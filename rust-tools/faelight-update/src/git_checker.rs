use anyhow::Result;
use std::process::Command;
use std::path::PathBuf;

pub fn check_git_updates() -> Vec<String> {
    let mut repos = Vec::new();
    
    // Check 0-core
    if let Ok(status) = check_repo("/home/christian/0-core") {
        if !status.is_empty() {
            repos.push(format!("0-core: {}", status));
        }
    }
    
    repos
}

fn check_repo(path: &str) -> Result<String> {
    let path = PathBuf::from(path);
    
    // Fetch first
    Command::new("git")
        .args(["fetch"])
        .current_dir(&path)
        .output()?;
    
    // Check for updates
    let output = Command::new("git")
        .args(["status", "-uno"])
        .current_dir(&path)
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    if stdout.contains("Your branch is behind") {
        Ok("behind origin".to_string())
    } else if stdout.contains("have diverged") {
        Ok("diverged from origin".to_string())
    } else {
        Ok(String::new())
    }
}
