//! faelight-sandbox v3.0.0
//! Controlled experimentation environment for Faelight Forest
//! Philosophy: Experiment freely. Understand completely. Revert instantly.

mod cgroup;
mod policy;
mod seccomp_filter;
use anyhow::{bail, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::os::unix::process::CommandExt;
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
        /// Apply a named security policy
        #[arg(long)]
        policy: Option<String>,
        /// Deep isolation level: net, full (net+pid+fs)
        #[arg(long)]
        isolate: Option<String>,
        /// Profile resource usage (memory, CPU, I/O)
        #[arg(long)]
        profile: bool,

        /// Run even when a control the policy REQUIRES could not be applied.
        ///
        /// Without this, a sandbox that cannot deliver what it promised refuses rather than
        /// running the command anyway and reporting success. With it, the run proceeds and the
        /// degradation is named in the report and the event.
        #[arg(long)]
        allow_degraded: bool,

        /// Watch a directory for changes (default: ~/0-core)
        #[arg(long)]
        watch: Option<String>,

        /// Command to run
        #[arg(required = true, trailing_var_arg = true)]
        cmd: Vec<String>,
    },

    /// Show what changed in last sandbox session
    Diff {
        /// Show the CHANGED LINES, not just which files changed.
        ///
        /// Only for files inside a git work tree, and deliberately so: the session stores a
        /// path, size, mtime and hash -- NOT content -- so there is no "before" to diff
        /// against. git already has it. A file outside a repo is reported honestly as
        /// unshowable rather than guessed at.
        #[arg(long)]
        patch: bool,
    },

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
    /// List available security policies
    PolicyList,
    /// Show details of a specific policy
    PolicyShow {
        /// Policy name
        name: String,
    },
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
    /// Whether the network was ACTUALLY isolated, as against net_off which records only
    /// whether the flag was passed.
    ///
    /// Both are kept, deliberately: what was asked for and what happened are different facts
    /// and a reader wants both. net_off alone was driving the report AND both event emitters,
    /// so every row in state.db recorded the flag rather than the outcome -- an --isolate full
    /// run was logged as not isolated, and an unshare fallback was logged as isolated.
    #[serde(default)]
    network_isolated: bool,
    watch_dir: String,
    exit_code: Option<i32>,
    /// Controls that were asked for and could not be delivered.
    ///
    /// A FACT ABOUT THE SESSION, not a line on stderr. The report reads net_off to print
    /// "OFF (isolated)" -- which is the INTENT, not what happened -- so after an unshare
    /// fallback it claimed isolation that was never applied. This is how the report, and
    /// anything reading the session later, finds out.
    #[serde(default)]
    degraded: Vec<String>,
    before: HashMap<String, FileSnapshot>,
    after: HashMap<String, FileSnapshot>,
}

