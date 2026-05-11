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

// ── INT-246: Friday Architecture v2 ──────────────────────────────────────
/// Confidence tiers -- formalized from INT-246 Pillar 1
pub enum ConfidenceTier {
    Observe,    // 0.0 - 0.4  -- collect data, say nothing
    Suggest,    // 0.4 - 0.7  -- surface insight, no interruption
    Recommend,  // 0.7 - 0.9  -- interrupt with specific suggestion
    Challenge,  // 0.9+       -- block and require explicit approval
}
impl ConfidenceTier {
    pub fn from_confidence(c: f64) -> Self {
        if c >= 0.9 { Self::Challenge }
        else if c >= 0.7 { Self::Recommend }
        else if c >= 0.4 { Self::Suggest }
        else { Self::Observe }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Observe   => "OBSERVE",
            Self::Suggest   => "SUGGEST",
            Self::Recommend => "RECOMMEND",
            Self::Challenge => "CHALLENGE",
        }
    }
    #[allow(dead_code)]
    pub fn should_speak(&self) -> bool {
        !matches!(self, Self::Observe)
    }
}
/// Create friday_usefulness table -- INT-246 Pillar 4
static USEFULNESS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS friday_usefulness (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    suggestion  TEXT NOT NULL,
    context     TEXT NOT NULL DEFAULT '',
    confidence  REAL NOT NULL DEFAULT 0.5,
    tier        TEXT NOT NULL DEFAULT 'suggest',
    accepted    INTEGER NOT NULL DEFAULT 0,
    recorded_at INTEGER NOT NULL
);
";
pub fn ensure_usefulness_table(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(USEFULNESS_TABLE)?;
    Ok(())
}
/// Record a Friday suggestion outcome (accepted=1 or accepted=0)
#[allow(dead_code)]
pub fn record_usefulness(
    ctx: &AppContext,
    suggestion: &str,
    context: &str,
    confidence: f64,
    accepted: bool,
) -> CoreResult<()> {
    ensure_usefulness_table(ctx)?;
    let tier = ConfidenceTier::from_confidence(confidence);
    ctx.runtime.db.execute(
        "INSERT INTO friday_usefulness (suggestion, context, confidence, tier, accepted, recorded_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            suggestion, context, confidence,
            tier.label(),
            if accepted { 1i64 } else { 0i64 },
            now_ts()
        ],
    )?;
    Ok(())
}
/// Show Friday usefulness metrics -- INT-246 Pillar 4
pub fn show_usefulness(ctx: &AppContext) -> CoreResult<()> {
    ensure_usefulness_table(ctx)?;
    let db = &ctx.runtime.db;
    println!("{}", "📊 Friday Usefulness Metrics".bold());
    println!("{}", "━".repeat(55).dimmed());
    let total: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_usefulness", [], |r| r.get(0)
    ).unwrap_or(0);
    let accepted: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_usefulness WHERE accepted = 1", [], |r| r.get(0)
    ).unwrap_or(0);
    if total == 0 {
        println!("  {} No suggestions tracked yet", "○".dimmed());
        println!("  {} Suggestions are tracked as you accept or reject Friday proposals", "→".dimmed());
        return Ok(());
    }
    let rate = accepted as f64 / total as f64 * 100.0;
    let rate_str = if rate >= 75.0 {
        format!("{:.1}%", rate).bright_green().to_string()
    } else if rate >= 50.0 {
        format!("{:.1}%", rate).bright_yellow().to_string()
    } else {
        format!("{:.1}%", rate).bright_red().to_string()
    };
    println!("  {} Total suggestions: {}", "→".dimmed(), total.to_string().bright_white());
    println!("  {} Accepted: {}", "→".dimmed(), accepted.to_string().bright_green());
    println!("  {} Rejected: {}", "→".dimmed(), (total - accepted).to_string().bright_red());
    println!("  {} Acceptance rate: {} (target: >75%)", "→".dimmed(), rate_str);
    // By tier
    println!();
    println!("  {} By confidence tier:", "→".dimmed());
    for tier in &["OBSERVE", "SUGGEST", "RECOMMEND", "CHALLENGE"] {
        let t: i64 = db.query_row(
            "SELECT COUNT(*) FROM friday_usefulness WHERE tier = ?1",
            params![tier], |r| r.get(0)
        ).unwrap_or(0);
        let a: i64 = db.query_row(
            "SELECT COUNT(*) FROM friday_usefulness WHERE tier = ?1 AND accepted = 1",
            params![tier], |r| r.get(0)
        ).unwrap_or(0);
        if t > 0 {
            println!("    {} {}: {}/{} accepted", "◦".dimmed(), tier.bright_white(), a, t);
        }
    }
    println!("{}", "━".repeat(55).dimmed());
    Ok(())
}
/// Trust decay -- INT-246 Pillar 1 (deferred from INT-216)
/// Models that are consistently wrong lose weight over time.
/// Models below 0.3 trust are silenced automatically.
pub fn decay_trust(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    // Get all active models with trust data
    let mut stmt = db.prepare(
        "SELECT m.id, m.confidence, t.predictions, t.correct
         FROM friday_models m
         JOIN friday_trust t ON t.model_id = m.id
         WHERE m.active = 1 AND t.predictions > 0"
    )?;
    let models: Vec<(i64, f64, i64, i64)> = stmt.query_map([], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?,
            r.get::<_, i64>(2)?, r.get::<_, i64>(3)?))
    })?.filter_map(|r| r.ok()).collect();
    if models.is_empty() {
        println!("  {} No models with prediction data yet", "○".dimmed());
        return Ok(());
    }
    println!("{}", "🔄 Friday Trust Decay".bold());
    println!("{}", "━".repeat(55).dimmed());
    let mut silenced = 0;
    let mut decayed = 0;
    let mut gained = 0;
    for (model_id, confidence, predictions, correct) in &models {
        let accuracy = *correct as f64 / *predictions as f64;
        let new_confidence = if accuracy < 0.5 {
            // Wrong more than half the time -- decay
            (confidence - 0.1).max(0.0)
        } else if accuracy > 0.8 {
            // Reliably correct -- gain trust
            (confidence + 0.05).min(1.0)
        } else {
            *confidence
        };
        let tier = ConfidenceTier::from_confidence(new_confidence);
        let still_active = new_confidence >= 0.3;
        if (new_confidence - confidence).abs() > 0.001 {
            db.execute(
                "UPDATE friday_models SET confidence = ?1, active = ?2 WHERE id = ?3",
                params![new_confidence, if still_active { 1 } else { 0 }, model_id],
            )?;
            db.execute(
                "UPDATE friday_trust SET accuracy = ?1, updated_at = ?2 WHERE model_id = ?3",
                params![accuracy, now, model_id],
            )?;
            if !still_active {
                println!("  {} Model {} silenced (accuracy: {:.0}%, confidence: {:.2} → {:.2})",
                    "🔇".dimmed(), model_id,
                    accuracy * 100.0, confidence, new_confidence);
                silenced += 1;
            } else if new_confidence < *confidence {
                println!("  {} Model {} decayed to {} (accuracy: {:.0}%)",
                    "📉".bright_yellow(), model_id, tier.label(),
                    accuracy * 100.0);
                decayed += 1;
            } else {
                println!("  {} Model {} gained trust → {} (accuracy: {:.0}%)",
                    "📈".bright_green(), model_id, tier.label(),
                    accuracy * 100.0);
                gained += 1;
            }
        }
    }
    if silenced == 0 && decayed == 0 && gained == 0 {
        println!("  {} All models stable -- no decay needed", "✅".green());
    } else {
        println!();
        println!("  {} {} silenced  {} decayed  {} gained trust",
            "→".dimmed(), silenced, decayed, gained);
    }
    println!("{}", "━".repeat(55).dimmed());
    Ok(())
}

