use once_cell::sync::Lazy;
use std::path::PathBuf;

pub const ALPN: &[u8] = b"poof/0";

pub static CONFIG_DIRECTORY: Lazy<PathBuf> = Lazy::new(|| {
    let home = dirs::home_dir().expect("Failed to get home directory");
    home.join(".config").join(env!("CARGO_PKG_NAME"))
});

pub const KEYS_FILE: &str = "keys.toml";
pub const DEFAULT_KEY_NAME: &str = "default";
