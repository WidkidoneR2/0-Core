//! Path helpers using faelight_core::paths
//! Provides convenient wrappers for faelight-link specific paths

use faelight_core::paths;
use std::path::PathBuf;

pub fn backup_dir() -> PathBuf {
    paths::faelight_link_backups()
}

pub fn home() -> PathBuf {
    paths::home()
}