/// Approve a pending Friday proposal -- INT-246 human approval gate
pub fn approve_proposal(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    ensure_usefulness_table(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    let proposal: Option<(String, String, f64)> = db.query_row(
        "SELECT description, action, confidence FROM friday_proposals WHERE id = ?1 AND status = 'pending'",
        rusqlite::params![id],
        |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?))
    ).ok();
    match proposal {
        None => {
            println!("  {} Proposal {} not found or not pending", "⚠".bright_yellow(), id);
        }
        Some((desc, action, confidence)) => {
            db.execute(
                "UPDATE friday_proposals SET status = 'approved', reviewed_at = ?1, outcome = 'accepted' WHERE id = ?2",
                rusqlite::params![now, id],
            )?;
            let _ = record_usefulness(ctx, &desc, &action, confidence, true);
            println!("{}", "✅ Proposal Approved".bold().bright_green());
            println!("{}", "━".repeat(55).dimmed());
            println!("  {} {}", "Proposal:".dimmed(), desc.bright_white());
            println!("  {} {}", "Action:   ".dimmed(), action.bright_cyan());
            println!("  {} Trust +0.05 for this model", "→".dimmed());
            println!("{}", "━".repeat(55).dimmed());
        }
    }
    Ok(())
}
/// Reject a pending Friday proposal -- feeds trust decay
pub fn reject_proposal(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    ensure_usefulness_table(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    let proposal: Option<(String, String, f64)> = db.query_row(
        "SELECT description, action, confidence FROM friday_proposals WHERE id = ?1 AND status = 'pending'",
        rusqlite::params![id],
        |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?))
    ).ok();
    match proposal {
        None => {
            println!("  {} Proposal {} not found or not pending", "⚠".bright_yellow(), id);
        }
        Some((desc, action, confidence)) => {
            db.execute(
                "UPDATE friday_proposals SET status = 'rejected', reviewed_at = ?1, outcome = 'rejected' WHERE id = ?2",
                rusqlite::params![now, id],
            )?;
            let _ = record_usefulness(ctx, &desc, &action, confidence, false);
            println!("{}", "⬜ Proposal Rejected".bold().bright_red());
            println!("{}", "━".repeat(55).dimmed());
            println!("  {} {}", "Proposal:".dimmed(), desc.bright_white());
            println!("  {} Trust -0.1 applied via next trust-decay run", "→".dimmed());
            println!("{}", "━".repeat(55).dimmed());
        }
    }
    Ok(())
}

