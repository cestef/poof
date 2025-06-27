use std::{path::PathBuf, sync::Arc};

use dashmap::DashMap;
use facet_pretty::FacetPretty;
use futures_lite::future::Boxed as BoxedFuture;
use iroh::{NodeId, protocol::ProtocolHandler};
use iroh_blobs::rpc::client::blobs::MemClient;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    core::ticket::{ResponseCode, Ticket},
    info,
    utils::format::ReducedId,
};

pub const ALPN: &[u8] = b"poof/0";

#[derive(Debug, Clone)]
pub struct PoofProtocol {
    pub endpoint: iroh::Endpoint,
    pub blobs: MemClient,
    pub tickets: Arc<DashMap<String, Ticket>>,
}

impl PoofProtocol {
    pub fn new(blobs: MemClient, endpoint: iroh::Endpoint) -> Arc<Self> {
        Arc::new(PoofProtocol {
            endpoint,
            blobs,
            tickets: Default::default(),
        })
    }

    pub async fn send(&self, file_path: PathBuf) -> anyhow::Result<Ticket> {
        tracing::debug!("Dropping file: {:?}", file_path);
        let res = self
            .blobs
            .add_from_path(
                file_path.clone(),
                true,
                iroh_blobs::util::SetTagOption::Auto,
                iroh_blobs::rpc::client::blobs::WrapOption::NoWrap,
            )
            .await?
            .await?;

        let ticket = Ticket::new(res.hash).with_filename(
            file_path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string()),
        );

        tracing::debug!("File dropped with ticket: {}", ticket.pretty());
        self.tickets
            .insert(ticket.query.to_string(), ticket.clone());

        Ok(ticket)
    }

    pub async fn receive(
        &self,
        node_id: NodeId,
        query: String,
        out_file: Option<PathBuf>,
    ) -> anyhow::Result<()> {
        tracing::debug!("Receiving file for node: {}, query: {}", node_id, query);
        let connection = self
            .connect_with_retry(node_id, 3)
            .await
            .map_err(|e| crate::error!("Failed to connect to node: {}", e))?;
        let (mut send, mut recv) = connection.open_bi().await?;

        tracing::debug!("Sending query: {}", query);
        send.write_u32(query.len() as u32).await?;
        send.write_all(query.as_bytes()).await?;

        send.finish()?;
        send.stopped().await?;

        let response_code = ResponseCode::from_u8(recv.read_u8().await?);
        tracing::debug!("Received response code: {:?}", response_code);

        match response_code {
            Some(ResponseCode::Ok) => {
                let size = recv.read_u32().await? as usize;
                tracing::debug!("Received ticket size: {}", size);
                let mut buffer = vec![0; size];
                recv.read_exact(&mut buffer).await?;

                let ticket: Ticket = facet_msgpack::from_slice(&buffer)
                    .map_err(|e| crate::error!("Failed to deserialize ticket: {}", e))?;

                let res = self
                    .blobs
                    .download(ticket.hash(), node_id.into())
                    .await?
                    .await?;
                tracing::debug!("Downloading file with ticket: {:?}", res);
                let file = if let Some(ref out_file) = out_file {
                    if out_file.is_absolute() {
                        out_file.clone()
                    } else {
                        std::env::current_dir()
                            .unwrap_or_else(|_| PathBuf::from("."))
                            .join(out_file)
                    }
                } else {
                    std::env::current_dir()
                        .unwrap_or_else(|_| PathBuf::from("."))
                        .join(ticket.filename.as_deref().unwrap_or(&ticket.hash[..8]))
                };

                tracing::debug!("Writing file to {:?}", file);
                self.blobs
                    .export(
                        ticket.hash(),
                        file,
                        iroh_blobs::store::ExportFormat::Blob,
                        iroh_blobs::store::ExportMode::Copy,
                    )
                    .await?
                    .await?;
            }
            Some(ResponseCode::NotFound) => {
                tracing::warn!("Ticket not found for query: {}", query);
            }
            Some(ResponseCode::Error) => {
                tracing::error!("An error occurred while processing the request");
            }
            None => {
                tracing::error!("Received invalid response code");
            }
        }

        Ok(())
    }

    async fn connect_with_retry(
        &self,
        node_id: NodeId,
        retries: usize,
    ) -> anyhow::Result<iroh::endpoint::Connection> {
        let mut attempts = 0;
        loop {
            match self.endpoint.connect(node_id, ALPN).await {
                Ok(connection) => return Ok(connection),
                Err(_) if attempts < retries => {
                    tracing::warn!(
                        "Connection failed, retrying... ({}/{})",
                        attempts + 1,
                        retries
                    );
                    attempts += 1;
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

impl ProtocolHandler for PoofProtocol {
    fn accept(&self, connection: iroh::endpoint::Connection) -> BoxedFuture<anyhow::Result<()>> {
        let this = self.clone();
        Box::pin(async move {
            tracing::debug!("Accepted blob ticket connection: {:?}", connection);

            let (mut send, mut recv) = connection.accept_bi().await?;

            let query_size = recv.read_u32().await?;
            tracing::debug!("Received query size: {}", query_size);
            if query_size == 0 {
                tracing::warn!("Received empty query, closing connection");
                send.write_u8(ResponseCode::Error.to_u8()).await?;
                send.write_u32(0).await?;
                send.finish()?;
                return Ok(());
            }

            let query = {
                let mut buf = vec![0; query_size as usize];
                recv.read_exact(&mut buf).await?;
                String::from_utf8(buf).map_err(|e| crate::error!("Invalid UTF-8: {}", e))?
            };

            tracing::debug!("Received query: {}", query);

            if let Some(ticket) = this.tickets.get(&query) {
                tracing::debug!("Found ticket: {}", ticket.pretty());
                send.write_u8(ResponseCode::Ok.to_u8()).await?;
                let ticket = ticket.value();
                let bytes = facet_msgpack::to_vec(ticket);
                send.write_u32(bytes.len() as u32).await?;
                send.write_all(&bytes).await?;
                info!(
                    "Node {} requested ticket: {}",
                    connection.remote_node_id()?.reduced(),
                    ticket.query.blue().bold()
                );
            } else {
                tracing::warn!("Ticket not found for query: {}", query);
                send.write_u8(ResponseCode::NotFound.to_u8()).await?;
                send.write_u32(0).await?;
            }

            send.finish()?;

            send.stopped().await?;

            Ok(())
        })
    }
}
