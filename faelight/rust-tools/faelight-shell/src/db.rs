#![allow(clippy::all)]
// faelight-shell — state.db connection
use anyhow::{Context, Result};
use rusqlite::Connection;
use rustyline::{history::FileHistory, Editor, Helper};

pub struct ForestDb {
    pub conn: Connection,
    pub core_root: String,
}

/// INT-191: the SQLite boundary for an execution id.
///
/// ⚠️ Saturates rather than panicking -- telemetry must never take the shell down -- but LOGS,
/// because a silent narrowing is a data mutation. Note i64::MAX is itself a valid integer, so
/// without the log a saturated value would be indistinguishable from a real one.
fn clamp_execution_id(id: u64) -> i64 {
    i64::try_from(id).unwrap_or_else(|_| {
        eprintln!("warning: execution_id {id} exceeds SQLite INTEGER range, saturating");
        i64::MAX
    })
}

/// INT-191: the lifecycle states of a command execution. Constants because callers would
/// otherwise write bare literals and recreate the drift problem at smaller scale, in a table
/// meant to be authoritative.
pub const EXEC_STARTED: &str = "started";
pub const EXEC_OK: &str = "ok";
pub const EXEC_ERROR: &str = "error";
pub const EXEC_EXIT: &str = "exit";
pub const EXEC_EMPTY: &str = "empty";
pub const EXEC_BLOCKED: &str = "blocked";

/// INT-191: the facts known when an execution BEGINS.
///
/// ⚠️ A named-field struct rather than loose parameters: `session_id`, `typed_text` and `cwd`
/// are all `&str`, so as positional arguments any two could be swapped with the compiler
/// silent -- and a swapped identity is precisely the defect class this table exists to end.
pub struct ExecutionStart<'a> {
    pub session_id: &'a str,
    pub execution_id: u64,
    pub typed_text: &'a str,
    pub cwd: &'a str,
    pub intent_id: Option<&'a str>,
    pub started_at: i64,
}

/// INT-191: the facts known only when an execution ENDS.
///
/// ⚠️ Separate from `ExecutionStart` because they have different OWNERS. postexec knows the
/// executed form; only the caller knows the final exit code, since the pipeline arms decide it
/// after `execute_with_context` returns. One `save_` method would hide that boundary.
pub struct ExecutionCompletion<'a> {
    pub session_id: &'a str,
    pub execution_id: u64,
    pub executed_text: Option<&'a str>,
    pub state: &'a str,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    pub finished_at: i64,
}

impl ForestDb {
    pub fn open() -> Result<Self> {
        // INT-061: derive both core_root and the db path from the single path
        // authority (paths.rs), not local format!/join. core_root is retained --
        // it is stored on ForestDb and exposed via core_root() for git ops etc.
        let core_root = faelight_core::paths::core_root_string();
        let db_path = faelight_core::paths::state_db();

        // Self-heal: ensure the runtime dir exists so a fresh environment (VM,
        // recovery shell, new machine) can create state.db instead of fsh dying
        // at startup. SQLite creates the .db file itself; it cannot create the dir.
        // The CREATE TABLE IF NOT EXISTS schema below then initialises a fresh db.
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Cannot open state.db at {:?}", db_path))?;
        // INT-249b: checkpoint WAL on startup so morning-after-suspend doesn't
        // hit SQLITE_READONLY from accumulated WAL frames. Errors ignored - if
        // checkpoint fails, normal operation continues; retry logic handles transients.
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)");

