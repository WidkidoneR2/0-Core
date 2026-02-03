use faelight_core::paths;
use std::path::PathBuf;

/// Get the sway config file path
pub fn sway_config_path() -> PathBuf {
    paths::sway_config()
}