/// Where bwrap is, if it is anywhere.
///
/// Looked up rather than assumed: 246 rules that a control which cannot be applied is a
/// DEGRADATION, and "bwrap is missing" is exactly that. Hardcoding /usr/bin/bwrap would turn a
/// reportable absence into a spawn failure with a confusing message.
fn which_bwrap() -> Option<String> {
    for dir in std::env::var("PATH").unwrap_or_default().split(':') {
        let p = std::path::Path::new(dir).join("bwrap");
        if p.is_file() {
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
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

/// The hunks for one path, or None when they cannot be produced.
///
/// ⭐ GIT IS THE BEFORE. The session records a hash, not content, so DevBox has no copy of
/// what a file looked like before the run. `~/0-core` is a git work tree and git does have it --
/// so for anything tracked, `git diff` gives exactly what is wanted: the changed lines only, in
/// red and green, without DevBox storing a byte.
///
/// ⚠️ AND IT RETURNS None RATHER THAN AN EMPTY STRING when it cannot show a file. A file
/// outside a repo, or a repo with no HEAD, has no before -- reporting that as "no changes" would
/// be the collapse this codebase keeps finding. The caller says so instead.
fn git_hunks(path: &str, added: bool) -> Option<String> {
    let dir = std::path::Path::new(path).parent()?;
    let inside = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(dir)
        .output()
        .ok()?;
    if !inside.status.success() {
        return None;
    }
    // An added file is untracked, so there is nothing in the index to compare against.
    // --no-index against /dev/null renders every line as green, which is the truth for a file
    // that did not exist before.
    let out = if added {
        Command::new("git")
            .args(["diff", "--no-index", "--color=always", "/dev/null", path])
            .current_dir(dir)
            .output()
            .ok()?
    } else {
        Command::new("git")
            .args(["diff", "--color=always", "--", path])
            .current_dir(dir)
            .output()
            .ok()?
    };
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

fn print_hunks(path: &str, added: bool) {
    match git_hunks(path, added) {
        Some(text) => {
            for line in text.lines().skip(4) {
                println!("      {}", line);
            }
        }
        None => println!(
            "      {}",
            "(no git history for this path -- the change cannot be shown, only reported)".dimmed()
        ),
    }
}

fn print_diff(session: &SandboxSession, patch: bool) {
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
        if session.network_isolated {
            "OFF (isolated)".bright_red()
        } else {
            "ON".bright_green()
        }
    );
    println!("   Watch:   {}", session.watch_dir.dimmed());
    // WHAT ACTUALLY HAPPENED, not what was asked for. The Network line above reads net_off,
    // which is the INTENT -- after an unshare fallback it printed "OFF (isolated)" for a run
    // that had no isolation at all. These lines are how the report stops claiming a posture it
    // did not deliver.
    if !session.degraded.is_empty() {
        println!(
            "   {}  THE SANDBOX DID NOT DELIVER EVERYTHING IT WAS ASKED FOR",
            "[!!]".bright_red()
        );
        for d in &session.degraded {
            println!("          {}", d.yellow());
        }
    }
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
            if patch {
                print_hunks(p, true);
            }
        }
    }

    if !modified.is_empty() {
        println!("\n  {} Modified ({}):", "~".bright_yellow(), modified.len());
        for p in &modified {
            let short = p.replace(&home(), "~");
            println!("    {} {}", "~".bright_yellow(), short);
            if patch {
                print_hunks(p, false);
            }
        }
    }

    if !removed.is_empty() {
        println!("\n  {} Removed ({}):", "✗".bright_red(), removed.len());
        for p in &removed {
            let short = p.replace(&home(), "~");
            println!("    {} {}", "-".bright_red(), short);
            if patch {
                print_hunks(p, false);
            }
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

fn get_io_bytes() -> (u64, u64) {
    // Returns (read_bytes, write_bytes) from /proc/self/io
    let content = std::fs::read_to_string("/proc/self/io").unwrap_or_default();
    let mut read = 0u64;
    let mut write = 0u64;
    for line in content.lines() {
        if line.starts_with("read_bytes:") {
            read = line
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        }
        if line.starts_with("write_bytes:") {
            write = line
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        }
    }
    (read, write)
}

fn get_memory_kb() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|n| n.parse().ok())
        })
        .unwrap_or(0)
}

fn emit_to_ledger_with_policy(
    session: &SandboxSession,
    duration_secs: u64,
    files_changed: usize,
    policy_name: Option<&str>,
) {
    let db_path = faelight_core::paths::state_db();
    if !db_path.exists() {
        return;
    }
    let Ok(conn) = rusqlite::Connection::open(&db_path) else {
        return;
    };
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let exit_code = session.exit_code.unwrap_or(-1);
    let result = if exit_code == 0 { "ok" } else { "fail" };
    let policy_str = policy_name.unwrap_or("none");
    let payload = format!(
        r#"{{"actor":"faelight-sandbox","result":"{}","detail":{{"command":"{}","exit_code":{},"duration_secs":{},"files_changed":{},"net_off":{},"policy":"{}"}}}}"#,
        result,
        session.command.replace('"', "'"),
        exit_code,
        duration_secs,
        files_changed,
        session.network_isolated,
        policy_str,
    );
    conn.execute(
        "INSERT INTO events (domain, action, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!["sandbox", "run", payload, ts],
    )
    .ok();
}

