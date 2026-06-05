// faelight-fm v4.0 -- Git plugin
// Handles: .git dirs, any file in a git repo

use std::{path::Path, process::Command};
use super::{Plugin, PluginAction};

pub struct GitPlugin;

impl Plugin for GitPlugin {
    fn name(&self) -> &str { "git" }

    fn handles(&self, path: &Path) -> bool {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        name == ".git"
            || name == "COMMIT_EDITMSG"
            || name == "CHANGELOG.md"
            || name.ends_with(".lock") && path.parent()
                .and_then(|p| p.join(".git").exists().then_some(()))
                .is_some()
    }

    fn preview(&self, path: &Path) -> String {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if name == ".git" || path.is_dir() {
            return git_repo_summary(path.parent().unwrap_or(path));
        }
        git_repo_summary(path.parent().unwrap_or(path))
    }

    fn actions(&self, _path: &Path) -> Vec<PluginAction> {
        vec![
            PluginAction {
                label: "git log".to_string(),
                key: 'l',
                description: "Show recent commits".to_string(),
            },
            PluginAction {
                label: "git status".to_string(),
                key: 's',
                description: "Show working tree status".to_string(),
            },
            PluginAction {
                label: "git diff".to_string(),
                key: 'd',
                description: "Show unstaged diff".to_string(),
            },
        ]
    }

    fn execute(&self, path: &Path, action: char) -> String {
        let dir = if path.is_dir() { path } else { path.parent().unwrap_or(path) };
        match action {
            'l' => git_log(dir),
            's' => git_status(dir),
            'd' => git_diff(dir),
            _ => String::new(),
        }
    }
}

fn git_repo_summary(path: &Path) -> String {
    let mut out = String::from("🌿 Git Repository\n━━━━━━━━━━━━━━━━\n");

    // Branch
    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if !branch.is_empty() {
        out.push_str(&format!("Branch:  {}\n", branch));
    }

    // Latest commit
    let log = Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if !log.is_empty() {
        out.push_str(&format!("Latest:  {}\n", log));
    }

    // Status summary
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    let modified = status.lines().filter(|l| !l.starts_with("??")).count();
    let untracked = status.lines().filter(|l| l.starts_with("??")).count();
    if modified > 0 || untracked > 0 {
        out.push_str(&format!("Changes: {} modified  {} untracked\n", modified, untracked));
    } else {
        out.push_str("Status:  clean\n");
    }

    // Recent commits
    let log5 = Command::new("git")
        .args(["log", "--oneline", "-5"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    if !log5.is_empty() {
        out.push_str("\nRecent commits:\n");
        for line in log5.lines() {
            out.push_str(&format!("  {}\n", line));
        }
    }
    out
}

#[allow(dead_code)]
fn git_log(path: &Path) -> String {
    Command::new("git")
        .args(["log", "--oneline", "-20"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "git log failed".to_string())
}

#[allow(dead_code)]
fn git_status(path: &Path) -> String {
    Command::new("git")
        .args(["status", "--short"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "git status failed".to_string())
}

#[allow(dead_code)]
fn git_diff(path: &Path) -> String {
    let out = Command::new("git")
        .args(["diff", "--stat"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    if out.is_empty() { "No unstaged changes".to_string() } else { out }
}
