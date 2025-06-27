use crate::cli::{HostCommand, KeyCommand};
use crate::core::hosts::{HostManager, KeyManager};
use crate::utils::format::{ReducedId, format_duration};
use crate::{Result, info, success, warning};
use iroh::SecretKey;
use owo_colors::OwoColorize;
use std::str::FromStr;

pub async fn handle_host_command(cmd: HostCommand, host_manager: &HostManager) -> Result<()> {
    match cmd {
        HostCommand::Add {
            alias,
            public_key,
            description,
        } => {
            host_manager.add_host(alias.clone(), public_key, description)?;
            success!(
                "Added host '{}' with public key {}",
                alias.bold(),
                public_key.reduced()
            );
        }

        HostCommand::Remove { alias } => {
            let host = host_manager.remove_host(&alias)?;
            success!(
                "Removed host '{}' ({})",
                alias.bold(),
                host.public_key().reduced()
            );
        }

        HostCommand::List { verbose } => {
            let hosts = host_manager.list_hosts()?;
            if hosts.is_empty() {
                info!("No hosts configured");
                return Ok(());
            }

            println!("\n{}", "Configured Hosts:".bold().underline());
            for host in hosts {
                if verbose {
                    println!(
                        "\n{}",
                        format!("  {} {}", "•".blue(), host.alias.bold()).bright_white()
                    );
                    println!("    {}: {}", "Public Key".dimmed(), host.public_key);
                    if let Some(desc) = &host.description {
                        println!("    {}: {}", "Description".dimmed(), desc);
                    }
                    println!(
                        "    {}: {}",
                        "Added".dimmed(),
                        format_duration(host.added_at().elapsed().unwrap_or_default())
                    );
                    if let Some(last_seen) = host.last_seen() {
                        println!(
                            "    {}: {}",
                            "Last Seen".dimmed(),
                            format_duration(last_seen.elapsed().unwrap_or_default())
                        );
                    }
                    if !host.metadata.is_empty() {
                        println!("    {}:", "Metadata".dimmed());
                        for (key, value) in &host.metadata {
                            println!("      {}: {}", key, value);
                        }
                    }
                } else {
                    println!(
                        "  {} {} ({})",
                        "•".blue(),
                        host.alias.bold(),
                        host.public_key().reduced()
                    );
                }
            }
            println!();
        }

        HostCommand::Show { alias } => {
            if let Some(host) = host_manager.get_host(&alias)? {
                println!("\n{}", format!("Host: {}", host.alias).bold().underline());
                println!("  {}: {}", "Public Key".dimmed(), host.public_key);
                if let Some(desc) = &host.description {
                    println!("  {}: {}", "Description".dimmed(), desc);
                }
                println!(
                    "  {}: {}",
                    "Added".dimmed(),
                    format_duration(host.added_at().elapsed().unwrap_or_default())
                );
                if let Some(last_seen) = host.last_seen() {
                    println!(
                        "  {}: {}",
                        "Last Seen".dimmed(),
                        format_duration(last_seen.elapsed().unwrap_or_default())
                    );
                }
                if !host.metadata.is_empty() {
                    println!("  {}:", "Metadata".dimmed());
                    for (key, value) in &host.metadata {
                        println!("    {}: {}", key, value);
                    }
                }
                println!();
            } else {
                warning!("Host '{}' not found", alias);
            }
        }

        HostCommand::Rename {
            old_alias,
            new_alias,
        } => {
            host_manager.rename_host(&old_alias, new_alias.clone())?;
            success!(
                "Renamed host '{}' to '{}'",
                old_alias.bold(),
                new_alias.bold()
            );
        }
    }

    Ok(())
}

pub async fn handle_key_command(cmd: KeyCommand, key_manager: &KeyManager) -> Result<()> {
    match cmd {
        KeyCommand::Generate {
            name,
            description,
            default,
        } => {
            let key = key_manager.generate_key(name.clone(), description)?;
            if default {
                key_manager.set_default_key(&name)?;
            }
            success!(
                "Generated new key '{}' with public key {}",
                name.bold(),
                key.public_key().reduced()
            );
            if default {
                info!("Set '{}' as default key", name.bold());
            }
        }

        KeyCommand::Add {
            name,
            secret_key,
            description,
            default,
        } => {
            let sk = SecretKey::from_str(&secret_key)
                .map_err(|e| crate::error!(source = e, "Invalid secret key format"))?;
            key_manager.add_key(name.clone(), sk.clone(), description)?;
            if default {
                key_manager.set_default_key(&name)?;
            }
            success!(
                "Added key '{}' with public key {}",
                name.bold(),
                sk.public().reduced()
            );
            if default {
                info!("Set '{}' as default key", name.bold());
            }
        }

        KeyCommand::Remove { name } => {
            let key = key_manager.remove_key(&name)?;
            success!(
                "Removed key '{}' ({})",
                name.bold(),
                key.public_key().reduced()
            );
        }

        KeyCommand::List { show_secret, full } => {
            let keys = key_manager.list_keys()?;
            let default_key = key_manager.get_default_key()?;

            if keys.is_empty() {
                info!("No keys configured");
                return Ok(());
            }

            println!("\n{}", "Configured Keys:".bold().underline());
            for key in keys {
                let is_default = default_key
                    .as_ref()
                    .map(|dk| dk.name == key.name)
                    .unwrap_or(false);
                let marker = if is_default {
                    "★".yellow().to_string()
                } else {
                    "•".blue().to_string()
                };

                println!("  {} {} ({})", marker, key.name.bold(), {
                    if full {
                        key.public_key().to_string().bold().blue().to_string()
                    } else {
                        key.public_key().reduced()
                    }
                });

                if show_secret {
                    println!("    {}: {}", "Secret Key".dimmed(), key.secret_key);
                }
                if let Some(desc) = &key.description {
                    println!("    {}: {}", "Description".dimmed(), desc);
                }
                println!(
                    "    {}: {} ago",
                    "Created".dimmed(),
                    format_duration(key.created_at().elapsed().unwrap_or_default())
                );
            }
            println!();

            if let Some(default) = default_key {
                info!(
                    "Default key: {} ({})",
                    default.name.bold(),
                    if full {
                        default.public_key().to_string().bold().blue().to_string()
                    } else {
                        default.public_key().reduced()
                    }
                );
            }
        }

        KeyCommand::Show { name, show_secret } => {
            if let Some(key) = key_manager.get_key(&name)? {
                let default_key = key_manager.get_default_key()?;
                let is_default = default_key
                    .as_ref()
                    .map(|dk| dk.name == key.name)
                    .unwrap_or(false);

                println!("\n{}", format!("Key: {}", key.name).bold().underline());
                if is_default {
                    println!("  {} {}", "Status".dimmed(), "Default".yellow().bold());
                }
                println!("  {}: {}", "Public Key".dimmed(), key.public_key());
                if show_secret {
                    println!("  {}: {}", "Secret Key".dimmed(), key.secret_key);
                }
                if let Some(desc) = &key.description {
                    println!("  {}: {}", "Description".dimmed(), desc);
                }
                println!(
                    "  {}: {} ago",
                    "Created".dimmed(),
                    format_duration(key.created_at().elapsed().unwrap_or_default())
                );
                println!();
            } else {
                warning!("Key '{}' not found", name);
            }
        }

        KeyCommand::Default { name } => {
            key_manager.set_default_key(&name)?;
            success!("Set '{}' as default key", name.bold());
        }
    }

    Ok(())
}
