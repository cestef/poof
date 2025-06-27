use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum PoofError {
    #[error(transparent)]
    #[diagnostic(code(punch::other))]
    Other(#[from] anyhow::Error),

    #[error(transparent)]
    #[diagnostic(code(punch::io))]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    #[diagnostic(code(punch::iroh::key))]
    Iroh(#[from] iroh::KeyParsingError),

    #[error(transparent)]
    #[diagnostic(code(punch::toml))]
    Toml(#[from] facet_toml::TomlSerError),

    #[error("An error occurred: {message}")]
    #[diagnostic(code(punch::error))]
    Error {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

pub type Result<T, E = PoofError> = std::result::Result<T, E>;

#[macro_export]
macro_rules! error {
    (source = $source:expr, $($arg:tt)*) => {
        {
            crate::utils::error::PoofError::Error {
                message: format!($($arg)*),
                source: Some(Box::new($source)),
            }
        }
    };
    ($($arg:tt)*) => {
        {
            crate::utils::error::PoofError::Error {
                message: format!($($arg)*),
                source: None,
            }
        }
    };
}
