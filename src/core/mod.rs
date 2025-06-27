use std::str::FromStr;

use crate::{
    cli::Opts,
    core::{
        commands::{handle_host_command, handle_key_command},
        hosts::{HostManager, KeyManager},
        protocol::{ALPN, PoofProtocol},
    },
    info, success,
    utils::format::ReducedId,
};
use iroh::{Endpoint, NodeId, SecretKey, protocol::Router};
use iroh_blobs::net_protocol::Blobs;
use rand::rngs::OsRng;
use tracing::debug;

pub mod commands;
pub mod config;
pub mod hosts;
pub mod protocol;
pub mod ticket;

pub async fn run(opts: Opts) -> crate::Result<()> {
    crate::utils::logging::init()?;

    debug!("{opts:?}");

    let hosts = HostManager::new();
    let keys = KeyManager::new();

    let sk = if let Some(key) = opts.key {
        if let Some(hk) = keys.get_key(&key)? {
            hk.secret_key().clone()
        } else {
            return Err(crate::error!("Key '{}' not found", key));
        }
    } else if let Some(hk) = keys.get_default_key()? {
        hk.secret_key().clone()
    } else {
        // Generate a new secret key if no key is provided and no default key exists
        let sk = SecretKey::generate(&mut OsRng);
        keys.add_key("default".to_string(), sk.clone(), None)?;
        info!("No key provided, generated a new default key");
        sk
    };

    let endpoint = Endpoint::builder()
        .discovery_n0()
        .discovery_local_network()
        .secret_key(sk)
        .bind()
        .await?;

    let blobs = Blobs::memory().build(&endpoint);
    let client = blobs.client();

    let proto = PoofProtocol::new(client.clone(), endpoint.clone());

    let router = Router::builder(endpoint.clone())
        .accept(iroh_blobs::ALPN, blobs)
        .accept(ALPN, proto.clone())
        .spawn();

    match opts.command {
        crate::cli::Command::Host(cmd) => handle_host_command(cmd, &hosts).await?,
        crate::cli::Command::Key(cmd) => handle_key_command(cmd, &keys).await?,
        crate::cli::Command::Drop { file } => {
            info!("Node started with ID: {}", endpoint.node_id());
            let file_path = file.canonicalize()?;
            let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
            let ticket = proto.send(file_path.clone()).await?;
            success!(
                "Dropped file '{}' with ticket {}",
                file_name.bold(),
                ticket.query.blue().bold()
            );
            tokio::signal::ctrl_c().await?;
        }
        crate::cli::Command::Catch {
            host,
            output,
            query,
        } => {
            let node_id = if let Some(host) = hosts.get_host(&host)? {
                hosts.update_last_seen(&host.alias)?;
                host.public_key().into()
            } else {
                if let Ok(node_id) = NodeId::from_str(&host) {
                    node_id
                } else {
                    return Err(crate::error!("Invalid host: {}", host));
                }
            };

            info!(
                "Catching file with query '{}' from node {}",
                query.bold(),
                node_id.reduced()
            );
            proto.receive(node_id, query, output).await?;
            success!("File received successfully");
        }
    }

    router.shutdown().await?;
    Ok(())
}
