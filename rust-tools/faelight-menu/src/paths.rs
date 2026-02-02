use std::env;

/// Get the scripts directory path
pub fn scripts_dir() -> String {
    let home = env::var("HOME").unwrap_or_default();
    format!("{}/0-core/scripts", home)
}

/// Get path to graceful-poweroff script
pub fn graceful_poweroff() -> String {
    format!("{}/graceful-poweroff", scripts_dir())
}

/// Get path to graceful-reboot script
pub fn graceful_reboot() -> String {
    format!("{}/graceful-reboot", scripts_dir())
}

/// Get path to a specific script by name
pub fn script_path(name: &str) -> String {
    format!("{}/{}", scripts_dir(), name)
}
