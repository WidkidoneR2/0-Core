//! Git status detection for file markers
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus {
    Clean,
    Modified,
    Added,
    Deleted,
    Untracked,
}

impl GitStatus {
    pub fn marker(&self) -> &str {
        match self {
            GitStatus::Clean => "  ",
            GitStatus::Modified => "M ",
            GitStatus::Added => "A ",
            GitStatus::Deleted => "D ",
            GitStatus::Untracked => "??",
        }
    }
}

/// Get git status for all files in a directory
pub fn get_status(dir: &Path) -> std::collections::HashMap<String, GitStatus> {
    let mut status_map = std::collections::HashMap::new();
    
    // Run git status --porcelain
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(dir)
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let status_text = String::from_utf8_lossy(&output.stdout);
            for line in status_text.lines() {
                if line.len() >= 3 {
                    let status_code = &line[0..2];
                    let filename = line[3..].trim();
                    
                    let status = match status_code.trim() {
                        "M" | " M" | "MM" => GitStatus::Modified,
                        "A" | "AM" => GitStatus::Added,
                        "D" | " D" => GitStatus::Deleted,
                        "??" => GitStatus::Untracked,
                        _ => GitStatus::Clean,
                    };
                    
                    status_map.insert(filename.to_string(), status);
                }
            }
        }
    }
    
    status_map
}
