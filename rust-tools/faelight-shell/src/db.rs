// faelight-shell — state.db connection
use anyhow::{Context, Result};
use rusqlite::Connection;
use rustyline::{history::FileHistory, Editor, Helper};
use std::path::PathBuf;

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

        // Ensure shell tables exist
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS shell_history (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                command   TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS shell_aliases (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                name      TEXT NOT NULL UNIQUE,
                command   TEXT NOT NULL,
                created   INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS shell_state (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )?;

        Ok(Self { conn, core_root })
    }

    pub fn core_root(&self) -> String {
        self.core_root.clone()
    }

    pub fn load_history<H: Helper>(&self, rl: &mut Editor<H, FileHistory>) {
        if let Ok(mut stmt) = self
            .conn
            .prepare("SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 100")
        {
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

    pub fn add_alias(&self, name: &str, command: &str) -> bool {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        self.conn
            .execute(
                "INSERT OR REPLACE INTO shell_aliases (name, command, created) VALUES (?1, ?2, ?3)",
                rusqlite::params![name, command, ts],
            )
            .is_ok()
    }

    pub fn remove_alias(&self, name: &str) -> bool {
        self.conn
            .execute(
                "DELETE FROM shell_aliases WHERE name = ?1",
                rusqlite::params![name],
            )
            .map(|n| n > 0)
            .unwrap_or(false)
    }

    pub fn get_alias(&self, name: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT command FROM shell_aliases WHERE name = ?1",
                rusqlite::params![name],
                |r| r.get(0),
            )
            .ok()
    }

    pub fn list_aliases(&self) -> Vec<(String, String)> {
        let mut stmt = match self
            .conn
            .prepare("SELECT name, command FROM shell_aliases ORDER BY name")
        {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    }

    pub fn load_plugins(&self) -> Vec<(String, String, String)> {
        // Returns Vec<(command_name, expansion, description)>
        let plugin_dir = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".config/faelight-shell/plugins");

        if !plugin_dir.exists() {
            return vec![];
        }

        let mut commands = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "fsh").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(parsed) = toml::from_str::<toml::Value>(&content) {
                            if let Some(cmds) = parsed.get("command").and_then(|c| c.as_array()) {
                                for cmd in cmds {
                                    let name = cmd
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let expand = cmd
                                        .get("expand")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let desc = cmd
                                        .get("description")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    if !name.is_empty() && !expand.is_empty() {
                                        commands.push((name, expand, desc));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        commands
    }

    pub fn save_history_entry(&self, command: &str) -> rusqlite::Result<()> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        self.conn
            .execute(
                "INSERT INTO shell_history (command, timestamp) VALUES (?1, ?2)",
                rusqlite::params![command, ts],
            )?;
        Ok(())
    }

    pub fn get_last_command(&self) -> Option<String> {
        self.conn
            .query_row(
                "SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .ok()
    }
    pub fn get_command_matching(&self, pattern: &str) -> Option<String> {
        let like = format!("%{}%", pattern);
        self.conn
            .query_row(
                "SELECT command FROM shell_history WHERE command LIKE ?1 ORDER BY timestamp DESC LIMIT 1",
                rusqlite::params![like],
                |r| r.get(0),
            )
            .ok()
    }

    pub fn query_events(
        &self,
        domain: Option<&str>,
        today_only: bool,
        limit: usize,
    ) -> Vec<(String, String, i64)> {
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
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    }

    pub fn set_focus_intent(&self, intent: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('focus_intent', ?1)",
            rusqlite::params![intent],
        )?;
        Ok(())
    }

    pub fn get_focus_intent(&self) -> Option<String> {
        self.conn
            .query_row(
                "SELECT value FROM shell_state WHERE key='focus_intent'",
                [],
                |r| r.get::<_, String>(0),
            )
            .ok()
    }

    pub fn set_theme(&self, theme: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('prompt_theme', ?1)",
            rusqlite::params![theme],
        )?;
        Ok(())
    }

    pub fn get_theme(&self) -> String {
        self.conn.query_row(
            "SELECT value FROM shell_state WHERE key='prompt_theme'",
            [],
            |r| r.get(0),
        ).unwrap_or_else(|_| "forest".to_string())
    }

    pub fn clear_focus_intent(&self) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM shell_state WHERE key='focus_intent'",
            [],
        )?;
        Ok(())
    }

    pub fn health_score(&self) -> Option<i64> {
        self.conn
            .query_row(
                "SELECT payload FROM events WHERE domain='doctor' ORDER BY timestamp DESC LIMIT 1",
                [],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .and_then(|p| serde_json::from_str::<serde_json::Value>(&p).ok())
            .and_then(|v| v["detail"]["health"].as_i64())
    }
}
