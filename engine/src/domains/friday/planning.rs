//! INT-234 -- Core v21: Friday Planning Layer
//! Session-aware context, forward-chaining inference, anticipation.
//! v20 predicts across the forest. v21 predicts within the session.
use crate::app::context::AppContext;
use crate::errors::CoreResult;
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
static INIT_TABLES: &str = "
CREATE TABLE IF NOT EXISTS friday_session_context (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id      TEXT NOT NULL,
    timestamp       INTEGER NOT NULL,
    exchange_kind   TEXT NOT NULL,
    content         TEXT NOT NULL,
    references_id   INTEGER,
    facts_cited     TEXT NOT NULL DEFAULT '',
    confidence      REAL NOT NULL DEFAULT 0.0,
    approved        INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_friday_session_ctx_session
    ON friday_session_context(session_id, timestamp);
";
pub fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(INIT_TABLES)?;
    Ok(())
}
/// Generate a session ID in YYYYMMDD-HHMMSS-pid format.
/// Sortable, human-readable, unique per fsh launch.
fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let secs_of_day = secs % 86400;
    let hh = secs_of_day / 3600;
    let mm = (secs_of_day % 3600) / 60;
    let ss = secs_of_day % 60;
    let mut y = 1970i64;
    let mut d = days as i64;
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let year_len = if leap { 366 } else { 365 };
        if d < year_len { break; }
        d -= year_len;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let months = [31, if leap {29} else {28}, 31,30,31,30,31,31,30,31,30,31];
    let mut m = 0usize;
    while m < 12 && d >= months[m] as i64 {
        d -= months[m] as i64;
        m += 1;
    }
    let pid = std::process::id();
    format!("{:04}{:02}{:02}-{:02}{:02}{:02}-{}", y, m + 1, d + 1, hh, mm, ss, pid)
}
/// Internal: start a new session. Returns the new session_id.
/// Does not print -- callers handle messaging.
fn start_session_internal(ctx: &AppContext) -> CoreResult<String> {
    let db = &ctx.runtime.db;
    let now = now_ts();
    let session_id = generate_session_id();
    db.execute(
        "INSERT OR REPLACE INTO friday_state (key, value, updated_at) VALUES ('current_session_id', ?1, ?2)",
        rusqlite::params![session_id, now],
    )?;
    db.execute(
        "INSERT INTO friday_session_context \
         (session_id, timestamp, exchange_kind, content, confidence) \
         VALUES (?1, ?2, 'signal', 'session_start', 1.0)",
        rusqlite::params![session_id, now],
    )?;
    Ok(session_id)
}
/// Internal: write a summary row for a session to friday_knowledge.
/// Does not clear current_session_id -- caller decides.
fn write_session_summary(ctx: &AppContext, sid: &str) -> CoreResult<()> {
    let db = &ctx.runtime.db;
    let now = now_ts();
    let mut stmt = db.prepare(
        "SELECT exchange_kind, content, confidence FROM friday_session_context \
         WHERE session_id = ?1 ORDER BY confidence DESC, timestamp DESC LIMIT 3",
    )?;
    let rows: Vec<(String, String, f64)> = stmt
        .query_map(rusqlite::params![sid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .filter_map(|r| r.ok())
        .collect();
    let summary = if rows.is_empty() {
        format!("session {} ended -- no exchanges", sid)
    } else {
        let parts: Vec<String> = rows.iter()
            .map(|(k, c, conf)| format!("[{}] {} ({:.0}%)", k, c, conf * 100.0))
            .collect();
        format!("session {} summary -- {}", sid, parts.join(" | "))
    };
    db.execute(
        "INSERT INTO friday_knowledge (domain, fact, confidence, source, created_at, updated_at) \
         VALUES ('session_summary', ?1, 0.8, 'planning', ?2, ?2)",
        rusqlite::params![summary, now],
    )?;
    Ok(())
}
/// core friday session-start -- manual session start.
pub fn session_start(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let session_id = start_session_internal(ctx)?;
    println!("  🌿 session {} started", session_id);
    Ok(())
}
/// Lazy session management: called from friday::ensure_tables on every friday command.
/// - If no current session: starts one.
/// - If current session idle > 30 min: writes summary, starts fresh session.
/// - If session active: silent no-op.
pub fn maybe_roll_session(ctx: &AppContext) -> CoreResult<()> {
    let db = &ctx.runtime.db;
    let now = now_ts();
    const IDLE_SECS: i64 = 30 * 60;
    let current: Option<String> = db.query_row(
        "SELECT value FROM friday_state WHERE key = 'current_session_id'",
        [], |r| r.get(0),
    ).ok();
    match current {
        None => {
            // No session. Start one.
            let sid = start_session_internal(ctx)?;
            println!("  🌿 session {} started (auto)", sid);
        }
        Some(sid) => {
            // Check last exchange timestamp in this session.
            let last_ts: i64 = db.query_row(
                "SELECT COALESCE(MAX(timestamp), 0) FROM friday_session_context WHERE session_id = ?1",
                rusqlite::params![sid], |r| r.get(0),
            ).unwrap_or(0);
            if now - last_ts > IDLE_SECS {
                // Stale -- summarize and roll.
                write_session_summary(ctx, &sid)?;
                db.execute(
                    "DELETE FROM friday_state WHERE key = 'current_session_id'",
                    [],
                )?;
                let new_sid = start_session_internal(ctx)?;
                println!("  🌿 session {} ended (idle), session {} started (auto)", sid, new_sid);
            }
            // else: active session, silent.
        }
    }
    Ok(())
}
/// core friday session-end -- manual session end.
pub fn session_end(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let session_id: Option<String> = db.query_row(
        "SELECT value FROM friday_state WHERE key = 'current_session_id'",
        [], |r| r.get(0),
    ).ok();
    let Some(sid) = session_id else {
        println!("  no active session");
        return Ok(());
    };
    write_session_summary(ctx, &sid)?;
    db.execute(
        "DELETE FROM friday_state WHERE key = 'current_session_id'",
        [],
    )?;
    println!("  🌿 session {} ended, summary written", sid);
    Ok(())
}
/// core friday context -- show current session buffer (last 10 exchanges).
/// If no active session, say so.
pub fn context(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    use colored::*;
    let db = &ctx.runtime.db;
    let session_id: Option<String> = db.query_row(
        "SELECT value FROM friday_state WHERE key = 'current_session_id'",
        [], |r| r.get(0),
    ).ok();
    let Some(sid) = session_id else {
        println!();
        println!("  {} Friday -- Session Context", "🌲".normal());
        println!("  {}", "━".repeat(50).dimmed());
        println!();
        println!("  {} No active session.", "💡".dimmed());
        println!("  {} A session will start automatically on the next friday command.", "→".dimmed());
        println!();
        return Ok(());
    };
    // Pull last 10 exchanges in chronological order (oldest first, so buffer reads top-to-bottom)
    let rows: Vec<(i64, String, String, f64, Option<i64>, String)> = {
        let mut stmt = db.prepare(
            "SELECT id, exchange_kind, content, confidence, references_id, facts_cited \
             FROM friday_session_context \
             WHERE session_id = ?1 \
             ORDER BY timestamp DESC LIMIT 10",
        )?;
        let v: Vec<_> = stmt.query_map(rusqlite::params![sid], |r| Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, f64>(3)?,
            r.get::<_, Option<i64>>(4)?,
            r.get::<_, String>(5)?,
        )))?.filter_map(|r| r.ok()).collect();
        v
    };
    println!();
    println!("  {} Friday -- Session Context", "🌲".normal());
    println!("  {}", "━".repeat(50).dimmed());
    println!();
    println!("  {:<28} {}", "Session:".dimmed(), sid.bright_white());
    println!("  {:<28} {}", "Exchanges:".dimmed(), rows.len().to_string().bright_white());
    println!();
    if rows.is_empty() {
        println!("  {} No exchanges recorded in this session yet.", "💡".dimmed());
        println!();
        return Ok(());
    }
    println!("  {} Last {} exchanges (newest first):", "→".bright_cyan(), rows.len());
    // Iterate newest-first (which is the query order)
    for (id, kind, content, conf, refs, facts) in &rows {
        let conf_str = if *conf > 0.0 {
            format!("{:.0}%", conf * 100.0)
        } else {
            "--".to_string()
        };
        let kind_colored = match kind.as_str() {
            "ask"          => kind.bright_yellow(),
            "observation"  => kind.bright_cyan(),
            "anticipation" => kind.bright_magenta(),
            "conclusion"   => kind.bright_green(),
            "signal"       => kind.dimmed(),
            _              => kind.white(),
        };
        let short = content.chars().take(72).collect::<String>();
        println!("    {} #{:<4} [{}] {} {}",
            "·".dimmed(),
            id.to_string().dimmed(),
            kind_colored,
            short.white(),
            format!("({})", conf_str).dimmed(),
        );
        if let Some(r) = refs {
            println!("         {} cites exchange #{}", "↳".dimmed(), r.to_string().dimmed());
        }
        if !facts.is_empty() {
            println!("         {} facts: {}", "↳".dimmed(), facts.dimmed());
        }
    }
    println!();
    Ok(())
}

