use std::path::PathBuf;

use crate::{
    cli::Opts,
    core::key::{KeyEntry, KeysManager},
};
use facet_pretty::FacetPretty;
use iroh::{Endpoint, SecretKey};
use owo_colors::OwoColorize;
use rand::rngs::OsRng;
use tracing::debug;

pub mod key;

pub async fn run(opts: Opts) -> crate::Result<()> {
    crate::utils::logging::init()?;

    debug!("{opts:?}");

    let mut manager = KeysManager::load().await?;

    match opts.command {
        crate::cli::Command::Key(cmd) => match cmd {
            crate::cli::KeyCommand::Default { .. } => todo!(),
            crate::cli::KeyCommand::List => {
                debug!("Available keys: {}", manager.pretty());
                for key in manager.keys() {
                    let is_default = if let Some(default_key) = manager.default_key_name() {
                        key.name == default_key
                    } else {
                        false
                    };
                    println!(
                        "{}: {} {}",
                        key.name,
                        key.path,
                        if is_default {
                            "(default)".dimmed().to_string()
                        } else {
                            "".to_string()
                        }
                    );
                }
            }
            crate::cli::KeyCommand::Generate { name } => {
                let key = KeyEntry::new(name);
                debug!("Generated key: {}", key.pretty());
                let should_generate = if PathBuf::from(&key.name).exists() {
                    inquire::Confirm::new("Key already exists. Do you want to overwrite it?")
                        .with_default(false)
                        .prompt()?
                } else {
                    true
                };

                if should_generate {
                    key.save(&SecretKey::generate(&mut OsRng)).await?;
                    manager.add_key(key).await?;
                } else {
                    debug!("Key generation aborted by user.");
                }
            }
            crate::cli::KeyCommand::Remove { .. } => todo!(),
        },
        crate::cli::Command::Catch { .. } => {}
        crate::cli::Command::Drop { .. } => {}
    }

    debug!("Loaded keys: {}", manager.pretty());
    let key_entry = if let Some(ref k) = opts.key {
        manager.get_key(&k)
    } else {
        manager.get_default_key()
    };

    let sk = key_entry
        .ok_or_else(|| crate::error!("Key not found: {}", opts.key.as_deref().unwrap_or_default()))?
        .load()
        .await?;

    let endpoint = Endpoint::builder()
        .discovery_n0()
        .discovery_local_network()
        .secret_key(sk)
        .bind()
        .await?;
    Ok(())
}
