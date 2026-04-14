//! INT-215 -- Canonical Signal struct and forest_events_v2
//! Append-only. Monotonic sequence. Schema validation. Causality chains.

/// The fundamental unit of forest knowledge
#[allow(dead_code)]
pub enum SignalKind {
    Observation,    // raw facts -- health=100, commit_made
    Interpretation, // meaning derived -- "high velocity session"
    Judgment,       // evaluation -- "health below threshold"
    Decision,       // chosen action (not yet executed)
    Proposal,       // candidate action (requires human gate)
    Outcome,        // result of execution -- feeds Friday learning
}
impl SignalKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalKind::Observation    => "observation",
            SignalKind::Interpretation => "interpretation",
            SignalKind::Judgment       => "judgment",
            SignalKind::Decision       => "decision",
            SignalKind::Proposal       => "proposal",
            SignalKind::Outcome        => "outcome",
        }
    }
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "interpretation" => SignalKind::Interpretation,
            "judgment"       => SignalKind::Judgment,
            "decision"       => SignalKind::Decision,
            "proposal"       => SignalKind::Proposal,
            "outcome"        => SignalKind::Outcome,
            _                => SignalKind::Observation,
        }
    }
}
/// Signal schema registry -- defines valid payload shapes per type_name
pub fn validate_payload(type_name: &str, payload: &str) -> Result<(), String> {
    // Parse as JSON first
    let v: serde_json::Value = serde_json::from_str(payload)
        .map_err(|e| format!("payload must be valid JSON: {}", e))?;
    match type_name {
        "health" => {
            v.get("health").and_then(|h| h.as_u64())
                .ok_or_else(|| "health payload requires {\"health\": u32}".to_string())?;
        }
        "git_commit" => {
            v.get("hash").and_then(|h| h.as_str())
                .ok_or_else(|| "git_commit payload requires {\"hash\": str, ...}".to_string())?;
        }
        "intent_start" | "intent_complete" => {
            v.get("id").and_then(|i| i.as_str())
                .ok_or_else(|| format!("{} payload requires {{\"id\": str, \"title\": str}}", type_name))?;
        }
        "deploy" => {
            v.get("tool").and_then(|t| t.as_str())
                .ok_or_else(|| "deploy payload requires {\"tool\": str, ...}".to_string())?;
        }
        "alignment" => {
            v.get("score").and_then(|s| s.as_f64())
                .ok_or_else(|| "alignment payload requires {\"score\": f64}".to_string())?;
        }
        "prediction" => {
            v.get("suggestion").and_then(|s| s.as_str())
                .ok_or_else(|| "prediction payload requires {\"suggestion\": str}".to_string())?;
        }
        "watchdog_alert" => {
            v.get("health").and_then(|h| h.as_u64())
                .ok_or_else(|| "watchdog_alert payload requires {\"health\": u32}".to_string())?;
        }
        // Unknown types allowed but warned
        _ => {}
    }
    Ok(())
}
/// Create the forest_events_v2 table
pub static CREATE_TABLE: &str = "
CREATE TABLE IF NOT EXISTS forest_events_v2 (
    seq         INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp   INTEGER NOT NULL,
    source      TEXT NOT NULL,
    kind        TEXT NOT NULL,
    type_name   TEXT NOT NULL,
    payload     TEXT NOT NULL DEFAULT '{}',
    intent_id   TEXT,
    session_id  TEXT,
    caused_by   INTEGER REFERENCES forest_events_v2(seq),
    schema_ver  INTEGER NOT NULL DEFAULT 1,
    confidence  REAL NOT NULL DEFAULT 1.0,
    weight      REAL NOT NULL DEFAULT 1.0
);
CREATE INDEX IF NOT EXISTS idx_fev2_timestamp ON forest_events_v2(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_fev2_type ON forest_events_v2(type_name);
CREATE INDEX IF NOT EXISTS idx_fev2_source ON forest_events_v2(source);
CREATE INDEX IF NOT EXISTS idx_fev2_caused_by ON forest_events_v2(caused_by);
";
/// Emit a validated signal to forest_events_v2
pub fn emit(
    db: &rusqlite::Connection,
    source: &str,
    kind: SignalKind,
    type_name: &str,
    payload: &str,
    intent_id: Option<&str>,
    caused_by: Option<i64>,
    confidence: f64,
) -> Result<i64, String> {
    // Ensure table exists
    db.execute_batch(CREATE_TABLE)
        .map_err(|e| format!("table creation failed: {}", e))?;
    // Validate payload
    validate_payload(type_name, payload)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let session_id = std::env::var("FSH_SESSION_ID").ok();
    db.execute(
        "INSERT INTO forest_events_v2 (timestamp, source, kind, type_name, payload, intent_id, session_id, caused_by, confidence)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            now, source, kind.as_str(), type_name, payload,
            intent_id, session_id.as_deref(), caused_by, confidence
        ],
    ).map_err(|e| format!("emit failed: {}", e))?;
    Ok(db.last_insert_rowid())
}
/// Read causality chain for a signal
pub fn causality_chain(db: &rusqlite::Connection, seq: i64) -> Vec<(i64, String, String, String)> {
    let mut chain = Vec::new();
    let mut current = seq;
    for _ in 0..20 {  // max depth 20 to prevent cycles
        let row: Option<(i64, String, String, Option<i64>)> = db.query_row(
            "SELECT seq, type_name, payload, caused_by FROM forest_events_v2 WHERE seq = ?1",
            rusqlite::params![current],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        ).ok();
        match row {
            Some((s, t, p, Some(parent))) => {
                chain.push((s, t, p, "→".to_string()));
                current = parent;
            }
            Some((s, t, p, None)) => {
                chain.push((s, t, p, "●".to_string()));
                break;
            }
            None => break,
        }
    }
    chain
}
