//! faelight-browser v0.1.0
//!
//! A web browser built on 0-Core principles:
//! - Security through transparency
//! - Intent over automation
//! - Recovery over perfection
//! - Comprehension over convenience

pub mod error;
pub mod navigation;
pub mod render;
pub mod security;
pub mod storage;
pub mod ui;

// Re-export common types
pub use error::{BrowserError, Result};
