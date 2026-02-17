//! Error types for faelight-browser
//!
//! Following 0-Core principle: Errors are explicit and actionable

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Wayland connection failed: {0}")]
    WaylandConnection(String),

    #[error("Rendering error: {0}")]
    Rendering(String),

    #[error("Storage error: {0}")]
    Storage(#[from] std::io::Error),

    #[error("Permission denied for domain: {0}")]
    PermissionDenied(String),

    #[error("Network request failed: {0}")]
    Network(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, BrowserError>;
