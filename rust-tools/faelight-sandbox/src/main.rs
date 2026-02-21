//! faelight-sandbox v1.0.0
//! Controlled experimentation environment for Faelight Forest
//! Philosophy: Experiment freely. Understand completely. Revert instantly.

use anyhow::{bail, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

#[derive(Parser)]
#[command(name = "faelight-sandbox")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "Controlled experimentation environment — experiment freely, understand completely"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a command in the sandbox
    Run {
        /// Disable network access
        #[arg(long)]
        net_off: bool,

        /// Watch a directory for changes (default: ~/0-core)
        #[arg(long)]
        watch: Option<String>,

        /// Command to run
        #[arg(required = true, trailing_var_arg = true)]
        cmd: Vec<String>,
    },

    /// Show what changed in last sandbox session
    Diff,

    /// Show current sandbox status
    Status,

    /// Clear sandbox session state
    Clear,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileSnapshot {
    path: String,
    size: u64,
    modified: u64,
    hash: u64, // simple hash for change detection
}

#[derive(Debug, Serialize, Deserialize)]
struct SandboxSession {
    id: String,
    started: String,
    finished: Option<String>,
    command: String,
    net_off: bool,
    watch_dir: String,
    exit_code: Option<i32>,
    before: HashMap<String, FileSnapshot>,
    after: HashMap<String, FileSnapshot>,
}

fn state_dir() -> PathBuf {
    let home = home();
    PathBuf::from(&home).join(".local/state/0-core/sandbox")
}

fn session_path() -> PathBuf {
    state_dir().join("session.json")
}

fn home() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/home/christian".to_string())
}

fn ensure_state_dir() -> Result<()> {
    fs::create_dir_all(state_dir())?;
    Ok(())
}

/// Simple hash for change detection — not cryptographic
fn hash_file(path: &Path) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let Ok(content) = fs::read(path) else {
        return 0;
    };
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

fn snapshot_dir(dir: &Path) -> HashMap<String, FileSnapshot> {
    let mut map = HashMap::new();

    let walker = walkdir::WalkDir::new(dir)
        .max_depth(6)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file());

    for entry in walker {
        let path = entry.path();
        // Skip target/ build dirs and git objects
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/") || path_str.contains("/.git/objects/") {
            continue;
        }

        let Ok(meta) = fs::metadata(path) else {
            continue;
        };
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        map.insert(
            path_str.to_string(),
            FileSnapshot {
                path: path_str.to_string(),
                size: meta.len(),
                modified,
                hash: hash_file(path),
            },
        );
    }

    map
}

