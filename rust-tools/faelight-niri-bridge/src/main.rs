//! faelight-niri-bridge v0.1.0
//! 🌲 Niri IPC → event ledger
//! Subscribes to niri event-stream and writes compositor events to runtime/state.db

use anyhow::{Context, Result};
use rusqlite::Connection;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn db_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/home/christian"))
        .join("0-core/runtime/state.db")
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn write_event(conn: &Connection, domain: &str, action: &str, payload: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO events (domain, action, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![domain, action, payload, now()],
    )?;
    Ok(())
}

fn parse_event(conn: &Connection, line: &str) -> Result<()> {
    let v: Value = serde_json::from_str(line)?;

    if let Some(obj) = v.get("WorkspacesChanged") {
        // Find the focused workspace
        if let Some(workspaces) = obj["workspaces"].as_array() {
            if let Some(focused) = workspaces
                .iter()
                .find(|w| w["is_focused"].as_bool().unwrap_or(false))
            {
                let id = focused["id"].as_u64().unwrap_or(0);
                let idx = focused["idx"].as_u64().unwrap_or(0);
                let payload = serde_json::json!({
                    "actor": "niri",
                    "result": "ok",
                    "detail": { "workspace_id": id, "workspace_idx": idx }
                });
                write_event(conn, "compositor", "workspace.switch", &payload.to_string())?;
                eprintln!("🌿 workspace.switch → idx:{}", idx);
            }
        }
    } else if let Some(obj) = v.get("WindowFocusChanged") {
        // Skip null focus (window closed, no focus)
        if let Some(window_id) = obj["id"].as_u64() {
            let payload = serde_json::json!({
                "actor": "niri",
                "result": "ok",
                "detail": { "window_id": window_id }
            });
            write_event(conn, "compositor", "window.focus", &payload.to_string())?;
            eprintln!("🌿 window.focus → {}", window_id);
        }
    } else if let Some(obj) = v.get("WindowsChanged") {
        // Track focused window changes
        if let Some(windows) = obj["windows"].as_array() {
            if let Some(focused) = windows
                .iter()
                .find(|w| w["is_focused"].as_bool().unwrap_or(false))
            {
                let app_id = focused["app_id"].as_str().unwrap_or("unknown");
                let window_id = focused["id"].as_u64().unwrap_or(0);
                let payload = serde_json::json!({
                    "actor": "niri",
                    "result": "ok",
                    "detail": { "window_id": window_id, "app_id": app_id }
                });
                write_event(conn, "compositor", "window.focus", &payload.to_string())?;
                eprintln!("🌿 window.focus → {} id:{}", app_id, window_id);
            }
        }
    } else if v.get("WindowOpenedOrChanged").is_some() {
        let obj = &v["WindowOpenedOrChanged"]["window"];
        let app_id = obj["app_id"].as_str().unwrap_or("unknown");
        let title = obj["title"].as_str().unwrap_or("");
        let payload = serde_json::json!({
            "actor": "niri",
            "result": "ok",
            "detail": { "app_id": app_id, "title": title }
        });
        write_event(conn, "compositor", "window.open", &payload.to_string())?;
        eprintln!("🌿 window.open → {}", app_id);
    } else if v.get("WindowClosed").is_some() {
        let id = v["WindowClosed"]["id"].as_u64().unwrap_or(0);
        let payload = serde_json::json!({
            "actor": "niri",
            "result": "ok",
            "detail": { "window_id": id }
        });
        write_event(conn, "compositor", "window.close", &payload.to_string())?;
        eprintln!("🌿 window.close → {}", id);
    }
    // Ignore: WorkspacesChanged, WindowsChanged (bulk), KeyboardLayoutsChanged, etc.

    Ok(())
}

fn main() -> Result<()> {
    let db = db_path();
    let conn = Connection::open(&db)
        .with_context(|| format!("cannot open state.db at {}", db.display()))?;

    eprintln!("🌲 faelight-niri-bridge v0.1.0 — compositor events → event ledger");
    eprintln!("   DB: {}", db.display());

    let mut child = Command::new("niri")
        .args(["msg", "--json", "event-stream"])
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to spawn niri msg event-stream")?;

    let stdout = child.stdout.take().context("no stdout")?;
    let reader = BufReader::new(stdout);

    eprintln!("   Listening...\n");

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Err(e) = parse_event(&conn, &line) {
            eprintln!(
                "⚠️  parse error: {} — line: {}",
                e,
                &line[..line.len().min(80)]
            );
        }
    }

    Ok(())
}
