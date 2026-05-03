//! INT-244 v22 -- Pillar 3: Persistent Decision Memory
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
pub fn decide(ctx: &AppContext, what: &str, why: &str, ties_to: &str) -> CoreResult<()> {
    super::ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let session_id = std::env::var("FSH_SESSION_ID").unwrap_or_default();
    db.execute(
        "INSERT INTO friday_decisions (timestamp, what, why, ties_to, session_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![now_ts(), what, why, ties_to, session_id],
    )?;
    println!("  {} Decision recorded", "✅".green());
    println!("  {} What:    {}", "→".dimmed(), what.bright_white());
    println!("  {} Why:     {}", "→".dimmed(), why.cyan());
    if !ties_to.is_empty() {
        println!("  {} Ties to: {}", "→".dimmed(), ties_to.dimmed());
    }
    Ok(())
}
pub fn why(ctx: &AppContext, topic: &str) -> CoreResult<()> {
    super::ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let search = format!("%{}%", topic.to_lowercase());
    let mut stmt = db.prepare(
        "SELECT id, timestamp, what, why, ties_to
         FROM friday_decisions
         WHERE lower(what) LIKE ?1 OR lower(why) LIKE ?1 OR lower(ties_to) LIKE ?1
         ORDER BY timestamp DESC LIMIT 10"
    )?;
    let rows: Vec<(i64, i64, String, String, String)> = stmt.query_map(
        params![search],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
    )?.filter_map(|r| r.ok()).collect();
    if rows.is_empty() {
        println!("  🤔 No decisions found for: {}", topic.yellow());
        println!("  → Record one: core friday decide \"<what>\" --why \"<why>\"");
        return Ok(());
    }
    println!("  🧠 Decisions matching: {}", topic.bright_white().bold());
    println!("{}", "─".repeat(48).dimmed());
    for (_id, ts, what, why, ties_to) in &rows {
        let dt = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "unknown".to_string());
        println!("  ▸ [{}] {}", dt.dimmed(), what.bright_white());
        println!("    {} {}", "why:".dimmed(), why.cyan());
        if !ties_to.is_empty() {
            println!("    {} {}", "ties_to:".dimmed(), ties_to.dimmed());
        }
        println!();
    }
    Ok(())
}
pub fn list_decisions(ctx: &AppContext) -> CoreResult<()> {
    super::ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let mut stmt = db.prepare(
        "SELECT id, timestamp, what, ties_to FROM friday_decisions ORDER BY timestamp DESC LIMIT 20"
    )?;
    let rows: Vec<(i64, i64, String, String)> = stmt.query_map(
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    )?.filter_map(|r| r.ok()).collect();
    if rows.is_empty() {
        println!("  No decisions recorded yet.");
        println!("  Record one: core friday decide \"what\" --why \"why\"");
        return Ok(());
    }
    println!("  🧠 Recorded Decisions ({})", rows.len());
    println!("{}", "─".repeat(48).dimmed());
    for (id, ts, what, ties_to) in &rows {
        let dt = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let ties = if ties_to.is_empty() { String::new() } else { format!(" [{}]", ties_to) };
        println!("  {} {} {}{}", id.to_string().dimmed(), dt.dimmed(), what.bright_white(), ties.dimmed());
    }
    Ok(())
}
