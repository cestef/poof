use std::str::FromStr;

use facet::Facet;
use iroh_blobs::Hash;

#[derive(Debug, Facet, Clone)]
pub struct Ticket {
    pub hash: String,
    pub query: String,
    pub filename: Option<String>,
}

impl Ticket {
    /// Creates a new ticket.
    pub fn new(hash: Hash) -> Self {
        let hash = hash.to_string();
        Self {
            query: Self::generate_query(&hash),
            hash,
            filename: None,
        }
    }

    /// Generates a query string (6-char)
    fn generate_query(hash: &str) -> String {
        String::from(&hash[..6])
    }

    pub fn with_filename(mut self, filename: Option<String>) -> Self {
        self.filename = filename;
        self
    }

    pub fn hash(&self) -> Hash {
        Hash::from_str(&self.hash).expect("Invalid hash format")
    }
}

#[derive(Debug, Facet, Clone, Copy)]
#[repr(u8)]
pub enum ResponseCode {
    Ok = 0,
    NotFound = 1,
    Error = 2,
}

impl ResponseCode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ResponseCode::Ok),
            1 => Some(ResponseCode::NotFound),
            2 => Some(ResponseCode::Error),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}
