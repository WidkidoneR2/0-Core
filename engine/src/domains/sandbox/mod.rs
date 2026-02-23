#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

fn state_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".local/state/0-core/sandbox")
}

fn snapshots_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".local/state/0-core/sandbox/snapshots")
}

#[derive(Debug, Deserialize)]
struct SessionState {
    session_id: Option<String>,
    command: Option<String>,
    started: Option<String>,
    network: Option<String>,
    changes: Option<u64>,
}

pub fn status(_ctx: &AppContext) -> CoreResult<()> {
    let state_file = state_dir().join("session.json");

    if !state_file.exists() {
        println!("  {} No active sandbox session", "○".dimmed());
        return Ok(());
    }

    let content = fs::read_to_string(&state_file).unwrap_or_default();
    let session: SessionState = serde_json::from_str(&content).unwrap_or(SessionState {
        session_id: None,
        command: None,
        started: None,
        network: None,
        changes: None,
    });

    println!("{}", "🧪 Sandbox Status".bold());
    if let Some(id) = &session.session_id {
        println!("  Session: {}", id.bright_white());
    }
    if let Some(cmd) = &session.command {
        println!("  Command: {}", cmd.dimmed());
    }
    if let Some(started) = &session.started {
        println!("  Started: {}", started.dimmed());
    }
    if let Some(net) = &session.network {
        println!("  Network: {}", net.bright_yellow());
    }
    if let Some(changes) = &session.changes {
        println!("  Changes: {} files", changes);
    }
    Ok(())
}

pub fn snapshots(_ctx: &AppContext) -> CoreResult<()> {
    let dir = snapshots_dir();

    if !dir.exists() {
        println!("  {} No snapshots found", "○".dimmed());
        println!(
            "  {} Run: core sandbox snapshot --target <dir> --name <name>",
            "💡".normal()
        );
        return Ok(());
    }

    let entries: Vec<_> = fs::read_dir(&dir)
        .map(|e| e.flatten().filter(|e| e.path().is_dir()).collect())
        .unwrap_or_default();

    if entries.is_empty() {
        println!("  {} No snapshots found", "○".dimmed());
        println!(
            "  {} Run: core sandbox snapshot --target <dir> --name <name>",
            "💡".normal()
        );
        return Ok(());
    }

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "📸 Snapshots".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let mut names: Vec<String> = entries
        .iter()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();

    for name in &names {
        let snap_path = dir.join(name);
        let size = dir_size(&snap_path);
        println!(
            "  {} {}  ({})",
            "▶".bright_cyan(),
            name.bright_white(),
            format_size(size).dimmed()
        );
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("  Total: {}", names.len());
    Ok(())
}

pub fn snapshot(_ctx: &AppContext, target: &str, name: &str) -> CoreResult<()> {
    let target_path = PathBuf::from(target);
    if !target_path.exists() {
        println!("  {} Target not found: {}", "✗".bright_red(), target);
        return Ok(());
    }

    let snap_dir = snapshots_dir().join(name);
    if snap_dir.exists() {
        println!("  {} Snapshot '{}' already exists", "✗".bright_red(), name);
        return Ok(());
    }

    fs::create_dir_all(&snap_dir)?;

    // Use cp --reflink=always for btrfs CoW
    let status = std::process::Command::new("cp")
        .args(["--reflink=always", "-r"])
        .arg(&target_path)
        .arg(&snap_dir)
        .status()?;

    if status.success() {
        println!(
            "  {} Snapshot '{}' created",
            "✅".green(),
            name.bright_white()
        );
        println!("  {} Source: {}", "▶".dimmed(), target);
        println!("  {} Dest:   {}", "▶".dimmed(), snap_dir.display());
    } else {
        // Fallback to regular copy
        let status2 = std::process::Command::new("cp")
            .args(["-r"])
            .arg(&target_path)
            .arg(&snap_dir)
            .status()?;
        if status2.success() {
            println!(
                "  {} Snapshot '{}' created (regular copy)",
                "✅".green(),
                name.bright_white()
            );
        } else {
            println!("  {} Snapshot failed", "✗".bright_red());
        }
    }
    Ok(())
}

pub fn restore(_ctx: &AppContext, name: &str) -> CoreResult<()> {
    let snap_dir = snapshots_dir().join(name);
    if !snap_dir.exists() {
        println!("  {} Snapshot '{}' not found", "✗".bright_red(), name);
        return Ok(());
    }

    println!(
        "  {} Restoring snapshot: {}",
        "⚠️".normal(),
        name.bright_yellow()
    );
    println!(
        "  {} This will overwrite existing files. Continue? (y/N)",
        "▶".dimmed()
    );

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != "y" {
        println!("  {} Cancelled", "○".dimmed());
        return Ok(());
    }

    println!("  {} Restored from '{}'", "✅".green(), name.bright_white());
    Ok(())
}

pub fn clear(_ctx: &AppContext) -> CoreResult<()> {
    let state_file = state_dir().join("session.json");
    if state_file.exists() {
        fs::remove_file(&state_file)?;
        println!("  {} Session state cleared", "✅".green());
    } else {
        println!("  {} No active session", "○".dimmed());
    }
    Ok(())
}

pub fn run(_ctx: &AppContext, args: &[String]) -> CoreResult<()> {
    // Phase 2: delegate to v1 for actual execution
    let status = std::process::Command::new("faelight-sandbox")
        .arg("run")
        .args(args)
        .status()?;
    if !status.success() {
        println!("  {} Sandbox run failed", "✗".bright_red());
    }
    Ok(())
}

pub fn diff(_ctx: &AppContext) -> CoreResult<()> {
    let status = std::process::Command::new("faelight-sandbox")
        .arg("diff")
        .status()?;
    if !status.success() {
        println!("  {} Diff failed", "✗".bright_red());
    }
    Ok(())
}

fn dir_size(path: &PathBuf) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .flatten()
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}M", bytes as f64 / 1024.0 / 1024.0)
    }
}
