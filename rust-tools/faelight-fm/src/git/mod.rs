// faelight-fm v3.1 -- git integration

use std::{collections::HashMap, path::PathBuf, process::Command};
use crate::types::GitStatus;

pub fn get_git_status(path: &PathBuf) -> HashMap<String, GitStatus> {
    let mut map = HashMap::new();
    let git_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| PathBuf::from(s.trim()));
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output();
    if let Ok(out) = output {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if line.len() > 3 {
                let status = &line[..2];
                let file_path = line[3..].to_string();
                let rel = if let Some(ref root) = git_root {
                    let abs = root.join(&file_path);
                    abs.strip_prefix(path)
                        .ok()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or(file_path.clone())
                } else { file_path.clone() };
                let first = rel.split('/').next().unwrap_or(&rel).to_string();
                if first.is_empty() || first.starts_with('.') { continue; }
                let gs = match &status[..1] {
                    "M" | "A" | "R" | "C" => GitStatus::Staged,
                    _ if status.contains('?') => GitStatus::Untracked,
                    _ => GitStatus::Modified,
                };
                map.entry(first).or_insert(gs);
            }
        }
    }
    map
}

#[allow(dead_code)]
pub fn branch_info(path: &PathBuf) -> String {
    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if branch.is_empty() || branch == "HEAD" { return String::new(); }
    // Count ahead/behind
    let ahead = Command::new("git")
        .args(["rev-list", "--count", "@{u}..HEAD"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if ahead.is_empty() || ahead == "0" {
        format!("branch: {}", branch)
    } else {
        format!("branch: {} ↑{}", branch, ahead)
    }
}

#[allow(dead_code)]
pub fn file_diff(path: &PathBuf, filename: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["diff", "HEAD", "--", filename])
        .current_dir(path)
        .output()
        .ok()?;
    let diff = String::from_utf8_lossy(&output.stdout).to_string();
    if diff.is_empty() {
        // Try staged diff
        let staged = Command::new("git")
            .args(["diff", "--cached", "--", filename])
            .current_dir(path)
            .output()
            .ok()?;
        let s = String::from_utf8_lossy(&staged.stdout).to_string();
        if s.is_empty() { return None; }
        return Some(s.lines().take(60).collect::<Vec<_>>().join("\n"));
    }
    Some(diff.lines().take(60).collect::<Vec<_>>().join("\n"))
}

pub fn stage_file(path: &PathBuf, filename: &str) -> Result<(), String> {
    let out = Command::new("git")
        .args(["add", filename])
        .current_dir(path)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() { Ok(()) }
    else { Err(String::from_utf8_lossy(&out.stderr).to_string()) }
}

pub fn unstage_file(path: &PathBuf, filename: &str) -> Result<(), String> {
    let out = Command::new("git")
        .args(["restore", "--staged", filename])
        .current_dir(path)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() { Ok(()) }
    else { Err(String::from_utf8_lossy(&out.stderr).to_string()) }
}
