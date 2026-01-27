use std::path::{Path, PathBuf};
use std::fs;
use crate::error::Result;

/// Read directory contents (non-recursive)
pub fn read_dir(path: &Path) -> Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        entries.push(entry.path());
    }
    
    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| {
        let a_is_dir = a.is_dir();
        let b_is_dir = b.is_dir();
        
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });
    
    Ok(entries)
}
