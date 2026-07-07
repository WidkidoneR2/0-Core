#![allow(dead_code)]
//! INT-277: Friday Attention Engine -- Core v24
//! Friday thinks before it speaks.
//! attention_score = novelty × risk × strategic_relevance × uncertainty × temporal_pressure
use colored::Colorize;
use rusqlite::Connection;

pub const ATTENTION_THRESHOLD: f64 = 0.6;
pub const INTERRUPT_THRESHOLD: f64 = 0.9;

#[derive(Debug, Clone)]
pub struct AttentionEvent {
    pub event_type: String,
    pub event_detail: String,
    pub novelty: f64,
    pub risk: f64,
    pub strategic_relevance: f64,
    pub uncertainty: f64,
    pub temporal_pressure: f64,
}

impl AttentionEvent {
    pub fn score(&self) -> f64 {
        // Geometric mean of all 5 dimensions -- all must be elevated for high score
        (self.novelty
            * self.risk
            * self.strategic_relevance
            * self.uncertainty
            * self.temporal_pressure)
            .powf(0.2) // 5th root = geometric mean
    }

    pub fn should_speak(&self) -> bool {
        self.score() >= ATTENTION_THRESHOLD
    }

    pub fn should_interrupt(&self) -> bool {
        self.score() >= INTERRUPT_THRESHOLD
    }
}

/// Compute novelty -- has Friday seen this exact pattern before?
pub fn compute_novelty(db: &Connection, event_type: &str, event_detail: &str) -> f64 {
    let count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM friday_attention WHERE event_type = ?1 AND event_detail LIKE ?2",
            rusqlite::params![
                event_type,
                format!("%{}%", &event_detail[..event_detail.len().min(20)])
            ],
            |r| r.get(0),
        )
        .unwrap_or(0);
    // First time: 1.0, decays with repetition
    match count {
        0 => 1.0,
        1..=2 => 0.7,
        3..=5 => 0.5,
        6..=10 => 0.3,
        _ => 0.1,
    }
}

/// Compute risk -- what is the potential downside if Friday is wrong or silent?
pub fn compute_risk(event_type: &str, event_detail: &str) -> f64 {
    let detail_lower = event_detail.to_lowercase();
    match event_type {
        "health_drop" => {
            // Extract health value if present
            if detail_lower.contains("below 80") || detail_lower.contains("critical") {
                1.0
            } else if detail_lower.contains("below 90") {
                0.9
            } else {
                0.7
            }
        }
        "schema_change" => 0.9,
        "commit_without_intent" => 0.6,
        "contradiction" => 0.8,
        "deploy" => {
            if detail_lower.contains("core") || detail_lower.contains("faelight-shell") {
                0.5
            } else {
                0.2
            }
        }
        "pattern_match" => 0.4,
        "idle_timeout" => 0.1,
        "routine_check" => 0.15,
        _ => 0.3,
    }
}

/// Compute strategic relevance -- does this relate to the active intent?
pub fn compute_strategic_relevance(db: &Connection, event_detail: &str) -> f64 {
    // Get active intent
    // INT-071: read focus.toml (written by cistart, source of truth) first; the
    // shell_state focus_intent row went stale at the NixOS migration. Matches
    // friday-chat and faelight-shell. Without this, every event scored at the
    // no-active-intent baseline even during focused work.
    let active_intent: Option<String> = {
        let home = std::env::var("HOME").unwrap_or_default();
        let focus_file =
            std::path::PathBuf::from(&home).join(".local/state/0-core/intent/focus.toml");
        let from_toml = std::fs::read_to_string(&focus_file)
            .ok()
            .and_then(|c| {
                c.lines().find_map(|line| {
                    line.strip_prefix("id = ")
                        .map(|r| r.trim().trim_matches('"').to_string())
                })
            })
            .filter(|s| !s.is_empty());
        from_toml.or_else(|| {
            db.query_row(
                "SELECT value FROM shell_state WHERE key = 'focus_intent'",
                [],
                |r| r.get(0),
            )
            .ok()
        })
    };

    match active_intent {
        None => 0.3, // No active intent -- lower relevance
        Some(intent) => {
            let detail_lower = event_detail.to_lowercase();
            let intent_lower = intent.to_lowercase();
            // Check if event mentions anything in the active intent domain
            if intent_lower
                .split_whitespace()
                .any(|w| w.len() > 4 && detail_lower.contains(w))
            {
                0.9
            } else if detail_lower.contains("core") || detail_lower.contains("friday") {
                0.6
            } else {
                0.2
            }
        }
    }
}

