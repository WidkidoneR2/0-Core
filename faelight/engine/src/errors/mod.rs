use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum CoreError {
    #[error("Domain error [{domain}]: {message}")]
    Domain { domain: String, message: String },
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),
    #[error("Registry error: {0}")]
    Registry(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}

pub type CoreResult<T> = Result<T, CoreError>;
