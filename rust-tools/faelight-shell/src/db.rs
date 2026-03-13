// faelight-shell — state.db connection
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use rustyline::DefaultEditor;

pub struct ForestDb {
    pub conn: Connection,
    pub core_root: String,
}

impl ForestDb {
    pub fn open() -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_default();
        let core_root = format!("{}/0-core", home);
        let db_path = PathBuf::from(&core_root).join("runtime/state.db");

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Cannot open state.db at {:?}", db_path))?;

        // Ensure shell history table exists
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS shell_history (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                command   TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            );"
        )?;

        Ok(Self { conn, core_root })
    }

    pub fn core_root(&self) -> String {
        self.core_root.clone()
    }

    pub fn load_history(&self, rl: &mut DefaultEditor) {
        if let Ok(mut stmt) = self.conn.prepare(
            "SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 100"
        ) {
            let history: Vec<String> = stmt
                .query_map([], |r| r.get(0))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                .unwrap_or_default();
            // Add in reverse so most recent is at top
            for cmd in history.iter().rev() {
                let _ = rl.add_history_entry(cmd.as_str());
            }
        }
    }

    pub fn save_history_entry(&self, command: &str) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        self.conn.execute(
            "INSERT INTO shell_history (command, timestamp) VALUES (?1, ?2)",
            rusqlite::params![command, ts],
        ).ok();
    }

    pub fn query_events(&self, domain: Option<&str>, today_only: bool, limit: usize) -> Vec<(String, String, i64)> {
        let today_ts = if today_only {
            // Start of today in unix time
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            // Subtract seconds since midnight
            let secs_since_midnight = now % 86400;
            now - secs_since_midnight
        } else {
            0
        };

        let sql = match (domain, today_only) {
            (Some(d), true) => format!(
                "SELECT domain, action, timestamp FROM events WHERE domain='{}' AND timestamp >= {} ORDER BY timestamp DESC LIMIT {}",
                d, today_ts, limit
            ),
            (Some(d), false) => format!(
                "SELECT domain, action, timestamp FROM events WHERE domain='{}' ORDER BY timestamp DESC LIMIT {}",
                d, limit
            ),
            (None, true) => format!(
                "SELECT domain, action, timestamp FROM events WHERE timestamp >= {} ORDER BY timestamp DESC LIMIT {}",
                today_ts, limit
            ),
            (None, false) => format!(
                "SELECT domain, action, timestamp FROM events ORDER BY timestamp DESC LIMIT {}",
                limit
            ),
        };
        let mut stmt = match self.conn.prepare(&sql) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map([], |r| {
            Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i64>(2)?))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    }

    pub fn health_score(&self) -> Option<i64> {
        self.conn.query_row(
            "SELECT payload FROM events WHERE domain='doctor' ORDER BY timestamp DESC LIMIT 1",
            [],
            |r| r.get::<_,String>(0),
        ).ok()
        .and_then(|p| serde_json::from_str::<serde_json::Value>(&p).ok())
        .and_then(|v| v["detail"]["health"].as_i64())
    }
}
