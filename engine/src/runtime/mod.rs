use crate::errors::CoreResult;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
/// Write a single event to the JSONL log file
/// Call this alongside any direct db.execute for events
pub fn write_event_log(domain: &str, action: &str, payload: &str, ts: i64) {
    let home = std::env::var("HOME").unwrap_or_default();
    let events_dir = std::path::PathBuf::from(&home).join("0-core/runtime/events");
    if !events_dir.exists() {
        if std::fs::create_dir_all(&events_dir).is_err() {
            return;
        }
    }
    let date = chrono::DateTime::from_timestamp(ts, 0)
        .map(|t| t.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let log_path = events_dir.join(format!("{}.jsonl", date));
    let mut line = String::with_capacity(256);
    line.push_str("{\"ts\":");
    line.push_str(&ts.to_string());
    line.push_str(",\"domain\":\"");
    line.push_str(domain);
    line.push_str("\",\"action\":\"");
    line.push_str(action);
    line.push_str("\",\"payload\":");
    line.push_str(payload);
    line.push_str("}\n");
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        f.write_all(line.as_bytes()).ok();
    }
}

#[allow(dead_code)]
pub struct Runtime {
    pub root: PathBuf,
    pub logs: PathBuf,
    pub cache: PathBuf,
    pub snapshots: PathBuf,
    pub locks: PathBuf,
    pub db: Connection,
}

impl Runtime {
    pub fn init() -> CoreResult<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        let root = PathBuf::from(&home).join("0-core/runtime");
        let logs = root.join("logs");
        let cache = root.join("cache");
        let snapshots = root.join("snapshots");
        let locks = root.join("locks");
        fs::create_dir_all(&logs)?;
        fs::create_dir_all(&cache)?;
        fs::create_dir_all(&snapshots)?;
        fs::create_dir_all(&locks)?;
        let db_path = root.join("state.db");
        let backups = root.join("backups");
        fs::create_dir_all(&backups)?;
        let db = Connection::open(&db_path)?;
        // INT-166: Enable WAL mode for corruption prevention
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        db.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS domain_state (
                domain      TEXT NOT NULL,
                key         TEXT NOT NULL,
                value       TEXT NOT NULL,
                updated_at  INTEGER NOT NULL,
                PRIMARY KEY (domain, key)
            );
            CREATE TABLE IF NOT EXISTS events (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                domain      TEXT NOT NULL,
                action      TEXT NOT NULL,
                payload     TEXT,
                timestamp   INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS capabilities_log (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                domain      TEXT NOT NULL,
                capability  TEXT NOT NULL,
                granted     INTEGER NOT NULL,
                timestamp   INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS forest_events (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                kind        TEXT NOT NULL,
                domain      TEXT NOT NULL,
                detail      TEXT,
                timestamp   INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS forest_insights (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                signal      TEXT NOT NULL,
                detail      TEXT NOT NULL,
                importance  REAL NOT NULL DEFAULT 0.0,
                confidence  REAL NOT NULL DEFAULT 0.0,
                expires_at  INTEGER,
                shown       INTEGER NOT NULL DEFAULT 0,
                created_at  INTEGER NOT NULL
            );
        ",
        )?;
        Ok(Self {
            root,
            logs,
            cache,
            snapshots,
            locks,
            db,
        })
    }
}

pub fn emit_forest_event(db: &Connection, kind: &str, domain: &str, detail: &str) {
    let ts = chrono::Utc::now().timestamp();
    let _ = db.execute(
        "INSERT INTO forest_events (kind, domain, detail, timestamp) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![kind, domain, detail, ts],
    );
}
pub struct RuntimeLock {
    path: PathBuf,
}

impl RuntimeLock {
    pub fn acquire(runtime: &Runtime) -> CoreResult<Self> {
        let path = runtime.locks.join("core.lock");
        // Try up to 3 times with short delays (handles bar polling collisions)
        for attempt in 0..3 {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(pid) = content.trim().parse::<u32>() {
                        let comm_path = PathBuf::from(format!("/proc/{}/comm", pid));
                        let is_core = fs::read_to_string(&comm_path)
                            .map(|c| c.trim() == "core")
                            .unwrap_or(false);
                        if is_core {
                            if attempt < 2 {
                                std::thread::sleep(std::time::Duration::from_millis(50));
                                continue;
                            }
                            return Err(crate::errors::CoreError::Runtime(format!(
                                "Another core process is running (pid {})",
                                pid
                            )));
                        }
                    }
                }
                // Stale lock — remove it
                fs::remove_file(&path).ok();
            }
            let pid = std::process::id();
            fs::write(&path, pid.to_string())?;
            return Ok(Self { path });
        }
        Err(crate::errors::CoreError::Runtime(
            "Could not acquire runtime lock".into(),
        ))
    }
}

impl Drop for RuntimeLock {
    fn drop(&mut self) {
        fs::remove_file(&self.path).ok();
    }
}

// ── Event Writer ─────────────────────────────────────────────────────────────

pub struct EventWriter<'a> {
    db: &'a Connection,
}

impl<'a> EventWriter<'a> {
    pub fn new(db: &'a Connection) -> Self {
        Self { db }
    }

    pub fn write(
        &self,
        domain: &str,
        action: &str,
        actor: &str,
        result: &str,
        payload: Option<&str>,
    ) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let full_payload = match payload {
            Some(p) => format!(
                r#"{{"actor":"{}","result":"{}","detail":{}}}"#,
                actor, result, p
            ),
            None => format!(r#"{{"actor":"{}","result":"{}"}}"#, actor, result),
        };

        self.db
            .execute(
                "INSERT INTO events (domain, action, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![domain, action, full_payload, ts],
            )
            .ok();

        // INT-129 — write to JSONL event log alongside SQLite
        self.append_jsonl(domain, action, &full_payload, ts);
    }

    fn append_jsonl(&self, domain: &str, action: &str, payload: &str, ts: i64) {
        let home = std::env::var("HOME").unwrap_or_default();
        let events_dir = std::path::PathBuf::from(&home).join("0-core/runtime/events");
        if !events_dir.exists() {
            if std::fs::create_dir_all(&events_dir).is_err() {
                return;
            }
        }

        // Daily rotation — one file per day
        let date = chrono::DateTime::from_timestamp(ts, 0)
            .map(|t| t.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let log_path = events_dir.join(format!("{}.jsonl", date));

        // Build JSONL line safely
        let mut line = String::with_capacity(256);
        line.push_str("{\"ts\":");
        line.push_str(&ts.to_string());
        line.push_str(",\"domain\":\"");
        line.push_str(domain);
        line.push_str("\",\"action\":\"");
        line.push_str(action);
        line.push_str("\",\"payload\":");
        line.push_str(payload);
        line.push_str("}\n");

        // Append to file
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            f.write_all(line.as_bytes()).ok();
        }
    }
}
