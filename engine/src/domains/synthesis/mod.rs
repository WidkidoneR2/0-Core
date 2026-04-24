//! INT-212 -- Core v18: Synthesis Engine
//! The forest speaks with one voice.
//! Combines v17 pattern weights, health, alignment, Friday patterns into one coherent brief.
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
static CREATE_TABLES: &str = "
CREATE TABLE IF NOT EXISTS synthesis_snapshots (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp       INTEGER NOT NULL,
    health          INTEGER NOT NULL DEFAULT 100,
    alignment       REAL NOT NULL DEFAULT 1.0,
    active_intent   TEXT NOT NULL DEFAULT '',
    session_commits INTEGER NOT NULL DEFAULT 0,
    top_pattern     TEXT NOT NULL DEFAULT '',
    friday_brief    TEXT NOT NULL DEFAULT '',
    brief_confidence REAL NOT NULL DEFAULT 0.0,
    contradiction   TEXT NOT NULL DEFAULT '',
    session_id      TEXT
);
";
fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(CREATE_TABLES)?;
    Ok(())
}
/// Gather all intelligence signals and synthesize into one brief
pub fn synthesize_now(ctx: &AppContext) -> CoreResult<SynthesisResult> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    // 1. Health
    let health: u32 = std::fs::read_to_string(
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".cache/faelight/health-status"),
    )
    .unwrap_or_else(|_| "100".into())
    .trim()
    .trim_end_matches('%')
    .parse()
    .unwrap_or(100);
    // 2. Alignment
    let alignment: f64 = db.query_row(
        "SELECT AVG(score) FROM alignment_checks WHERE checked_at > (strftime('%s','now') - 604800)",
        [], |r| r.get::<_, Option<f64>>(0)
    ).unwrap_or(None).unwrap_or(1.0).min(1.0).max(0.0);
    // 3. Active intent -- read from filesystem like predict domain
    let active_intent: String = {
        let future_dir = std::path::PathBuf::from(&ctx.core_root).join("intents/future");
        std::fs::read_dir(&future_dir)
            .ok()
            .and_then(|d| {
                d.filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let content = std::fs::read_to_string(e.path()).ok()?;
                        if content.contains("status: in-progress") {
                            content
                                .lines()
                                .find(|l| l.starts_with("title:"))?
                                .trim_start_matches("title:")
                                .trim()
                                .trim_matches('"')
                                .to_string()
                                .into()
                        } else {
                            None
                        }
                    })
                    .next()
            })
            .unwrap_or_default()
    };
    // 4. Session commits today
    let today_start = now - (now % 86400);
    let session_commits: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM commit_patterns WHERE timestamp > ?1",
            params![today_start],
            |r| r.get(0),
        )
        .unwrap_or(0);
    // 5. Top pattern from Friday
    let top_pattern: String = {
        let mut s = db.prepare(
            "SELECT trigger, action, confidence FROM friday_patterns ORDER BY confidence DESC LIMIT 1"
        )?;
        let x: Vec<(String, String, f64)> = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, f64>(2)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        x.first()
            .map(|(t, a, c)| format!("{} → {} ({:.0}%)", t, a, c * 100.0))
            .unwrap_or_default()
    };
    // 6. Top weight signal from v17
    let top_weight: Option<(String, f64)> = db
        .query_row(
            "SELECT id, final_weight FROM pattern_weights ORDER BY final_weight DESC LIMIT 1",
            [],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?)),
        )
        .ok();
    // 7. Contradiction detection
    let mut contradictions: Vec<String> = Vec::new();
    // Active intents count vs focus value
    let active_count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM intents WHERE status = 'in-progress'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if active_count > 2 {
        contradictions.push(format!(
            "{} intents in progress -- values declare focus > speed",
            active_count
        ));
    }
    // Health vs momentum
    if health < 95 && session_commits > 10 {
        contradictions.push(format!(
            "High commit velocity ({} today) while health is at {}%",
            session_commits, health
        ));
    }
    // 8. Generate brief
    let (brief, confidence) = generate_brief(
        health,
        alignment,
        &active_intent,
        session_commits,
        &top_pattern,
        &top_weight,
        &contradictions,
    );
    // 9. Store snapshot
    let contradiction_str = contradictions.join("; ");
    let _ = db.execute(
        "INSERT INTO synthesis_snapshots (timestamp, health, alignment, active_intent, session_commits, top_pattern, friday_brief, brief_confidence, contradiction)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![now, health, alignment, active_intent, session_commits, top_pattern, brief, confidence, contradiction_str],
    );
    // 10. Emit to forest_events_v2
    let payload = format!(
        r#"{{"brief":"{}","confidence":{:.2}}}"#,
        brief.replace('"', "'"),
        confidence
    );
    let _ = crate::domains::events::signal::emit(
        db,
        "synthesis",
        crate::domains::events::signal::SignalKind::Interpretation,
        "synthesis",
        &payload,
        None,
        None,
        confidence,
    );
    Ok(SynthesisResult {
        health,
        alignment,
        active_intent,
        session_commits,
        top_pattern,
        brief,
        confidence,
        contradictions,
    })
}
pub struct SynthesisResult {
    pub health: u32,
    pub alignment: f64,
    pub active_intent: String,
    pub session_commits: i64,
    pub top_pattern: String,
    pub brief: String,
    pub confidence: f64,
    pub contradictions: Vec<String>,
}
fn generate_brief(
    health: u32,
    alignment: f64,
    active_intent: &str,
    session_commits: i64,
    top_pattern: &str,
    top_weight: &Option<(String, f64)>,
    contradictions: &[String],
) -> (String, f64) {
    let mut parts: Vec<String> = Vec::new();
    let mut confidence: f64 = 0.7;
    // Lead with contradiction if any
    if !contradictions.is_empty() {
        parts.push(format!("⚠ Contradiction: {}", contradictions[0]));
        confidence *= 0.9;
    }
    // Intent context
    if !active_intent.is_empty() {
        let short_intent = active_intent.chars().take(50).collect::<String>();
        let momentum = if session_commits >= 10 {
            "strong momentum"
        } else if session_commits >= 5 {
            "steady progress"
        } else {
            "early session"
        };
        parts.push(format!(
            "Working on: {} -- {} ({} commits today)",
            short_intent, momentum, session_commits
        ));
        confidence += 0.1;
    }
    // Health and alignment
    if health == 100 && alignment >= 0.99 {
        parts.push("Forest is healthy and aligned. No concerns.".to_string());
    } else if health < 95 {
        parts.push(format!(
            "Health at {}% -- investigate before continuing.",
            health
        ));
        confidence -= 0.1;
    }
    // Top pattern insight
    if !top_pattern.is_empty() {
        parts.push(format!("Pattern: {}", top_pattern));
        confidence += 0.05;
    }
    // Weight engine signal
    if let Some((id, weight)) = top_weight {
        if *weight > 0.5 {
            parts.push(format!("Dominant signal: {} (weight {:.2})", id, weight));
        }
    }
    if parts.is_empty() {
        parts.push("Synthesis has insufficient data. Continue building.".to_string());
        confidence = 0.3;
    }
    (parts.join(" "), confidence.clamp(0.0, 1.0))
}
/// core synthesize now -- generate and display synthesis snapshot
pub fn cmd_now(ctx: &AppContext) -> CoreResult<()> {
    let result = synthesize_now(ctx)?;
    println!();
    println!("  {} Core v18 -- Synthesis Snapshot", "🌲".normal());
    println!("  {}", "━".repeat(55).dimmed());
    println!();
    if !result.contradictions.is_empty() {
        for c in &result.contradictions {
            println!("  {} {}", "⚠".bright_red(), c.bright_yellow());
        }
        println!();
    }
    println!(
        "  {:<24} {}%",
        "Health:".dimmed(),
        if result.health == 100 {
            result.health.to_string().bright_green()
        } else {
            result.health.to_string().bright_yellow()
        }
    );
    let align_str = format!("{:.0}%", result.alignment * 100.0);
    println!(
        "  {:<24} {}",
        "Alignment:".dimmed(),
        align_str.bright_white()
    );
    println!(
        "  {:<24} {}",
        "Active intent:".dimmed(),
        result
            .active_intent
            .chars()
            .take(45)
            .collect::<String>()
            .bright_cyan()
    );
    println!(
        "  {:<24} {} today",
        "Session commits:".dimmed(),
        result.session_commits.to_string().bright_white()
    );
    if !result.top_pattern.is_empty() {
        println!(
            "  {:<24} {}",
            "Top pattern:".dimmed(),
            result.top_pattern.bright_white()
        );
    }
    println!();
    println!("  {} Friday brief:", "💡".normal());
    println!("  {}", result.brief.bright_white().bold());
    println!(
        "  {} confidence: {:.0}%",
        "·".dimmed(),
        result.confidence * 100.0
    );
    println!();
    Ok(())
}
/// core synthesize brief -- show latest stored brief
pub fn cmd_brief(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let row: Option<(String, f64, i64)> = ctx.runtime.db.query_row(
        "SELECT friday_brief, brief_confidence, timestamp FROM synthesis_snapshots ORDER BY timestamp DESC LIMIT 1",
        [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
    ).ok();
    match row {
        Some((brief, conf, ts)) => {
            let age = (now_ts() - ts) / 60;
            println!();
            println!(
                "  {} Friday brief ({} min ago, {:.0}% confidence)",
                "🌲".normal(),
                age,
                conf * 100.0
            );
            println!("  {}", "─".repeat(55).dimmed());
            println!("  {}", brief.bright_white());
            println!();
        }
        None => {
            println!("  No synthesis snapshot yet -- run: core synthesize now");
        }
    }
    Ok(())
}
/// core synthesize history -- past snapshots
pub fn cmd_history(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let rows: Vec<(i64, u32, String, i64, String)> = {
        let mut s = ctx.runtime.db.prepare(
            "SELECT timestamp, health, active_intent, session_commits, friday_brief
             FROM synthesis_snapshots ORDER BY timestamp DESC LIMIT 10",
        )?;
        let x: Vec<(i64, u32, String, i64, String)> = s
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get::<_, String>(2)?,
                    r.get(3)?,
                    r.get::<_, String>(4)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    println!();
    println!("  {} Synthesis History", "🌲".normal());
    println!("  {}", "─".repeat(55).dimmed());
    for (ts, health, intent, commits, brief) in &rows {
        let time = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|t| t.format("%m/%d %H:%M").to_string())
            .unwrap_or_default();
        let short_intent = intent.chars().take(30).collect::<String>();
        let short_brief = brief.chars().take(60).collect::<String>();
        println!(
            "  {} {}% {} {}c  {}",
            time.dimmed(),
            health.to_string().bright_green(),
            short_intent.bright_cyan(),
            commits.to_string().dimmed(),
            short_brief.white()
        );
    }
    println!();
    Ok(())
}
