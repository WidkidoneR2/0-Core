//! Intent-aware commit — stages, verifies, and commits

use crate::git::GitRepo;
use crate::is_locked;
use crate::risk::RiskScore;
use anyhow::{bail, Result};
use colored::*;
use std::io::{self, Write};


fn log_commit_pattern(hash: &str, message: &str, intent_ref: &Option<String>, pushed: bool) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/christian".to_string());
    let db_path = std::path::PathBuf::from(&home).join("0-core/runtime/state.db");
    if !db_path.exists() { return; }
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS commit_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                hash TEXT NOT NULL,
                message TEXT NOT NULL,
                intent_id TEXT,
                outcome TEXT NOT NULL,
                velocity_per_hour REAL NOT NULL DEFAULT 0.0,
                session_depth INTEGER NOT NULL DEFAULT 0
            );"
        );
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_secs() as i64;
        let one_hour_ago = ts - 3600;
        let today_start = ts - (ts % 86400);
        let velocity: f64 = conn.query_row(
            "SELECT COUNT(*) FROM commit_patterns WHERE timestamp > ?1",
            rusqlite::params![one_hour_ago],
            |r| r.get::<_, i64>(0),
        ).unwrap_or(0) as f64;
        let session_depth: i64 = conn.query_row(
            "SELECT COUNT(*) FROM commit_patterns WHERE timestamp > ?1",
            rusqlite::params![today_start],
            |r| r.get(0),
        ).unwrap_or(0);
        let outcome = if pushed { "pushed" } else { "local-only" };
        let intent_id = intent_ref.as_deref().unwrap_or("");
        let _ = conn.execute(
            "INSERT INTO commit_patterns (timestamp, hash, message, intent_id, outcome, velocity_per_hour, session_depth) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![ts, hash, message, intent_id, outcome, velocity, session_depth],
        );
    }
}
fn emit_git_event(action: &str, detail: &str) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/christian".to_string());
    let db_path = std::path::PathBuf::from(&home).join("0-core/runtime/state.db");
    if !db_path.exists() {
        return;
    }
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let payload = format!(
            r#"{{"actor":"faelight-git","result":"ok","detail":{{{}}}}}"#,
            detail
        );
        let _ = conn.execute(
            "INSERT INTO events (domain, action, payload, timestamp) VALUES ('git', ?, ?, ?)",
            rusqlite::params![action, payload, ts],
        );
    }
}

