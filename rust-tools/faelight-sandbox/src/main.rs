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
use std::time::{SystemTime, UNIX_EPOCH};

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

    /// Create a reflink snapshot of a directory
    Snapshot {
        /// Directory to snapshot (default: ~/0-core)
        #[arg(long)]
        target: Option<String>,

        /// Name for this snapshot
        #[arg(long)]
        name: Option<String>,
    },

    /// Restore from a snapshot
    Restore {
        /// Snapshot name to restore
        name: String,
    },

    /// List available snapshots
    Snapshots,
    /// Show session history (last 10 runs)
    History,
    /// Query audit trail from state.db
    Audit {
        /// Filter by tool name
        #[arg(long)]
        tool: Option<String>,
        /// Show last N runs (default: 10)
        #[arg(long, default_value = "10")]
        limit: usize,
    },
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

fn history_dir() -> PathBuf {
    state_dir().join("history")
}

fn save_session_with_history(session: &SandboxSession) -> Result<()> {
    let json = serde_json::to_string_pretty(session)?;

    // Write current session
    fs::write(session_path(), &json)?;

    // Archive to history ring buffer (keep last 10)
    let hist = history_dir();
    fs::create_dir_all(&hist)?;
    let archive_name = format!("{}.json", session.id);
    fs::write(hist.join(&archive_name), &json)?;

    // Prune to 10 most recent
    let mut entries: Vec<_> = fs::read_dir(&hist)?
        .flatten()
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    if entries.len() > 10 {
        for old_entry in &entries[..entries.len() - 10] {
            let _ = fs::remove_file(old_entry.path());
        }
    }

    Ok(())
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


fn emit_to_ledger(session: &SandboxSession, duration_secs: u64, files_changed: usize) {
    let db_path = PathBuf::from(home()).join("0-core/runtime/state.db");
    if !db_path.exists() { return; }
    let Ok(conn) = rusqlite::Connection::open(&db_path) else { return; };
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let exit_code = session.exit_code.unwrap_or(-1);
    let result = if exit_code == 0 { "ok" } else { "fail" };
    let payload = format!(
        r#"{{"actor":"faelight-sandbox","result":"{}","detail":{{"command":"{}","exit_code":{},"duration_secs":{},"files_changed":{},"net_off":{}}}}}"#,
        result,
        session.command.replace('"', "'"),
        exit_code,
        duration_secs,
        files_changed,
        session.net_off,
    );
    conn.execute(
        "INSERT INTO events (domain, action, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!["sandbox", "run", payload, ts],
    ).ok();
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

            let run_start = std::time::Instant::now();
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

            // Save session + archive to history ring buffer
            save_session_with_history(&session)?;
            // Calculate duration and files changed
            let duration_secs = run_start.elapsed().as_secs();
            let files_changed = {
                let mut n = 0usize;
                for (p, af) in &session.after {
                    match session.before.get(p) {
                        None => n += 1,
                        Some(bf) => if bf.hash != af.hash { n += 1; }
                    }
                }
                for p in session.before.keys() {
                    if !session.after.contains_key(p) { n += 1; }
                }
                n
            };
            emit_to_ledger(&session, duration_secs, files_changed);

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
            // Compute actual changed file count, not total watched files
            let changed = {
                let mut n = 0usize;
                for (path, after_snap) in &session.after {
                    match session.before.get(path) {
                        None => n += 1,
                        Some(before_snap) => {
                            if before_snap.hash != after_snap.hash {
                                n += 1;
                            }
                        }
                    }
                }
                for path in session.before.keys() {
                    if !session.after.contains_key(path) {
                        n += 1;
                    }
                }
                n
            };
            if changed == 0 {
                println!("  Changes: none");
            } else {
                println!("  Changes: {} files", changed);
            }
        }

        Commands::Clear => {
            if session_path().exists() {
                fs::remove_file(session_path())?;
                println!("  {} Sandbox session cleared", "✅".green());
            } else {
                println!("  {} No session to clear", "○".bright_black());
            }
        }

        Commands::Snapshot { target, name } => {
            ensure_state_dir()?;
            let target_dir = target.unwrap_or_else(|| format!("{}/0-core", home()));
            let snap_name =
                name.unwrap_or_else(|| Local::now().format("%Y%m%d-%H%M%S").to_string());
            let snap_dir = state_dir().join("snapshots").join(&snap_name);

            fs::create_dir_all(snap_dir.parent().unwrap())?;

            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("{}", "📸 faelight-sandbox snapshot".bold().bright_cyan());
            println!("   Source:   {}", target_dir.dimmed());
            println!("   Snapshot: {}", snap_name.bright_white());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

            print!(
                "
  {} Creating reflink snapshot...",
                "📸".cyan()
            );

            let status = Command::new("cp")
                .args([
                    "--reflink=auto",
                    "-r",
                    &target_dir,
                    snap_dir.to_str().unwrap(),
                ])
                .status()?;

            if status.success() {
                println!(" {}", "done".bright_green());
                println!(
                    "  {} Snapshot '{}' created",
                    "✅".green(),
                    snap_name.bright_white()
                );
                println!(
                    "  {} Location: {}",
                    "💾".dimmed(),
                    snap_dir.display().to_string().dimmed()
                );
            } else {
                println!(" {}", "failed".bright_red());
                println!(
                    "  {} Reflink failed — ensure source and destination are on same btrfs volume",
                    "✗".bright_red()
                );
            }
        }

        Commands::Restore { name } => {
            let snap_dir = state_dir().join("snapshots").join(&name);
            if !snap_dir.exists() {
                println!("  {} Snapshot '{}' not found", "⚠️".yellow(), name);
                println!("  {} Run: faelight-sandbox snapshots", "💡".dimmed());
                return Ok(());
            }

            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!(
                "  {} Restore from snapshot '{}'",
                "⚠️".bright_yellow(),
                name.bright_white()
            );
            println!("  This will overwrite the current target directory.");
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

            use dialoguer::Confirm;
            if !Confirm::new()
                .with_prompt("Proceed with restore?")
                .default(false)
                .interact()?
            {
                println!("  {} Cancelled", "○".bright_black());
                return Ok(());
            }

            // Find what was snapshotted (single dir inside snap_dir)
            let entries: Vec<_> = fs::read_dir(&snap_dir)?.flatten().collect();
            if entries.len() != 1 {
                println!("  {} Unexpected snapshot structure", "✗".bright_red());
                return Ok(());
            }

            let snap_content = &entries[0].path();
            let target = PathBuf::from(&home()).join(entries[0].file_name());

            print!("  {} Restoring...", "🔄".cyan());
            let status = Command::new("cp")
                .args([
                    "--reflink=auto",
                    "-r",
                    "--backup=numbered",
                    snap_content.to_str().unwrap(),
                    target.parent().unwrap().to_str().unwrap(),
                ])
                .status()?;

            if status.success() {
                println!(" {}", "done".bright_green());
                println!("  {} Restored from '{}'", "✅".green(), name.bright_white());
            } else {
                println!(" {}", "failed".bright_red());
            }
        }

        Commands::History => {
            let hist = history_dir();
            if !hist.exists() || fs::read_dir(&hist)?.count() == 0 {
                println!("  {} No session history found", "○".bright_black());
                println!("  {} Run: faelight-sandbox run <cmd>", "💡".dimmed());
                return Ok(());
            }
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("{}", "🧪 Session History (last 10)".bold());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            let mut entries: Vec<_> = fs::read_dir(&hist)?
                .flatten()
                .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
                .collect();
            entries.sort_by_key(|e| e.file_name());
            entries.reverse();
            for entry in &entries {
                let data = fs::read_to_string(entry.path()).unwrap_or_default();
                if let Ok(s) = serde_json::from_str::<SandboxSession>(&data) {
                    // Compute changed count
                    let mut changed = 0usize;
                    for (p, after_snap) in &s.after {
                        match s.before.get(p) {
                            None => changed += 1,
                            Some(b) => {
                                if b.hash != after_snap.hash {
                                    changed += 1;
                                }
                            }
                        }
                    }
                    for p in s.before.keys() {
                        if !s.after.contains_key(p) {
                            changed += 1;
                        }
                    }
                    let exit = s
                        .exit_code
                        .map(|c| {
                            if c == 0 {
                                "✓".to_string()
                            } else {
                                format!("exit {}", c)
                            }
                        })
                        .unwrap_or_default();
                    println!(
                        "  {} {}  {}  {} changed  {}",
                        "▶".dimmed(),
                        s.id.bright_white(),
                        s.command.dimmed(),
                        changed.to_string().cyan(),
                        exit.green(),
                    );
                }
            }
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("  View session: faelight-sandbox diff (loads most recent)");
        }
        Commands::Audit { tool, limit } => {
            let db_path = PathBuf::from(home()).join("0-core/runtime/state.db");
            if !db_path.exists() {
                println!("  {} state.db not found — no audit data yet", "○".dimmed());
                return Ok(());
            }
            let conn = rusqlite::Connection::open(&db_path)?;
            let query = if tool.is_some() {
                format!(
                    "SELECT payload, timestamp FROM events WHERE domain='sandbox' AND action='run' AND payload LIKE '%{}%' ORDER BY timestamp DESC LIMIT {}",
                    tool.as_ref().unwrap(), limit
                )
            } else {
                format!(
                    "SELECT payload, timestamp FROM events WHERE domain='sandbox' AND action='run' ORDER BY timestamp DESC LIMIT {}",
                    limit
                )
            };
            let mut stmt = conn.prepare(&query)?;
            let rows: Vec<(String, i64)> = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                .filter_map(|r| r.ok())
                .collect();
            println!();
            println!("{}", "  ╭─ 🧪 Sandbox Audit Trail ──────────────────────────────".bright_cyan());
            if rows.is_empty() {
                println!("  │  {} No sandbox runs recorded yet", "○".dimmed());
                println!("  │  Use {} to run commands", "faelight-sandbox run".bright_cyan());
            } else {
                for (payload, ts) in &rows {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) {
                        let cmd = v["detail"]["command"].as_str().unwrap_or("unknown");
                        let exit = v["detail"]["exit_code"].as_i64().unwrap_or(-1);
                        let dur = v["detail"]["duration_secs"].as_u64().unwrap_or(0);
                        let changed = v["detail"]["files_changed"].as_u64().unwrap_or(0);
                        let result = v["result"].as_str().unwrap_or("?");
                        let result_icon = if result == "ok" { "✅".to_string() } else { "❌".to_string() };
                        let short_cmd = if cmd.len() > 40 { format!("{}...", &cmd[..40]) } else { cmd.to_string() };
                        println!("  │  {} {}  {}s  {} files  exit:{}",
                            result_icon,
                            short_cmd.bright_white(),
                            dur.to_string().dimmed(),
                            changed.to_string().cyan(),
                            exit.to_string().dimmed(),
                        );
                    }
                }
            }
            println!("{}", "  ╰─────────────────────────────────────────────────────".dimmed());
            println!();
        }
        Commands::Snapshots => {
            let snap_root = state_dir().join("snapshots");
            if !snap_root.exists() || fs::read_dir(&snap_root)?.count() == 0 {
                println!("  {} No snapshots found", "○".bright_black());
                println!("  {} Run: faelight-sandbox snapshot", "💡".dimmed());
                return Ok(());
            }

            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("{}", "📸 Available Snapshots".bold());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

            let mut snaps: Vec<_> = fs::read_dir(&snap_root)?.flatten().collect();
            snaps.sort_by_key(|e| e.file_name());

            for snap in snaps {
                let name = snap.file_name().to_string_lossy().to_string();
                // Get size
                let size = Command::new("du")
                    .args(["-sh", snap.path().to_str().unwrap()])
                    .output()
                    .map(|o| {
                        String::from_utf8_lossy(&o.stdout)
                            .split_whitespace()
                            .next()
                            .unwrap_or("?")
                            .to_string()
                    })
                    .unwrap_or_else(|_| "?".to_string());

                println!(
                    "  {} {}  {}",
                    "▶".dimmed(),
                    name.bright_white(),
                    size.dimmed()
                );
            }
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("  Restore with: faelight-sandbox restore <name>");
        }
    }

    Ok(())
}
