//! planning domain — break accepted goals into concrete steps (Core v9 Phase 2)
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS forest_plans (
    id          TEXT PRIMARY KEY,
    goal_id     TEXT NOT NULL,
    steps       TEXT NOT NULL,
    sessions    INTEGER NOT NULL DEFAULT 1,
    risk        TEXT NOT NULL DEFAULT 'LOW',
    reversible  INTEGER NOT NULL DEFAULT 1,
    status      TEXT NOT NULL DEFAULT 'draft',
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);";

fn ensure_schema(ctx: &AppContext) {
    let _ = ctx.runtime.db.execute_batch(SCHEMA);
}

fn next_id(ctx: &AppContext) -> String {
    let count: i64 = ctx.runtime.db
        .query_row("SELECT COUNT(*) FROM forest_plans", [], |r| r.get(0))
        .unwrap_or(0);
    format!("PLAN-{:03}", count + 1)
}

fn generate_steps(title: &str, plan_hint: &str) -> Vec<String> {
    let lower = title.to_lowercase();
    if lower.contains("retire") || lower.contains("redundant") {
        vec![
            "Run: core evolution tools — identify all dormant/redundant candidates".into(),
            "Review each candidate: confirm shell or core replacement exists".into(),
            "Remove binary from PATH and rust-tools/ directory".into(),
            "Update aliases.zsh — remove related aliases".into(),
            "Run: d — verify 100% health after each removal".into(),
            "Commit with intent reference".into(),
        ]
    } else if lower.contains("health") || lower.contains("restore") {
        vec![
            "Run: d — identify all failing checks".into(),
            "Triage: separate fixable from upstream-pending".into(),
            "Fix each failing check in order of impact".into(),
            "Run: d — verify improvement after each fix".into(),
            "Run: core forecast — confirm trend stable".into(),
        ]
    } else if lower.contains("shell") || lower.contains("faelight-shell") {
        vec![
            "Review INT-120 — identify next incomplete phase".into(),
            "Design: sketch the phase interface before coding".into(),
            plan_hint.to_string(),
            "Build and test in faelight-shell".into(),
            "Update INT-120 gate check".into(),
            "Run: d — verify health after change".into(),
            "Commit with intent reference".into(),
        ]
    } else {
        vec![
            plan_hint.to_string(),
            "Review current state: run relevant core commands".into(),
            "Design approach before coding".into(),
            "Implement in stages — test after each".into(),
            "Run: d — verify health".into(),
            "Commit with intent reference".into(),
        ]
    }
}

fn risk_colored(risk: &str) -> colored::ColoredString {
    match risk {
        "HIGH"   => "HIGH".bright_red(),
        "MEDIUM" => "MEDIUM".yellow(),
        _        => "LOW".bright_green(),
    }
}

fn status_colored(status: &str) -> colored::ColoredString {
    match status {
        "approved" => "[OK]".bright_green(),
        "complete" => "[✓] ".bright_cyan(),
        _          => "[..]".dimmed(),
    }
}

pub fn generate(ctx: &AppContext, goal_id: &str) -> CoreResult<()> {
    ensure_schema(ctx);

    let goal = ctx.runtime.db.query_row(
        "SELECT id, title, reason, plan, priority FROM forest_goals WHERE id=?1",
        params![goal_id],
        |r| Ok((
            r.get::<_,String>(0)?,
            r.get::<_,String>(1)?,
            r.get::<_,String>(2)?,
            r.get::<_,String>(3)?,
            r.get::<_,String>(4)?,
        )),
    );

    let (id, title, reason, plan_hint, priority) = match goal {
        Ok(g) => g,
        Err(_) => {
            println!("  Goal not found: {}", goal_id);
            println!("  Run: core goals list");
            return Ok(());
        }
    };

    // One plan per goal — guard against duplicates
    let existing: i64 = ctx.runtime.db
        .query_row(
            "SELECT COUNT(*) FROM forest_plans WHERE goal_id=?1",
            params![goal_id], |r| r.get(0),
        )
        .unwrap_or(0);

    if existing > 0 {
        let plan_id: String = ctx.runtime.db
            .query_row(
                "SELECT id FROM forest_plans WHERE goal_id=?1 ORDER BY created_at DESC LIMIT 1",
                params![goal_id], |r| r.get(0),
            )
            .unwrap_or_default();
        println!();
        println!("  Plan already exists for {}: {}", goal_id, plan_id.bright_cyan());
        println!("  Run: core plan review {}  to view it", plan_id);
        return Ok(());
    }

    let steps = generate_steps(&title, &plan_hint);
    let sessions: i64 = match priority.as_str() { "HIGH" => 3, "MEDIUM" => 2, _ => 1 };
    let risk     = match priority.as_str() { "HIGH" => "MEDIUM", _ => "LOW" };

    let steps_json = serde_json::to_string(&steps)
        .unwrap_or_else(|_| "[]".to_string());
    let plan_id = next_id(ctx);
    let now = chrono::Utc::now().timestamp();

    ctx.runtime.db.execute(
        "INSERT INTO forest_plans \
         (id,goal_id,steps,sessions,risk,reversible,status,created_at,updated_at) \
         VALUES (?1,?2,?3,?4,?5,1,'draft',?6,?6)",
        params![plan_id, id, steps_json, sessions, risk, now],
    ).map_err(|e| crate::errors::CoreError::Runtime(e.to_string()))?;

    println!();
    println!("  {} {}", "Plan generated:".bright_white().bold(), plan_id.bright_cyan());
    println!("  Goal:     {} — {}", goal_id.yellow(), title);
    println!("  Reason:   {}", reason.dimmed());
    println!("  Risk:     {}  Reversible: {}  Est. sessions: {}",
        risk_colored(risk), "YES".bright_green(), sessions);
    println!();
    println!("  {}", "Steps (generated — edit to refine):".bright_white().bold());
    for (i, step) in steps.iter().enumerate() {
        println!("    {}. {}", (i + 1).to_string().dimmed(), step);
    }
    println!();
    println!("  Status: {}", "draft".dimmed());
    println!("  {}  core plan review {}    — view and refine", "→".dimmed(), plan_id);
    println!("  {}  core plan simulate {}  — risk analysis",   "→".dimmed(), plan_id);

    let _ = ctx.runtime.db.execute(
        "INSERT INTO events (domain,action,payload,timestamp) VALUES ('planning','generated',?1,?2)",
        params![format!("plan:{} goal:{}", plan_id, goal_id), now],
    );

    Ok(())
}