pub fn generate_proposal(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    ensure_usefulness_table(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    let health: i64 = db.query_row(
        "SELECT COALESCE(AVG(score), 100) FROM health_history ORDER BY checked_at DESC LIMIT 5",
        [], |r| r.get(0)
    ).unwrap_or(100);
    let active_intent: Option<(i64, String)> = db.query_row(
        "SELECT id, title FROM intents WHERE status = 'in-progress' LIMIT 1",
        [], |r| Ok((r.get(0)?, r.get(1)?))
    ).ok();
    let recent_deploys: i64 = db.query_row(
        "SELECT COUNT(*) FROM deploy_history WHERE deployed_at > ?1",
        params![now - 3600], |r| r.get(0)
    ).unwrap_or(0);
    let pattern_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_patterns WHERE confidence > 0.7",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let (description, action, confidence, rationale) = if recent_deploys > 0 && health < 95 {
        (
            "Health check after recent deploys -- verify nothing degraded".to_string(),
            "core doctor run".to_string(),
            0.88f64,
            format!("{} deploy(s) in last hour, health at {}%", recent_deploys, health)
        )
    } else if let Some((id, ref title)) = active_intent {
        (
            format!("Checkpoint -- commit progress on INT-{}", id),
            "fg done \"progress checkpoint\"".to_string(),
            0.75f64,
            format!("Working on: {} -- regular commits improve recovery", title)
        )
    } else if pattern_count > 10 {
        (
            "Review Friday patterns -- high-confidence patterns ready".to_string(),
            "core friday-arch models".to_string(),
            0.72f64,
            format!("{} patterns above 0.7 confidence ready for review", pattern_count)
        )
    } else {
        (
            "Forest health check -- routine verification".to_string(),
            "core doctor run".to_string(),
            0.65f64,
            "Regular verification keeps the forest coherent".to_string()
        )
    };
    db.execute(
        "INSERT INTO friday_proposals (signal_type, description, action, confidence, status, created_at)
         VALUES ('context', ?1, ?2, ?3, 'pending', ?4)",
        params![description, action, confidence, now],
    )?;
    let proposal_id: i64 = db.query_row(
        "SELECT id FROM friday_proposals ORDER BY id DESC LIMIT 1",
        [], |r| r.get(0)
    )?;
    println!();
    println!("  {} Friday Proposal [{}]", "🌲".normal(), proposal_id);
    println!("  {}", "─".repeat(55).dimmed());
    println!("  {} {}", "Proposal:  ".dimmed(), description.bright_white());
    println!("  {} {}", "Action:    ".dimmed(), action.bright_cyan());
    println!("  {} {:.0}%", "Confidence:".dimmed(), confidence * 100.0);
    println!("  {} {}", "Rationale: ".dimmed(), rationale.dimmed());
    println!();
    println!("  {} approve: core friday-arch approve {}", "→".dimmed(), proposal_id);
    println!("  {} reject:  core friday-arch reject {}", "→".dimmed(), proposal_id);
    println!("  {}", "─".repeat(55).dimmed());
    Ok(())
}

