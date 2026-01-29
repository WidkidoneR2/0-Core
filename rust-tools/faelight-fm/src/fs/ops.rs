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

/// Copy a file to a new location
pub fn copy_file(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    fs::copy(src, dst)?;
    Ok(())
}

/// Move a file to a new location
pub fn move_file(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    fs::rename(src, dst)?;
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
    Ok(())
}

/// Check if core zone is locked
pub fn is_core_locked() -> bool {
    let core_path = std::path::PathBuf::from("/home")
        .join(std::env::var("USER").unwrap_or_else(|_| "christian".to_string()))
        .join("0-core");
    
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
