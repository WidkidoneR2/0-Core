// INT-151 Core v12 — Strategy Domain
// Phase 1: Horizon Engine (now/week/quarter planning synthesis)
//
// The forest plans across multiple horizons.
// v11 tells you what WILL happen.
// v12 tells you what TO DO about it.
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::time::{SystemTime, UNIX_EPOCH};

// ── DB init ───────────────────────────────────────────────────────────────────
pub fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(
        "CREATE TABLE IF NOT EXISTS forest_strategies (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            horizon      TEXT    NOT NULL,
            proposal     TEXT    NOT NULL,
            evidence     TEXT,
            priority     INTEGER NOT NULL DEFAULT 50,
            created_at   INTEGER NOT NULL,
            acted_on     INTEGER DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS horizon_snapshots (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            horizon      TEXT    NOT NULL,
            snapshot     TEXT    NOT NULL,
            created_at   INTEGER NOT NULL
        );",
    )?;
    Ok(())
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ── Shared helpers ────────────────────────────────────────────────────────────
fn get_health(_ctx: &AppContext) -> u32 {
    let home = std::env::var("HOME").unwrap_or_default();
    std::fs::read_to_string(format!("{}/.cache/faelight/health-status", home))
        .unwrap_or_else(|_| "100".into())
        .trim()
        .trim_end_matches('%')
        .parse()
        .unwrap_or(100)
}

fn get_in_progress_intents(ctx: &AppContext) -> Vec<String> {
    let root = std::path::PathBuf::from(&ctx.core_root);
    std::fs::read_dir(root.join("intents/future"))
        .map(|entries| entries.flatten()
            .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
            .filter_map(|e| {
                let content = std::fs::read_to_string(e.path()).ok()?;
                if !content.contains("status: in-progress") { return None; }
                let title = content.lines()
                    .find(|l| l.starts_with("title:"))?
                    .trim_start_matches("title:")
                    .trim()
                    .trim_matches('"')
                    .to_string();
                let id = e.file_name()
                    .to_string_lossy()
                    .split('-')
                    .next()
                    .unwrap_or("?")
                    .to_string();
                Some(format!("INT-{}: {}", id, title))
            })
            .collect())
        .unwrap_or_default()
}

