use std::env;
use std::path::PathBuf;

/// Get the sway config file path
pub fn sway_config_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".config/sway/config")
}
