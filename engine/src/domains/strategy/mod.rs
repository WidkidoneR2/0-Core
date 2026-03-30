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
        );
        CREATE TABLE IF NOT EXISTS jarvis_readiness_log (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            score        INTEGER NOT NULL,
            factors      TEXT    NOT NULL,
            recorded_at  INTEGER NOT NULL
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

// ── Phase 3: Cross-Intent Coherence ──────────────────────────────────────────

#[derive(Debug)]
struct IntentMeta {
    id: String,
    title: String,
    status: String,
    tags: Vec<String>,
    priority: String,
    #[allow(dead_code)]
    version: String,
    depends_on: Vec<String>,
}

fn load_intent_meta(core_root: &str) -> Vec<IntentMeta> {
    let root = std::path::PathBuf::from(core_root);
    let mut intents = Vec::new();
    for dir in &["future", "complete"] {
        if let Ok(entries) = std::fs::read_dir(root.join("intents").join(dir)) {
            for entry in entries.flatten() {
                if !entry.path().extension().map(|e| e == "md").unwrap_or(false) { continue; }
                let Ok(content) = std::fs::read_to_string(entry.path()) else { continue };
                let id = content.lines()
                    .find(|l| l.starts_with("id:"))
                    .map(|l| l.trim_start_matches("id:").trim().to_string())
                    .unwrap_or_default();
                let title = content.lines()
                    .find(|l| l.starts_with("title:"))
                    .map(|l| l.trim_start_matches("title:").trim().trim_matches('"').to_string())
                    .unwrap_or_default();
                let status = content.lines()
                    .find(|l| l.starts_with("status:"))
                    .map(|l| l.trim_start_matches("status:").trim().to_string())
                    .unwrap_or_default();
                let priority = content.lines()
                    .find(|l| l.starts_with("priority:"))
                    .map(|l| l.trim_start_matches("priority:").trim().to_string())
                    .unwrap_or_default();
                let version = content.lines()
                    .find(|l| l.starts_with("version:"))
                    .map(|l| l.trim_start_matches("version:").trim().to_string())
                    .unwrap_or_default();
                let tags: Vec<String> = content.lines()
                    .find(|l| l.starts_with("tags:"))
                    .map(|l| l.trim_start_matches("tags:")
                        .trim()
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .split(',')
                        .map(|t| t.trim().to_string())
                        .collect())
                    .unwrap_or_default();
                let depends_on: Vec<String> = content.lines()
                    .find(|l| l.starts_with("depends_on:"))
                    .map(|l| l.trim_start_matches("depends_on:")
                        .trim()
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect())
                    .unwrap_or_default();
                if !id.is_empty() {
                    intents.push(IntentMeta { id, title, status, tags, priority, version, depends_on });
                }
            }
        }
    }
    intents
}

/// core strategy conflicts — which intents are pulling in opposite directions?
pub fn conflicts(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let intents = load_intent_meta(&ctx.core_root);
    let in_progress: Vec<&IntentMeta> = intents.iter()
        .filter(|i| i.status == "in-progress")
        .collect();

    println!();
    println!("  {}", "🌲 Strategy — Conflicts".bright_green().bold());
    println!("  {}", "Which intents are pulling in opposite directions?".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    let mut conflict_found = false;

    // Check 1: Dependency violations — in-progress intent depends on planned intent
    println!("  {} {}", "▶".bright_cyan(), "Dependency check:".bright_white().bold());
    let mut dep_issues = 0;
    for intent in &in_progress {
        for dep in &intent.depends_on {
            // Find the dependency
            let dep_status = intents.iter()
                .find(|i| i.id.to_string() == dep.as_str() ||
                    format!("{}", i.id).contains(dep.as_str()))
                .map(|i| i.status.as_str())
                .unwrap_or("unknown");
            if dep_status == "planned" {
                println!("    {} {} depends on {} which is still planned",
                    "⚠".bright_yellow(),
                    intent.id.bright_white(),
                    dep.bright_yellow());
                dep_issues += 1;
                conflict_found = true;
            }
        }
    }
    if dep_issues == 0 {
        println!("    {} No dependency violations", "✅".bright_green());
    }
    println!();

    // Check 2: Tag overlap — multiple in-progress intents touching same domain
    println!("  {} {}", "▶".bright_cyan(), "Domain overlap:".bright_white().bold());
    let mut tag_counts: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for intent in &in_progress {
        for tag in &intent.tags {
            tag_counts.entry(tag.clone()).or_default().push(intent.id.clone());
        }
    }
    let mut overlap_found = false;
    for (tag, ids) in &tag_counts {
        if ids.len() > 1 && !["shell", "fsh", "core"].contains(&tag.as_str()) {
            println!("    {} [{}] touched by: {}",
                "⚠".bright_yellow(),
                tag.bright_yellow(),
                ids.join(", ").bright_white());
            overlap_found = true;
            conflict_found = true;
        }
    }
    if !overlap_found {
        println!("    {} No domain overlap conflicts", "✅".bright_green());
    }
    println!();

    // Check 3: Too many in-progress
    println!("  {} {}", "▶".bright_cyan(), "Focus check:".bright_white().bold());
    if in_progress.len() > 3 {
        println!("    {} {} intents in flight — recommended max is 3",
            "⚠".bright_yellow(), in_progress.len());
        for i in &in_progress {
            println!("      {} {}: {}", "·".dimmed(), i.id.bright_white(), i.title);
        }
        conflict_found = true;
    } else {
        println!("    {} {} intents in flight — within focus limit",
            "✅".bright_green(), in_progress.len());
    }
    println!();

    if !conflict_found {
        println!("  {} No conflicts detected — the forest is coherent", "✅".bright_green());
    }

    Ok(())
}

/// core strategy coherence — is the current work plan internally consistent?
pub fn coherence(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let intents = load_intent_meta(&ctx.core_root);
    let in_progress: Vec<&IntentMeta> = intents.iter()
        .filter(|i| i.status == "in-progress")
        .collect();
    let planned: Vec<&IntentMeta> = intents.iter()
        .filter(|i| i.status == "planned")
        .collect();

    println!();
    println!("  {}", "🌲 Strategy — Coherence".bright_green().bold());
    println!("  {}", "Is the current work plan internally consistent?".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    let health = get_health(ctx);
    let mut score = 100i32;
    let mut issues: Vec<String> = Vec::new();

    // Factor 1: Health
    if health < 95 { score -= 10; issues.push(format!("Health at {}% (target ≥ 95%)", health)); }

    // Factor 2: Focus
    if in_progress.len() > 3 {
        score -= 20;
        issues.push(format!("{} intents in flight (recommended ≤ 3)", in_progress.len()));
    }

    // Factor 3: All in-progress have high/critical priority
    let low_priority_active: Vec<&str> = in_progress.iter()
        .filter(|i| i.priority == "low" || i.priority == "medium")
        .map(|i| i.id.as_str())
        .collect();
    if !low_priority_active.is_empty() {
        score -= 10;
        issues.push(format!("Low priority intents in flight: {}", low_priority_active.join(", ")));
    }

    // Factor 4: Planned count reasonable
    if planned.len() > 20 {
        score -= 5;
        issues.push(format!("{} planned intents — backlog growing", planned.len()));
    }

    let score_display = if score >= 85 {
        format!("{}/100", score).bright_green().to_string()
    } else if score >= 70 {
        format!("{}/100", score).bright_yellow().to_string()
    } else {
        format!("{}/100", score).bright_red().to_string()
    };

    println!("  {} Coherence score: {}", "▶".bright_cyan(), score_display);
    println!();

    if issues.is_empty() {
        println!("  {} Work plan is fully coherent", "✅".bright_green());
    } else {
        println!("  {} {} issue(s) affecting coherence:", "⚠".bright_yellow(), issues.len());
        println!();
        for issue in &issues {
            println!("    {} {}", "·".dimmed(), issue.bright_white());
        }
    }
    println!();

    // Show active intents summary
    println!("  {} {}", "▶".bright_cyan(), "Active intents:".bright_white().bold());
    for intent in &in_progress {
        let pri_color = match intent.priority.as_str() {
            "critical" => intent.priority.bright_red().to_string(),
            "high" => intent.priority.bright_yellow().to_string(),
            _ => intent.priority.dimmed().to_string(),
        };
        println!("    {} {}: {} [{}]",
            "·".dimmed(), intent.id.bright_white(), intent.title, pri_color);
    }
    println!();

    Ok(())
}

/// core strategy merge <goal1> <goal2> — can these goals be pursued together?
pub fn merge(ctx: &AppContext, goal1: &str, goal2: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;

    let get_goal = |id: &str| -> Option<(String, String, String, String)> {
        ctx.runtime.db.query_row(
            "SELECT id, title, reason, priority FROM forest_goals WHERE id = ?1",
            rusqlite::params![id],
            |r| Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            )),
        ).ok()
    };

    println!();
    println!("  {}", "🌲 Strategy — Merge".bright_green().bold());
    println!("  {}", format!("Can {} and {} be pursued together?", goal1, goal2).dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    let g1 = get_goal(goal1);
    let g2 = get_goal(goal2);

    match (g1, g2) {
        (None, _) => println!("  {} Goal {} not found — run: core goals list", "❌".bright_red(), goal1),
        (_, None) => println!("  {} Goal {} not found — run: core goals list", "❌".bright_red(), goal2),
        (Some((id1, title1, reason1, pri1)), Some((id2, title2, reason2, pri2))) => {
            println!("  {} {} — {}", "▶".bright_cyan(), id1.bright_white().bold(), title1);
            println!("  {} {} — {}", "▶".bright_cyan(), id2.bright_white().bold(), title2);
            println!();

            // Check for shared keywords in reasons
            let words1: std::collections::HashSet<&str> = reason1.split_whitespace().collect();
            let words2: std::collections::HashSet<&str> = reason2.split_whitespace().collect();
            let shared: Vec<&&str> = words1.intersection(&words2)
                .filter(|w| w.len() > 4)
                .collect();

            println!("  {} {}", "▶".bright_cyan(), "Compatibility analysis:".bright_white().bold());

            if !shared.is_empty() {
                println!("    {} Shared concepts: {}", "✅".bright_green(),
                    shared.iter().take(5).map(|w| **w).collect::<Vec<_>>().join(", "));
            }

            // Priority compatibility
            let compatible = (pri1.as_str(), pri2.as_str());
            match compatible {
                ("low", "low") | ("medium", "medium") =>
                    println!("    {} Both {} priority — can share sessions", "✅".bright_green(), pri1),
                ("high", "high") | ("critical", "critical") =>
                    println!("    {} Both {} priority — may compete for focus", "⚠".bright_yellow(), pri1),
                _ =>
                    println!("    {} Different priorities ({} vs {}) — sequence instead of parallel",
                        "⚠".bright_yellow(), pri1, pri2),
            }
            println!();

            println!("  {} {}", "▶".bright_cyan(), "Recommendation:".bright_white().bold());
            if pri1 == "high" && pri2 == "high" {
                println!("    {} Pursue sequentially — complete {} first", "→".bright_green(), id1);
            } else {
                println!("    {} These goals can be pursued in parallel", "→".bright_green());
                println!("    {} Consider creating a combined plan: core plan generate {}", "→".bright_green(), id1);
            }
        }
    }
    println!();
    Ok(())
}

// ── Phase 4: Jarvis Readiness Tracking ───────────────────────────────────────

fn compute_jarvis_score(ctx: &AppContext) -> (i32, Vec<(String, i32, String)>) {
    let mut factors: Vec<(String, i32, String)> = Vec::new(); // (name, score, note)
    let mut total = 0i32;

    // Factor 1: Core intelligence layers (max 40)
    // v9 Intent (+10), v10 Reaction (+10), v11 Prediction (+10), v12 Strategy (+10)
    factors.push(("v9 Intent Engine".to_string(), 10, "Complete — goals/planning/prioritization".to_string()));
    factors.push(("v10 Reaction Engine".to_string(), 10, "Complete — rules/signals/narrative".to_string()));
    factors.push(("v11 Prediction Engine".to_string(), 10, "Complete — 9 predict commands, 85% confidence".to_string()));
    // v12 partial — in progress
    factors.push(("v12 Strategy Engine".to_string(), 5, "In progress — horizon/sequence/coherence built".to_string()));
    total += 35;

    // Factor 2: Health stability (max 10)
    let health = get_health(ctx);
    let health_score = if health >= 95 { 10 } else if health >= 80 { 5 } else { 0 };
    factors.push(("System Health".to_string(), health_score,
        format!("{}% health (target ≥ 95%)", health)));
    total += health_score;

    // Factor 3: Intent velocity (max 10)
    let root = std::path::PathBuf::from(&ctx.core_root);
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
    let velocity_score = if complete_count >= 100 { 10 } else if complete_count >= 50 { 7 } else { 3 };
    factors.push(("Intent Velocity".to_string(), velocity_score,
        format!("{} intents complete", complete_count)));
    total += velocity_score;

    // Factor 4: Commit cadence (max 10)
    let commits = get_recent_commits(ctx);
    let commit_score = if commits >= 1000 { 10 } else if commits >= 500 { 7 } else { 3 };
    factors.push(("Commit Cadence".to_string(), commit_score,
        format!("{} total commits", commits)));
    total += commit_score;

    // Factor 5: Shell intelligence (max 10) — fsh as daily driver
    // Partial — fsh exists but not yet daily driver
    factors.push(("Shell Intelligence".to_string(), 5,
        "faelight-shell at 90% native coverage — not yet daily driver".to_string()));
    total += 5;

    // Factor 6: Prediction accuracy (max 10) — not yet measured
    factors.push(("Prediction Accuracy".to_string(), 0,
        "Not yet measured — INT-167 Feedback Loop needed".to_string()));

    // Log this score to DB
    let factors_json = factors.iter()
        .map(|(n, s, note)| format!("{}:{}/{}", n, s, note))
        .collect::<Vec<_>>()
        .join("|");
    let _ = ctx.runtime.db.execute(
        "INSERT INTO jarvis_readiness_log (score, factors, recorded_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![total, factors_json, now_ts()],
    );

    (total, factors)
}

/// core strategy jarvis — how close is the forest to Jarvis-level capability?
pub fn jarvis(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let (score, factors) = compute_jarvis_score(ctx);

    println!();
    println!("  {}", "🌲 Strategy — Jarvis Readiness".bright_green().bold());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    // Score display
    let score_bar = {
        let filled = (score as usize) / 5;
        let empty = 20usize.saturating_sub(filled);
        format!("[{}{}]", "█".repeat(filled).bright_green(), "░".repeat(empty).dimmed())
    };
    let score_color = if score >= 80 { format!("{}/100", score).bright_green().to_string() }
        else if score >= 60 { format!("{}/100", score).bright_yellow().to_string() }
        else { format!("{}/100", score).bright_red().to_string() };

    println!("  {} Jarvis Score: {} {}", "▶".bright_cyan(), score_color, score_bar);
    println!();

    // Level description
    let level = match score {
        80..=100 => "Strategic Advisor — approaching Jarvis",
        60..=79  => "Anticipatory Partner — forest sees ahead",
        40..=59  => "Reactive Assistant — forest responds",
        20..=39  => "Aware System — forest observes",
        _        => "Basic Tool — forest executes",
    };
    println!("  {} Level: {}", "·".dimmed(), level.bright_white());
    println!();

    // Factor breakdown
    println!("  {} {}", "▶".bright_cyan(), "Score breakdown:".bright_white().bold());
    for (name, pts, note) in &factors {
        let pts_str = if *pts > 0 {
            format!("+{}", pts).bright_green().to_string()
        } else {
            "+0".dimmed().to_string()
        };
        println!("    {} {} {}  {}", "·".dimmed(), pts_str, name.bright_white(), note.dimmed());
    }
    println!();

    // Milestone targets
    println!("  {} {}", "▶".bright_cyan(), "Milestones:".bright_white().bold());
    println!("    {} 65/100 — Anticipatory partner  {} (current)",
        if score >= 65 { "✅" } else { "⬜" }, "←".bright_yellow());
    println!("    {} 80/100 — Strategic advisor     (complete v12)",
        if score >= 80 { "✅" } else { "⬜" });
    println!("    {} 95/100 — Autonomous agent      (complete v13)",
        if score >= 95 { "✅" } else { "⬜" });
    println!();

    Ok(())
}

/// core strategy trust — what evidence would justify more autonomy?
pub fn trust(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let (score, _) = compute_jarvis_score(ctx);

    println!();
    println!("  {}", "🌲 Strategy — Trust".bright_green().bold());
    println!("  {}", "What evidence would justify more autonomy?".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    println!("  {} {}", "▶".bright_cyan(), "Current trust level:".bright_white().bold());
    println!("    {} Jarvis score: {}/100", "·".dimmed(), score);
    println!("    {} v13 Autonomy requires: 95/100", "·".dimmed());
    println!("    {} Gap: {} points", "·".dimmed(), (95 - score).max(0));
    println!();

    println!("  {} {}", "▶".bright_cyan(), "Evidence required for more autonomy:".bright_white().bold());

    // Trust gates
    let gates = vec![
        (score >= 80, "Complete Core v12 Strategy — all 5 phases", "core strategy gap"),
        (false, "Prediction accuracy > 75% measured over 30 days", "core predict accuracy"),
        (false, "faelight-shell as primary daily driver", "intent show 146"),
        (false, "faelight-context + memory operational", "intent show 159"),
        (false, "Zero critical health failures in 30 days", "core doctor run"),
    ];

    for (met, requirement, command) in &gates {
        let icon = if *met { "✅" } else { "⬜" };
        println!("    {} {}", icon, requirement.bright_white());
        if !met {
            println!("       {} Next step: {}", "→".bright_green(), command.dimmed());
        }
    }
    println!();

    println!("  {} v13 Autonomy is earned, not given.", "·".dimmed());
    println!("  {} The forest must demonstrate it is right more often than wrong.", "·".dimmed());
    println!();

    Ok(())
}

/// core strategy gap — what capabilities are missing for full Jarvis?
pub fn gap(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let (score, _) = compute_jarvis_score(ctx);

    println!();
    println!("  {}", "🌲 Strategy — Gap Analysis".bright_green().bold());
    println!("  {}", "What capabilities are missing for full Jarvis?".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    println!("  {} Current: {}/100 → Target: 95/100", "▶".bright_cyan(), score);
    println!();

    let gaps = vec![
        (false, "HIGH",   "Prediction Accuracy Feedback Loop",  "INT-167", "Forest cannot learn if predictions are right"),
        (false, "HIGH",   "faelight-shell Daily Driver",        "INT-146", "Shell not yet primary interface"),
        (false, "HIGH",   "Core v12 Strategy — Phases 4+5",     "INT-151", "Jarvis tracking + strategy memory incomplete"),
        (false, "MEDIUM", "faelight-context",                   "INT-159", "No deep codebase understanding"),
        (false, "MEDIUM", "faelight-memory",                    "INT-160", "No persistent project knowledge"),
        (false, "MEDIUM", "Shell Architecture Hardening",       "INT-162", "ExecContext + layer separation needed"),
        (false, "LOW",    "Core v13 Autonomy",                  "INT-156", "The destination — requires all above"),
    ];

    println!("  {} {}", "▶".bright_cyan(), "Capability gaps (ordered by impact):".bright_white().bold());
    for (done, priority, capability, intent, reason) in &gaps {
        let icon = if *done { "✅" } else { "⬜" };
        let pri_color = match *priority {
            "HIGH"   => priority.bright_red().to_string(),
            "MEDIUM" => priority.bright_yellow().to_string(),
            _        => priority.dimmed().to_string(),
        };
        println!("    {} [{}] {} ({})", icon, pri_color, capability.bright_white(), intent);
        println!("       {} {}", "·".dimmed(), reason.dimmed());
    }
    println!();

    let remaining = 95 - score;
    println!("  {} {} points needed to reach v13 Autonomy", "·".dimmed(), remaining);
    println!("  {} Estimated sessions: {}", "·".dimmed(),
        if remaining > 20 { "10-15 focused sessions" } else { "5-10 focused sessions" });
    println!();

    Ok(())
}

// ── Phase 5: Strategy Memory ──────────────────────────────────────────────────

/// core strategy history — past strategies and outcomes
pub fn history(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;

    println!();
    println!("  {}", "🌲 Strategy — History".bright_green().bold());
    println!("  {}", "Past strategies and did they help?".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    // Horizon snapshots
    let mut snapshots: Vec<(String, String, i64)> = Vec::new();
    {
        let mut stmt = ctx.runtime.db.prepare(
            "SELECT horizon, snapshot, created_at FROM horizon_snapshots ORDER BY created_at DESC LIMIT 10"
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            snapshots.push((row.get(0)?, row.get(1)?, row.get(2)?));
        }
    }

    // Jarvis score history
    let mut scores: Vec<(i32, i64)> = Vec::new();
    {
        let mut stmt = ctx.runtime.db.prepare(
            "SELECT score, recorded_at FROM jarvis_readiness_log ORDER BY recorded_at DESC LIMIT 5"
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            scores.push((row.get(0)?, row.get(1)?));
        }
    }

    // Strategy proposals
    let mut strategies: Vec<(String, String, i32, i64)> = Vec::new();
    {
        let mut stmt = ctx.runtime.db.prepare(
            "SELECT horizon, proposal, acted_on, created_at FROM forest_strategies ORDER BY created_at DESC LIMIT 10"
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            strategies.push((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?));
        }
    }

    // Jarvis score trend
    println!("  {} {}", "▶".bright_cyan(), "Jarvis readiness trend:".bright_white().bold());
    if scores.is_empty() {
        println!("    {} No score history yet — run: core strategy jarvis", "·".dimmed());
    } else {
        for (score, ts) in &scores {
            let dt = chrono::DateTime::from_timestamp(*ts, 0)
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| ts.to_string());
            let bar = "█".repeat((*score as usize) / 10);
            println!("    {} {} [{}] {}", "·".dimmed(), dt.dimmed(),
                bar.bright_green(), format!("{}/100", score).bright_white());
        }
    }
    println!();

    // Horizon snapshots
    println!("  {} {}", "▶".bright_cyan(), "Recent horizon snapshots:".bright_white().bold());
    if snapshots.is_empty() {
        println!("    {} No snapshots yet — run: core strategy now/week/quarter", "·".dimmed());
    } else {
        for (horizon, snapshot, ts) in &snapshots {
            let dt = chrono::DateTime::from_timestamp(*ts, 0)
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| ts.to_string());
            println!("    {} [{}] {} — {}", "·".dimmed(),
                horizon.bright_cyan(), dt.dimmed(), snapshot.dimmed());
        }
    }
    println!();

    // Strategy proposals
    println!("  {} {}", "▶".bright_cyan(), "Strategy proposals:".bright_white().bold());
    if strategies.is_empty() {
        println!("    {} No strategies recorded yet", "·".dimmed());
    } else {
        for (horizon, proposal, acted_on, ts) in &strategies {
            let dt = chrono::DateTime::from_timestamp(*ts, 0)
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| ts.to_string());
            let status = if *acted_on == 1 { "✅ acted".bright_green().to_string() }
                else { "⬜ pending".dimmed().to_string() };
            println!("    {} [{}] {} — {} — {}",
                "·".dimmed(), horizon.bright_cyan(),
                dt.dimmed(), proposal.bright_white(), status);
        }
    }
    println!();

    Ok(())
}

/// core strategy learn <strategy_id> <outcome> — record that a strategy worked or didn't
pub fn learn(ctx: &AppContext, strategy_id: &str, outcome: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;

    let acted = match outcome.to_lowercase().as_str() {
        "yes" | "true" | "worked" | "good" | "1" => 1,
        _ => 0,
    };

    // Try to update an existing strategy
    let updated = ctx.runtime.db.execute(
        "UPDATE forest_strategies SET acted_on = ?1 WHERE id = ?2",
        rusqlite::params![acted, strategy_id.parse::<i64>().unwrap_or(0)],
    )?;

    println!();
    println!("  {}", "🌲 Strategy — Learn".bright_green().bold());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    if updated > 0 {
        let outcome_display = if acted == 1 {
            "✅ worked".bright_green().to_string()
        } else {
            "❌ did not work".bright_red().to_string()
        };
        println!("  {} Strategy {} recorded as: {}", "✅".bright_green(),
            strategy_id.bright_white(), outcome_display);
        println!("  {} The forest remembers.", "·".dimmed());
    } else {
        // Insert as a new learned outcome
        ctx.runtime.db.execute(
            "INSERT INTO forest_strategies (horizon, proposal, priority, created_at, acted_on) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["learned", strategy_id, 50, now_ts(), acted],
        )?;
        println!("  {} Outcome recorded for: {}", "✅".bright_green(), strategy_id.bright_white());
    }
    println!();

    Ok(())
}

/// core strategy review — what worked, what didn't?
pub fn review(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;

    println!();
    println!("  {}", "🌲 Strategy — Review".bright_green().bold());
    println!("  {}", "What worked? What didn't?".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    let mut worked: Vec<(String, String, i64)> = Vec::new();
    {
        let mut stmt = ctx.runtime.db.prepare(
            "SELECT horizon, proposal, created_at FROM forest_strategies WHERE acted_on = 1 ORDER BY created_at DESC LIMIT 5"
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            worked.push((row.get(0)?, row.get(1)?, row.get(2)?));
        }
    }

    let mut did_not_work: Vec<(String, String, i64)> = Vec::new();
    {
        let mut stmt = ctx.runtime.db.prepare(
            "SELECT horizon, proposal, created_at FROM forest_strategies WHERE acted_on = 0 AND horizon != 'now' AND horizon != 'week' AND horizon != 'quarter' ORDER BY created_at DESC LIMIT 5"
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            did_not_work.push((row.get(0)?, row.get(1)?, row.get(2)?));
        }
    }

    // Score trajectory
    let scores: Vec<i32> = {
        let mut stmt = ctx.runtime.db.prepare(
            "SELECT score FROM jarvis_readiness_log ORDER BY recorded_at ASC"
        )?;
        stmt.query_map([], |r| r.get::<_, i32>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };

    println!("  {} {}", "▶".bright_cyan(), "Jarvis score trajectory:".bright_white().bold());
    if scores.len() < 2 {
        println!("    {} Not enough data yet — run core strategy jarvis over time", "·".dimmed());
    } else {
        let first = scores.first().copied().unwrap_or(0);
        let last = scores.last().copied().unwrap_or(0);
        let delta = last - first;
        let trend = if delta > 0 { format!("↑ +{}", delta).bright_green().to_string() }
            else if delta < 0 { format!("↓ {}", delta).bright_red().to_string() }
            else { "→ stable".dimmed().to_string() };
        println!("    {} {} → {} ({})", "·".dimmed(), first, last, trend);
    }
    println!();

    println!("  {} {}", "▶".bright_cyan(), "What worked:".bright_white().bold());
    if worked.is_empty() {
        println!("    {} No outcomes recorded yet — run: core strategy learn <id> yes", "·".dimmed());
    } else {
        for (horizon, proposal, _) in &worked {
            println!("    {} [{}] {}", "✅".bright_green(), horizon.bright_cyan(), proposal);
        }
    }
    println!();

    println!("  {} {}", "▶".bright_cyan(), "What didn't work:".bright_white().bold());
    if did_not_work.is_empty() {
        println!("    {} No negative outcomes recorded", "·".dimmed());
    } else {
        for (horizon, proposal, _) in &did_not_work {
            println!("    {} [{}] {}", "❌".bright_red(), horizon.bright_cyan(), proposal);
        }
    }
    println!();

    // Key insight
    println!("  {} {}", "▶".bright_cyan(), "Key insight:".bright_white().bold());
    println!("    {} The forest learns by recording outcomes.", "·".dimmed());
    println!("    {} Run: core strategy learn <id> yes/no — after acting on a strategy", "→".bright_green());
    println!();

    Ok(())
}
