//! INT-216 -- Friday Formal Architecture
//! Meta-Interpretation Engine: sees all layers simultaneously
//! Friday produces insight, not authority.
//! Every proposal requires human approval before execution.
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
CREATE TABLE IF NOT EXISTS friday_models (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    description TEXT NOT NULL,
    domain      TEXT NOT NULL DEFAULT 'cross-layer',
    confidence  REAL NOT NULL DEFAULT 0.5,
    stability   REAL NOT NULL DEFAULT 0.5,
    predictions INTEGER NOT NULL DEFAULT 0,
    correct     INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    validated_at INTEGER,
    active      INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS friday_trust (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    model_id    INTEGER NOT NULL,
    predictions INTEGER NOT NULL DEFAULT 0,
    correct     INTEGER NOT NULL DEFAULT 0,
    accuracy    REAL NOT NULL DEFAULT 0.0,
    updated_at  INTEGER NOT NULL,
    FOREIGN KEY (model_id) REFERENCES friday_models(id)
);
CREATE TABLE IF NOT EXISTS friday_proposals (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    signal_type TEXT NOT NULL,
    description TEXT NOT NULL,
    action      TEXT NOT NULL,
    confidence  REAL NOT NULL DEFAULT 0.5,
    status      TEXT NOT NULL DEFAULT 'pending',
    created_at  INTEGER NOT NULL,
    reviewed_at INTEGER,
    outcome     TEXT
);
CREATE TABLE IF NOT EXISTS friday_contradictions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    engine_a    TEXT NOT NULL,
    signal_a    TEXT NOT NULL,
    engine_b    TEXT NOT NULL,
    signal_b    TEXT NOT NULL,
    description TEXT NOT NULL,
    severity    TEXT NOT NULL DEFAULT 'low',
    resolved    INTEGER NOT NULL DEFAULT 0,
    detected_at INTEGER NOT NULL,
    resolved_at INTEGER
);
";
pub fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(INIT_TABLES)?;
    Ok(())
}
/// Seed initial Friday models from known cross-layer patterns
pub fn seed_models(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    let models = vec![
        (
            "high_velocity_health_risk",
            "High commit velocity (>15/day) correlates with health dips if intents > 2",
            "health+prediction",
            0.75f64,
        ),
        (
            "deploy_after_complete",
            "cicomplete reliably followed by deploy within same session",
            "prediction+shell",
            0.90,
        ),
        (
            "multi_intent_focus_violation",
            "More than 2 active intents contradicts focus>speed alignment value",
            "alignment+intent",
            0.85,
        ),
        (
            "late_session_risk",
            "Commands after 30+ session deploys show higher error rates",
            "shell+health",
            0.70,
        ),
        (
            "health_gate_pattern",
            "d (doctor) run before and after every intent -- the health-check-loop",
            "shell+health",
            0.95,
        ),
    ];
    let mut seeded = 0;
    for (desc_key, description, domain, confidence) in &models {
        let exists: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM friday_models WHERE description = ?1",
                params![description],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if exists == 0 {
            let model_id = db.execute(
                "INSERT INTO friday_models (description, domain, confidence, stability, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![description, domain, confidence, confidence, now],
            ).map(|_| db.last_insert_rowid())?;
            let _ = db.execute(
                "INSERT INTO friday_trust (model_id, predictions, correct, accuracy, updated_at)
                 VALUES (?1, 0, 0, ?2, ?3)",
                params![model_id, confidence, now],
            );
            let _ = db.execute(
                "INSERT INTO friday_knowledge (domain, fact, confidence, source, created_at, updated_at)
                 VALUES ('friday_model', ?1, ?2, 'friday_arch', ?3, ?3)",
                params![format!("model[{}]: {}", desc_key, description), confidence, now],
            );
            seeded += 1;
        }
    }
    println!("  {} Seeded {} Friday models", "✅".green(), seeded);
    Ok(())
}
/// Cross-layer pattern detection -- Friday sees what no single engine can
pub fn detect_patterns(ctx: &AppContext) -> CoreResult<Vec<String>> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    let mut patterns_found: Vec<String> = Vec::new();
    // Pattern 1: Active intent count vs alignment
    let active_intents: i64 = {
        let future_dir = std::path::PathBuf::from(&ctx.core_root).join("intents/future");
        std::fs::read_dir(&future_dir)
            .ok()
            .map(|d| {
                d.filter_map(|e| e.ok())
                    .filter(|e| {
                        std::fs::read_to_string(e.path())
                            .ok()
                            .map(|c| c.contains("status: in-progress"))
                            .unwrap_or(false)
                    })
                    .count() as i64
            })
            .unwrap_or(0)
    };
    if active_intents > 2 {
        let desc = format!(
            "CONTRADICTION: {} active intents -- values declare focus>speed",
            active_intents
        );
        patterns_found.push(desc.clone());
        let _ = db.execute(
            "INSERT OR IGNORE INTO friday_contradictions
             (engine_a, signal_a, engine_b, signal_b, description, severity, detected_at)
             VALUES ('alignment', 'focus>speed', 'intent', ?1, ?2, 'medium', ?3)",
            params![format!("{} active intents", active_intents), desc, now],
        );
    } else {
        // Auto-resolve stale intent contradictions when count is back to normal
        let _ = db.execute(
            "UPDATE friday_contradictions SET resolved = 1, resolved_at = ?1
             WHERE engine_b = 'intent' AND resolved = 0",
            params![now],
        );
    }
    // Pattern 2: Session commit velocity
    let today_start = now - (now % 86400);
    let commits_today: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM commit_patterns WHERE timestamp > ?1",
            params![today_start],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if commits_today > 15 {
        let desc = format!(
            "HIGH VELOCITY: {} commits today -- monitor health trajectory",
            commits_today
        );
        patterns_found.push(desc);
    }
    // Pattern 3: Deploy success rate
    let deploy_count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM deploy_patterns WHERE timestamp > ?1",
            params![today_start],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let deploy_success: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM deploy_patterns WHERE timestamp > ?1 AND outcome = 'success'",
            params![today_start],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if deploy_count > 3 {
        let success_rate = deploy_success as f64 / deploy_count as f64;
        if success_rate < 0.8 {
            let desc = format!(
                "SIGNAL: Deploy success rate {:.0}% -- investigate before continuing",
                success_rate * 100.0
            );
            patterns_found.push(desc);
        }
    }
    // Pattern 4: Check health trend from synthesis snapshots
    let recent_health: Vec<i64> = {
        let mut s = db
            .prepare("SELECT health FROM synthesis_snapshots ORDER BY timestamp DESC LIMIT 5")
            .unwrap();
        s.query_map([], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };
    if recent_health.len() >= 3 {
        let trend: i64 = recent_health[0] - recent_health[recent_health.len() - 1];
        if trend < -5 {
            patterns_found.push(format!(
                "TREND: Health declining {} points across last {} snapshots",
                trend.abs(),
                recent_health.len()
            ));
        }
    }
    Ok(patterns_found)
}
/// Detect contradictions between engine signals
pub fn detect_contradictions(ctx: &AppContext) -> CoreResult<Vec<(String, String, String)>> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let mut contradictions: Vec<(String, String, String)> = Vec::new();
    // Check unresolved contradictions detected in last 24h only
    let cutoff = now_ts() - 86400;
    let rows: Vec<(String, String, String)> = {
        let mut s = db.prepare(
            "SELECT engine_a, engine_b, description FROM friday_contradictions
             WHERE resolved = 0 AND detected_at > ?1 ORDER BY detected_at DESC LIMIT 5",
        )?;
        let x: Vec<(String, String, String)> = s
            .query_map(rusqlite::params![cutoff], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    contradictions.extend(rows);
    Ok(contradictions)
}
/// List Friday models and their trust scores
pub fn show_models(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let rows: Vec<(i64, String, String, f64, i64, i64)> = {
        let mut s = db.prepare(
            "SELECT m.id, m.description, m.domain, t.accuracy, t.predictions, t.correct
             FROM friday_models m
             LEFT JOIN friday_trust t ON t.model_id = m.id
             WHERE m.active = 1 ORDER BY t.accuracy DESC NULLS LAST",
        )?;
        let x: Vec<(i64, String, String, f64, i64, i64)> = s
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, f64>(3).unwrap_or(0.5),
                    r.get::<_, i64>(4).unwrap_or(0),
                    r.get::<_, i64>(5).unwrap_or(0),
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    println!();
    println!(
        "  {} Friday Models -- Meta-Interpretation Engine",
        "🌲".normal()
    );
    println!("  {}", "─".repeat(60).dimmed());
    for (id, desc, domain, accuracy, predictions, correct) in &rows {
        let short_desc = desc.chars().take(55).collect::<String>();
        let acc_colored = if *accuracy >= 0.8 {
            format!("{:.0}%", accuracy * 100.0)
                .bright_green()
                .to_string()
        } else if *accuracy >= 0.6 {
            format!("{:.0}%", accuracy * 100.0)
                .bright_yellow()
                .to_string()
        } else {
            format!("{:.0}%", accuracy * 100.0).bright_red().to_string()
        };
        println!(
            "  {} [{}] {} ({}) {}/{} {}",
            "→".bright_green(),
            id.to_string().dimmed(),
            short_desc.white(),
            domain.dimmed(),
            correct,
            predictions,
            acc_colored
        );
    }
    println!();
    Ok(())
}

/// Friday speaks when it detects a known error pattern
pub fn speak_on_error(ctx: &AppContext, error_output: &str) -> CoreResult<()> {
    if let Some((desc, resolution, confidence)) =
        crate::domains::knowledge::query_for_error(ctx, error_output)
    {
        if confidence >= 0.85 {
            println!();
            println!("  {} Friday knows this pattern:", "🌲".normal());
            println!(
                "  {} {} ({:.0}% confidence)",
                "→".bright_green(),
                desc.bright_white(),
                confidence * 100.0
            );
            println!("  {} {}", "fix:".bright_cyan(), resolution.white());
            println!();
        }
    }
    Ok(())
}
/// Run the full Friday formal architecture cycle
pub fn run(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    seed_models(ctx)?;
    println!();
    println!("  {} Friday -- Meta-Interpretation Engine", "🌲".normal());
    println!("  {}", "━".repeat(55).dimmed());
    // Phase 1: Pattern detection
    let patterns = detect_patterns(ctx)?;
    if !patterns.is_empty() {
        println!();
        println!("  {} Cross-layer patterns detected:", "🔍".normal());
        for p in &patterns {
            let colored = if p.starts_with("CONTRADICTION") {
                p.bright_red().to_string()
            } else if p.starts_with("SIGNAL") || p.starts_with("TREND") {
                p.bright_yellow().to_string()
            } else {
                p.bright_white().to_string()
            };
            println!("    → {}", colored);
        }
    } else {
        println!(
            "  {} No cross-layer patterns detected -- forest is coherent",
            "✅".green()
        );
    }
    // Phase 2: Contradictions
    let contradictions = detect_contradictions(ctx)?;
    if !contradictions.is_empty() {
        println!();
        println!("  {} Active contradictions:", "⚠️ ".yellow());
        for (a, b, desc) in &contradictions {
            println!(
                "    {} {} ↔ {}: {}",
                "→".bright_red(),
                a.bright_cyan(),
                b.bright_cyan(),
                desc.chars().take(60).collect::<String>().white()
            );
        }
    }
    // Phase 3: Brief
    let now = now_ts();
    let brief = if !patterns.is_empty() {
        format!(
            "Friday sees {} cross-layer signal(s). {}{}",
            patterns.len(),
            if !contradictions.is_empty() {
                format!("{} contradiction(s) active. ", contradictions.len())
            } else {
                String::new()
            },
            patterns
                .first()
                .map(|p| p.chars().take(80).collect::<String>())
                .unwrap_or_default()
        )
    } else {
        "Forest is coherent. No cross-layer conflicts detected. All engines aligned.".to_string()
    };
    // Store brief in synthesis
    let _ = ctx.runtime.db.execute(
        "INSERT INTO synthesis_snapshots (timestamp, health, alignment, active_intent,
         session_commits, top_pattern, friday_brief, brief_confidence, contradiction)
         VALUES (?1, 100, 1.0, 'INT-216', 0, 'meta-interpretation', ?2, 0.85, ?3)",
        params![
            now,
            brief,
            if contradictions.is_empty() { "" } else { "yes" }
        ],
    );
    println!();
    println!("  {} Friday brief:", "💡".normal());
    println!("  {}", brief.bright_white());
    println!();
    println!("  {} Friday produces insight, not authority.", "→".dimmed());
    Ok(())
}
/// Show pending proposals awaiting human review
pub fn show_proposals(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let rows: Vec<(i64, String, String, f64)> = {
        let mut s = db.prepare(
            "SELECT id, description, action, confidence FROM friday_proposals
             WHERE status = 'pending' ORDER BY confidence DESC LIMIT 10",
        )?;
        let x: Vec<(i64, String, String, f64)> = s
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, f64>(3)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    println!();
    println!(
        "  {} Friday Proposals -- Pending Human Review",
        "🌲".normal()
    );
    println!("  {}", "─".repeat(55).dimmed());
    if rows.is_empty() {
        println!("  {} No pending proposals.", "→".dimmed());
    } else {
        for (id, desc, action, conf) in &rows {
            println!(
                "  [{}] {} ({:.0}%)",
                id,
                desc.chars().take(50).collect::<String>(),
                conf * 100.0
            );
            println!("      → {}", action.bright_cyan());
        }
    }
    println!();
    Ok(())
}
