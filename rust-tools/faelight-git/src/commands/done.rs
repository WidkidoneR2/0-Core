//! fg done -- stage all, commit with active intent prefix, push. No prompts.
//! INT-256: the fastest path from work to pushed commit.
use crate::git::GitRepo;
use crate::is_locked;
use anyhow::{bail, Result};
use rusqlite;
use colored::*;
fn get_active_intent() -> Option<(String, String)> {
    // Scan intents/future/ for in-progress intents -- same as fsh prompt
    let home = std::env::var("HOME").ok()?;
    let future_dir = format!("{}/0-core/intents/future", home);
    let mut entries: Vec<_> = std::fs::read_dir(&future_dir).ok()?.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".md") { continue; }
        let content = std::fs::read_to_string(entry.path()).ok()?;
        if content.contains("status: in-progress") {
            let id = name.split('-').next().unwrap_or("").to_string();
            // Extract title from frontmatter
            let title = content.lines()
                .find(|l| l.starts_with("title:"))
                .map(|l| l.trim_start_matches("title:").trim().trim_matches('"').to_string())
                .unwrap_or_else(|| format!("INT-{}", id));
            let short = title.split(" -- ").next()
                .unwrap_or(&title)
                .split(" -- ").next()
                .unwrap_or(&title)
                .trim()
                .to_string();
            return Some((id, short));
        }
    }
    None
}
pub fn run(extra: Option<&str>) -> Result<()> {
    if is_locked() {
        bail!("Core is locked. Run 'unlock-core' first.");
    }
    let repo = GitRepo::open()?;
    let status = repo.status()?;
    if status.is_empty() {
        println!("{}", "  ✅ Nothing to commit -- tree is clean".green());
        return Ok(());
    }
    println!("{}", "🌲 fg done".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    // Show staged files
    for f in &status.files {
        println!("  {} {}", f.state.symbol().yellow(), f.path.dimmed());
    }
    println!();
    // Build commit message from active intent
    let message = if let Some((id, title)) = get_active_intent() {
        if let Some(extra) = extra {
            format!("INT-{}: {} -- {}", id, title, extra)
        } else {
            format!("INT-{}: {}", id, title)
        }
    } else if let Some(extra) = extra {
        extra.to_string()
    } else {
        bail!("No active intent and no message provided. Use: fg done \"message\"");
    };
    // Stage all
    repo.stage_all()?;
    println!("  {} All changes staged", "✅".green());
    // Commit
    let hash = repo.commit(&message)?;
    println!(
        "  {} {} {}",
        "✅".green(),
        hash[..8.min(hash.len())].yellow().bold(),
        message.white()
    );
    // INT-312: write to intent_commits genealogy table
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let db_path = format!("{}/0-core/runtime/state.db", home);
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let intent_id: Option<i64> = if let Some((ref id, _)) = get_active_intent() {
                id.parse().ok()
            } else { None };
            let health: Option<i64> = conn.query_row(
                "SELECT CAST(value AS INTEGER) FROM shell_state WHERE key = 'last_health'",
                [], |r| r.get(0)).ok();
            let friday_facts: Option<i64> = conn.query_row(
                "SELECT COUNT(*) FROM friday_knowledge", [], |r| r.get(0)).ok();
            let friday_patterns: Option<i64> = conn.query_row(
                "SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0)).ok();
            let session_id: Option<String> = conn.query_row(
                "SELECT value FROM shell_state WHERE key = 'current_session'",
                [], |r| r.get(0)).ok();
            // Parse phase hint from message
            let phase_hint = message.split_whitespace()
                .zip(message.split_whitespace().skip(1))
                .find_map(|(a, b)| if a.to_lowercase() == "phase" {
                    Some(format!("Phase {}", b.trim_end_matches(':').trim_end_matches(',')))
                } else { None });
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs() as i64;
            let _ = conn.execute(
                "INSERT OR IGNORE INTO intent_commits
                 (commit_hash, intent_id, intent_status, phase_hint, health_at,
                  friday_facts, friday_patterns, session_id, committed_at, message)
                 VALUES (?1,?2,'in-progress',?3,?4,?5,?6,?7,?8,?9)",
                rusqlite::params![
                    &hash[..12.min(hash.len())],
                    intent_id, phase_hint, health,
                    friday_facts, friday_patterns, session_id, now, &message
                ],
            );
        }
    }
    // Push
    println!("  {} Pushing...", "→".cyan());
    let push = std::process::Command::new("git").arg("push").status()?;
    if push.success() {
        println!("{}", "  🚀 Pushed to origin".green().bold());
    } else {
        println!("{}", "  ❌ Push failed -- run 'git push' manually".red());
    }
    println!("{}", "━".repeat(52).dimmed());
    println!("{}", "  🌲 The forest remembers.".dimmed());
    Ok(())
}