/// Compute temporal pressure -- is time a factor?
pub fn compute_temporal_pressure(db: &Connection) -> f64 {
    // Check session activity -- how long has session been active?
    let session_events: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM events WHERE timestamp > strftime('%s','now') - 3600",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if session_events > 50 {
        0.6
    }
    // Active deep session
    else if session_events > 20 {
        0.4
    }
    // Normal session
    else if session_events > 5 {
        0.3
    }
    // Light session
    else {
        0.1
    } // Idle / just started
}

/// Record attention event to state.db
pub fn record_attention(
    db: &Connection,
    event: &AttentionEvent,
    spoke: bool,
    intent_id: Option<i64>,
    session_id: Option<&str>,
) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let score = event.score();
    let _ = db.execute(
        "INSERT INTO friday_attention
         (timestamp, event_type, event_detail, novelty, risk, strategic_relevance,
          uncertainty, temporal_pressure, attention_score, threshold, spoke, intent_id, session_id)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
        rusqlite::params![
            now,
            &event.event_type,
            &event.event_detail,
            event.novelty,
            event.risk,
            event.strategic_relevance,
            event.uncertainty,
            event.temporal_pressure,
            score,
            ATTENTION_THRESHOLD,
            if spoke { 1 } else { 0 },
            intent_id,
            session_id
        ],
    );
}

/// Get attention stats for friday status output
pub fn attention_stats(db: &Connection) -> (i64, i64, f64) {
    let total: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_attention", [], |r| r.get(0))
        .unwrap_or(0);
    let spoke: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM friday_attention WHERE spoke = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let avg_score: f64 = db
        .query_row(
            "SELECT AVG(attention_score) FROM friday_attention",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0.0);
    (total, spoke, avg_score)
}

/// Show attention debug output
pub fn show_debug(db: &Connection) {
    println!();
    println!(
        "  {} {}",
        "🌲 Friday Attention Debug".bright_green().bold(),
        "Core v24".dimmed()
    );
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    let (total, spoke, avg) = attention_stats(db);
    let silence_rate = if total > 0 {
        100.0 * (total - spoke) as f64 / total as f64
    } else {
        0.0
    };
    println!("  {} Events evaluated: {}", "→".bright_cyan(), total);
    println!(
        "  {} Spoke: {} ({:.0}% silence rate)",
        "→".bright_cyan(),
        spoke,
        silence_rate
    );
    println!("  {} Avg attention score: {:.3}", "→".bright_cyan(), avg);
    println!(
        "  {} Threshold: {:.1} (speak) / {:.1} (interrupt)",
        "→".bright_cyan(),
        ATTENTION_THRESHOLD,
        INTERRUPT_THRESHOLD
    );
    println!();
    // Recent events
    println!("  {} Recent events:", "→".bright_cyan());
    let mut stmt = db
        .prepare(
            "SELECT event_type, attention_score, spoke, event_detail
         FROM friday_attention ORDER BY timestamp DESC LIMIT 5",
        )
        .unwrap();
    let _ = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, f64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .unwrap()
        .flatten()
        .for_each(|(etype, score, spoke, detail)| {
            let spoke_str = if spoke == 1 {
                "spoke".bright_yellow()
            } else {
                "silent".dimmed()
            };
            let score_color = if score >= ATTENTION_THRESHOLD {
                format!("{:.3}", score).bright_yellow()
            } else {
                format!("{:.3}", score).dimmed()
            };
            println!(
                "    {} {} {} {}",
                score_color,
                spoke_str,
                etype.bright_cyan(),
                detail[..detail.len().min(40)].dimmed()
            );
        });
    println!();
}
