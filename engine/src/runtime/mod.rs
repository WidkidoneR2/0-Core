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
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/christian".to_string());
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
