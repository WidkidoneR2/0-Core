use faelight_core::paths;

/// Get path to graceful-poweroff script
pub fn graceful_poweroff() -> String {
    paths::scripts_dir()
        .join("graceful-poweroff")
        .display()
        .to_string()
}

/// Get path to graceful-reboot script
pub fn graceful_reboot() -> String {
    paths::scripts_dir()
        .join("graceful-reboot")
        .display()
        .to_string()
}

/// Get path to a specific script by name
pub fn script_path(name: &str) -> String {
    paths::scripts_dir().join(name).display().to_string()
}
