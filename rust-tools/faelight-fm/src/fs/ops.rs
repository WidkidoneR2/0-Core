use faelight_core::paths;
use std::path::Path;

/// Check if path is readable
#[allow(dead_code)]
pub fn is_readable(path: &Path) -> bool {
    path.exists() && path.metadata().is_ok()
}

/// Get parent directory
#[allow(dead_code)]
pub fn parent(path: &Path) -> Option<&Path> {
    path.parent()
}

use crate::error::Result;
use std::fs;
fn log_fm_event(action: &str, path: &str) {
    let home = std::env::var("HOME").unwrap_or_default();
    let db_path = format!("{}/0-core/runtime/state.db", home);
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64).unwrap_or(0);
        let payload = format!("action={} path={}", action, path);
        let _ = conn.execute(
            "INSERT INTO events (domain, action, payload, timestamp) VALUES ('fm', ?1, ?2, ?3)",
            rusqlite::params![action, payload, ts],
        );
    }
}

/// Copy a file to a new location
pub fn copy_file(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    fs::copy(src, dst)?;
    log_fm_event("copy", &src.to_string_lossy());
    Ok(())
}

/// Move a file to a new location
pub fn move_file(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    fs::rename(src, dst)?;
    log_fm_event("move", &src.to_string_lossy());
    Ok(())
}

/// Rename a file in place
pub fn rename_file(old: &std::path::Path, new_name: &str) -> Result<()> {
    if let Some(parent) = old.parent() {
        let new_path = parent.join(new_name);
        fs::rename(old, new_path)?;
    }
    Ok(())
}

/// Delete a file
pub fn delete_file(path: &std::path::Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    log_fm_event("delete", &path.to_string_lossy());
    Ok(())
}

/// Check if core zone is locked
pub fn is_core_locked() -> bool {
    let core_path = paths::core_dir();

    // Check if directory has immutable attribute
    if let Ok(output) = std::process::Command::new("lsattr")
        .arg("-d")
        .arg(&core_path)
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout.split_whitespace().collect();
        if let Some(attrs) = parts.first() {
            return attrs.contains('i');
        }
    }

    false
}