        // INT-104: command_snapshots -- destructive-command audit ledger (INT-322 Phase 4).
        // Split out of shell_snapshots: distinct purpose (pre-destructive command context),
        // its own authoritative schema. No ALTER-patching of shell_snapshots (fresh-db safe).
        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS command_snapshots (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                name      TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                health    INTEGER,
                command   TEXT,
                git_hash  TEXT,
                cwd       TEXT,
                intent_id TEXT
            );",
        );

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

        // INT-101: enrich shell_history AFTER its CREATE above, so a FRESH db
        // already has the table -- ALTERs succeed (no "no column named cwd"
        // warning on first run) and no-op on existing dbs. (INT-250 columns.)
        let _ = conn.execute_batch("ALTER TABLE shell_history ADD COLUMN cwd TEXT");
        let _ = conn.execute_batch("ALTER TABLE shell_history ADD COLUMN exit_code INTEGER");
        let _ = conn.execute_batch("ALTER TABLE shell_history ADD COLUMN duration_ms INTEGER");
        let _ = conn.execute_batch("ALTER TABLE shell_history ADD COLUMN intent_id TEXT");
        // INT-191: the command lifecycle table. `shell_history` carries THREE concepts at once --
        // submission ("the user entered this"), execution ("the producer ran this"), and enrichment
        // ("attach exit/duration to the thing we tracked") -- and the enrichment lands on the wrong
        // one: measured 2026-07-26, the table says `c` exited 0 in 96ms when the process that ran
        // was `clear`. 50,293 rows carry completion metadata and at least 15,957 of them are bare
        // alias names, which is a floor rather than the rate.
        //
        // This table is born correct instead of being repaired. The key is the PAIR: `execution_id`
        // restarts at 1 in every shell process, so alone it is not an identity -- two sessions would
        // both claim 1, 2, 3. `session_id` supplies the process boundary.
        //
        // ⚠️ NULLABLE ON PURPOSE. `executed_text` is null when the command never reached expansion
        // (blocked by the safety guard). `exit_code` is null when the lifecycle had no process exit.
        // `finished_at` is null while running -- and a row left in state `started` is EVIDENCE that
        // the shell died mid-command, not a gap to be filled in later.
        //
        // No trigger. INT-134's trigger suits immutable auditing of an existing write stream; here
        // the point is the opposite -- define the lifecycle owner and let storage follow it.
        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS command_execution (
                session_id      TEXT    NOT NULL,
                execution_id    INTEGER NOT NULL,
                typed_text      TEXT    NOT NULL,
                executed_text   TEXT,
                execution_state TEXT    NOT NULL,
                exit_code       INTEGER,
                duration_ms     INTEGER,
                started_at      INTEGER NOT NULL,
                finished_at     INTEGER,
                cwd             TEXT,
                intent_id       TEXT,
                PRIMARY KEY (session_id, execution_id)
            );",
        );
        // INT-134: immutable command audit log. A separate append-only table
        // captures every REAL command (internal SUGGEST:/TIMING:/doctor-test rows
        // excluded). Auto-populated by an AFTER INSERT trigger on shell_history, so
        // capture rides on the existing insert -- no write-path change, can't drift.
        // UPDATE/DELETE on the audit table are blocked by triggers -> true immutability,
        // demonstrable (try to delete an audit row -> SQLite ABORTs).
        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS shell_history_audit (
                audit_id   INTEGER PRIMARY KEY AUTOINCREMENT,
                command    TEXT NOT NULL,
                timestamp  INTEGER NOT NULL,
                cwd        TEXT,
                intent_id  TEXT,
                audited_at INTEGER NOT NULL
            );
            CREATE TRIGGER IF NOT EXISTS trg_audit_capture
            AFTER INSERT ON shell_history
            WHEN NEW.command NOT LIKE 'SUGGEST:%'
                 AND NEW.command NOT LIKE 'TIMING:%'
                 AND NEW.command <> '__fsh_doctor_test__'
            BEGIN
                INSERT INTO shell_history_audit (command, timestamp, cwd, intent_id, audited_at)
                VALUES (NEW.command, NEW.timestamp, NEW.cwd, NEW.intent_id, strftime('%s','now'));
            END;
            CREATE TRIGGER IF NOT EXISTS trg_audit_no_update
            BEFORE UPDATE ON shell_history_audit
            BEGIN
                SELECT RAISE(ABORT, 'shell_history_audit is immutable: updates not permitted');
            END;
            CREATE TRIGGER IF NOT EXISTS trg_audit_no_delete
            BEFORE DELETE ON shell_history_audit
            BEGIN
                SELECT RAISE(ABORT, 'shell_history_audit is immutable: deletes not permitted');
            END;",
        );
        // INT-322 Phase 2: command failures table for Friday learning
        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS command_failures (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                command   TEXT NOT NULL,
                exit_code INTEGER NOT NULL,
                cwd       TEXT,
                timestamp INTEGER NOT NULL
            );",
        );

        Ok(Self { conn, core_root })
    }

    pub fn core_root(&self) -> String {
        self.core_root.clone()
    }

    pub fn load_history<H: Helper>(&self, rl: &mut Editor<H, FileHistory>) {
        if let Ok(mut stmt) = self
            .conn
            .prepare("SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 10000")
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

    pub fn save_history_entry(&self, command: &str) -> rusqlite::Result<i64> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let cwd = std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(String::from));
        // INT-249b: retry on transient SQLite errors (BUSY, LOCKED) with backoff.
        // Avoids noisy warnings during WAL contention (e.g. just after boot, while
        // multiple forest processes are checkpointing).
        let max_attempts = 3;
        let mut last_err: Option<rusqlite::Error> = None;
        for attempt in 0..max_attempts {
            match self.conn.execute(
                "INSERT INTO shell_history (command, timestamp, cwd, intent_id) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![command, ts, cwd, self.get_focus_intent()],
            ) {
                Ok(_) => return Ok(self.conn.last_insert_rowid()),
                Err(e) => {
                    let transient = matches!(
                        &e,
                        rusqlite::Error::SqliteFailure(err, _)
                            if err.code == rusqlite::ErrorCode::DatabaseBusy
                                || err.code == rusqlite::ErrorCode::DatabaseLocked
                                || err.code == rusqlite::ErrorCode::ReadOnly
                    );
                    if transient && attempt + 1 < max_attempts {
                        std::thread::sleep(std::time::Duration::from_millis(
                            50 * (attempt as u64 + 1),
                        ));
                        last_err = Some(e);
                        continue;
                    }
                    return Err(e);
                }
            }
        }
        Err(last_err.unwrap_or(rusqlite::Error::ExecuteReturnedResults))
    }

    /// INT-250: backfill completion data (exit_code, duration_ms) for an existing
    /// history row. Called AFTER command execution. Errors silently ignored --
    /// completion data is best-effort.
    pub fn update_history_completion(
        &self,
        id: i64,
        exit_code: Option<i32>,
        duration_ms: Option<u64>,
    ) {
        let _ = self.conn.execute(
            "UPDATE shell_history SET exit_code = ?1, duration_ms = ?2 WHERE id = ?3",
            rusqlite::params![exit_code, duration_ms.map(|d| d as i64), id],
        );
    }

    /// INT-191: record that an execution BEGAN. Errors are returned rather than swallowed --
    /// `update_history_completion` above is documented best-effort, but this table is meant to be
    /// authoritative, so a caller that cannot write must be able to say so.
    ///
    /// ⚠️ Plain INSERT, not INSERT OR REPLACE. `(session_id, execution_id)` should never collide;
    /// if it somehow does, the ORIGINAL row survives and the write fails loudly, rather than
    /// evidence being silently overwritten by whatever came second.
    pub fn begin_command_execution(&self, start: &ExecutionStart) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO command_execution
                 (session_id, execution_id, typed_text, execution_state, started_at, cwd, intent_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                start.session_id,
                clamp_execution_id(start.execution_id),
                start.typed_text,
                EXEC_STARTED,
                start.started_at,
                start.cwd,
                start.intent_id,
            ],
        )?;
        Ok(())
    }

    /// INT-191: attach the facts known only at the END of an execution.
    ///
    /// ⚠️ A row left in state `started` is EVIDENCE -- the shell died mid-command -- so this method
    /// never inserts. If no `begin` happened, nothing is updated and the absence is itself true.
    pub fn complete_command_execution(&self, done: &ExecutionCompletion) -> rusqlite::Result<()> {
        let changed = self.conn.execute(
            "UPDATE command_execution
                SET executed_text = ?1, execution_state = ?2, exit_code = ?3,
                    duration_ms = ?4, finished_at = ?5
              WHERE session_id = ?6 AND execution_id = ?7",
            rusqlite::params![
                done.executed_text,
                done.state,
                done.exit_code,
                done.duration_ms.map(|d| d as i64),
                done.finished_at,
                done.session_id,
                clamp_execution_id(done.execution_id),
            ],
        )?;
        // ⚠️ Zero rows updated is NOT benign: it means begin never ran, the session mismatched, the
        // id was wrong, or the write was lost. Completion must not float unattached to a lifecycle.
        if changed != 1 {
            return Err(rusqlite::Error::StatementChangedRows(changed));
        }
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
        // Read from focus.toml (written by cistart via core engine)
        let home = std::env::var("HOME").unwrap_or_default();
        let focus_file =
            std::path::PathBuf::from(&home).join(".local/state/0-core/intent/focus.toml");
        if let Ok(content) = std::fs::read_to_string(&focus_file) {
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("id = ") {
                    return Some(rest.trim().trim_matches('"').to_string());
                }
            }
        }
        // Fallback: shell_state table
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
        self.conn
            .query_row(
                "SELECT value FROM shell_state WHERE key='prompt_theme'",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "forest".to_string())
    }

    pub fn clear_focus_intent(&self) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM shell_state WHERE key='focus_intent'", [])?;
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

    /// INT-322 Phase 4: capture a lightweight snapshot before destructive commands
    pub fn capture_snapshot(&self, command: &str, intent_id: Option<&str>) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let health = self.health_score().unwrap_or(0);
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let git_hash = std::process::Command::new("git")
            .args(["-C", &self.core_root, "rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        // INT-195: name the snapshot from the canonical quote-aware derivation, so a
        // quoted command word is attributed as auto-rm rather than auto-"rm. This site
        // governs ATTRIBUTION; main.rs's destructive-command check governs CREATION.
        // In scope only because capture_snapshot is reached solely from the execution
        // path -- if a second caller appears, revisit that.
        let word = crate::commands::command_word(command);
        let name = format!(
            "auto-{}",
            if word.is_empty() {
                "cmd"
            } else {
                word.as_str()
            }
        );
        let _ = self.conn.execute(
            "INSERT INTO command_snapshots (name, timestamp, health, command, git_hash, cwd, intent_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![name, ts, health as i64, command, git_hash, cwd, intent_id],
        );
    }
}

// INT-249: shared spawn + heredoc-leak detection for sh -c invocations
pub fn spawn_sh_with_leak_check(cmd: &str) -> std::io::Result<std::process::ExitStatus> {
    use std::io::{BufRead, BufReader, Write};
    let mut child = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()?;
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line_result in reader.lines() {
            if let Ok(out_line) = line_result {
                println!("{}", out_line);
                let trimmed = out_line.trim();
                let is_leak = trimmed.len() >= 4
                    && trimmed.ends_with("EOF")
                    && trimmed[..trimmed.len() - 3].len() >= 1
                    && trimmed[..trimmed.len() - 3]
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c == '_');
                if is_leak {
                    eprintln!("  WARN possible unclosed heredoc -- {:?} appeared as standalone output line", trimmed);
                }
                let _ = std::io::stdout().flush();
            }
        }
    }
    child.wait()
}
