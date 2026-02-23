use crate::errors::CoreResult;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

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
        let db = Connection::open(&db_path)?;
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
