//! Error types for faelight-core

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FaelightError {
    #[error("Font loading failed: {0}")]
    FontLoad(String),
    
    #[error("Glyph rasterization failed for '{0}'")]
    GlyphRasterize(char),
    
    #[error("Canvas operation failed: {0}")]
    Canvas(String),
    
    #[error("Wayland error: {0}")]
    Wayland(String),
}

pub type Result<T> = std::result::Result<T, FaelightError>;