// ─── Forward-Chaining Inference (Gate 6) ─────────────────────────────────
// Templates combine at least one knowledge fact with at least one live
// observation to derive a conclusion. Conclusions are stored in
// friday_session_context with exchange_kind='conclusion' and facts_cited.
/// Parse "health:N%" out of a friday_observations.content string.
/// Returns None if not present or not parseable.
fn parse_health_from_content(content: &str) -> Option<u32> {
    let marker = "health:";
    let idx = content.find(marker)?;
    let rest = &content[idx + marker.len()..];
    let end = rest.find('%')?;
    rest[..end].trim().parse::<u32>().ok()
}
/// Write a conclusion row to friday_session_context for the current session.
/// Returns Ok(()) if written or no active session (silent no-op).
fn write_conclusion(
    ctx: &AppContext,
    content: &str,
    facts_cited: &str,
    confidence: f64,
) -> CoreResult<()> {
    let db = &ctx.runtime.db;
    let sid: Option<String> = db.query_row(
        "SELECT value FROM friday_state WHERE key = 'current_session_id'",
        [], |r| r.get(0),
    ).ok();
    let Some(sid) = sid else { return Ok(()); };
    let now = now_ts();
    db.execute(
        "INSERT INTO friday_session_context \
         (session_id, timestamp, exchange_kind, content, facts_cited, confidence) \
         VALUES (?1, ?2, 'conclusion', ?3, ?4, ?5)",
        rusqlite::params![sid, now, content, facts_cited, confidence],
    )?;
    Ok(())
}
/// Template 1: Health Threshold Breach
fn check_health_threshold(ctx: &AppContext) -> CoreResult<Option<String>> {
    let db = &ctx.runtime.db;
    let recent: Option<String> = db.query_row(
        "SELECT content FROM friday_observations \
         WHERE kind = 'command' AND content LIKE '%health:%' \
         ORDER BY timestamp DESC LIMIT 1",
        [], |r| r.get(0),
    ).ok();
    let Some(content) = recent else { return Ok(None); };
    let Some(health) = parse_health_from_content(&content) else { return Ok(None); };
    if health < 95 {
        let msg = format!(
            "Current health at {}% -- below the 95% ship floor established by facts #77 and #60. Investigate before shipping.",
            health
        );
        return Ok(Some(msg));
    }
    Ok(None)
}
/// Template 2: Session Velocity
fn check_session_velocity(ctx: &AppContext) -> CoreResult<Option<String>> {
    let db = &ctx.runtime.db;
    let now = now_ts();
    let day_ago = now - 86400;
    let today_commits: i64 = db.query_row(
        "SELECT COUNT(*) FROM events \
         WHERE domain = 'git' AND action = 'commit' AND timestamp > ?1",
        rusqlite::params![day_ago], |r| r.get(0),
    ).unwrap_or(0);
    let total_commits_text: Option<String> = db.query_row(
        "SELECT fact FROM friday_knowledge WHERE domain = 'forest' AND key = 'forest_stats'",
        [], |r| r.get(0),
    ).ok();
    let total_commits: i64 = total_commits_text
        .and_then(|s| {
            let marker = "representing ";
            let idx = s.find(marker)?;
            let rest = &s[idx + marker.len()..];
            let end = rest.find(' ')?;
            rest[..end].parse::<i64>().ok()
        })
        .unwrap_or(0);
    if today_commits >= 20 {
        let pct = if total_commits > 0 {
            (today_commits * 100) / total_commits
        } else { 0 };
        let msg = format!(
            "{} commits today -- high velocity. {}% of total forest commits ({}) in one day. Per facts #255 and #256, this session exceeds sustainable cadence.",
            today_commits, pct, total_commits
        );
        return Ok(Some(msg));
    } else if today_commits >= 10 {
        let msg = format!(
            "{} commits today -- elevated velocity. Per facts #255 and #256, the session is above typical cadence.",
            today_commits
        );
        return Ok(Some(msg));
    }
    Ok(None)
}
/// Template 3: Intent State Drift
fn check_intent_drift(ctx: &AppContext) -> CoreResult<Option<String>> {
    let db = &ctx.runtime.db;
    let now = now_ts();
    let day_ago = now - 86400;
    let recent_lifecycle: i64 = db.query_row(
        "SELECT COUNT(*) FROM shell_history \
         WHERE timestamp > ?1 \
         AND (command LIKE 'cistart%' OR command LIKE 'cicomplete%' \
              OR command LIKE 'dc %' OR command LIKE 'ds %')",
        rusqlite::params![day_ago], |r| r.get(0),
    ).unwrap_or(0);
    if recent_lifecycle > 0 {
        return Ok(None);
    }
    let intents_dir = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join("0-core/intents/future");
    let in_progress_count = if let Ok(entries) = std::fs::read_dir(&intents_dir) {
        entries.filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
            .filter(|e| {
                std::fs::read_to_string(e.path())
                    .map(|c| c.contains("status: in-progress"))
                    .unwrap_or(false)
            })
            .count()
    } else {
        0
    };
    if in_progress_count > 0 {
        let msg = format!(
            "{} intents marked in-progress but no cistart/cicomplete activity in 24h. Per facts #158 and #159, intent state may be stale -- review intent ledger.",
            in_progress_count
        );
        return Ok(Some(msg));
    }
    Ok(None)
}
/// core friday infer -- run forward-chaining inference across all templates.
pub fn infer(ctx: &AppContext, verbose: bool) -> CoreResult<()> {
    ensure_tables(ctx)?;
    use colored::*;
    println!();
    println!("  {} Friday -- Forward-Chaining Inference", "🌲".normal());
    println!("  {}", "━".repeat(50).dimmed());
    println!();
    let templates: Vec<(&str, &str, fn(&AppContext) -> CoreResult<Option<String>>, &str, f64)> = vec![
        ("health_threshold", "health < 95% breach", check_health_threshold as fn(&AppContext) -> CoreResult<Option<String>>, "77,60", 0.95),
        ("session_velocity", "elevated/high commit velocity", check_session_velocity as fn(&AppContext) -> CoreResult<Option<String>>, "255,256", 0.9),
        ("intent_drift",     "stale in-progress intents", check_intent_drift as fn(&AppContext) -> CoreResult<Option<String>>, "158,159", 0.85),
    ];
    let mut fired = 0;
    for (name, desc, check_fn, facts, conf) in &templates {
        let result = check_fn(ctx)?;
        match result {
            Some(conclusion) => {
                write_conclusion(ctx, &conclusion, facts, *conf)?;
                println!("  {} {} -- FIRED", "✓".bright_green(), name.bright_white());
                println!("    {} {}", "→".bright_cyan(), conclusion.white());
                println!("    {} facts_cited: {}  confidence: {:.0}%", "·".dimmed(), facts.dimmed(), conf * 100.0);
                println!();
                fired += 1;
            }
            None => {
                if verbose {
                    println!("  {} {} -- not fired ({})", "·".dimmed(), name.dimmed(), desc.dimmed());
                }
            }
        }
    }
    if fired == 0 && !verbose {
        println!("  {} No conclusions drawn. Conditions did not meet thresholds.", "💡".dimmed());
        println!("  {} Run with --verbose to see template evaluations.", "→".dimmed());
    } else if fired > 0 {
        println!("  {} {} conclusion(s) written to session context.", "🌲".normal(), fired);
    }
    println!();
    Ok(())
}