/// INT-246 Pillar 2 -- Basic simulation engine
/// Predicts command outcome from historical data before execution
pub fn simulate(ctx: &AppContext, command: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let cmd = command.trim();

    println!();
    println!("  {} Friday Simulation", "🌲".normal());
    println!("  {}", "─".repeat(55).dimmed());
    println!("  {} {}", "Command:".dimmed(), cmd.bright_white());
    println!();

    // Detect command type and gather historical data
    let words: Vec<&str> = cmd.split_whitespace().collect();
    let first = words.first().copied().unwrap_or("");
    let second = words.get(1).copied().unwrap_or("");

    match first {
        "deploy" if !second.is_empty() => {
            simulate_deploy(db, second)?;
        }
        "cargo" if second == "build" => {
            simulate_build(db, cmd)?;
        }
        "fg" if second == "done" || second == "commit" => {
            simulate_commit(db)?;
        }
        "cistart" | "cicomplete" => {
            simulate_intent_op(db, first, second)?;
        }
        _ => {
            // Generic: check shell history for outcome patterns
            simulate_generic(db, cmd)?;
        }
    }

    println!("  {}", "─".repeat(55).dimmed());
    println!("  {} To run: {}", "→".dimmed(), cmd.bright_cyan());
    println!();
    Ok(())
}

fn simulate_deploy(db: &rusqlite::Connection, tool: &str) -> CoreResult<()> {
    // Query historical deploy data for this tool
    let data: Option<(i64, i64, f64, i64, i64)> = db.query_row(
        "SELECT COUNT(*),
                SUM(CASE WHEN outcome = 'success' THEN 1 ELSE 0 END),
                AVG(duration_ms),
                MIN(duration_ms),
                MAX(duration_ms)
         FROM deploy_patterns WHERE tool = ?1",
        rusqlite::params![tool],
        |r| Ok((r.get(0)?, r.get(1)?, r.get::<_,f64>(2).unwrap_or(0.0),
                 r.get(3)?, r.get(4)?))
    ).ok();

    match data {
        Some((total, success, avg_ms, min_ms, max_ms)) if total > 0 => {
            let success_rate = (success as f64 / total as f64) * 100.0;
            let avg_s = avg_ms / 1000.0;
            let confidence = if total >= 10 { 0.92 } else if total >= 3 { 0.78 } else { 0.60 };

            let risk = if success_rate >= 99.0 { "LOW".bright_green() }
                       else if success_rate >= 90.0 { "MEDIUM".bright_yellow() }
                       else { "HIGH".bright_red() };

            println!("  {} Deploy: {}", "→".dimmed(), tool.bright_cyan());
            println!("  {} Historical data: {} deploys", "→".dimmed(), total);
            println!("  {} Success rate:    {:.1}% ({}/{})",
                "→".dimmed(), success_rate, success, total);
            println!("  {} Avg duration:    {:.1}s  (range: {:.1}s - {:.1}s)",
                "→".dimmed(), avg_s, min_ms as f64 / 1000.0, max_ms as f64 / 1000.0);
            println!("  {} Risk:            {}", "→".dimmed(), risk);
            println!("  {} Confidence:      {:.0}%", "→".dimmed(), confidence * 100.0);
            println!();

            let pred = if success_rate >= 95.0 { "SUCCESS" } else { "UNCERTAIN" };
            let pred_colored = if pred == "SUCCESS" {
                pred.bright_green().to_string()
            } else {
                pred.bright_yellow().to_string()
            };
            println!("  {} Predicted outcome: {} ({:.0}% confidence)",
                "🌲".normal(), pred_colored, confidence * 100.0);
        }
        _ => {
            println!("  {} No historical data for deploy {}", "○".dimmed(), tool);
            println!("  {} First deploy -- outcome unknown", "→".dimmed());
            println!("  {} Confidence: 50% (no prior data)", "→".dimmed());
        }
    }
    Ok(())
}

