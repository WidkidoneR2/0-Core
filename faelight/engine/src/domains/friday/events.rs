//! INT-251 v23 Pillar 1 -- Unified Event Bus
//! friday::events::emit is the canonical way all tools report to Friday
fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;

/// Emit a structured event to the forest event bus.
/// This is the v23 canonical emit -- all tools should use this.
/// domain: the tool/subsystem emitting (e.g. "deploy", "shell", "git")
/// kind:   what happened (e.g. "deploy_completed", "command_run")
/// payload: JSON string with event details
/// source_tool: which binary emitted this (e.g. "core", "faelight-shell")
/// correlation_id: optional session/workflow id for tracing causality
#[allow(dead_code)]
pub fn emit(
    ctx: &AppContext,
    domain: &str,
    kind: &str,
    payload: &str,
    source_tool: &str,
    correlation_id: Option<&str>,
) -> CoreResult<()> {
    let db = &ctx.runtime.db;
    let ts = now_ts();
    let corr = correlation_id.unwrap_or("");

    db.execute(
        "INSERT INTO events (domain, action, payload, timestamp, source_tool, correlation_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![domain, kind, payload, ts, source_tool, corr],
    )?;
    Ok(())
}

/// Emit a simple event with just domain + kind (common case)
#[allow(dead_code)]
pub fn emit_simple(ctx: &AppContext, domain: &str, kind: &str) -> CoreResult<()> {
    emit(ctx, domain, kind, "{}", domain, None)
}

/// Query recent events for a domain, returns (kind, payload, timestamp) tuples
#[allow(dead_code)]
pub fn recent(ctx: &AppContext, domain: &str, limit: usize) -> Vec<(String, String, i64)> {
    let db = &ctx.runtime.db;
    let mut s = match db.prepare(
        "SELECT action, payload, timestamp FROM events WHERE domain=?1 ORDER BY timestamp DESC LIMIT ?2"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    s.query_map(rusqlite::params![domain, limit as i64], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1).unwrap_or_default(),
            r.get::<_, i64>(2)?,
        ))
    })
    .ok()
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

/// Query events by kind across all domains (for cross-tool reasoning)
#[allow(dead_code)]
pub fn by_kind(ctx: &AppContext, kind: &str, since_ts: i64) -> Vec<(String, String, i64)> {
    let db = &ctx.runtime.db;
    let mut s = match db.prepare(
        "SELECT domain, payload, timestamp FROM events WHERE action=?1 AND timestamp > ?2 ORDER BY timestamp DESC LIMIT 50"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    s.query_map(rusqlite::params![kind, since_ts], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1).unwrap_or_default(),
            r.get::<_, i64>(2)?,
        ))
    })
    .ok()
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

/// Show recent events -- core friday events command
pub fn show_recent(
    ctx: &AppContext,
    limit: usize,
    domain: Option<&str>,
    json: bool,
) -> CoreResult<()> {
    let db = &ctx.runtime.db;
    let now = now_ts();
    let day_ago = now - 86400;

    let rows: Vec<(String, String, String, String, i64)> = if let Some(d) = domain {
        let mut s = db.prepare(
            "SELECT domain, action, source_tool, payload, timestamp FROM events
             WHERE timestamp > ?1 AND domain = ?2 ORDER BY timestamp DESC LIMIT ?3",
        )?;
        let x = s
            .query_map(rusqlite::params![day_ago, d, limit as i64], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2).unwrap_or_default(),
                    r.get::<_, String>(3).unwrap_or_default(),
                    r.get::<_, i64>(4)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        x
    } else {
        let mut s = db.prepare(
            "SELECT domain, action, source_tool, payload, timestamp FROM events
             WHERE timestamp > ?1 ORDER BY timestamp DESC LIMIT ?2",
        )?;
        let x = s
            .query_map(rusqlite::params![day_ago, limit as i64], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2).unwrap_or_default(),
                    r.get::<_, String>(3).unwrap_or_default(),
                    r.get::<_, i64>(4)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        x
    };

    if json {
        println!("[");
        let len = rows.len();
        for (i, (dom, kind, source, payload, ts)) in rows.iter().enumerate() {
            let comma = if i + 1 < len { "," } else { "" };
            println!(
                "  {{\"domain\":\"{}\",\"kind\":\"{}\",\"source\":\"{}\",\"payload\":{},\"ts\":{}}}{}",
                dom, kind, source, payload, ts, comma
            );
        }
        println!("]");
        return Ok(());
    }

    println!();
    println!("  {} Event Bus -- Last 24h", "🌲".normal());
    println!("  {}", "─".repeat(50).dimmed());
    println!();

    if rows.is_empty() {
        println!("  {} No events in last 24h", "·".dimmed());
    } else {
        let mut last_domain = String::new();
        for (domain, kind, _source, payload, ts) in &rows {
            if *domain != last_domain {
                println!("  {} {}", "▸".bright_cyan(), domain.bright_white());
                last_domain = domain.clone();
            }
            let short_payload = payload.chars().take(60).collect::<String>();
            let secs = ts % 86400;
            let h = secs / 3600;
            let m = (secs % 3600) / 60;
            let s = secs % 60;
            let time = format!("{:02}:{:02}:{:02}", h, m, s);
            println!(
                "    {} {} [{}] {}",
                "·".dimmed(),
                time.dimmed(),
                kind.bright_green(),
                short_payload.white()
            );
        }
    }
    println!();
    Ok(())
}
