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

