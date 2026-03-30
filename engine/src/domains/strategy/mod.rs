// INT-151 Core v12 — Strategy Domain
// Phase 1: Horizon Engine (now/week/quarter planning synthesis)
//
// The forest plans across multiple horizons.
// v11 tells you what WILL happen.
// v12 tells you what TO DO about it.
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use serde_json;
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

// ── Phase 2: Action Sequencing ────────────────────────────────────────────────

/// core strategy sequence <goal_id> — optimal path to a goal
pub fn sequence(ctx: &AppContext, goal_id: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;

    // Look up the goal
    let goal: Option<(String, String, String, String)> = ctx.runtime.db
        .query_row(
            "SELECT id, title, plan, status FROM forest_goals WHERE id = ?1",
            rusqlite::params![goal_id],
            |r| Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            )),
        )
        .ok();

    println!();
    println!("  {}", "🌲 Strategy — Sequence".bright_green().bold());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    match goal {
        None => {
            println!("  {} Goal {} not found", "❌".bright_red(), goal_id.bright_white());
            println!("  {} Run: core goals list", "·".dimmed());
        }
        Some((id, title, plan, status)) => {
            println!("  {} {} — {}", "▶".bright_cyan(),
                id.bright_white().bold(), title.bright_white());
            println!("  {} Status: {}", "·".dimmed(), status.bright_yellow());
            println!();

            // Show the plan from forest_plans if it exists
            let plan_steps: Option<(String, String, i64)> = ctx.runtime.db
                .query_row(
                    "SELECT steps, risk, sessions FROM forest_plans WHERE goal_id = ?1 ORDER BY created_at DESC LIMIT 1",
                    rusqlite::params![&id],
                    |r| Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?,
                    )),
                )
                .ok();

            if let Some((steps, risk, sessions)) = plan_steps {
                println!("  {} {}", "▶".bright_cyan(), "Execution sequence:".bright_white().bold());
                // Steps may be JSON array or newline-separated text
                let parsed: Vec<String> = if steps.trim_start().starts_with('[') {
                    serde_json::from_str(&steps).unwrap_or_else(|_| vec![steps.clone()])
                } else {
                    steps.lines()
                        .map(|l| l.trim().trim_start_matches('-').trim().to_string())
                        .filter(|l| !l.is_empty())
                        .collect()
                };
                for (i, step) in parsed.iter().enumerate() {
                    println!("    {} [{}] {}", "→".bright_green(), i + 1, step);
                }
                println!();
                println!("  {} {}", "▶".bright_cyan(), "Execution profile:".bright_white().bold());
                println!("    {} Estimated sessions: {}", "·".dimmed(), sessions);
                println!("    {} Risk level: {}", "·".dimmed(), risk.bright_yellow());
            } else {
                println!("  {} {}", "▶".bright_cyan(), "Recommended sequence:".bright_white().bold());
                // Fall back to the goal's own plan field
                for (i, step) in plan.lines().enumerate().take(8) {
                    let step = step.trim().trim_start_matches('-').trim();
                    if !step.is_empty() {
                        println!("    {} [{}] {}", "→".bright_green(), i + 1, step);
                    }
                }
                println!();
                println!("  {} Run: core plan generate {} — to build a detailed plan",
                    "💡".bright_yellow(), id);
            }

            // Check for related intents
            let root = std::path::PathBuf::from(&ctx.core_root);
            let related: Vec<String> = std::fs::read_dir(root.join("intents/future"))
                .map(|entries| entries.flatten()
                    .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
                    .filter_map(|e| {
                        let c = std::fs::read_to_string(e.path()).ok()?;
                        if !c.contains("status: in-progress") { return None; }
                        let t = c.lines()
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
                        Some(format!("INT-{}: {}", id, t))
                    })
                    .collect())
                .unwrap_or_default();

            if !related.is_empty() {
                println!();
                println!("  {} {}", "▶".bright_cyan(), "Active intents supporting this goal:".bright_white().bold());
                for intent in &related {
                    println!("    {} {}", "·".dimmed(), intent);
                }
            }
        }
    }
    println!();
    Ok(())
}