fn get_recent_commits(ctx: &AppContext) -> u64 {
    std::process::Command::new("git")
        .args(["-C", &ctx.core_root, "rev-list", "--count", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn get_planned_intents(ctx: &AppContext) -> Vec<String> {
    let root = std::path::PathBuf::from(&ctx.core_root);
    std::fs::read_dir(root.join("intents/future"))
        .map(|entries| entries.flatten()
            .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
            .filter_map(|e| {
                let content = std::fs::read_to_string(e.path()).ok()?;
                if !content.contains("status: planned") { return None; }
                let title = content.lines()
                    .find(|l| l.starts_with("title:"))?
                    .trim_start_matches("title:")
                    .trim()
                    .trim_matches('"')
                    .to_string();
                let id = e.file_name()
                    .to_string_lossy()
                    .split('-')
                    .next()
                    .unwrap_or("?")
                    .to_string();
                Some(format!("INT-{}: {}", id, title))
            })
            .collect())
        .unwrap_or_default()
}

// ── Phase 1: Horizon Engine ───────────────────────────────────────────────────

/// core strategy now — what needs attention this session?
pub fn now(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let health = get_health(ctx);
    let in_progress = get_in_progress_intents(ctx);
    let commits = get_recent_commits(ctx);

    println!();
    println!("  {}", "🌲 Strategy — Now".bright_green().bold());
    println!("  {}", "What needs attention this session?".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    // Health signal
    if health < 80 {
        println!("  {} {} Health is at {}% — address before new work",
            "⚠".bright_yellow(),
            "URGENT:".bright_yellow().bold(),
            health);
        println!();
    } else if health < 95 {
        println!("  {} Health at {}% — run d to investigate warnings",
            "·".dimmed(), health);
        println!();
    }

    // Active intents
    println!("  {} {}", "▶".bright_cyan(), "Active intents in flight:".bright_white().bold());
    if in_progress.is_empty() {
        println!("    {} No intents in progress — pick one and cistart it", "·".dimmed());
    } else {
        for intent in &in_progress {
            println!("    {} {}", "·".dimmed(), intent.bright_white());
        }
    }
    println!();

    // Session focus recommendation
    println!("  {} {}", "▶".bright_cyan(), "Recommended session focus:".bright_white().bold());
    if in_progress.is_empty() {
        println!("    {} Start a new intent — run: predict next", "·".dimmed());
    } else if in_progress.len() == 1 {
        println!("    {} Complete your active intent before starting another", "·".dimmed());
        println!("    {} {}", "→".bright_green(), in_progress[0].bright_white());
    } else {
        println!("    {} {} intents in flight — consider focusing on one", "·".dimmed(), in_progress.len());
        println!("    {} Finish: {}", "→".bright_green(), in_progress[0].bright_white());
    }
    println!();

    // Quick stats
    println!("  {} {}", "▶".bright_cyan(), "Session context:".bright_white().bold());
    println!("    {} Health:  {}%", "·".dimmed(), health);
    println!("    {} Commits: {}", "·".dimmed(), commits);
    println!("    {} Active:  {}", "·".dimmed(), in_progress.len());
    println!();

    // Save snapshot
    let snapshot = format!(
        "health={} in_progress={} commits={}",
        health, in_progress.len(), commits
    );
    ctx.runtime.db.execute(
        "INSERT INTO horizon_snapshots (horizon, snapshot, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params!["now", snapshot, now_ts()],
    )?;

    Ok(())
}

/// core strategy week — what should the next 7 days focus on?
pub fn week(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let health = get_health(ctx);
    let in_progress = get_in_progress_intents(ctx);
    let planned = get_planned_intents(ctx);
    let commits = get_recent_commits(ctx);

    println!();
    println!("  {}", "🌲 Strategy — Week".bright_green().bold());
    println!("  {}", "What should the next 7 days focus on?".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    // Current momentum
    println!("  {} {}", "▶".bright_cyan(), "Current momentum:".bright_white().bold());
    println!("    {} {} intents in progress", "·".dimmed(), in_progress.len());
    println!("    {} {} intents planned", "·".dimmed(), planned.len());
    println!("    {} {} total commits", "·".dimmed(), commits);
    println!();

    // This week's priority
    println!("  {} {}", "▶".bright_cyan(), "This week — complete in order:".bright_white().bold());
    if in_progress.is_empty() && planned.is_empty() {
        println!("    {} Intent ledger is clear — define next goals", "·".dimmed());
    } else {
        // Show in-progress first
        for (i, intent) in in_progress.iter().take(3).enumerate() {
            println!("    {} [{}] {}", "→".bright_green(), i + 1, intent.bright_white());
        }
        // Then top planned
        let remaining = 3usize.saturating_sub(in_progress.len());
        for (i, intent) in planned.iter().take(remaining).enumerate() {
            println!("    {} [{}] {}", "·".dimmed(), in_progress.len() + i + 1, intent);
        }
    }
    println!();

    // Weekly health target
    println!("  {} {}", "▶".bright_cyan(), "Weekly targets:".bright_white().bold());
    println!("    {} Maintain health ≥ 95%", "·".dimmed());
    println!("    {} Complete all in-progress intents", "·".dimmed());
    if !planned.is_empty() {
        println!("    {} Start {} planned intent(s)", "·".dimmed(),
            planned.len().min(2));
    }
    println!();

    // Save snapshot
    let snapshot = format!(
        "health={} in_progress={} planned={} commits={}",
        health, in_progress.len(), planned.len(), commits
    );
    ctx.runtime.db.execute(
        "INSERT INTO horizon_snapshots (horizon, snapshot, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params!["week", snapshot, now_ts()],
    )?;

    Ok(())
}

/// core strategy quarter — what is the 90-day arc toward Jarvis?
pub fn quarter(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let root = std::path::PathBuf::from(&ctx.core_root);

    // Count complete intents — scan all categories by status: complete (mirrors doctor)
    let complete_count = {
        let intent_dir = root.join("intents");
        let categories = ["complete", "decisions", "experiments", "philosophy",
                          "future", "cancelled", "deferred", "incidents", "active"];
        let mut count = 0usize;
        for cat in &categories {
            if let Ok(entries) = std::fs::read_dir(intent_dir.join(cat)) {
                for entry in entries.flatten() {
                    if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                        if let Ok(c) = std::fs::read_to_string(entry.path()) {
                            if c.contains("status: complete") { count += 1; }
                        }
                    }
                }
            }
        }
        count
    };

    let planned = get_planned_intents(ctx);
    let commits = get_recent_commits(ctx);

    println!();
    println!("  {}", "🌲 Strategy — Quarter".bright_green().bold());
    println!("  {}", "The 90-day arc toward Jarvis.".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    // Jarvis readiness
    println!("  {} {}", "▶".bright_cyan(), "Jarvis Readiness:".bright_white().bold());
    println!("    {} Current score: {} / 100", "·".dimmed(), "65".bright_yellow());
    println!("    {} Target (v12):  {} / 100", "·".dimmed(), "85".bright_green());
    println!("    {} Destination:   {} / 100 (v13 Autonomy)", "·".dimmed(), "95");
    println!();

    // Core timeline
    println!("  {} {}", "▶".bright_cyan(), "Core intelligence timeline:".bright_white().bold());
    println!("    {} v9  Intent     {} the forest chooses where to grow",
        "·".dimmed(), "✅".bright_green());
    println!("    {} v10 Reaction   {} the forest responds without being asked",
        "·".dimmed(), "✅".bright_green());
    println!("    {} v11 Prediction {} the forest anticipates before it happens",
        "·".dimmed(), "✅".bright_green());
    println!("    {} v12 Strategy   {} the forest plans across horizons  ← NOW",
        "·".dimmed(), "🔄".bright_yellow());
    println!("    {} v13 Autonomy   {} the forest chooses its own purpose",
        "·".dimmed(), "⬜");
    println!();

    // 90-day focus areas
    println!("  {} {}", "▶".bright_cyan(), "90-day focus areas:".bright_white().bold());
    println!("    {} Complete Core v12 Strategy (INT-151)", "→".bright_green());
    println!("    {} Shell Architecture Hardening (INT-162)", "→".bright_green());
    println!("    {} faelight-shell daily driver (INT-146)", "→".bright_green());
    println!("    {} faelight-context + memory (INT-159, INT-160)", "→".bright_green());
    println!();

    // Forest stats
    println!("  {} {}", "▶".bright_cyan(), "Forest health:".bright_white().bold());
    println!("    {} {} intents complete", "·".dimmed(), complete_count);
    println!("    {} {} intents planned", "·".dimmed(), planned.len());
    println!("    {} {} total commits", "·".dimmed(), commits);
    println!();

    // Save snapshot
    let snapshot = format!(
        "complete={} planned={} commits={} jarvis=65",
        complete_count, planned.len(), commits
    );
    ctx.runtime.db.execute(
        "INSERT INTO horizon_snapshots (horizon, snapshot, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params!["quarter", snapshot, now_ts()],
    )?;

    Ok(())
}
