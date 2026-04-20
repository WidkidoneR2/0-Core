//! INT-219 -- Core v20: Friday Phase 2: Deep Pattern Synthesis and Predictive Strategy
//! Phase 2 builds temporal models, multi-step plans, and persistent Friday state.
//! Friday thinks ahead -- not just what is happening, but what will happen.
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
static INIT_TABLES: &str = "
CREATE TABLE IF NOT EXISTS friday_temporal_models (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    name                TEXT NOT NULL UNIQUE,
    time_horizon_hours  INTEGER NOT NULL DEFAULT 24,
    pattern_signature   TEXT NOT NULL,
    prediction          TEXT NOT NULL,
    confidence          REAL NOT NULL DEFAULT 0.5,
    historical_accuracy REAL NOT NULL DEFAULT 0.0,
    validated_count     INTEGER NOT NULL DEFAULT 0,
    correct_count       INTEGER NOT NULL DEFAULT 0,
    created_at          INTEGER NOT NULL,
    last_validated      INTEGER
);
CREATE TABLE IF NOT EXISTS friday_state (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS friday_plan_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_text   TEXT NOT NULL,
    confidence  REAL NOT NULL DEFAULT 0.5,
    created_at  INTEGER NOT NULL,
    approved    INTEGER NOT NULL DEFAULT 0
);
";
pub fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(INIT_TABLES)?;
    Ok(())
}
#[allow(dead_code)]
pub struct TemporalModel {
    pub id: i64,
    pub name: String,
    pub time_horizon_hours: i64,
    pub pattern_signature: String,
    pub prediction: String,
    pub confidence: f64,
    pub historical_accuracy: f64,
    pub validated_count: i64,
    pub correct_count: i64,
}
pub fn seed_temporal_models(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    let existing: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_temporal_models", [], |r| r.get(0)
    ).unwrap_or(0);
    if existing > 0 { return Ok(()); }
    let models: &[(&str, i64, &str, &str, f64)] = &[
        ("open-intents-health-degradation",     168,
         "active_intent_count >= 4 AND days_open >= 3",
         "Health drops below 95% within 7 days when 4+ intents stay open for 3+ days",
         0.72),
        ("commit-velocity-recovery",            48,
         "commit_velocity > 20 AND streak_days >= 5",
         "Commit velocity drops for 1-2 days after sustained high-velocity periods",
         0.68),
        ("deploy-after-cicomplete",             1,
         "cicomplete_detected",
         "Deploy follows cicomplete within 5 minutes",
         0.95),
        ("intelligence-arc-stability",          720,
         "intent_domain = intelligence AND version >= 15",
         "Intelligence-arc intents (v15+) complete without regressions",
         0.95),
        ("session-depth-commits",               24,
         "session_commands > 50",
         "Sessions with 50+ commands produce at least one commit",
         0.85),
    ];
    let mut seeded = 0;
    for (name, horizon, sig, pred, conf) in models {
        let r = db.execute(
            "INSERT OR IGNORE INTO friday_temporal_models
             (name, time_horizon_hours, pattern_signature, prediction,
              confidence, historical_accuracy, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)",
            params![name, horizon, sig, pred, conf, now],
        );
        if r.is_ok() { seeded += 1; }
    }
    println!("  {} {} temporal models seeded", "✅".green(), seeded.to_string().bright_white());
    Ok(())
}
pub fn show_temporal_models(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    seed_temporal_models(ctx)?;
    let db = &ctx.runtime.db;
    let models: Vec<TemporalModel> = {
        let mut s = db.prepare(
            "SELECT id, name, time_horizon_hours, pattern_signature, prediction,
                    confidence, historical_accuracy, validated_count, correct_count
             FROM friday_temporal_models ORDER BY confidence DESC"
        )?;
        let x = s.query_map([], |r| Ok(TemporalModel {
            id:                  r.get(0)?,
            name:                r.get(1)?,
            time_horizon_hours:  r.get(2)?,
            pattern_signature:   r.get(3)?,
            prediction:          r.get(4)?,
            confidence:          r.get(5)?,
            historical_accuracy: r.get(6)?,
            validated_count:     r.get(7)?,
            correct_count:       r.get(8)?,
        }))?.filter_map(|r| r.ok()).collect(); x
    };
    println!();
    println!("  {} Friday Phase 2 -- Temporal Models", "🌲".normal());
    println!("  {}", "━".repeat(55).dimmed());
    println!();
    for m in &models {
        let horizon = if m.time_horizon_hours >= 24 {
            format!("{}d", m.time_horizon_hours / 24)
        } else {
            format!("{}h", m.time_horizon_hours)
        };
        let accuracy = if m.validated_count > 0 {
            format!("{:.0}% accuracy ({}/{})",
                m.historical_accuracy * 100.0, m.correct_count, m.validated_count)
        } else {
            "not yet validated".to_string()
        };
        println!("  {} {} [{}]", "→".bright_cyan(),
            m.name.bright_white().bold(), horizon.dimmed());
        println!("    {} {}", "predict:".dimmed(), m.prediction.white());
        println!("    {} {:.0}% confidence  ·  {}",
            "·".dimmed(), m.confidence * 100.0, accuracy.dimmed());
        println!();
    }
    let state_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_state", [], |r| r.get(0)
    ).unwrap_or(0);
    println!("  {} {} state keys persisted across sessions", "💡".dimmed(), state_count);
    println!();
    Ok(())
}
pub fn plan(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    seed_temporal_models(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    let root = std::path::PathBuf::from(&ctx.core_root);
    println!();
    println!("  {} Friday Phase 2 -- Strategic Plan", "🌲".normal());
    println!("  {}", "━".repeat(55).dimmed());
    println!();
    // Read planned + in-progress intents from filesystem
    let mut open_intents: Vec<(String, String, String)> = Vec::new(); // (id, title, status)
    for dir in &["future", "active"] {
        let path = root.join("intents").join(dir);
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Ok(c) = std::fs::read_to_string(&p) {
                        let is_open = c.contains("status: planned")
                            || c.contains("status: in-progress");
                        if is_open {
                            let fname = p.file_stem()
                                .unwrap_or_default().to_string_lossy().to_string();
                            let id = fname.split('-').next().unwrap_or("").to_string();
                            let title = c.lines()
                                .find(|l| l.starts_with("title:"))
                                .map(|l| l[6..].trim().trim_matches('"').to_string())
                                .unwrap_or_else(|| fname.clone());
                            let status = if c.contains("status: in-progress") {
                                "in-progress".to_string()
                            } else {
                                "planned".to_string()
                            };
                            open_intents.push((id, title, status));
                        }
                    }
                }
            }
        }
    }
    open_intents.sort_by(|a, b| a.0.cmp(&b.0));
    // Current health
    let health: i64 = db.query_row(
        "SELECT COALESCE(value, '100') FROM domain_state WHERE key = 'last_health' LIMIT 1",
        [], |r| r.get::<_, String>(0)
    ).ok().and_then(|s| s.parse().ok()).unwrap_or(100);
    // Commit velocity (last 7 days)
    let seven_days_ago = now - 604800;
    let velocity: i64 = db.query_row(
        "SELECT COUNT(*) FROM commit_patterns WHERE timestamp > ?1",
        params![seven_days_ago], |r| r.get(0)
    ).unwrap_or(0);
    // Top temporal model warning
    let top_model: Option<(String, String, f64)> = db.query_row(
        "SELECT name, prediction, confidence FROM friday_temporal_models
         WHERE confidence >= 0.70 ORDER BY confidence DESC LIMIT 1",
        [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
    ).ok();
    // Top pattern
    let top_pattern: Option<(String, String, f64)> = db.query_row(
        "SELECT trigger, action, confidence FROM friday_patterns
         WHERE confidence >= 0.75 ORDER BY confidence DESC LIMIT 1",
        [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
    ).ok();
    // Plan confidence from forest state
    let open_count = open_intents.len();
    let plan_conf: f64 = if health == 100 && open_count <= 3 { 0.84 }
                         else if health >= 95 && open_count <= 5 { 0.74 }
                         else { 0.58 };
    // Print state snapshot
    println!("  {} Forest state:", "→".bright_cyan());
    println!("    {} Health: {}%  ·  Open intents: {}  ·  Velocity: {}/7d",
        "·".dimmed(),
        health.to_string().bright_green(),
        open_count.to_string().bright_white(),
        velocity.to_string().bright_white()
    );
    if let Some((ref trig, ref act, conf)) = top_pattern {
        println!("    {} Strongest pattern: {} → {} ({:.0}%)",
            "·".dimmed(), trig.bright_yellow(), act.bright_white(), conf * 100.0);
    }
    println!();
    // Temporal model warning if relevant
    if let Some((ref name, ref pred, conf)) = top_model {
        println!("  {} Temporal model: {} ({:.0}%)", "⚠️ ".yellow(), name.bright_yellow(), conf * 100.0);
        println!("    {} {}", "→".dimmed(), pred.white());
        println!();
    }
    // Multi-step plan
    println!("  {} Proposed path ({:.0}% confidence):", "🌲".normal(), plan_conf * 100.0);
    println!("  {}", "─".repeat(52).dimmed());
    println!();
    let priority = ["203", "219", "234", "235", "239", "232", "147", "213"];
    let mut step = 1usize;
    for pid in &priority {
        if let Some((id, title, status)) = open_intents.iter().find(|(i, _, _)| i == pid) {
            let icon = if status == "in-progress" { "●".bright_cyan() } else { "○".dimmed() };
            let est = match id.as_str() {
                "203" => ("this session", "low -- one behavioral gate remains"),
                "219" => ("this session", "low -- foundation gates in progress"),
                "234" => ("2-3 sessions", "medium -- builds directly on v20"),
                "235" => ("1-2 sessions", "low -- daemon extension pattern"),
                "239" => ("2-3 sessions", "low -- UI layer, no core changes"),
                "232" => ("3-4 sessions", "medium -- terminal rebuild"),
                "147" => ("2-3 sessions", "medium -- Piper TTS integration"),
                "213" => ("2-3 sessions", "medium -- 2FA system"),
                _     => ("unknown", "unknown"),
            };
            println!("  {} Step {}: INT-{} [{}]", icon, step, id, status.dimmed());
            println!("    {} {}", "→".dimmed(), title.white());
            println!("    {} Est: {}  ·  Risk: {}", "·".dimmed(), est.0.dimmed(), est.1.dimmed());
            println!();
            step += 1;
            if step > 6 { break; }
        }
    }
    println!("  {}", "─".repeat(52).dimmed());
    if plan_conf < 0.70 {
        println!("  {} Confidence {:.0}% -- directional, not prescriptive. Validate with d first.",
            "⚠️ ".yellow(), plan_conf * 100.0);
    } else {
        println!("  {} Confidence {:.0}% -- Friday has enough signal to recommend this path.",
            "💡".dimmed(), plan_conf * 100.0);
    }
    println!();
    // Persist plan
    let summary = format!(
        "Plan: {} open intents, health {}%, {}/7d commits, conf {:.0}%",
        open_count, health, velocity, plan_conf * 100.0
    );
    let _ = db.execute(
        "INSERT INTO friday_plan_history (plan_text, confidence, created_at) VALUES (?1, ?2, ?3)",
        params![summary, plan_conf, now],
    );
    let _ = db.execute(
        "INSERT OR REPLACE INTO friday_state (key, value, updated_at) VALUES ('last_plan_ts', ?1, ?2)",
        params![now.to_string(), now],
    );
    let _ = db.execute(
        "INSERT OR REPLACE INTO friday_state (key, value, updated_at) VALUES ('last_plan_confidence', ?1, ?2)",
        params![plan_conf.to_string(), now],
    );
    println!("  {} Plan recorded to friday_plan_history.", "🌲".normal());
    println!();
    Ok(())
}
pub fn init(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    println!();
    println!("  {} Friday Phase 2 -- Initializing", "🌲".normal());
    println!("  {}", "━".repeat(50).dimmed());
    println!();
    seed_temporal_models(ctx)?;
    let health: i64 = db.query_row(
        "SELECT COALESCE(value, '100') FROM domain_state WHERE key = 'last_health' LIMIT 1",
        [], |r| r.get::<_, String>(0)
    ).ok().and_then(|s| s.parse().ok()).unwrap_or(100);
    let patterns: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0)
    ).unwrap_or(0);
    let observations: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0)
    ).unwrap_or(0);
    let now_str      = now.to_string();
    let health_str   = health.to_string();
    let pattern_str  = patterns.to_string();
    let obs_str      = observations.to_string();
    let entries: &[(&str, &str)] = &[
        ("phase2_initialized",    "true"),
        ("phase2_init_ts",        &now_str),
        ("baseline_health",       &health_str),
        ("baseline_patterns",     &pattern_str),
        ("baseline_observations", &obs_str),
        ("phase",                 "2"),
    ];
    for (key, value) in entries {
        let _ = db.execute(
            "INSERT OR REPLACE INTO friday_state (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, value, now],
        );
    }
    println!("  {} Tables: friday_temporal_models, friday_state, friday_plan_history", "✅".green());
    println!("  {} State: health {}%, {} patterns, {} observations",
        "✅".green(), health, patterns, observations);
    println!();
    println!("  {} Phase 2 active. Friday now thinks ahead.", "🌲".normal());
    println!("  {} Next: core friday plan", "→".dimmed());
    println!();
    Ok(())
}
pub fn phase2_status(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let initialized = db.query_row(
        "SELECT value FROM friday_state WHERE key = 'phase2_initialized'",
        [], |r| r.get::<_, String>(0)
    ).ok().map(|v| v == "true").unwrap_or(false);
    if !initialized {
        println!();
        println!("  {} Phase 2 not initialized -- run: core friday phase2-init", "⚠️ ".yellow());
        println!();
        return Ok(());
    }
    let models: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_temporal_models", [], |r| r.get(0)
    ).unwrap_or(0);
    let plans: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_plan_history", [], |r| r.get(0)
    ).unwrap_or(0);
    let last_conf: f64 = db.query_row(
        "SELECT value FROM friday_state WHERE key = 'last_plan_confidence'",
        [], |r| r.get::<_, String>(0)
    ).ok().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    println!();
    println!("  {} Friday -- Phase 2", "🌲".normal());
    println!("  {}", "━".repeat(50).dimmed());
    println!();
    println!("  {:<30} {}", "Temporal models:".dimmed(), models.to_string().bright_white());
    println!("  {:<30} {}", "Plans generated:".dimmed(), plans.to_string().bright_white());
    if plans > 0 {
        println!("  {:<30} {}", "Last plan confidence:".dimmed(),
            format!("{:.0}%", last_conf * 100.0).bright_white());
    }
    println!();
    println!("  {} Capabilities:", "→".bright_cyan());
    println!("    {} Temporal models persisted across sessions", "✅".green());
    println!("    {} Multi-step strategy proposals (core friday plan)", "✅".green());
    println!("    {} Friday state persists across sessions", "✅".green());
    println!("    {} Predictive health trajectory 24-72h", "⬜".dimmed());
    println!("    {} Contradiction resolution proposals", "⬜".dimmed());
    println!("    {} Trust-modulated interrupt levels", "⬜".dimmed());
    println!();
    Ok(())
}