/// core strategy unblock — what is blocking the most progress?
pub fn unblock(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let health = get_health(ctx);
    let in_progress = get_in_progress_intents(ctx);
    let commits = get_recent_commits(ctx);

    println!();
    println!("  {}", "🌲 Strategy — Unblock".bright_green().bold());
    println!("  {}", "What is blocking the most progress?".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    let mut blockers: Vec<(u8, String, String)> = Vec::new(); // (priority, blocker, action)

    // Health blocker
    if health < 80 {
        blockers.push((10, format!("Health at {}% — system needs attention", health),
            "Run: d — investigate and fix warnings".to_string()));
    }

    // Too many intents in flight
    if in_progress.len() > 3 {
        blockers.push((20, format!("{} intents in flight — cognitive overload", in_progress.len()),
            format!("Focus on one: {}", in_progress.first().cloned().unwrap_or_default())));
    }

    // Check for stale in-progress intents (no recent commits)
    let _root = std::path::PathBuf::from(&ctx.core_root);
    let _recent = commits;

    // Goals without plans
    let goals_no_plan: Vec<String> = {
        let mut stmt = ctx.runtime.db.prepare(
            "SELECT id, title FROM forest_goals WHERE status = 'accepted' AND id NOT IN (SELECT goal_id FROM forest_plans)"
        )?;
        stmt.query_map([], |r| Ok(format!("{}: {}", r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };

    if !goals_no_plan.is_empty() {
        for goal in &goals_no_plan {
            blockers.push((30, format!("Goal {} has no execution plan", goal),
                format!("Run: core plan generate {}", goal.split(':').next().unwrap_or(""))));
        }
    }

    // No accepted goals
    let goal_count: i64 = ctx.runtime.db
        .query_row("SELECT COUNT(*) FROM forest_goals WHERE status = 'accepted'", [], |r| r.get(0))
        .unwrap_or(0);

    if goal_count == 0 {
        blockers.push((40, "No accepted goals — forest has no direction".to_string(),
            "Run: core goals generate — to create goals from evidence".to_string()));
    }

    if blockers.is_empty() {
        println!("  {} No blockers detected — the forest is clear to build", "✅".bright_green());
        println!();
        println!("  {} {}", "▶".bright_cyan(), "Current state:".bright_white().bold());
        println!("    {} Health:  {}%", "·".dimmed(), health);
        println!("    {} Active:  {} intents", "·".dimmed(), in_progress.len());
        println!("    {} Goals:   {} accepted", "·".dimmed(), goal_count);
    } else {
        blockers.sort_by_key(|b| b.0);
        println!("  {} {} blocker(s) detected:", "⚠".bright_yellow(), blockers.len());
        println!();
        for (i, (_, blocker, action)) in blockers.iter().enumerate() {
            println!("  {} [{}] {}", "🔴".bright_red(), i + 1, blocker.bright_white());
            println!("       {} {}", "→".bright_green(), action);
            println!();
        }
    }

    Ok(())
}

/// core strategy tradeoff <action> — what do we give up to do this now?
pub fn tradeoff(ctx: &AppContext, action: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let in_progress = get_in_progress_intents(ctx);
    let health = get_health(ctx);

    println!();
    println!("  {}", "🌲 Strategy — Tradeoff".bright_green().bold());
    println!("  {}", format!("What do we give up to do \"{}\" now?", action).dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    println!("  {} {}", "▶".bright_cyan(), "Proposed action:".bright_white().bold());
    println!("    {} {}", "→".bright_green(), action.bright_white());
    println!();

    println!("  {} {}", "▶".bright_cyan(), "What you give up:".bright_white().bold());

    // Currently in progress — these get delayed
    if in_progress.is_empty() {
        println!("    {} Nothing currently in flight — clean slate", "·".dimmed());
    } else {
        println!("    {} Delays these in-progress intents:", "·".dimmed());
        for intent in &in_progress {
            println!("      {} {}", "·".dimmed(), intent);
        }
    }
    println!();

    // Health risk
    println!("  {} {}", "▶".bright_cyan(), "Risk assessment:".bright_white().bold());
    if health < 95 {
        println!("    {} Health at {}% — adding work increases risk", "⚠".bright_yellow(), health);
    } else {
        println!("    {} Health at {}% — safe to take on new work", "✅".bright_green(), health);
    }

    if in_progress.len() > 2 {
        println!("    {} {} intents already in flight — high context-switch cost",
            "⚠".bright_yellow(), in_progress.len());
    } else {
        println!("    {} {} intents in flight — manageable", "✅".bright_green(), in_progress.len());
    }
    println!();

    // Check historical tradeoffs for similar actions
    let similar: Option<(String, String)> = ctx.runtime.db
        .query_row(
            "SELECT description, recommendation FROM forest_tradeoffs ORDER BY created_at DESC LIMIT 1",
            [],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .ok();

    if let Some((desc, rec)) = similar {
        println!("  {} {}", "▶".bright_cyan(), "Most recent tradeoff analysis:".bright_white().bold());
        println!("    {} {}", "·".dimmed(), desc);
        println!("    {} {}", "→".bright_green(), rec);
        println!();
    }

    println!("  {} Run: core tradeoff analyze — for a full tradeoff analysis", "💡".bright_yellow());
    println!();

    Ok(())
}