#[allow(dead_code)]
fn emit_to_ledger(session: &SandboxSession, duration_secs: u64, files_changed: usize) {
    let db_path = faelight_core::paths::state_db();
    if !db_path.exists() {
        return;
    }
    let Ok(conn) = rusqlite::Connection::open(&db_path) else {
        return;
    };
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
        session.network_isolated,
    );
    conn.execute(
        "INSERT INTO events (domain, action, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!["sandbox", "run", payload, ts],
    )
    .ok();
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            net_off,
            policy,
            isolate,
            profile,
            watch,
            cmd,
            allow_degraded,
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
            // Load policy if specified
            let active_policy = if let Some(policy_name) = &policy {
                match policy::SandboxPolicy::load(policy_name) {
                    Ok(p) => Some(p),
                    Err(e) => {
                        eprintln!("  ✗ Policy error: {}", e);
                        return Ok(());
                    }
                }
            } else {
                None
            };

            println!("   Session: {}", session_id.dimmed());
            println!("   Command: {}", command_str.bright_white());
            let policy_controls_net = active_policy
                .as_ref()
                .map(|p| !p.allow_net)
                .unwrap_or(false);
            // "requested" because this prints BEFORE the run, so it can only state intent.
            // The session report at the end prints what actually happened, and the two can
            // legitimately differ -- an --isolate flag, or an unshare fallback.
            if !policy_controls_net {
                println!(
                    "   Network: {} (requested)",
                    if net_off {
                        "OFF (isolated)".bright_red()
                    } else {
                        "ON".bright_green()
                    }
                );
            }
            println!("   Watch:   {}", watch_dir.dimmed());
            if let Some(ref p) = active_policy {
                println!("   Policy:  {}", p.name.bright_yellow());
                for restriction in p.restrictions() {
                    println!("             {}", restriction.dimmed());
                }
                if !p.allow_net {
                    println!("   Network: {}", "OFF (policy)".bright_red());
                }
            }
            if let Some(ref level) = isolate {
                println!("   Isolate: {}", level.bright_magenta().bold());
                match level.as_str() {
                    "seccomp" => {
                        println!("             syscalls: filtered (dangerous blocked)");
                    }
                    "full" => {
                        println!("             network: isolated");
                        println!("             pid: isolated");
                        println!("             filesystem: tmpfs overlay");
                    }
                    "net" => println!("             network: isolated"),
                    _ => println!("             unknown level — use: net, full"),
                }
            }
            if profile {
                println!(
                    "   Profile: {}",
                    "enabled — resource tracking".bright_cyan()
                );
            }
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

            // Snapshot before
            print!("\n  {} Snapshotting {}...", "📸".cyan(), watch_dir.dimmed());
            let before = snapshot_dir(&watch_path);
            println!(" {} files", before.len().to_string().bright_white());

            // Build session
            let mut session = SandboxSession {
                // Cloned because apply_env below reads the id for {session} substitution in
                // set_env values. The session struct owns one copy; the closure borrows another.
                id: session_id.clone(),
                started: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                finished: None,
                command: command_str.clone(),
                net_off,
                // Set after the run: network_isolated is not known until the spawn has been
                // attempted, because the unshare fallback can take it away again.
                network_isolated: false,
                watch_dir: watch_dir.clone(),
                exit_code: None,
                // Empty here and filled after the run: unshare failing is only discovered at
                // spawn time, a couple of hundred lines below this.
                degraded: Vec::new(),
                before,
                after: HashMap::new(),
            };

            let run_start = std::time::Instant::now();
            println!("\n  {} Running command...\n", "▶".bright_green());
            println!("{}", "─".repeat(42).dimmed());

            // Execute — respect policy + isolate flag
            let isolate_level = isolate.as_deref().unwrap_or("none");
            let network_isolated = net_off
                || active_policy
                    .as_ref()
                    .map(|p| !p.allow_net)
                    .unwrap_or(false)
                || matches!(isolate_level, "net" | "full");
            let pid_isolated = isolate_level == "full";
            let fs_isolated = isolate_level == "full";
            let seccomp_enabled = matches!(isolate_level, "seccomp" | "full");

            // ⭐ WHAT THE POLICY REQUIRES, AND WHAT ACTUALLY HAPPENED.
            //
            // Everything below records DEGRADATIONS: a control that was asked for and could not be
            // delivered. They are collected rather than printed and forgotten, because three
            // separate paths used to let one slip past (2026-09-12):
            //   - seccomp failing printed "continuing without" and carried on
            //   - unshare failing fell back to NO isolation at all, after the header had already
            //     printed "Network: OFF (isolated)"
            //   - seccomp was silently dropped whenever net isolation was on, with no message
            let required: Vec<String> = active_policy
                .as_ref()
                .map(|p| p.require.clone())
                .unwrap_or_default();
            let mut degraded: Vec<String> = Vec::new();

            // ⚠ THIS WAS SILENT AND IS NOW NOT. `--isolate full` sets BOTH seccomp and net,
            // and the old line dropped seccomp without a word. Whether the two are genuinely
            // incompatible or the filter is merely too strict is UNRESOLVED -- the comment claimed
            // "unshare needs syscalls blocked by strict seccomp" and nobody has tested it since.
            // Until that is settled the behaviour is unchanged; what changed is that it says so.
            let apply_seccomp = seccomp_enabled && !network_isolated;
            if seccomp_enabled && network_isolated {
                degraded.push(
                    "seccomp: not applied because network isolation is on (unresolved: the filter \
                     may block the syscalls unshare needs)"
                        .to_string(),
                );
            }
            if apply_seccomp {
                match seccomp_filter::apply_filter(false) {
                    Ok(()) => eprintln!("  ✅ Seccomp filter applied"),
                    Err(e) => degraded.push(format!("seccomp: filter failed to apply: {}", e)),
                }
            }

            // The refusal happens BEFORE the command runs. A sandbox that has already executed
            // something cannot un-execute it, so the check sits here rather than in the report.
            let unmet: Vec<String> = degraded
                .iter()
                .filter(|d| required.iter().any(|r| d.starts_with(r.as_str())))
                .cloned()
                .collect();
            if !unmet.is_empty() && !allow_degraded {
                println!();
                println!(
                    "  {} REFUSED -- the policy requires a control that could not be applied",
                    "✗".bright_red()
                );
                for d in &unmet {
                    println!("      {}", d);
                }
                println!();
                println!("  run with --allow-degraded to proceed anyway; the degradation is then");
                println!("  recorded in the session report rather than hidden");
                std::process::exit(3);
            }

            // ⭐ THE CHILD ENVIRONMENT IS BUILT, NOT INHERITED (2026-09-06).
            //
            // Every spawn below used to hand the child this process's whole environment, so a
            // command run under any policy saw the real HOME, the real NSH_CONFIG and the real
            // state.db. That is what made allow_env decorative and made a shell sandbox
            // impossible: you cannot test a shell in isolation while its own config is visible.
            //
            // Order is deliberate: CLEAR, then allow named variables through, then apply set_env
            // over the top. A name in both lists takes the set_env value -- overriding is the
            // more specific instruction.
            //
            // ⚠️ NO POLICY MEANS NO CHANGE. `faelight-sandbox run` without --policy keeps
            // inheriting, because clearing the environment for callers who asked for nothing
            // would break every existing use of this tool for a benefit they did not request.
            let apply_env = |c: &mut Command| {
                let Some(ref p) = active_policy else { return };
                c.env_clear();
                for name in &p.allow_env {
                    if let Ok(v) = std::env::var(name) {
                        c.env(name, v);
                    }
                }
                // The same {session} substitution and the same create-if-absent rule as set_env
                // below, because a cwd that does not exist fails the spawn with a message about
                // the command rather than about the policy.
                if let Some(dir) = &p.set_cwd {
                    let dir = dir.replace("{session}", &session_id);
                    let _ = std::fs::create_dir_all(&dir);
                    c.current_dir(dir);
                }
                for (k, v) in &p.set_env {
                    let value = v.replace("{session}", &session_id);
                    // The sandbox creates the roots it points at. A policy that redirects HOME to a
                    // directory that does not exist asks the child to fail in a way that looks
                    // like a shell bug -- and it would be debugged as one.
                    if k.starts_with("HOME") || k.starts_with("XDG_") {
                        let _ = std::fs::create_dir_all(&value);
                    }
                    c.env(k, value);
                }
            };

            let max_cpu = active_policy
                .as_ref()
                .map(|p| p.max_cpu_seconds)
                .unwrap_or(0);

            // Warn about fs write restriction (full enforcement in v3 Phase 3)
            if let Some(ref p) = active_policy {
                if !p.allow_fs_write {
                    println!(
                        "  {} Policy: filesystem is READ-ONLY except the sandbox home",
                        "🛡".yellow()
                    );
                }
                if max_cpu > 0 && max_cpu < 300 {
                    println!(
                        "  {} Policy: CPU limit {}s (enforcement in Phase 3)",
                        "🛡".yellow(),
                        max_cpu
                    );
                }
            }

            // Record start time for profiling
            let profile_start = std::time::Instant::now();
            let start_mem = if profile { get_memory_kb() } else { 0 };
            let start_io = if profile { get_io_bytes() } else { (0, 0) };

            // ⭐ INT-246: THE MEMORY CAP, CREATED BEFORE ANYTHING SPAWNS.
            //
            // Its degradations join the same list the seccomp and unshare paths use, so a policy
            // with require = ["memory"] refuses through the gate that already exists rather than
            // needing one of its own.
            let (cgroup, cg_degraded) = cgroup::Cgroup::create(
                &session_id,
                active_policy.as_ref().and_then(|p| {
                    // 1024 is the struct default and means "no cap declared". A policy that
                    // genuinely wants 1GB says so and gets it; this only skips the default.
                    if p.max_memory_mb > 0 && p.max_memory_mb < 1024 {
                        Some(p.max_memory_mb)
                    } else {
                        None
                    }
                }),
            );
            degraded.extend(cg_degraded);

            // ⚠️ THE REFUSAL GATE RUNS AGAIN, because the cap is only now known to have failed.
            // The earlier gate could not see this: the cgroup did not exist when it ran.
            let unmet_now: Vec<String> = degraded
                .iter()
                .filter(|d| required.iter().any(|r| d.starts_with(r.as_str())))
                .cloned()
                .collect();
            if !unmet_now.is_empty() && !allow_degraded {
                println!();
                println!(
                    "  {} REFUSED -- the policy requires a control that could not be applied",
                    "✗".bright_red()
                );
                for d in &unmet_now {
                    println!("      {}", d);
                }
                println!("  run with --allow-degraded to proceed anyway");
                std::process::exit(3);
            }

            // Joining is done by the CHILD, between fork and exec, by writing 0 to cgroup.procs
            // -- 0 means "the process doing the writing". Doing it here would move the sandbox
            // itself under the cap.
            let procs_path = cgroup.as_ref().map(|c| c.procs_file());
            let join_cgroup = move |c: &mut Command| {
                let Some(ref p) = procs_path else { return };
                let p = p.clone();
                unsafe {
                    c.pre_exec(move || {
                        std::fs::write(&p, "0").map_err(|e| {
                            std::io::Error::other(format!("could not join cgroup: {}", e))
                        })?;
                        Ok(())
                    });
                }
            };

            // ⭐ INT-246: THE FILESYSTEM LIMIT, VIA BWRAP.
            //
            // allow_fs_write = false printed "filesystem: read-only" and blocked nothing -- writes
            // were DETECTED afterwards by the snapshot diff, which is a report, not a control.
            //
            // ⚠️ "READ-ONLY" HAS ONE EXCEPTION AND IT IS DELIBERATE. A sandbox where nothing at all
            // is writable fails for reasons that have nothing to do with what is being tested:
            // python3 cannot make a temp file, cargo cannot build. The exception is the sandbox's
            // OWN HOME -- /tmp/devbox-{session}, which the devbox policy already creates, already
            // redirects HOME to, and already throws away. Writes go to the disposable directory
            // and nowhere else.
            //
            // MEASURED 2026-09-12: with / read-only and that one --bind, `python3 tempfile` lands
            // inside the sandbox home and `cargo --version` runs. TMPDIR is set as well so that is
            // deterministic rather than a fallback that happens to work.
            //
            // ⚠️ BWRAP NESTS INSIDE unshare RATHER THAN REPLACING IT. bwrap has --unshare-net and
            // could do both jobs; measured, it works. It is NOT used that way because unshare
            // already carries --pid/--fork/--mount for `--isolate full`, all of which would need
            // re-verifying for nothing measurable. Each mechanism owns one domain: unshare the
            // namespaces, bwrap the filesystem, cgroups the resources.
            let fs_readonly = active_policy
                .as_ref()
                .map(|p| !p.allow_fs_write)
                .unwrap_or(false);

            let cmd: Vec<String> = if fs_readonly {
                match which_bwrap() {
                    Some(bw) => {
                        // The writable exception: whatever the policy set HOME to, or the real one
                        // if it set nothing. A policy that blocks writes without redirecting HOME
                        // would otherwise leave the user's actual home writable.
                        let writable = active_policy
                            .as_ref()
                            .and_then(|p| p.set_env.get("HOME").cloned())
                            .map(|h| h.replace("{session}", &session_id))
                            // ⚠️ NEVER THE REAL HOME. This fell back to std::env::var("HOME") and then
                            // bound it READ-WRITE -- so , whose whole point is blocking
                            // writes, made /home/christian writable and a probe file landed there.
                            // Measured 2026-09-12; the comment above this block claimed to guard
                            // against exactly that and did the opposite.
                            //
                            // A policy that blocks writes and names no writable HOME gets a fresh
                            // disposable directory. Something must be writable or nothing runs;
                            // it must not be anything the user cares about.
                            .unwrap_or_else(|| format!("/tmp/sandbox-rw-{}", session_id));
                        let _ = std::fs::create_dir_all(&writable);
                        let mut v: Vec<String> = vec![
                            bw,
                            "--ro-bind".into(),
                            "/".into(),
                            "/".into(),
                            "--dev".into(),
                            "/dev".into(),
                            "--proc".into(),
                            "/proc".into(),
                            "--bind".into(),
                            writable.clone(),
                            writable.clone(),
                            "--setenv".into(),
                            "TMPDIR".into(),
                            writable,
                            "--".into(),
                        ];
                        v.extend(cmd.iter().cloned());
                        v
                    }
                    None => {
                        degraded.push(
                            "fs: bwrap is not installed, so allow_fs_write = false blocked nothing \
                             -- writes are only detected afterwards"
                                .to_string(),
                        );
                        cmd.iter().cloned().collect()
                    }
                }
            } else {
                cmd.iter().cloned().collect()
            };

            // The refusal gate runs once more: fs is only now known to have failed, for the same
            // reason the cgroup gate needed its own pass.
            let unmet_fs: Vec<String> = degraded
                .iter()
                .filter(|d| required.iter().any(|r| d.starts_with(r.as_str())))
                .cloned()
                .collect();
            if !unmet_fs.is_empty() && !allow_degraded {
                println!();
                println!(
                    "  {} REFUSED -- the policy requires a control that could not be applied",
                    "✗".bright_red()
                );
                for d in &unmet_fs {
                    println!("      {}", d);
                }
                println!("  run with --allow-degraded to proceed anyway");
                std::process::exit(3);
            }

            let exit_code = if network_isolated {
                let mut unshare_cmd = Command::new("unshare");
                // Build namespace flags based on isolation level
                let mut ns_args = vec!["--net", "--map-root-user"];
                if pid_isolated {
                    ns_args.push("--pid");
                    ns_args.push("--fork");
                }
                if fs_isolated {
                    ns_args.push("--mount");
                }
                ns_args.push("--");
                unshare_cmd.args(&ns_args);
                unshare_cmd.args(&cmd);
                apply_env(&mut unshare_cmd);
                join_cgroup(&mut unshare_cmd);
                match unshare_cmd.status() {
                    Ok(s) => s.code().unwrap_or(1),
                    Err(e) => {
                        // ⭐ THE WORST OF THE THREE, AND IT RAN AFTER THE HEADER HAD ALREADY LIED.
                        //
                        // This printed "Falling back to normal execution" and then ran the command
                        // with NO ISOLATION OF ANY KIND -- no namespace, no seccomp -- while the
                        // header above had already printed "Network: OFF (isolated)" and the session
                        // report at the end would say the policy was applied. A sandbox that stops
                        // sandboxing and keeps going is the defect this whole tool exists to find.
                        //
                        // ⚠️ THE FALLBACK IS KEPT, DELIBERATELY. unshare fails for reasons the
                        // caller cannot fix -- a nested container, a kernel without user namespaces --
                        // and refusing outright would make DevBox unusable there even for policies
                        // that never asked for network isolation. What changed is that it is now a
                        // RECORDED DEGRADATION rather than a line that scrolls past.
                        //
                        // ⚠️ AND THE GATE ABOVE CANNOT CATCH THIS ONE. That gate runs before the
                        // command; this failure is only discovered at spawn time. So a policy that
                        // REQUIRES net has to be refused here, separately, or the requirement would
                        // be checked and then quietly broken a few lines later.
                        if required.iter().any(|r| r == "net") && !allow_degraded {
                            println!();
                            println!(
                                "  {} REFUSED -- the policy requires network isolation and unshare \
                                 could not provide it: {}",
                                "✗".bright_red(),
                                e
                            );
                            println!("  run with --allow-degraded to run without it");
                            std::process::exit(3);
                        }
                        degraded.push(format!(
                            "net: unshare failed, ran with NO isolation at all: {}",
                            e
                        ));
                        println!(
                            "  {} Failed to run with network isolation: {}",
                            "✗".bright_red(),
                            e
                        );
                        println!(
                            "  {} Running WITHOUT isolation -- this is recorded in the report",
                            "⚠️".yellow()
                        );
                        let mut fallback = Command::new(&cmd[0]);
                        if cmd.len() > 1 {
                            fallback.args(&cmd[1..]);
                        }
                        apply_env(&mut fallback);
                        join_cgroup(&mut fallback);
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
                apply_env(&mut proc);
                join_cgroup(&mut proc);
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
            // The degradations travel WITH the session, so the report and anything that reads
            // the session later see what actually happened rather than what was intended.
            // ⭐ A BARE 137 TELLS A USER NOTHING. memory.events records whether the kernel
            // OOM-killed anything in this cgroup, so the report can name the cap as the reason
            // instead of leaving a mysterious signal death. Read BEFORE the cgroup drops.
            if let Some(ref cg) = cgroup {
                if cg.oom_killed() {
                    degraded.push(format!(
                        "memory: the {}MB cap KILLED the command (kernel OOM in this sandbox's \
                         cgroup) -- this is the limit working, not a crash",
                        active_policy.as_ref().map(|p| p.max_memory_mb).unwrap_or(0)
                    ));
                }
            }
            session.degraded = degraded;
            // WHAT HAPPENED, not what was asked for. If the fallback ran, this is false even
            // though the flag or the policy asked for isolation.
            session.network_isolated =
                network_isolated && !session.degraded.iter().any(|d| d.starts_with("net:"));
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
                        Some(bf) => {
                            if bf.hash != af.hash {
                                n += 1;
                            }
                        }
                    }
                }
                for p in session.before.keys() {
                    if !session.after.contains_key(p) {
                        n += 1;
                    }
                }
                n
            };
            let policy_name = active_policy.as_ref().map(|p| p.name.as_str());
            emit_to_ledger_with_policy(&session, duration_secs, files_changed, policy_name);

            // Phase 4 — Resource profiling output
            if profile {
                let end_mem = get_memory_kb();
                let profile_elapsed = profile_start.elapsed();
                println!();
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
                println!("{}", "📊 Resource Profile".bright_cyan().bold());
                println!("   Duration:  {:.3}s", profile_elapsed.as_secs_f64());
                println!(
                    "   Memory Δ:  {} KB",
                    (end_mem as i64 - start_mem as i64)
                        .to_string()
                        .bright_yellow()
                );
                let end_io = get_io_bytes();
                let read_mb = (end_io.0.saturating_sub(start_io.0)) / 1024 / 1024;
                let write_mb = (end_io.1.saturating_sub(start_io.1)) / 1024 / 1024;
                println!(
                    "   Disk read: {} MB  write: {} MB",
                    read_mb.to_string().bright_yellow(),
                    write_mb.to_string().bright_yellow()
                );
                println!("   Exit code: {}", exit_code);
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            }

            // Print diff
            println!();
            // false: the post-run report stays a SUMMARY. Twenty diffs after a devbox test
            // would bury the result. Hunks are a deliberate second look: devbox diff --patch.
            print_diff(&session, false);
            println!(
                "\n  {} Session saved — run 'faelight-sandbox diff' to review again",
                "💾".dimmed()
            );
        }

        Commands::Diff { patch } => {
            if !session_path().exists() {
                println!(
                    "  {} No sandbox session found — run: faelight-sandbox run <cmd>",
                    "⚠️".yellow()
                );
                return Ok(());
            }
            let data = fs::read_to_string(session_path())?;
            let session: SandboxSession = serde_json::from_str(&data)?;
            print_diff(&session, patch);
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
                if session.network_isolated {
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
        Commands::PolicyList => {
            let policies = policy::SandboxPolicy::list_all()?;
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("{}", "🛡️  Sandbox Policies".bold());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            if policies.is_empty() {
                println!("  No policies found");
            } else {
                for p in &policies {
                    println!(
                        "  {} {:<16} {}",
                        "▶".dimmed(),
                        p.name.bright_cyan(),
                        p.description.dimmed()
                    );
                }
            }
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("  Use with: faelight-sandbox run --policy <name> -- <cmd>");
        }
        Commands::PolicyShow { name } => {
            let p = policy::SandboxPolicy::load(&name)?;
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("  Policy: {}", p.name.bright_cyan().bold());
            println!("  {}", p.description.dimmed());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!(
                "  Network:    {}",
                if p.allow_net {
                    "allowed".bright_green()
                } else {
                    "isolated".bright_red()
                }
            );
            println!(
                "  FS writes:  {}",
                if p.allow_fs_write {
                    "allowed".bright_green()
                } else {
                    "blocked".bright_red()
                }
            );
            println!(
                "  CPU limit:  {}s (declared; enforcement in Phase 3)",
                p.max_cpu_seconds
            );
            println!(
                "  Memory:     {}MB (declared; not enforced)",
                p.max_memory_mb
            );
            // A policy whose display hides the two fields that define it is a policy nobody can
            // review. devbox is ENTIRELY allow_env + set_env; without these lines policy-show
            // rendered it as identical to default.
            if p.allow_env.is_empty() {
                println!("  Env in:     none (cleared)");
            } else {
                println!("  Env in:     {}", p.allow_env.join(", "));
            }
            // Displayed for the same reason allow_env and set_env are: a policy field the
            // policy viewer hides is a field nobody reviews.
            if !p.require.is_empty() {
                // The field that defines a policy like  -- omitting it from the
                // viewer would make it look identical to one that promises nothing.
                println!("  Requires:   {}", p.require.join(", "));
            }
            match &p.set_cwd {
                Some(d) => println!("  Cwd:        {}", d),
                None => println!("  Cwd:        inherited"),
            }
            if !p.set_env.is_empty() {
                let mut keys: Vec<&String> = p.set_env.keys().collect();
                keys.sort();
                println!("  Env set:");
                for k in keys {
                    println!("    {} = {}", k, p.set_env[k]);
                }
            }
            println!(
                "  Events:     {}",
                if p.emit_events {
                    "yes".green()
                } else {
                    "no".dimmed()
                }
            );
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
        }
        Commands::Audit { tool, limit } => {
            let db_path = faelight_core::paths::state_db();
            if !db_path.exists() {
                println!("  {} state.db not found — no audit data yet", "○".dimmed());
                return Ok(());
            }
            let conn = rusqlite::Connection::open(&db_path)?;
            let query = if let Some(tool_name) = &tool {
                format!(
                    "SELECT payload, timestamp FROM events WHERE domain='sandbox' AND action='run' AND payload LIKE '%{}%' ORDER BY timestamp DESC LIMIT {}",
                    tool_name, limit
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
            println!(
                "{}",
                "  ╭─ 🧪 Sandbox Audit Trail ──────────────────────────────".bright_cyan()
            );
            if rows.is_empty() {
                println!("  │  {} No sandbox runs recorded yet", "○".dimmed());
                println!(
                    "  │  Use {} to run commands",
                    "faelight-sandbox run".bright_cyan()
                );
            } else {
                for (payload, _ts) in &rows {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) {
                        let cmd = v["detail"]["command"].as_str().unwrap_or("unknown");
                        let exit = v["detail"]["exit_code"].as_i64().unwrap_or(-1);
                        let dur = v["detail"]["duration_secs"].as_u64().unwrap_or(0);
                        let changed = v["detail"]["files_changed"].as_u64().unwrap_or(0);
                        let result = v["result"].as_str().unwrap_or("?");
                        let result_icon = if result == "ok" {
                            "✅".to_string()
                        } else {
                            "❌".to_string()
                        };
                        let short_cmd = if cmd.len() > 40 {
                            format!("{}...", &cmd[..40])
                        } else {
                            cmd.to_string()
                        };
                        println!(
                            "  │  {} {}  {}s  {} files  exit:{}",
                            result_icon,
                            short_cmd.bright_white(),
                            dur.to_string().dimmed(),
                            changed.to_string().cyan(),
                            exit.to_string().dimmed(),
                        );
                    }
                }
            }
            println!(
                "{}",
                "  ╰─────────────────────────────────────────────────────".dimmed()
            );
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