pub fn run(intent: Option<String>, no_intent: bool) -> Result<()> {
    let repo = GitRepo::open()?;

    // ── Guard: core must be unlocked ──────────────────────────
    if is_locked() {
        bail!("Core is locked. Run 'unlock-core' before committing.");
    }

    // ── Guard: must have changes ───────────────────────────────
    let status = repo.status()?;
    if status.is_empty() {
        println!("{}", "✅ Working tree clean — nothing to commit".green());
        return Ok(());
    }

    // ── Show current state ────────────────────────────────────
    println!("{}", "🌲 faelight-git commit".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    // INT-207 L1 — Show active intents context
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let intents_dir = std::path::PathBuf::from(&home).join("0-core/intents/future");
        let active: Vec<String> = std::fs::read_dir(&intents_dir)
            .map(|d| d.filter_map(|e| e.ok())
                .filter(|e| {
                    if let Ok(c) = std::fs::read_to_string(e.path()) {
                        c.contains("status: in-progress") || c.contains("type: in-progress")
                    } else { false }
                })
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let num = name.split('-').next().unwrap_or("").to_string();
                    if !num.is_empty() && num.parse::<u32>().is_ok() {
                        Some(format!("INT-{}", num))
                    } else { None }
                })
                .collect())
            .unwrap_or_default();
        if !active.is_empty() {
            println!("  {} {}", "Working on:".dimmed(), active.join(", ").bright_cyan());
        }
    }

    let staged = status.staged_files();
    let unstaged = status.unstaged_files();
    let untracked = status.untracked_files();

    // Show staged files
    if !staged.is_empty() {
        println!("{}", "  Staged".green().bold());
        for f in &staged {
            println!("  {} {}", "●".green(), f.path.green());
        }
        println!();
    }

    // Show unstaged files
    if !unstaged.is_empty() {
        println!("{}", "  Unstaged".yellow().bold());
        for f in &unstaged {
            println!("  {} {}", "○".yellow(), f.path.yellow());
        }
        println!();
    }

    // Show untracked
    if !untracked.is_empty() {
        println!("{}", "  Untracked".dimmed().bold());
        for f in &untracked {
            println!("  {} {}", "?".dimmed(), f.path.dimmed());
        }
        println!();
    }

    // ── Risk check ────────────────────────────────────────────
    let risk = RiskScore::calculate(&repo)?;
    println!("  {} {} {}/100", "risk".dimmed(), risk.emoji(), risk.total);
    println!("{}", "━".repeat(52).dimmed());
    println!();

    // ── Stage unstaged files? ─────────────────────────────────
    if staged.is_empty() && (!unstaged.is_empty() || !untracked.is_empty()) {
        print!("  Stage all changes? (y/n): ");
        io::stdout().flush()?;
        let mut ans = String::new();
        io::stdin().read_line(&mut ans)?;
        if ans.trim().to_lowercase() != "y" {
            println!("{}", "  ⚠️  Commit cancelled — nothing staged".yellow());
            return Ok(());
        }
        repo.stage_all()?;
        println!("{}", "  ✅ All changes staged".green());
        println!();
    } else if !unstaged.is_empty() {
        print!("  Also stage {} unstaged file(s)? (y/n): ", unstaged.len());
        io::stdout().flush()?;
        let mut ans = String::new();
        io::stdin().read_line(&mut ans)?;
        if ans.trim().to_lowercase() == "y" {
            repo.stage_all()?;
            println!("{}", "  ✅ All changes staged".green());
            println!();
        }
    }

    // ── v4 Risk Assessment ─────────────────────────────────
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let db_path = std::path::PathBuf::from(&home).join("0-core/runtime/state.db");
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs() as i64;
            // Velocity warning
            let recent: i64 = conn.query_row(
                "SELECT COUNT(*) FROM commit_patterns WHERE timestamp > ?1",
                rusqlite::params![ts - 3600], |r| r.get(0)
            ).unwrap_or(0);
            if recent >= 8 {
                println!("  {} high velocity: {} commits in last hour -- verify before pushing",
                    "⚠️ ".yellow(), recent.to_string().bright_yellow());
            }
            // Health warning
            let health: i64 = std::fs::read_to_string(
                std::path::Path::new(&home).join(".cache/faelight/health-status")
            ).unwrap_or_else(|_| "100".to_string())
            .trim().parse().unwrap_or(100);
            if health < 95 {
                println!("  {} health: {}% -- below peak, review before committing",
                    "⚠️ ".yellow(), health.to_string().bright_red());
            }
        }
        // Large change warning -- count staged lines
        if let Ok(output) = std::process::Command::new("git")
            .args(["diff", "--staged", "--stat"])
            .output() {
            let stat = String::from_utf8_lossy(&output.stdout);
            // Parse "X insertions, Y deletions" from last line
            if let Some(last) = stat.lines().last() {
                let nums: Vec<i64> = last.split_whitespace()
                    .filter_map(|w| w.parse().ok()).collect();
                let total_lines: i64 = nums.iter().sum();
                let file_count = nums.first().copied().unwrap_or(0);
                if total_lines >= 800 && file_count >= 10 {
                    println!("  {} large change: {} lines across {} files -- consider splitting",
                        "⚠️ ".yellow(), total_lines.to_string().bright_yellow(),
                        file_count.to_string().bright_yellow());
                }
            }
        }
    }

    // ── Intent ── v4 Auto-Detection ──────────────────────────────
    let intent_ref = if no_intent {
        println!("{}", "  ⚠️  Proceeding without intent (--no-intent)".yellow());
        None
    } else if let Some(ref i) = intent {
        println!("  {} linked to intent {}", "✅".green(), i.cyan());
        Some(i.clone())
    } else {
        let home = std::env::var("HOME").unwrap_or_default();
        let intents_dir = std::path::PathBuf::from(&home).join("0-core/intents/future");
        let active: Vec<(String, String)> = std::fs::read_dir(&intents_dir)
            .map(|d| d.filter_map(|e| e.ok())
                .filter(|e| {
                    if let Ok(c) = std::fs::read_to_string(e.path()) {
                        c.contains("status: in-progress") || c.contains("type: in-progress")
                    } else { false }
                })
                .filter_map(|e| {
                    let fname = e.file_name().to_string_lossy().to_string();
                    let num = fname.split('-').next().unwrap_or("").to_string();
                    if !num.is_empty() && num.parse::<u32>().is_ok() {
                        Some((format!("INT-{}", num), String::new()))
                    } else { None }
                })
                .collect())
            .unwrap_or_default();
        if active.len() == 1 {
            // Single active intent -- auto-attach
            let (id, title) = &active[0];
            println!("  {} auto-linked: {} {}", "✅".green(), id.bright_cyan(),
                if title.is_empty() { "".dimmed() } else { format!("({})", title).dimmed() });
            Some(id.clone())
        } else {
            // Multiple or zero -- ask
            if !active.is_empty() {
                println!("  {} {}", "💡 Active:".dimmed(),
                    active.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>().join(", ").bright_cyan());
            }
            print!("  Intent reference (INT-0XX or 'skip'): ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_string();
            if input == "skip" || input.is_empty() {
                println!("{}", "  ⚠️  Committing without intent".yellow());
                None
            } else {
                println!("  {} linked to intent {}", "✅".green(), input.cyan());
                Some(input)
            }
        }
    };

    // ── Commit message ────────────────────────────────────────
    println!();
    print!("  Commit message: ");
    io::stdout().flush()?;
    let mut message = String::new();
    io::stdin().read_line(&mut message)?;
    let message = message.trim().to_string();

    if message.is_empty() {
        bail!("Commit cancelled — empty message");
    }

    // Build full message with intent footer if provided
    let full_message = match intent_ref {
        Some(ref i) => format!("{}\n\nIntent: {}", message, i),
        None => message.clone(),
    };

    // ── Preview ───────────────────────────────────────────────
    println!();
    println!("{}", "  Preview".dimmed());
    println!("{}", "  ─".repeat(26).dimmed());
    println!("  {}", full_message.replace('\n', "\n  ").white().bold());
    println!("{}", "  ─".repeat(26).dimmed());
    println!();

    print!("  Confirm commit? (y/n): ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if confirm.trim().to_lowercase() != "y" {
        println!("{}", "  ⚠️  Commit cancelled".yellow());
        return Ok(());
    }

    // ── Commit ────────────────────────────────────────────────
    let hash = repo.commit(&full_message)?;
    emit_git_event(
        "commit",
        &format!(
            r#""hash":"{}","message":"{}""#,
            hash,
            message.replace('"', "'")
        ),
    );
    println!();
    println!(
        "  {} commit {} {}",
        "✅".green(),
        hash.yellow().bold(),
        message.white()
    );

    // ── Push? ─────────────────────────────────────────────────
    println!();
    print!("  Push to origin now? (y/n): ");
    io::stdout().flush()?;
    let mut push_ans = String::new();
    io::stdin().read_line(&mut push_ans)?;

    if push_ans.trim().to_lowercase() == "y" {
        println!("  {} Pushing...", "→".cyan());
        let push = std::process::Command::new("git").arg("push").status()?;
        if push.success() {
            emit_git_event("push", &format!(r#""hash":"{}","branch":"main""#, hash));
            log_commit_pattern(&hash, &message, &intent_ref, true);
            println!("{}", "  🚀 Pushed to origin".green().bold());
        } else {
            println!("{}", "  ❌ Push failed — run 'git push' manually".red());
        }
    } else {
        log_commit_pattern(&hash, &message, &intent_ref, false);
        println!("{}", "  ℹ️  Committed locally — push when ready".dimmed());
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    println!("{}", "  🌲 The forest remembers.".cyan().dimmed());

    Ok(())
}
