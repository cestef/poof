use std::path::PathBuf;

use clap::{Parser, Subcommand};
use iroh_blobs::ticket::BlobTicket;

#[derive(Parser, Debug)]
pub struct Opts {
    /// The command to run
    #[clap(subcommand)]
    pub command: Command,

    /// The key to use
    #[clap(long, short = 'k')]
    pub key: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Drop a file (send)
    #[clap(alias = "d")]
    Drop {
        /// The file to drop
        file: PathBuf,
    },

    /// Catch a file (receive)
    #[clap(alias = "c")]
    Catch {
        /// The ticket for the file to catch
        ticket: BlobTicket,
    },

    /// Key management commands
    #[clap(subcommand, aliases = ["k", "keys"])]
    Key(KeyCommand),
}

#[derive(Subcommand, Debug)]
pub enum KeyCommand {
    /// Generate a new key
    Generate {
        /// The name of the key to generate
        name: String,
    },

    /// Set or display the default key
    Default {
        /// The name of the key to set as default
        name: Option<String>,
    },

    /// List all keys
    List,

    /// Remove a key
    Remove {
        /// The name of the key to remove
        name: String,
    },
}
