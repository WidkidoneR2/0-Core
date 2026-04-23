//! INT-234 -- Core v21: Friday Planning Layer
//! Session-aware context, forward-chaining inference, anticipation.
//! v20 predicts across the forest. v21 predicts within the session.
use crate::app::context::AppContext;
use crate::errors::CoreResult;
#[allow(dead_code)]
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