pub fn review(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ensure_schema(ctx);

    let row = ctx.runtime.db.query_row(
        "SELECT p.id, p.goal_id, g.title, p.steps, p.sessions, p.risk, \
                p.reversible, p.status, p.created_at
         FROM forest_plans p
         LEFT JOIN forest_goals g ON p.goal_id = g.id
         WHERE p.id=?1",
        params![id],
        |r| Ok((
            r.get::<_,String>(0)?,
            r.get::<_,String>(1)?,
            r.get::<_,String>(2).unwrap_or_else(|_| "Unknown goal".into()),
            r.get::<_,String>(3)?,
            r.get::<_,i64>(4)?,
            r.get::<_,String>(5)?,
            r.get::<_,i64>(6)?,
            r.get::<_,String>(7)?,
            r.get::<_,i64>(8)?,
        )),
    );

    let (plan_id, goal_id, goal_title, steps_json,
         sessions, risk, reversible, status, created) = match row {
        Ok(r) => r,
        Err(_) => {
            println!("  Plan not found: {}", id);
            println!("  Run: core plan list");
            return Ok(());
        }
    };

    let steps: Vec<String> = serde_json::from_str(&steps_json).unwrap_or_default();
    let date = chrono::DateTime::from_timestamp(created, 0)
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    println!();
    println!("  {} {}", "Plan:".bright_white().bold(), plan_id.bright_cyan());
    println!("{}", "━".repeat(52).dimmed());
    println!("  Goal:      {} — {}", goal_id.yellow(), goal_title);
    println!("  Risk:      {}  Reversible: {}  Sessions: {}",
        risk_colored(&risk),
        if reversible == 1 { "YES".bright_green() } else { "NO".bright_red() },
        sessions.to_string().yellow());
    println!("  Status:    {}  Created: {}", status_colored(&status), date.dimmed());
    println!();
    println!("  {}", "Steps:".bright_white().bold());
    for (i, step) in steps.iter().enumerate() {
        println!("    {}. {}", (i + 1).to_string().bright_cyan(), step);
    }
    println!();
    println!("  {}  core plan simulate {}  — risk analysis", "→".dimmed(), plan_id);
    println!("{}", "━".repeat(52).dimmed());
    println!();

    Ok(())
}

pub fn list(ctx: &AppContext) -> CoreResult<()> {
    ensure_schema(ctx);

    let mut stmt = match ctx.runtime.db.prepare(
        "SELECT p.id, p.goal_id, g.title, p.risk, p.status, p.sessions
         FROM forest_plans p
         LEFT JOIN forest_goals g ON p.goal_id = g.id
         ORDER BY p.created_at DESC",
    ) {
        Ok(s) => s,
        Err(_) => {
            println!("  No plans yet — run: core plan generate <goal_id>");
            return Ok(());
        }
    };

    let plans: Vec<(String,String,String,String,String,i64)> = stmt
        .query_map([], |r| Ok((
            r.get(0)?, r.get(1)?,
            r.get::<_,String>(2).unwrap_or_else(|_| "?".into()),
            r.get(3)?, r.get(4)?, r.get(5)?,
        )))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    println!();
    if plans.is_empty() {
        println!("  No plans yet — run: core plan generate <goal_id>");
    } else {
        for (id, goal_id, title, risk, status, sessions) in &plans {
            println!("  {} {}  {} — {}",
                status_colored(status), id.bright_cyan(), goal_id.yellow(), title);
            println!("     Risk: {}  Sessions: {}", risk_colored(risk), sessions);
        }
    }
    println!();
    Ok(())
}

pub fn simulate_plan(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ensure_schema(ctx);

    let row = ctx.runtime.db.query_row(
        "SELECT p.goal_id, g.title
         FROM forest_plans p
         LEFT JOIN forest_goals g ON p.goal_id = g.id
         WHERE p.id=?1",
        params![id],
        |r| Ok((
            r.get::<_,String>(0)?,
            r.get::<_,String>(1).unwrap_or_else(|_| "Unknown goal".into()),
        )),
    );

    match row {
        Ok((goal_id, title)) => {
            println!();
            println!("  Simulating plan {} — goal {} — {}",
                id.bright_cyan(), goal_id.yellow(), title.bright_white());
            println!();
            crate::domains::simulate::scenario(ctx, &title)
        }
        Err(_) => {
            println!("  Plan not found: {}", id);
            println!("  Run: core plan list");
            Ok(())
        }
    }
}
