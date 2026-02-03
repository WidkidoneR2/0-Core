//! Path helpers using faelight_core::paths
//! Provides convenient wrappers for faelight-link specific paths

use faelight_core::paths;
use std::path::PathBuf;

/// Get the stow directory
pub fn stow_dir() -> PathBuf {
    paths::stow_dir()
}

/// Get backup directory for faelight-link
pub fn backup_dir() -> PathBuf {
    paths::faelight_link_backups()
}

/// Get home directory
pub fn home() -> PathBuf {
    paths::home()
}

/// Build stow package path format for searching
pub fn stow_package_pattern(package: &str) -> String {
    format!("0-core/03-interfaces/stow/{}", package)
}
