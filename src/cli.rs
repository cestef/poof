use clap::{Parser, Subcommand};
use iroh::PublicKey;
use std::path::PathBuf;

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
        /// Host identifier or alias
        host: String,

        /// File identifier or ticket
        query: String,

        /// Optional destination path
        #[clap(long, short = 'o')]
        output: Option<PathBuf>,
    },

    /// Host management commands
    #[clap(subcommand, aliases = ["h", "hosts"])]
    Host(HostCommand),

    /// Key management commands
    #[clap(subcommand, aliases = ["k", "keys"])]
    Key(KeyCommand),
}

#[derive(Subcommand, Debug)]
pub enum HostCommand {
    /// Add a new host
    #[clap(alias = "a")]
    Add {
        /// Alias for the host
        alias: String,
        /// Public key of the host
        public_key: PublicKey,
        /// Optional description
        #[clap(long, short = 'd')]
        description: Option<String>,
    },

    /// Remove a host
    #[clap(alias = "r")]
    Remove {
        /// Alias of the host to remove
        alias: String,
    },

    /// List all hosts
    #[clap(alias = "l")]
    List {
        /// Show detailed information
        #[clap(long)]
        verbose: bool,
    },

    /// Show host details
    #[clap(alias = "s")]
    Show {
        /// Alias of the host to show
        alias: String,
    },

    /// Rename a host
    #[clap(alias = "rn")]
    Rename {
        /// Current alias
        old_alias: String,
        /// New alias
        new_alias: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum KeyCommand {
    /// Generate a new key
    #[clap(alias = "g")]
    Generate {
        /// Name for the key
        name: String,
        /// Optional description
        #[clap(long, short = 'd')]
        description: Option<String>,
        /// Set as default key
        #[clap(long)]
        default: bool,
    },

    /// Add an existing key
    #[clap(alias = "a")]
    Add {
        /// Name for the key
        name: String,
        /// Secret key to add
        secret_key: String,
        /// Optional description
        #[clap(long, short)]
        description: Option<String>,
        /// Set as default key
        #[clap(long)]
        default: bool,
    },

    /// Remove a key
    #[clap(alias = "r")]
    Remove {
        /// Name of the key to remove
        name: String,
    },

    /// List all keys
    #[clap(alias = "l")]
    List {
        /// Show secret keys (use with caution)
        #[clap(long)]
        show_secret: bool,

        /// Show full key
        #[clap(short, long)]
        full: bool,
    },

    /// Show key details
    #[clap(alias = "s")]
    Show {
        /// Name of the key to show
        name: String,

        /// Show secret key (use with caution)
        #[clap(long)]
        show_secret: bool,
    },

    /// Set default key
    #[clap(alias = "d")]
    Default {
        /// Name of the key to set as default
        name: String,
    },
}