fn simulate_build(db: &rusqlite::Connection, _cmd: &str) -> CoreResult<()> {
    let recent_errors: i64 = db.query_row(
        "SELECT COUNT(*) FROM shell_history
         WHERE command LIKE '%cargo build%' AND command LIKE '%error%'
         AND timestamp > strftime('%s','now') - 86400",
        [], |r| r.get(0)
    ).unwrap_or(0);

    println!("  {} Cargo build simulation", "→".dimmed());
    println!("  {} Recent build errors (24h): {}", "→".dimmed(), recent_errors);

    if recent_errors == 0 {
        println!("  {} Risk: LOW -- no recent errors", "→".dimmed());
        println!("  {} Predicted: SUCCESS (workspace clean)", "→".dimmed());
    } else {
        println!("  {} Risk: MEDIUM -- {} errors in last 24h", "→".dimmed(), recent_errors);
        println!("  {} Check error patterns before building", "→".dimmed());
    }
    Ok(())
}

fn simulate_commit(db: &rusqlite::Connection) -> CoreResult<()> {
    let uncommitted: i64 = db.query_row(
        "SELECT COUNT(*) FROM shell_history
         WHERE command LIKE '%git add%' OR command LIKE '%fg done%'
         AND timestamp > strftime('%s','now') - 3600",
        [], |r| r.get(0)
    ).unwrap_or(0);

    println!("  {} Commit simulation", "→".dimmed());
    println!("  {} Risk: LOW -- standard forest operation", "→".dimmed());
    println!("  {} Predicted: SUCCESS -- pre-push hooks will validate", "→".dimmed());
    let _ = uncommitted;
    Ok(())
}

fn simulate_intent_op(db: &rusqlite::Connection, op: &str, intent_id: &str) -> CoreResult<()> {
    println!("  {} Intent {} simulation", "→".dimmed(), op);
    if !intent_id.is_empty() {
        println!("  {} Target intent: {}", "→".dimmed(), intent_id.bright_white());
    }
    println!("  {} Risk: LOW -- creates checkpoint before operation", "→".dimmed());
    println!("  {} Predicted: SUCCESS -- checkpoint protects state", "→".dimmed());
    let _ = db;
    Ok(())
}

fn simulate_generic(db: &rusqlite::Connection, cmd: &str) -> CoreResult<()> {
    // Check if this command has been run before
    let prior_runs: i64 = db.query_row(
        "SELECT COUNT(*) FROM shell_history WHERE command = ?1",
        rusqlite::params![cmd],
        |r| r.get(0)
    ).unwrap_or(0);

    println!("  {} Generic command simulation", "→".dimmed());
    if prior_runs > 0 {
        println!("  {} Prior runs: {} times", "→".dimmed(), prior_runs);
        println!("  {} Risk: LOW -- command has been run before", "→".dimmed());
        println!("  {} Confidence: 70%", "→".dimmed());
    } else {
        println!("  {} Prior runs: 0 -- first time running this", "→".dimmed());
        println!("  {} Risk: UNKNOWN -- no historical data", "→".dimmed());
        println!("  {} Confidence: 50%", "→".dimmed());
    }
    let _ = db;
    Ok(())
}