fn print_diff(session: &SandboxSession) {
    let mut added: Vec<&str> = vec![];
    let mut modified: Vec<&str> = vec![];
    let mut removed: Vec<&str> = vec![];

    // Files in after but not before = added
    for (path, after_snap) in &session.after {
        match session.before.get(path) {
            None => added.push(path.as_str()),
            Some(before_snap) => {
                if before_snap.hash != after_snap.hash {
                    modified.push(path.as_str());
                }
            }
        }
    }

    // Files in before but not after = removed
    for path in session.before.keys() {
        if !session.after.contains_key(path) {
            removed.push(path.as_str());
        }
    }

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "🧪 Sandbox Session Report".bold());
    println!("   Session: {}", session.id.dimmed());
    println!("   Command: {}", session.command.bright_white());
    println!(
        "   Network: {}",
        if session.net_off {
            "OFF (isolated)".bright_red()
        } else {
            "ON".bright_green()
        }
    );
    println!("   Watch:   {}", session.watch_dir.dimmed());
    if let Some(code) = session.exit_code {
        println!(
            "   Exit:    {}",
            if code == 0 {
                "0 ✓".bright_green()
            } else {
                code.to_string().bright_red()
            }
        );
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    if added.is_empty() && modified.is_empty() && removed.is_empty() {
        println!("\n  {} No file changes detected\n", "✅".green());
        return;
    }

    if !added.is_empty() {
        println!("\n  {} Added ({}):", "✚".bright_green(), added.len());
        for p in &added {
            let short = p.replace(&home(), "~");
            println!("    {} {}", "+".bright_green(), short);
        }
    }

    if !modified.is_empty() {
        println!("\n  {} Modified ({}):", "~".bright_yellow(), modified.len());
        for p in &modified {
            let short = p.replace(&home(), "~");
            println!("    {} {}", "~".bright_yellow(), short);
        }
    }

    if !removed.is_empty() {
        println!("\n  {} Removed ({}):", "✗".bright_red(), removed.len());
        for p in &removed {
            let short = p.replace(&home(), "~");
            println!("    {} {}", "-".bright_red(), short);
        }
    }

    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!(
        "  Total: {} added  {} modified  {} removed",
        added.len().to_string().bright_green(),
        modified.len().to_string().bright_yellow(),
        removed.len().to_string().bright_red(),
    );
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            net_off,
            watch,
            cmd,
        } => {
            ensure_state_dir()?;

            let watch_dir = watch.unwrap_or_else(|| format!("{}/0-core", home()));
            let watch_path = PathBuf::from(&watch_dir);

            if !watch_path.exists() {
                bail!("Watch directory does not exist: {}", watch_dir);
            }

            let session_id = Local::now().format("%Y%m%d-%H%M%S").to_string();
            let command_str = cmd.join(" ");

            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("{}", "🧪 faelight-sandbox".bold().bright_cyan());
            println!("   Session: {}", session_id.dimmed());
            println!("   Command: {}", command_str.bright_white());
            println!(
                "   Network: {}",
                if net_off {
                    "OFF (isolated)".bright_red()
                } else {
                    "ON".bright_green()
                }
            );
            println!("   Watch:   {}", watch_dir.dimmed());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

            // Snapshot before
            print!("\n  {} Snapshotting {}...", "📸".cyan(), watch_dir.dimmed());
            let before = snapshot_dir(&watch_path);
            println!(" {} files", before.len().to_string().bright_white());

            // Build session
            let mut session = SandboxSession {
                id: session_id,
                started: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                finished: None,
                command: command_str.clone(),
                net_off,
                watch_dir: watch_dir.clone(),
                exit_code: None,
                before,
                after: HashMap::new(),
            };

            println!("\n  {} Running command...\n", "▶".bright_green());
            println!("{}", "─".repeat(42).dimmed());

            // Execute
            let exit_code = if net_off {
                // Use unshare to create network namespace
                let mut unshare_cmd = Command::new("unshare");
                unshare_cmd.args(["--net", "--map-root-user", "--"]);
                unshare_cmd.args(&cmd);
                match unshare_cmd.status() {
                    Ok(s) => s.code().unwrap_or(1),
                    Err(e) => {
                        println!(
                            "  {} Failed to run with network isolation: {}",
                            "✗".bright_red(),
                            e
                        );
                        println!("  {} Falling back to normal execution", "⚠️".yellow());
                        let mut fallback = Command::new(&cmd[0]);
                        if cmd.len() > 1 {
                            fallback.args(&cmd[1..]);
                        }
                        fallback
                            .status()
                            .map(|s| s.code().unwrap_or(1))
                            .unwrap_or(1)
                    }
                }
            } else {
                let mut proc = Command::new(&cmd[0]);
                if cmd.len() > 1 {
                    proc.args(&cmd[1..]);
                }
                proc.status().map(|s| s.code().unwrap_or(1)).unwrap_or(1)
            };

            println!("{}", "─".repeat(42).dimmed());
            println!(
                "\n  {} Exit code: {}",
                "▶".dimmed(),
                if exit_code == 0 {
                    "0 ✓".bright_green()
                } else {
                    exit_code.to_string().bright_red()
                }
            );

            // Snapshot after
            print!("  {} Scanning for changes...", "🔍".cyan());
            let after = snapshot_dir(&watch_path);
            println!(" done");

            session.after = after;
            session.exit_code = Some(exit_code);
            session.finished = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

            // Save session
            fs::write(session_path(), serde_json::to_string_pretty(&session)?)?;

            // Print diff
            println!();
            print_diff(&session);
            println!(
                "\n  {} Session saved — run 'faelight-sandbox diff' to review again",
                "💾".dimmed()
            );
        }

        Commands::Diff => {
            if !session_path().exists() {
                println!(
                    "  {} No sandbox session found — run: faelight-sandbox run <cmd>",
                    "⚠️".yellow()
                );
                return Ok(());
            }
            let data = fs::read_to_string(session_path())?;
            let session: SandboxSession = serde_json::from_str(&data)?;
            print_diff(&session);
        }

        Commands::Status => {
            if !session_path().exists() {
                println!("  {} No active sandbox session", "○".bright_black());
                return Ok(());
            }
            let data = fs::read_to_string(session_path())?;
            let session: SandboxSession = serde_json::from_str(&data)?;
            println!("{}", "🧪 Sandbox Status".bold());
            println!("  Session: {}", session.id.bright_white());
            println!("  Command: {}", session.command.dimmed());
            println!("  Started: {}", session.started.dimmed());
            println!(
                "  Network: {}",
                if session.net_off {
                    "isolated".bright_red()
                } else {
                    "normal".bright_green()
                }
            );
            println!("  Changes: {} files", session.after.len());
        }

        Commands::Clear => {
            if session_path().exists() {
                fs::remove_file(session_path())?;
                println!("  {} Sandbox session cleared", "✅".green());
            } else {
                println!("  {} No session to clear", "○".bright_black());
            }
        }
    }

    Ok(())
}
