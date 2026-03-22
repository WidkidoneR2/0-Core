use rusqlite::{Connection, params};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use crate::crypto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub name:      String,
    pub cred_type: String,
    pub age_days:  i64,
}

#[derive(Debug)]
pub struct VaultEntryFull {
    pub name:      String,
    pub cred_type: String,
    pub secret:    String,
    pub age_days:  i64,
}

pub fn vault_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("faelight/vault.db")
}

fn open_db() -> Result<Connection, String> {
    let path = vault_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create vault dir: {}", e))?;
    }
    Connection::open(&path)
        .map_err(|e| format!("Cannot open vault: {}", e))
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS vault_meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS credentials (
            name       TEXT PRIMARY KEY,
            cred_type  TEXT NOT NULL,
            secret_enc TEXT NOT NULL,
            salt       TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS access_log (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT NOT NULL,
            action     TEXT NOT NULL,
            timestamp  INTEGER NOT NULL
        );
    ").map_err(|e| format!("Schema init failed: {}", e))
}

pub fn init_vault(master: &str) -> Result<(), String> {
    let conn = open_db()?;
    init_schema(&conn)?;

    let salt = crypto::random_salt();
    let key = crypto::derive_key(master, &salt);
    let salt_hex = hex::encode(&salt);

    // Store master hash for validation
    let master_check = crypto::encrypt("vault-initialized", &key);
    conn.execute(
        "INSERT OR REPLACE INTO vault_meta (key, value) VALUES ('master_check', ?1)",
        params![master_check]
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO vault_meta (key, value) VALUES ('master_salt', ?1)",
        params![salt_hex]
    ).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn validate_master(master: &str) -> Result<bool, String> {
    let conn = open_db()?;
    let salt_hex: String = conn.query_row(
        "SELECT value FROM vault_meta WHERE key='master_salt'",
        [], |r| r.get(0)
    ).map_err(|_| "Vault not initialized — run: faelight-vault init".to_string())?;

    let salt = hex::decode(&salt_hex).map_err(|e| e.to_string())?;
    let key = crypto::derive_key(master, &salt);

    let check: String = conn.query_row(
        "SELECT value FROM vault_meta WHERE key='master_check'",
        [], |r| r.get(0)
    ).map_err(|e| e.to_string())?;

    Ok(crypto::decrypt(&check, &key).as_deref() == Some("vault-initialized"))
}

fn get_key(conn: &Connection, master: &str) -> Result<[u8; 32], String> {
    let salt_hex: String = conn.query_row(
        "SELECT value FROM vault_meta WHERE key='master_salt'",
        [], |r| r.get(0)
    ).map_err(|_| "Vault not initialized".to_string())?;
    let salt = hex::decode(&salt_hex).map_err(|e| e.to_string())?;

    if !validate_master(master)? {
        return Err("Invalid master password".to_string());
    }
    Ok(crypto::derive_key(master, &salt))
}

pub fn add_credential(master: &str, name: &str, cred_type: &str, secret: &str) -> Result<(), String> {
    let conn = open_db()?;
    init_schema(&conn)?;
    let key = get_key(&conn, master)?;
    let salt = crypto::random_salt();
    let enc = crypto::encrypt(secret, &key);
    let salt_hex = hex::encode(&salt);
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT OR REPLACE INTO credentials (name, cred_type, secret_enc, salt, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![name, cred_type, enc, salt_hex, now]
    ).map_err(|e| e.to_string())?;

    log_access(&conn, name, "add")?;
    Ok(())
}

pub fn get_credential(master: &str, name: &str) -> Result<Option<VaultEntryFull>, String> {
    let conn = open_db()?;
    let key = get_key(&conn, master)?;

    let result = conn.query_row(
        "SELECT name, cred_type, secret_enc, updated_at FROM credentials WHERE name=?1",
        params![name],
        |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,i64>(3)?))
    );

    match result {
        Ok((n, ct, enc, updated)) => {
            let secret = crypto::decrypt(&enc, &key)
                .ok_or("Decryption failed")?;
            let age_days = (chrono::Utc::now().timestamp() - updated) / 86400;
            log_access(&conn, name, "get")?;
            Ok(Some(VaultEntryFull { name: n, cred_type: ct, secret, age_days }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn list_credentials(master: &str) -> Result<Vec<VaultEntry>, String> {
    let conn = open_db()?;
    validate_master(master).and_then(|ok| {
        if !ok { Err("Invalid master password".to_string()) } else { Ok(()) }
    })?;

    let mut stmt = conn.prepare(
        "SELECT name, cred_type, updated_at FROM credentials ORDER BY name"
    ).map_err(|e| e.to_string())?;

    let entries: Vec<VaultEntry> = stmt.query_map([], |r| {
        Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i64>(2)?))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .map(|(name, cred_type, updated)| {
        let age_days = (chrono::Utc::now().timestamp() - updated) / 86400;
        VaultEntry { name, cred_type, age_days }
    })
    .collect();

    Ok(entries)
}

pub fn update_credential(master: &str, name: &str, new_secret: &str) -> Result<(), String> {
    let conn = open_db()?;
    let key = get_key(&conn, master)?;
    let enc = crypto::encrypt(new_secret, &key);
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE credentials SET secret_enc=?1, updated_at=?2 WHERE name=?3",
        params![enc, now, name]
    ).map_err(|e| e.to_string())?;
    log_access(&conn, name, "rotate")?;
    Ok(())
}

pub fn remove_credential(master: &str, name: &str) -> Result<(), String> {
    let conn = open_db()?;
    validate_master(master).and_then(|ok| {
        if !ok { Err("Invalid master password".to_string()) } else { Ok(()) }
    })?;
    conn.execute("DELETE FROM credentials WHERE name=?1", params![name])
        .map_err(|e| e.to_string())?;
    log_access(&conn, name, "remove")?;
    Ok(())
}

pub fn write_session_cache(master: &str, _ttl: &str) {
    let cache_path = std::env::temp_dir().join("faelight-vault.session");
    std::fs::write(&cache_path, master).ok();
}

pub fn clear_session_cache() {
    let cache_path = std::env::temp_dir().join("faelight-vault.session");
    let _ = std::fs::remove_file(&cache_path);
}

pub fn export_vault(master: &str, path: &str) -> Result<(), String> {
    validate_master(master).and_then(|ok| {
        if !ok { Err("Invalid master password".to_string()) } else { Ok(()) }
    })?;
    std::fs::copy(vault_path(), path)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub fn import_vault(master: &str, path: &str) -> Result<usize, String> {
    std::fs::copy(path, vault_path())
        .map_err(|e| e.to_string())?;
    let entries = list_credentials(master)?;
    Ok(entries.len())
}

fn log_access(conn: &Connection, name: &str, action: &str) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO access_log (name, action, timestamp) VALUES (?1, ?2, ?3)",
        params![name, action, now]
    ).map_err(|e| e.to_string())?;
    Ok(())
}
