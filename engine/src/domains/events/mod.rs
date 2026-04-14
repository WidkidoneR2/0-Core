//! events domain -- query the event ledger
pub mod signal;
use crate::app::context::AppContext;
use crate::capabilities::Capability;
extern crate flate2;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;

pub fn list(ctx: &AppContext) -> CoreResult<()> {
    let today = chrono_today();
    query_events(ctx, &format!("timestamp >= {}", today), None)
}

pub fn since(ctx: &AppContext, duration: &str) -> CoreResult<()> {
    let seconds = parse_duration(duration);
    let ts = now_ts() - seconds;
    query_events(ctx, &format!("timestamp >= {}", ts), None)
}

pub fn filter(ctx: &AppContext, domain: &str) -> CoreResult<()> {
    query_events(
        ctx,
        &format!("timestamp >= {}", chrono_today()),
        Some(domain),
    )
}

fn query_events(
    ctx: &AppContext,
    where_clause: &str,
    domain_filter: Option<&str>,
) -> CoreResult<()> {
    let sql = if let Some(d) = domain_filter {
        format!(
            "SELECT domain, action, payload, timestamp FROM events WHERE {} AND domain = '{}' ORDER BY timestamp DESC LIMIT 100",
            where_clause, d
        )
    } else {
        format!(
            "SELECT domain, action, payload, timestamp FROM events WHERE {} ORDER BY timestamp DESC LIMIT 100",
            where_clause
        )
    };

    let mut stmt = ctx.runtime.db.prepare(&sql)?;
    let mut count = 0;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, i64>(3)?,
        ))
    })?;

    println!("{}", "🌲 Event Ledger".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    for row in rows {
        let (domain, action, payload, ts) = row?;
        let time = format_ts(ts);

        // Extract result from payload JSON simply
        let result = payload
            .as_deref()
            .and_then(|p| {
                p.find("\"result\":\"")
                    .map(|i| &p[i + 10..])
                    .and_then(|s| s.find('"').map(|e| &s[..e]))
            })
            .unwrap_or("ok");

        let result_colored = if result == "ok" || result == "pass" {
            result.green().to_string()
        } else {
            result.yellow().to_string()
        };

        println!(
            "  {} {} {} {}  {}",
            time.dimmed(),
            domain.cyan(),
            "›".dimmed(),
            action.white(),
            result_colored,
        );
        count += 1;
    }

    if count == 0 {
        println!("  {}", "No events recorded yet.".dimmed());
        println!("  Events are written as you use core commands.");
    } else {
        println!();
        println!("  {} events", count.to_string().dimmed());
    }

    Ok(())
}

/// core events emit <type> <payload> [--caused-by SEQ] -- validated signal emission to v2
pub fn emit_v2(ctx: &AppContext, type_name: &str, payload: &str, caused_by: Option<i64>) -> CoreResult<()> {
    if let Err(e) = signal::emit(
        &ctx.runtime.db,
        "core",
        signal::SignalKind::Observation,
        type_name,
        payload,
        None,
        caused_by,
        1.0,
    ) {
        println!("  {} emit failed: {}", "✗".bright_red(), e);
        return Ok(());
    }
    println!("  {} signal emitted: {} ({})", "✅".green(), type_name.bright_cyan(), payload.dimmed());
    Ok(())
}
/// core events replay --from SEQ --to SEQ -- show event sequence range
pub fn replay(ctx: &AppContext, from_seq: i64, to_seq: i64) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(signal::CREATE_TABLE)?;
    println!();
    println!("  {} Event Replay: seq {} → {}", "🔄".normal(), from_seq, to_seq);
    println!("  {}", "─".repeat(60).dimmed());
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT seq, timestamp, source, kind, type_name, payload, caused_by, confidence
         FROM forest_events_v2
         WHERE seq >= ?1 AND seq <= ?2
         ORDER BY seq ASC"
    )?;
    let rows: Vec<(i64, i64, String, String, String, String, Option<i64>, f64)> = stmt
        .query_map(params![from_seq, to_seq], |r| Ok((
            r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?,
            r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?
        )))?.filter_map(|r| r.ok()).collect();
    if rows.is_empty() {
        println!("  {} No events in range {} → {}", "○".dimmed(), from_seq, to_seq);
    }
    for (seq, ts, source, _kind, type_name, payload, caused_by, conf) in &rows {
        let time = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|t| t.format("%H:%M:%S").to_string())
            .unwrap_or_default();
        let causal = caused_by.map(|c| format!(" ← seq{}", c)).unwrap_or_default();
        println!("  #{:<6} {} {:<12} {:<20} {:.2}{}",
            seq.to_string().bright_cyan(),
            time.dimmed(),
            source.bright_white(),
            type_name.bright_green(),
            conf,
            causal.dimmed());
        if payload.len() > 2 && payload != "{}" {
            println!("         {}", payload.dimmed());
        }
    }
    println!();
    Ok(())
}
/// core events chain <SEQ> -- show causality chain for a signal
pub fn chain(ctx: &AppContext, seq: i64) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(signal::CREATE_TABLE)?;
    let chain = signal::causality_chain(&ctx.runtime.db, seq);
    println!();
    println!("  {} Causality chain for seq #{}", "🔗".normal(), seq.to_string().bright_cyan());
    println!("  {}", "─".repeat(50).dimmed());
    if chain.is_empty() {
        println!("  {} No events found at seq {}", "○".dimmed(), seq);
    }
    for (s, t, p, arrow) in &chain {
        println!("  {} #{:<6} {} {}",
            arrow.bright_cyan(),
            s.to_string().bright_white(),
            t.bright_green(),
            p.dimmed());
    }
    println!();
    Ok(())
}
/// core events status -- show forest_events_v2 health and counts
pub fn status_v2(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(signal::CREATE_TABLE)?;
    let total: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM forest_events_v2", [], |r| r.get(0)
    ).unwrap_or(0);
    let by_kind: Vec<(String, i64)> = {
        let mut s = ctx.runtime.db.prepare(
            "SELECT type_name, COUNT(*) FROM forest_events_v2 GROUP BY type_name ORDER BY COUNT(*) DESC LIMIT 10"
        )?;
        let x: Vec<(String, i64)> = s.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?)))
            ?.filter_map(|r| r.ok()).collect();
        x
    };
    let max_seq: i64 = ctx.runtime.db.query_row(
        "SELECT COALESCE(MAX(seq), 0) FROM forest_events_v2", [], |r| r.get(0)
    ).unwrap_or(0);
    let with_causality: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM forest_events_v2 WHERE caused_by IS NOT NULL", [], |r| r.get(0)
    ).unwrap_or(0);
    println!();
    println!("  {} forest_events_v2 -- Canonical Signal Log", "📡".normal());
    println!("  {}", "─".repeat(50).dimmed());
    println!("  {:<25} {}", "Total signals:".dimmed(), total.to_string().bright_white());
    println!("  {:<25} {}", "Max sequence:".dimmed(), max_seq.to_string().bright_cyan());
    println!("  {:<25} {} ({:.0}%)", "With causality:".dimmed(),
        with_causality.to_string().bright_green(),
        if total > 0 { with_causality as f64 / total as f64 * 100.0 } else { 0.0 });
    if !by_kind.is_empty() {
        println!();
        println!("  {} Signal types:", "→".dimmed());
        for (t, c) in &by_kind {
            println!("    {:<22} {}", t.bright_white(), c.to_string().dimmed());
        }
    }
    println!();
    Ok(())
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn chrono_today() -> i64 {
    // Midnight today (approximate: now minus seconds since midnight)
    let ts = now_ts();
    ts - (ts % 86400)
}

fn parse_duration(s: &str) -> i64 {
    let s = s.trim();
    if let Some(n) = s.strip_suffix('h') {
        n.parse::<i64>().unwrap_or(1) * 3600
    } else if let Some(n) = s.strip_suffix('d') {
        n.parse::<i64>().unwrap_or(1) * 86400
    } else if let Some(n) = s.strip_suffix('m') {
        n.parse::<i64>().unwrap_or(1) * 60
    } else {
        3600 // default 1h
    }
}

fn format_ts(ts: i64) -> String {
    // Use local time via `date` command
    let output = std::process::Command::new("date")
        .args(["-d", &format!("@{}", ts), "+%H:%M:%S"])
        .output();
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => {
            let secs = ts % 86400;
            format!(
                "{:02}:{:02}:{:02}",
                secs / 3600,
                (secs % 3600) / 60,
                secs % 60
            )
        }
    }
}

// ── Phase 2: Causality Engine ─────────────────────────────────────────────────

pub fn why_summary(ctx: &AppContext) -> CoreResult<()> {
    let today = chrono_today();
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT domain, action, payload, timestamp FROM events WHERE timestamp >= ? ORDER BY timestamp ASC LIMIT 200"
    )?;

    let rows: Vec<(String, String, Option<String>, i64)> = stmt
        .query_map(params![today], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    println!("{}", "🌲 Why — System Activity Summary".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    if rows.is_empty() {
        println!("  {}", "No activity recorded today yet.".dimmed());
        println!("  Run doctor, git status, or security scan to generate events.");
        return Ok(());
    }

    // Group by domain
    let mut domains: std::collections::BTreeMap<String, Vec<(String, Option<String>, i64)>> =
        std::collections::BTreeMap::new();
    for (domain, action, payload, ts) in &rows {
        domains
            .entry(domain.clone())
            .or_default()
            .push((action.clone(), payload.clone(), *ts));
    }

    let total = rows.len();
    let first_ts = rows.first().map(|r| r.3).unwrap_or(0);
    let last_ts = rows.last().map(|r| r.3).unwrap_or(0);

    println!(
        "  {} events today  •  {} to {}",
        total.to_string().bright_white(),
        format_ts(first_ts).dimmed(),
        format_ts(last_ts).dimmed(),
    );
    println!();

    for (domain, events) in &domains {
        let count = events.len();
        let last = events.last().map(|e| format_ts(e.2)).unwrap_or_default();
        let warns = events
            .iter()
            .filter(|e| {
                e.1.as_deref()
                    .map(|p| p.contains("\"result\":\"warn\""))
                    .unwrap_or(false)
            })
            .count();

        let status = if warns > 0 {
            format!("⚠️  {} warning(s)", warns).yellow().to_string()
        } else {
            "✓ all ok".green().to_string()
        };

        println!(
            "  {} {}  ×{}  last: {}  {}",
            "▶".dimmed(),
            domain.bright_white(),
            count.to_string().cyan(),
            last.dimmed(),
            status,
        );

        // Show payload detail for health/security
        if domain == "doctor" {
            if let Some(last_event) = events.last() {
                if let Some(ref p) = last_event.1 {
                    let health = extract_field(p, "health");
                    if !health.is_empty() {
                        println!("    {} health: {}", "→".dimmed(), health.bright_white());
                    }
                }
            }
        }
        if domain == "security" {
            if let Some(last_event) = events.last() {
                if let Some(ref p) = last_event.1 {
                    let critical = extract_field(p, "critical");
                    let high = extract_field(p, "high");
                    if !critical.is_empty() {
                        println!(
                            "    {} findings: {} critical, {} high",
                            "→".dimmed(),
                            critical,
                            high
                        );
                    }
                }
            }
        }
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn why_health(ctx: &AppContext) -> CoreResult<()> {
    let today = chrono_today();
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT payload, timestamp FROM events WHERE timestamp >= ? AND domain = 'doctor' ORDER BY timestamp ASC LIMIT 50"
    )?;

    let rows: Vec<(Option<String>, i64)> = stmt
        .query_map(params![today], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    println!("{}", "🌲 Why — Health Trajectory".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    if rows.is_empty() {
        println!("  {}", "No doctor runs recorded today.".dimmed());
        println!("  Run: core doctor run");
        return Ok(());
    }

    let mut prev_health: Option<i64> = None;

    for (payload, ts) in &rows {
        let health_str = payload
            .as_deref()
            .map(|p| extract_field(p, "health"))
            .unwrap_or_default();
        let health: i64 = health_str.parse().unwrap_or(0);
        let passed = payload
            .as_deref()
            .map(|p| extract_field(p, "passed"))
            .unwrap_or_default();
        let warnings = payload
            .as_deref()
            .map(|p| extract_field(p, "warnings"))
            .unwrap_or_default();
        let failed = payload
            .as_deref()
            .map(|p| extract_field(p, "failed"))
            .unwrap_or_default();

        let delta = match prev_health {
            Some(prev) => {
                let d = health - prev;
                if d > 0 {
                    format!(" (+{})", d).green().to_string()
                } else if d < 0 {
                    format!(" ({})", d).bright_red().to_string()
                } else {
                    " (no change)".dimmed().to_string()
                }
            }
            None => " (first run today)".dimmed().to_string(),
        };

        let health_colored = if health >= 95 {
            format!("{}%", health).green().to_string()
        } else if health >= 80 {
            format!("{}%", health).yellow().to_string()
        } else {
            format!("{}%", health).bright_red().to_string()
        };

        println!(
            "  {}  health {}{}",
            format_ts(*ts).dimmed(),
            health_colored,
            delta,
        );
        if !passed.is_empty() {
            println!(
                "          passed: {}  warnings: {}  failed: {}",
                passed.green(),
                if warnings == "0" {
                    warnings.dimmed().to_string()
                } else {
                    warnings.yellow().to_string()
                },
                if failed == "0" {
                    failed.dimmed().to_string()
                } else {
                    failed.bright_red().to_string()
                },
            );
        }

        prev_health = Some(health);
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn why_domain(ctx: &AppContext, domain: &str) -> CoreResult<()> {
    let today = chrono_today();
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT action, payload, timestamp FROM events WHERE timestamp >= ? AND domain = ? ORDER BY timestamp ASC LIMIT 100"
    )?;

    let rows: Vec<(String, Option<String>, i64)> = stmt
        .query_map(params![today, domain], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    println!(
        "{}",
        format!("🌲 Why — {} activity today", domain).cyan().bold()
    );
    println!("{}", "━".repeat(52).dimmed());
    println!();

    if rows.is_empty() {
        println!(
            "  {}",
            format!("No {} events recorded today.", domain).dimmed()
        );
        return Ok(());
    }

    for (action, payload, ts) in &rows {
        let result = payload
            .as_deref()
            .map(|p| extract_field(p, "result"))
            .unwrap_or_default();
        let result_str = if result == "ok" || result.is_empty() {
            "ok".green().to_string()
        } else {
            result.yellow().to_string()
        };

        println!(
            "  {}  {} {}  {}",
            format_ts(*ts).dimmed(),
            action.bright_white(),
            "›".dimmed(),
            result_str,
        );

        // Show meaningful payload fields
        if let Some(ref p) = payload {
            for field in &["health", "branch", "modified", "risk", "critical", "high"] {
                let val = extract_field(p, field);
                if !val.is_empty() {
                    println!("          {}: {}", field.dimmed(), val.white());
                }
            }
        }
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn trace_last(ctx: &AppContext) -> CoreResult<()> {
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT domain, action, payload, timestamp FROM events ORDER BY timestamp DESC LIMIT 10",
    )?;

    let rows: Vec<(String, String, Option<String>, i64)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    println!("{}", "🌲 Trace — Last 10 Events".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    for (domain, action, payload, ts) in rows.iter().rev() {
        println!(
            "  {}  {} › {}",
            format_ts(*ts).dimmed(),
            domain.cyan(),
            action.bright_white(),
        );
        if let Some(ref p) = payload {
            print_payload_fields(p);
        }
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn trace_domain(ctx: &AppContext, domain: &str) -> CoreResult<()> {
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT action, payload, timestamp FROM events WHERE domain = ? ORDER BY timestamp DESC LIMIT 20"
    )?;

    let rows: Vec<(String, Option<String>, i64)> = stmt
        .query_map(params![domain], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    println!(
        "{}",
        format!("🌲 Trace — {} (last 20)", domain).cyan().bold()
    );
    println!("{}", "━".repeat(52).dimmed());
    println!();

    if rows.is_empty() {
        println!(
            "  {}",
            format!("No events found for domain: {}", domain).dimmed()
        );
        return Ok(());
    }

    for (action, payload, ts) in rows.iter().rev() {
        println!("  {}  {}", format_ts(*ts).dimmed(), action.bright_white(),);
        if let Some(ref p) = payload {
            print_payload_fields(p);
        }
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

fn print_payload_fields(payload: &str) {
    // Extract meaningful fields from JSON payload, skip actor/result/detail wrappers
    let skip = ["actor", "result", "detail"];
    for field in &[
        "health", "passed", "warnings", "failed", "branch", "modified", "staged", "risk",
        "critical", "high", "medium", "low",
    ] {
        let val = extract_field(payload, field);
        if !val.is_empty() && !skip.contains(field) {
            println!(
                "          {}  {}",
                format!("{}:", field).dimmed(),
                val.white()
            );
        }
    }
}

fn extract_field(payload: &str, field: &str) -> String {
    let needle = format!("\"{}\":", field);
    payload
        .find(&needle)
        .map(|i| {
            let rest = &payload[i + needle.len()..].trim_start();
            if let Some(inner) = rest.strip_prefix('"') {
                // String value
                let inner = inner;
                inner[..inner.find('"').unwrap_or(inner.len())].to_string()
            } else {
                // Numeric value
                let end = rest
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(rest.len());
                rest[..end].to_string()
            }
        })
        .unwrap_or_default()
}

// ── Phase 4: Live Event Watch ─────────────────────────────────────────────────

pub fn watch(_ctx: &AppContext) -> CoreResult<()> {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;

    let home = std::env::var("HOME").unwrap_or_default();
    let socket_path = format!("{}/.local/state/0-core/daemon.sock", home);

    let stream = match UnixStream::connect(&socket_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "  {} Cannot connect to faelight-daemon: {}",
                "✗".bright_red(),
                e
            );
            eprintln!("  {} Is faelight-daemon running?", "💡".yellow());
            eprintln!("  {} systemctl --user status faelight-daemon", "→".dimmed());
            return Ok(());
        }
    };

    println!("{}", "🌲 Event Watch — Live Stream".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!("  {} Connected to faelight-daemon", "✓".green());
    println!("  {} Waiting for events... (Ctrl+C to stop)", "→".dimmed());
    println!();

    // Send EventStream command
    let msg = serde_json::json!({
        "id": 1,
        "payload": { "EventStream": null }
    });
    let mut writer = stream.try_clone()?;
    writeln!(writer, "{}", msg)?;

    // Read responses
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };

        let payload = &parsed["payload"];

        // Skip the Subscribed confirmation
        if payload.get("Subscribed").is_some() {
            println!("  {} Subscription confirmed", "✓".green().dimmed());
            continue;
        }

        // Handle Event
        if let Some(event) = payload.get("Event") {
            let domain = event["domain"].as_str().unwrap_or("?");
            let action = event["action"].as_str().unwrap_or("?");
            let ts = event["timestamp"].as_i64().unwrap_or(0);
            let time = format_ts(ts);

            let result = event["payload"]
                .as_str()
                .and_then(|p| {
                    p.find("\"result\":\"")
                        .map(|i| &p[i + 10..])
                        .and_then(|s| s.find('"').map(|e| &s[..e]))
                })
                .unwrap_or("ok");

            let result_colored = if result == "ok" || result == "pass" {
                result.green().to_string()
            } else {
                result.yellow().to_string()
            };

            println!(
                "  {} {} {} {}  {}",
                time.dimmed(),
                domain.cyan(),
                "›".dimmed(),
                action.white(),
                result_colored,
            );
        }
    }

    Ok(())
}

pub fn why_visual(ctx: &AppContext) -> CoreResult<()> {
    let today = chrono_today();
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT action, payload, timestamp FROM events WHERE domain='compositor' AND timestamp >= ? ORDER BY timestamp ASC"
    )?;
    let rows: Vec<(String, Option<String>, i64)> = stmt
        .query_map(params![today], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    println!("{}", "🌲 Why — Visual Topology Today".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    if rows.is_empty() {
        println!("  {}", "No compositor events today.".dimmed());
        println!("  Ensure faelight-niri-bridge is running in Niri autostart.");
        return Ok(());
    }

    // Count by action type
    let mut focus_count = 0u32;
    let mut open_count = 0u32;
    let mut switch_count = 0u32;
    let mut app_time: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let mut last_app = String::new();
    let mut last_ts = 0i64;

    for (action, payload, ts) in &rows {
        match action.as_str() {
            "window.focus" => {
                focus_count += 1;
                // Track time spent per app
                let app = payload
                    .as_deref()
                    .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
                    .and_then(|v| v["detail"]["app_id"].as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "unknown".to_string());
                if !last_app.is_empty() && last_ts > 0 {
                    let duration = (ts - last_ts) as u32;
                    *app_time.entry(last_app.clone()).or_insert(0) += duration.min(300);
                }
                last_app = app;
                last_ts = *ts;
            }
            "window.open" => open_count += 1,
            "workspace.switch" => switch_count += 1,
            _ => {}
        }
    }

    println!();
    println!(
        "  {} compositor events today",
        rows.len().to_string().bright_white()
    );
    println!(
        "  {} window focuses  •  {} windows opened  •  {} workspace switches",
        focus_count.to_string().cyan(),
        open_count.to_string().green(),
        switch_count.to_string().yellow(),
    );
    println!();

    // App time breakdown
    if !app_time.is_empty() {
        println!("  {}", "Time by app (estimated):".dimmed());
        let mut apps: Vec<(String, u32)> = app_time.into_iter().collect();
        apps.sort_by(|a, b| b.1.cmp(&a.1));
        for (app, secs) in apps.iter().take(5) {
            let mins = secs / 60;
            let bar_len = (mins.min(30)) as usize;
            let bar = "█".repeat(bar_len);
            println!("  {:20}  {}  {}m", app.bright_white(), bar.green(), mins,);
        }
        println!();
    }

    // Recent activity timeline
    println!("  {}", "Recent activity:".dimmed());
    for (action, payload, ts) in rows.iter().rev().take(8) {
        let app = payload
            .as_deref()
            .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
            .and_then(|v| {
                v["detail"]["app_id"]
                    .as_str()
                    .map(|s| s.to_string())
                    .or_else(|| {
                        v["detail"]["workspace_idx"]
                            .as_u64()
                            .map(|n| format!("workspace {}", n))
                    })
            })
            .unwrap_or_default();
        println!(
            "  {}  {}  {}",
            format_ts(*ts).dimmed(),
            action.cyan(),
            app.white(),
        );
    }

    Ok(())
}

pub fn why_attention(ctx: &AppContext) -> CoreResult<()> {
    let today = chrono_today();
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT action, payload, timestamp FROM events WHERE domain='compositor' AND timestamp >= ? ORDER BY timestamp ASC"
    )?;
    let rows: Vec<(String, i64)> = stmt
        .query_map(params![today], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    println!("{}", "🌲 Why — Attention Analysis".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    if rows.is_empty() {
        println!("  {}", "No compositor events today.".dimmed());
        return Ok(());
    }

    // Detect attention fragmentation: 3+ workspace switches within 60 seconds
    let switches: Vec<i64> = rows
        .iter()
        .filter(|(a, _)| a == "workspace.switch")
        .map(|(_, ts)| *ts)
        .collect();

    let mut fragments = 0u32;
    for window in switches.windows(3) {
        if window[2] - window[0] < 60 {
            fragments += 1;
        }
    }

    let focus_events: Vec<i64> = rows
        .iter()
        .filter(|(a, _)| a == "window.focus")
        .map(|(_, ts)| *ts)
        .collect();

    // Average focus duration
    let avg_focus = if focus_events.len() > 1 {
        let diffs: Vec<i64> = focus_events
            .windows(2)
            .map(|w| w[1] - w[0])
            .filter(|&d| d < 300)
            .collect();
        if diffs.is_empty() {
            0
        } else {
            diffs.iter().sum::<i64>() / diffs.len() as i64
        }
    } else {
        0
    };

    println!();
    let focus_quality = if avg_focus > 120 {
        "🟢 Deep focus"
    } else if avg_focus > 30 {
        "🟡 Moderate focus"
    } else {
        "🔴 Fragmented"
    };

    println!("  Focus quality:     {}", focus_quality.bright_white());
    println!("  Avg focus duration: {}s", avg_focus.to_string().cyan());
    println!(
        "  Workspace switches: {}",
        switches.len().to_string().yellow()
    );
    println!(
        "  Attention fragments: {} (rapid switch bursts)",
        fragments.to_string().bright_white()
    );
    println!();

    if fragments > 3 {
        println!(
            "  {} Attention highly fragmented today — consider single-task focus",
            "⚠️ ".yellow()
        );
    } else if fragments == 0 && avg_focus > 60 {
        println!("  {} Excellent focus discipline today", "✅".green());
    } else {
        println!("  {} Normal attention pattern", "·".dimmed());
    }

    Ok(())
}

// ─── LEDGER COMMANDS (Core v5 Phase 1) ───────────────────────────────────────

pub fn ledger_indexes(ctx: &AppContext) -> CoreResult<()> {
    println!("{}", "🌲 Ledger — Creating indexes...".cyan().bold());
    ctx.runtime.db.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_events_domain ON events(domain);
        CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
        CREATE INDEX IF NOT EXISTS idx_events_domain_ts ON events(domain, timestamp);
        CREATE INDEX IF NOT EXISTS idx_events_action ON events(action);
        CREATE INDEX IF NOT EXISTS idx_capabilities_ts ON capabilities_log(timestamp);
    ",
    )?;
    println!("  ✅ idx_events_domain");
    println!("  ✅ idx_events_timestamp");
    println!("  ✅ idx_events_domain_ts");
    println!("  ✅ idx_events_action");
    println!("  ✅ idx_capabilities_ts");
    println!();
    println!("  {} Ledger queries now use indexed scans", "⚡".yellow());
    Ok(())
}

pub fn ledger_stats(ctx: &AppContext) -> CoreResult<()> {
    // Total events
    let total: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))?;
    let first_ts: i64 = ctx
        .runtime
        .db
        .query_row("SELECT MIN(timestamp) FROM events", [], |r| r.get(0))
        .unwrap_or(0);
    let last_ts: i64 = ctx
        .runtime
        .db
        .query_row("SELECT MAX(timestamp) FROM events", [], |r| r.get(0))
        .unwrap_or(0);

    // Events per domain
    let mut stmt = ctx
        .runtime
        .db
        .prepare("SELECT domain, COUNT(*) as cnt FROM events GROUP BY domain ORDER BY cnt DESC")?;
    let domains: Vec<(String, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    // Events today
    let today = chrono_today();
    let today_count: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM events WHERE timestamp >= ?",
        [today],
        |r| r.get(0),
    )?;

    // Database size
    let db_size = std::fs::metadata(
        std::path::PathBuf::from(&ctx.core_root)
            .join("runtime")
            .join("state.db"),
    )
    .map(|m| m.len())
    .unwrap_or(0);

    println!("{}", "🌲 Ledger Stats".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();
    println!("  {} total events", total.to_string().bright_white().bold());
    println!("  {} events today", today_count.to_string().cyan());
    println!("  {} database size", format_bytes(db_size).green());
    println!();

    let first = format_ts(first_ts);
    let last = format_ts(last_ts);
    let span_days = (last_ts - first_ts) / 86400;
    println!(
        "  {} first event  •  {} last event  •  {} days of history",
        first.dimmed(),
        last.dimmed(),
        span_days.to_string().yellow()
    );
    println!();

    println!("  {}", "Events by domain:".dimmed());
    let max_count = domains.first().map(|d| d.1).unwrap_or(1);
    for (domain, count) in &domains {
        let bar_len = (count * 20 / max_count) as usize;
        let bar = "█".repeat(bar_len);
        println!(
            "  {:15}  {}  {}",
            domain.bright_white(),
            bar.cyan(),
            count.to_string().dimmed(),
        );
    }
    println!();

    // Capabilities log
    let cap_count: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM capabilities_log", [], |r| r.get(0))
        .unwrap_or(0);
    println!(
        "  {} capability log entries",
        cap_count.to_string().dimmed()
    );

    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / 1024.0 / 1024.0)
    }
}

pub fn ledger_query(ctx: &AppContext, domain: &str) -> CoreResult<()> {
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT id, action, payload, timestamp FROM events WHERE domain = ? ORDER BY timestamp DESC LIMIT 50"
    )?;
    let rows: Vec<(i64, String, Option<String>, i64)> = stmt
        .query_map([domain], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    println!(
        "{}",
        format!("🌲 Ledger — domain: {}", domain).cyan().bold()
    );
    println!("{}", "━".repeat(52).dimmed());

    if rows.is_empty() {
        println!("  {} No events found for domain '{}'", "·".dimmed(), domain);
        return Ok(());
    }

    println!(
        "  {} events (showing last {})",
        rows.len().to_string().bright_white(),
        rows.len().min(50)
    );
    println!();

    for (id, action, payload, ts) in &rows {
        let time = format_ts(*ts);
        // Extract result from payload
        let result = payload
            .as_deref()
            .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
            .and_then(|v| v["result"].as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "ok".to_string());

        // Extract health if doctor domain
        let extra = payload
            .as_deref()
            .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
            .and_then(|v| v["detail"]["health"].as_u64())
            .map(|h| format!("  health:{}%", h))
            .unwrap_or_default();

        let result_colored = if result == "ok" || result == "pass" {
            result.green().to_string()
        } else {
            result.yellow().to_string()
        };

        println!(
            "  {} {:6}  {}  {}{}",
            id.to_string().dimmed(),
            time.dimmed(),
            action.bright_white(),
            result_colored,
            extra.dimmed(),
        );
    }
    Ok(())
}

pub fn ledger_export(ctx: &AppContext) -> CoreResult<()> {
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT id, domain, action, payload, timestamp FROM events ORDER BY timestamp ASC",
    )?;
    let rows: Vec<serde_json::Value> = stmt
        .query_map([], |r| {
            let payload: Option<String> = r.get(3)?;
            Ok(serde_json::json!({
                "id": r.get::<_, i64>(0)?,
                "domain": r.get::<_, String>(1)?,
                "action": r.get::<_, String>(2)?,
                "payload": payload.as_deref()
                    .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
                    .unwrap_or(serde_json::Value::Null),
                "timestamp": r.get::<_, i64>(4)?,
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let export = serde_json::json!({
        "exported_at": chrono::Local::now().to_rfc3339(),
        "total": rows.len(),
        "events": rows,
    });

    let json = serde_json::to_string_pretty(&export)
        .map_err(|e| crate::errors::CoreError::Runtime(e.to_string()))?;
    println!("{}", json);
    Ok(())
}

// ─── CAUSALITY ENGINE (Core v5 Phase 3) ──────────────────────────────────────

pub fn why_health_since(ctx: &AppContext, since: &str) -> CoreResult<()> {
    // Parse since date — accept YYYY-MM-DD or "7d", "30d"
    let since_ts = if since.ends_with('d') {
        let days: i64 = since.trim_end_matches('d').parse().unwrap_or(7);
        chrono::Local::now().timestamp() - days * 86400
    } else {
        chrono::NaiveDate::parse_from_str(since, "%Y-%m-%d")
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
            .unwrap_or_else(|_| chrono::Local::now().timestamp() - 7 * 86400)
    };

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='doctor' AND timestamp >= ? ORDER BY timestamp ASC"
    )?;
    let rows: Vec<(Option<String>, i64)> = stmt
        .query_map(params![since_ts], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    println!(
        "{}",
        format!("🌲 Why — Health since {}", since).cyan().bold()
    );
    println!("{}", "━".repeat(52).dimmed());

    if rows.is_empty() {
        println!("  No doctor runs found since {}", since);
        return Ok(());
    }

    println!("  {} doctor runs over {} period\n", rows.len(), since);

    let mut prev: Option<i64> = None;
    let mut drops = vec![];

    for (payload, ts) in &rows {
        let health: i64 = payload
            .as_deref()
            .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
            .and_then(|v| v["detail"]["health"].as_i64())
            .unwrap_or(95);

        let date = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|d| {
                d.with_timezone(&chrono::Local)
                    .format("%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_default();

        let delta_str = match prev {
            Some(p) => {
                let d = health - p;
                if d > 0 {
                    format!(" ▲{}", d).green().to_string()
                } else if d < 0 {
                    drops.push((*ts, health, p));
                    format!(" ▼{}", d.abs()).bright_red().to_string()
                } else {
                    "  ·".dimmed().to_string()
                }
            }
            None => String::new(),
        };

        let bar = "█".repeat((health / 5) as usize);
        let health_colored = if health >= 95 {
            format!("{}%", health).green()
        } else if health >= 80 {
            format!("{}%", health).yellow()
        } else {
            format!("{}%", health).bright_red()
        };

        println!(
            "  {}  {} {}{}",
            date.dimmed(),
            bar.cyan(),
            health_colored,
            delta_str,
        );
        prev = Some(health);
    }

    if !drops.is_empty() {
        println!();
        println!("  {} Health drops detected:", "⚠️".yellow());
        for (ts, health, prev) in &drops {
            let date = chrono::DateTime::from_timestamp(*ts, 0)
                .map(|d| {
                    d.with_timezone(&chrono::Local)
                        .format("%Y-%m-%d %H:%M")
                        .to_string()
                })
                .unwrap_or_default();
            println!(
                "  {}  {}% → {}%  — run 'core why chain' for context",
                date.dimmed(),
                prev.to_string().green(),
                health.to_string().yellow()
            );
        }
    }

    println!("\n{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn why_causal(ctx: &AppContext, domain: &str) -> CoreResult<()> {
    // Find health drops and show what happened in the given domain around that time
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='doctor' ORDER BY timestamp DESC LIMIT 20"
    )?;
    let health_events: Vec<(i64, i64)> = stmt
        .query_map([], |r| {
            let payload: Option<String> = r.get(0)?;
            let ts: i64 = r.get(1)?;
            let h = payload
                .as_deref()
                .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
                .and_then(|v| v["detail"]["health"].as_i64())
                .unwrap_or(95);
            Ok((h, ts))
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Find drops
    let drops: Vec<(i64, i64, i64)> = health_events
        .windows(2)
        .filter(|w| w[0].0 < w[1].0) // current < previous = drop
        .map(|w| (w[1].1, w[1].0, w[0].0)) // (ts, prev_health, new_health)
        .collect();

    println!(
        "{}",
        format!("🌲 Why — Causal analysis: {}", domain)
            .cyan()
            .bold()
    );
    println!("{}", "━".repeat(52).dimmed());

    if drops.is_empty() {
        println!("  No health drops found in recent history");
        println!("  {} events in domain '{}' — system stable", domain, domain);
        return Ok(());
    }

    for (drop_ts, prev_h, new_h) in drops.iter().take(3) {
        let date = chrono::DateTime::from_timestamp(*drop_ts, 0)
            .map(|d| {
                d.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_default();

        println!();
        println!(
            "  📉 Health drop at {}  {}% → {}%",
            date.dimmed(),
            prev_h.to_string().green(),
            new_h.to_string().yellow()
        );

        // Find events in this domain within ±1 hour of the drop
        let window_start = drop_ts - 3600;
        let window_end = drop_ts + 3600;

        let mut dstmt = ctx.runtime.db.prepare(
            "SELECT action, payload, timestamp FROM events WHERE domain=? AND timestamp BETWEEN ? AND ? ORDER BY timestamp ASC"
        )?;
        let domain_events: Vec<(String, i64)> = dstmt
            .query_map(rusqlite::params![domain, window_start, window_end], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        if domain_events.is_empty() {
            println!("  {} No {} events in ±1h window", "·".dimmed(), domain);
        } else {
            println!(
                "  {} events in '{}' domain ±1h:",
                domain_events.len(),
                domain
            );
            for (action, ts) in &domain_events {
                let rel = ts - drop_ts;
                let rel_str = if rel < 0 {
                    format!("{}m before", (-rel) / 60)
                } else {
                    format!("{}m after", rel / 60)
                };
                println!(
                    "    {}  {}  {}",
                    format_ts(*ts).dimmed(),
                    action.cyan(),
                    rel_str.dimmed()
                );
            }
        }
    }

    println!("\n{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn why_chain(ctx: &AppContext) -> CoreResult<()> {
    // Find the most recent health drop and build a full causal chain
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='doctor' ORDER BY timestamp DESC LIMIT 10"
    )?;
    let health_events: Vec<(i64, i64)> = stmt
        .query_map([], |r| {
            let payload: Option<String> = r.get(0)?;
            let ts: i64 = r.get(1)?;
            let h = payload
                .as_deref()
                .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
                .and_then(|v| v["detail"]["health"].as_i64())
                .unwrap_or(95);
            Ok((h, ts))
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Find most recent drop
    let drop = health_events
        .windows(2)
        .find(|w| w[0].0 < w[1].0)
        .map(|w| (w[1].1, w[1].0, w[0].0));

    println!("{}", "🌲 Why — Causal Chain".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    let (drop_ts, prev_h, new_h) = match drop {
        Some(d) => d,
        None => {
            println!("  ✅ No health drops found in recent history");
            println!("  The forest is stable.");
            return Ok(());
        }
    };

    let date = chrono::DateTime::from_timestamp(drop_ts, 0)
        .map(|d| {
            d.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_default();

    println!();
    println!("  Last health drop: {}", date.bright_white());
    println!(
        "  {}% → {}%  (delta: {})",
        prev_h.to_string().green(),
        new_h.to_string().yellow(),
        format!("-{}", prev_h - new_h).bright_red(),
    );
    println!();
    println!("  {}", "Events in 2h window before drop:".dimmed());

    let window_start = drop_ts - 7200;

    let mut estmt = ctx.runtime.db.prepare(
        "SELECT domain, action, payload, timestamp FROM events WHERE timestamp BETWEEN ? AND ? ORDER BY timestamp ASC"
    )?;
    let all_events: Vec<(String, String, Option<String>, i64)> = estmt
        .query_map(rusqlite::params![window_start, drop_ts], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if all_events.is_empty() {
        println!("  No events found in the 2h window before the drop");
    } else {
        for (domain, action, payload, ts) in &all_events {
            let rel_secs = drop_ts - ts;
            let rel_str = format!("-{}m{}s", rel_secs / 60, rel_secs % 60);

            // Extract result
            let result = payload
                .as_deref()
                .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
                .and_then(|v| v["result"].as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "ok".to_string());

            let result_colored = if result == "ok" {
                result.green().to_string()
            } else {
                result.yellow().to_string()
            };

            println!(
                "  {}  {:12}  {:20}  {}  {}",
                format_ts(*ts).dimmed(),
                rel_str.dimmed(),
                domain.cyan(),
                action.white(),
                result_colored,
            );
        }
    }

    println!();
    println!("  {}", "After drop:".dimmed());

    let mut astmt = ctx.runtime.db.prepare(
        "SELECT domain, action, timestamp FROM events WHERE timestamp > ? ORDER BY timestamp ASC LIMIT 5"
    )?;
    let after: Vec<(String, String, i64)> = astmt
        .query_map(rusqlite::params![drop_ts], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    for (domain, action, ts) in &after {
        let rel_secs = ts - drop_ts;
        let rel_str = format!("+{}m{}s", rel_secs / 60, rel_secs % 60);
        println!(
            "  {}  {:12}  {:20}  {}",
            format_ts(*ts).dimmed(),
            rel_str.green().dimmed(),
            domain.cyan(),
            action.white(),
        );
    }

    println!("\n{}", "━".repeat(52).dimmed());
    Ok(())
}

// ─── PATTERN RECOGNITION + SUGGESTIONS (Core v5 Phase 4) ────────────────────

pub fn why_correlate(ctx: &AppContext, domain_a: &str, domain_b: &str) -> CoreResult<()> {
    println!(
        "{}",
        format!("🌲 Why — Correlate: {} ↔ {}", domain_a, domain_b)
            .cyan()
            .bold()
    );
    println!("{}", "━".repeat(52).dimmed());

    // Get all events for both domains
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT domain, action, timestamp FROM events WHERE domain IN (?, ?) ORDER BY timestamp ASC"
    )?;
    let events: Vec<(String, String, i64)> = stmt
        .query_map(rusqlite::params![domain_a, domain_b], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let a_events: Vec<i64> = events
        .iter()
        .filter(|(d, _, _)| d == domain_a)
        .map(|(_, _, ts)| *ts)
        .collect();
    let b_events: Vec<i64> = events
        .iter()
        .filter(|(d, _, _)| d == domain_b)
        .map(|(_, _, ts)| *ts)
        .collect();

    println!();
    println!(
        "  {} events in '{}'",
        a_events.len().to_string().cyan(),
        domain_a
    );
    println!(
        "  {} events in '{}'",
        b_events.len().to_string().cyan(),
        domain_b
    );
    println!();

    if a_events.is_empty() || b_events.is_empty() {
        println!("  Not enough data for correlation analysis");
        return Ok(());
    }

    // Find proximity patterns — how often does a B event follow an A event within 1h?
    let mut close_count = 0u32;
    let mut total_a = 0u32;
    for a_ts in &a_events {
        total_a += 1;
        let close = b_events.iter().any(|b_ts| {
            let diff = (b_ts - a_ts).abs();
            diff < 3600 && diff > 0
        });
        if close {
            close_count += 1;
        }
    }

    let proximity_pct = if total_a > 0 {
        close_count * 100 / total_a
    } else {
        0
    };

    // Average time between A and nearest B
    let avg_lag: i64 = a_events
        .iter()
        .filter_map(|a_ts| {
            b_events
                .iter()
                .filter(|b_ts| **b_ts > *a_ts)
                .map(|b_ts| b_ts - a_ts)
                .min()
        })
        .take(20)
        .sum::<i64>()
        / a_events.len().max(1) as i64;

    println!(
        "  {}% of '{}' events have '{}' activity within 1h",
        proximity_pct.to_string().bright_white(),
        domain_a,
        domain_b
    );

    if avg_lag > 0 && avg_lag < 86400 {
        println!(
            "  Avg time from '{}' to next '{}': {}m",
            domain_a,
            domain_b,
            (avg_lag / 60).to_string().cyan()
        );
    }

    println!();

    // Specific correlation insights
    if domain_a == "git" && domain_b == "doctor" {
        println!("  📊 Pattern: git activity → doctor runs");
        if proximity_pct > 70 {
            println!("  ✅ Strong correlation — you run doctor after git commits");
        } else {
            println!("  💡 Consider running doctor after major git changes");
        }
    } else if domain_a == "security" && domain_b == "doctor" {
        println!("  📊 Pattern: security scans → health impact");
        if proximity_pct > 50 {
            println!("  ✅ Security scans are regularly followed by health checks");
        }
    } else if domain_a == "update" && domain_b == "security" {
        println!("  📊 Pattern: updates → security scan follow-up");
        if proximity_pct > 50 {
            println!("  ✅ Good practice: security scanned after updates");
        } else {
            println!("  💡 Consider scanning security after system updates");
        }
    } else {
        if proximity_pct > 60 {
            println!("  ✅ Strong temporal correlation between domains");
        } else if proximity_pct > 30 {
            println!("  ·  Moderate correlation — some relationship detected");
        } else {
            println!("  ·  Weak correlation — domains appear independent");
        }
    }

    println!("\n{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn why_suggest(ctx: &AppContext) -> CoreResult<()> {
    println!("{}", "🌲 Why — Suggestions".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    let now = chrono::Local::now().timestamp();
    let mut suggestions: Vec<(u8, String, String)> = vec![]; // (priority, icon, message)

    // ── 1. Doctor run frequency ─────────────────────────────────────────────
    let last_doctor: Option<i64> = ctx
        .runtime
        .db
        .query_row(
            "SELECT MAX(timestamp) FROM events WHERE domain='doctor'",
            [],
            |r| r.get(0),
        )
        .ok()
        .flatten();

    if let Some(ts) = last_doctor {
        let hours_since = (now - ts) / 3600;
        if hours_since > 24 {
            suggestions.push((
                1,
                "⚠️".to_string(),
                format!(
                    "No doctor run in {}h — health drift risk elevated. Run: d",
                    hours_since
                ),
            ));
        } else if hours_since > 8 {
            suggestions.push((
                3,
                "💡".to_string(),
                format!("Last doctor run {}h ago — consider a check", hours_since),
            ));
        }
    }

    // ── 2. Security findings aging ───────────────────────────────────────────
    let last_security: Option<i64> = ctx
        .runtime
        .db
        .query_row(
            "SELECT MAX(timestamp) FROM events WHERE domain='security'",
            [],
            |r| r.get(0),
        )
        .ok()
        .flatten();

    if let Some(ts) = last_security {
        let days_since = (now - ts) / 86400;
        if days_since > 7 {
            suggestions.push((
                2,
                "🛡️".to_string(),
                format!(
                    "Security scan {}d ago — run: core security scan",
                    days_since
                ),
            ));
        }
    }

    // ── 3. Checkpoint age ────────────────────────────────────────────────────
    let core_root = std::path::PathBuf::from(&ctx.core_root);
    let cp_dir = core_root.join("runtime/checkpoints");
    let latest_cp = std::fs::read_dir(&cp_dir).ok().and_then(|d| {
        d.filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
            .map(|e| e.metadata().and_then(|m| m.modified()).ok())
            .flatten()
            .max()
    });

    if let Some(modified) = latest_cp {
        let age_secs = std::time::SystemTime::now()
            .duration_since(modified)
            .unwrap_or_default()
            .as_secs();
        let age_days = age_secs / 86400;
        if age_days > 7 {
            suggestions.push((
                3,
                "📸".to_string(),
                format!("Last checkpoint {}d ago — consider: cpc <name>", age_days),
            ));
        }
    }

    // ── 4. Health trend ──────────────────────────────────────────────────────
    let mut hstmt = ctx
        .runtime
        .db
        .prepare("SELECT payload FROM events WHERE domain='doctor' ORDER BY id DESC LIMIT 5")?;
    let recent_health: Vec<i64> = hstmt
        .query_map([], |r| {
            let p: Option<String> = r.get(0)?;
            Ok(p.as_deref()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                .and_then(|v| v["detail"]["health"].as_i64())
                .unwrap_or(95))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if recent_health.len() >= 3 {
        let avg: f64 =
            recent_health.iter().map(|h| *h as f64).sum::<f64>() / recent_health.len() as f64;
        let below_95 = recent_health.iter().filter(|&&h| h < 95).count();
        if below_95 >= 2 {
            suggestions.push((
                1,
                "📉".to_string(),
                format!(
                    "Health below 95% in {}/{} recent runs (avg: {:.0}%) — investigate warnings",
                    below_95,
                    recent_health.len(),
                    avg
                ),
            ));
        }
    }

    // ── 5. In-progress intents ───────────────────────────────────────────────
    let intents_dir = core_root.join("intents/future");
    let in_progress: Vec<String> = std::fs::read_dir(&intents_dir)
        .ok()
        .map(|d| {
            d.filter_map(|e| e.ok())
                .filter(|e| {
                    let content = std::fs::read_to_string(e.path()).unwrap_or_default();
                    content.contains("status: in-progress")
                })
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default();

    if in_progress.len() > 2 {
        suggestions.push((
            2,
            "🎯".to_string(),
            format!(
                "{} intents in-progress — focus on one: cistart <id>",
                in_progress.len()
            ),
        ));
    }

    // ── 6. Event ledger growth ───────────────────────────────────────────────
    let total_events: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
        .unwrap_or(0);

    let today_events: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM events WHERE timestamp >= ?",
            [chrono_today()],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // ── Output ───────────────────────────────────────────────────────────────
    println!();
    println!(
        "  {} Based on {} events across {} days of history",
        "📊".cyan(),
        total_events.to_string().bright_white(),
        ((now
            - ctx
                .runtime
                .db
                .query_row(
                    "SELECT MIN(timestamp) FROM events",
                    [],
                    |r: &rusqlite::Row| r.get::<_, i64>(0)
                )
                .unwrap_or(now))
            / 86400)
            .to_string()
            .dimmed(),
    );
    println!("  {} events today", today_events.to_string().cyan());
    println!();

    if suggestions.is_empty() {
        println!("  ✅ No suggestions — forest is in excellent shape");
        println!(
            "  {} Keep running d regularly to maintain health data",
            "💡".cyan()
        );
    } else {
        // Sort by priority
        suggestions.sort_by_key(|(p, _, _)| *p);
        for (_, icon, msg) in &suggestions {
            println!("  {}  {}", icon, msg.bright_white());
        }
    }

    println!("\n{}", "━".repeat(52).dimmed());
    Ok(())
}

// ─── COMPOSITOR INTELLIGENCE (Core v5 Phase 5) ───────────────────────────────

pub fn why_workspace(ctx: &AppContext) -> CoreResult<()> {
    let week_ago = chrono::Local::now().timestamp() - 7 * 86400;

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT action, payload, timestamp FROM events WHERE domain='compositor' AND timestamp >= ? ORDER BY timestamp ASC"
    )?;
    let events: Vec<(String, Option<String>, i64)> = stmt
        .query_map(rusqlite::params![week_ago], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    println!("{}", "🌲 Why — Workspace Activity (7 days)".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    if events.is_empty() {
        println!("  No compositor events — ensure faelight-niri-bridge is in autostart");
        return Ok(());
    }

    // Count by action
    let mut focus_count = 0u32;
    let mut switch_count = 0u32;
    let mut open_count = 0u32;
    let mut app_focus: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for (action, payload, _) in &events {
        match action.as_str() {
            "window.focus" => {
                focus_count += 1;
                let app = payload
                    .as_deref()
                    .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
                    .and_then(|v| v["detail"]["app_id"].as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "unknown".to_string());
                if app != "unknown" {
                    *app_focus.entry(app).or_insert(0) += 1;
                }
            }
            "workspace.switch" => switch_count += 1,
            "window.open" => open_count += 1,
            _ => {}
        }
    }

    println!();
    println!(
        "  {} total compositor events",
        events.len().to_string().bright_white()
    );
    println!(
        "  {} window focuses  •  {} workspace switches  •  {} windows opened",
        focus_count.to_string().cyan(),
        switch_count.to_string().yellow(),
        open_count.to_string().green(),
    );
    println!();

    // Top apps by focus count
    if !app_focus.is_empty() {
        println!("  {}", "Most focused apps (7 days):".dimmed());
        let mut apps: Vec<(String, u32)> = app_focus.into_iter().collect();
        apps.sort_by(|a, b| b.1.cmp(&a.1));
        let max = apps.first().map(|a| a.1).unwrap_or(1);
        for (app, count) in apps.iter().take(6) {
            let bar_len = (count * 20 / max) as usize;
            let bar = "█".repeat(bar_len);
            println!(
                "  {:25}  {}  {}x",
                app.bright_white(),
                bar.cyan(),
                count.to_string().dimmed(),
            );
        }
        println!();
    }

    // Workspace switch rate — fragmentation indicator
    let days = events.len() as f64 / 24.0; // rough session estimate
    let switch_rate = switch_count as f64 / days.max(1.0);

    let focus_quality = if switch_rate < 2.0 {
        "🟢 Deep focus sessions"
    } else if switch_rate < 5.0 {
        "🟡 Moderate switching"
    } else {
        "🔴 High fragmentation"
    };

    println!("  Focus quality: {}", focus_quality.bright_white());
    println!("  Workspace switch rate: {:.1}/session", switch_rate);

    // Correlation with health drops
    let mut hstmt = ctx.runtime.db.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='doctor' AND timestamp >= ? ORDER BY timestamp ASC"
    )?;
    let health_events: Vec<(i64, i64)> = hstmt
        .query_map(rusqlite::params![week_ago], |r| {
            let p: Option<String> = r.get(0)?;
            let h = p
                .as_deref()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                .and_then(|v| v["detail"]["health"].as_i64())
                .unwrap_or(95);
            Ok((h, r.get::<_, i64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let drops: Vec<i64> = health_events
        .windows(2)
        .filter(|w| w[0].0 < w[1].0)
        .map(|w| w[1].1)
        .collect();

    if !drops.is_empty() {
        let mut drop_switch_count = 0u32;
        for drop_ts in &drops {
            let switches_near_drop = events
                .iter()
                .filter(|(a, _, ts)| a == "workspace.switch" && (ts - drop_ts).abs() < 1800)
                .count();
            if switches_near_drop > 0 {
                drop_switch_count += 1;
            }
        }
        let drop_pct = drop_switch_count * 100 / drops.len().max(1) as u32;
        println!();
        println!(
            "  {}% of health drops had workspace switching within 30m",
            drop_pct.to_string().bright_white()
        );
        if drop_pct > 50 {
            println!("  💡 Attention fragmentation may precede health drift");
        }
    }

    println!("\n{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn why_focus(ctx: &AppContext) -> CoreResult<()> {
    let week_ago = chrono::Local::now().timestamp() - 7 * 86400;

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT action, timestamp FROM events WHERE domain='compositor' AND timestamp >= ? ORDER BY timestamp ASC"
    )?;
    let events: Vec<(String, i64)> = stmt
        .query_map(rusqlite::params![week_ago], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    println!("{}", "🌲 Why — Focus Analysis (7 days)".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    if events.is_empty() {
        println!("  No compositor events found");
        return Ok(());
    }

    // Group by day
    let mut daily_switches: std::collections::BTreeMap<String, u32> =
        std::collections::BTreeMap::new();
    let mut daily_focuses: std::collections::BTreeMap<String, u32> =
        std::collections::BTreeMap::new();

    for (action, ts) in &events {
        let day = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|d| d.with_timezone(&chrono::Local).format("%m-%d").to_string())
            .unwrap_or_default();
        match action.as_str() {
            "workspace.switch" => *daily_switches.entry(day).or_insert(0) += 1,
            "window.focus" => *daily_focuses.entry(day).or_insert(0) += 1,
            _ => {}
        }
    }

    println!();
    println!("  {}", "Daily focus quality:".dimmed());
    println!(
        "  {:8}  {:6}  {:7}  {}",
        "Date", "Focus", "Switch", "Quality"
    );
    println!("  {}", "─".repeat(35).dimmed());

    for day in daily_focuses.keys() {
        let focuses = daily_focuses.get(day).copied().unwrap_or(0);
        let switches = daily_switches.get(day).copied().unwrap_or(0);
        let ratio = if switches > 0 {
            focuses as f64 / switches as f64
        } else {
            focuses as f64
        };
        let quality = if ratio > 10.0 {
            "🟢 Deep"
        } else if ratio > 5.0 {
            "🟡 Moderate"
        } else {
            "🔴 Fragmented"
        };
        println!(
            "  {:8}  {:6}  {:7}  {}",
            day.bright_white(),
            focuses.to_string().cyan(),
            switches.to_string().yellow(),
            quality,
        );
    }

    // Fragmentation detection — 3+ switches in 60 seconds
    let switches: Vec<i64> = events
        .iter()
        .filter(|(a, _)| a == "workspace.switch")
        .map(|(_, ts)| *ts)
        .collect();

    let fragments = switches.windows(3).filter(|w| w[2] - w[0] < 60).count();

    println!();
    println!(
        "  Attention fragments detected: {}",
        fragments.to_string().bright_white()
    );
    if fragments > 5 {
        println!("  ⚠️  High fragmentation — consider single-workspace focus sessions");
    } else if fragments == 0 {
        println!("  ✅ No attention fragmentation — excellent focus discipline");
    } else {
        println!("  ·  Normal attention pattern");
    }

    println!("\n{}", "━".repeat(52).dimmed());
    Ok(())
}

// ── INT-129 — Event Log File Management ──────────────────────────────────────

pub fn status(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("events", &[Capability::FilesystemReadHome])?;
    let home = std::env::var("HOME").unwrap_or_default();
    let events_dir = std::path::PathBuf::from(&home).join("0-core/runtime/events");

    println!();
    println!(
        "{}",
        "  ╭─ 📋 Event Log Status ───────────────────────────────".bright_cyan()
    );

    if !events_dir.exists() {
        println!("  │  {} No event log directory yet", "○".dimmed());
        println!(
            "{}",
            "  ╰────────────────────────────────────────────────────".dimmed()
        );
        return Ok(());
    }

    let mut total_lines: usize = 0;
    let mut total_bytes: u64 = 0;
    let mut files: Vec<(String, u64, usize)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&events_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".jsonl") {
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let lines = std::fs::read_to_string(&path)
                    .map(|c| c.lines().count())
                    .unwrap_or(0);
                total_bytes += size;
                total_lines += lines;
                files.push((name, size, lines));
            }
        }
    }

    files.sort_by(|a, b| b.0.cmp(&a.0));

    println!(
        "  │  {} events across {} days",
        total_lines.to_string().bright_white(),
        files.len().to_string().bright_white()
    );
    println!("  │  Size: {}", format_bytes(total_bytes).bright_white());
    println!(
        "{}",
        "  ├────────────────────────────────────────────────────".dimmed()
    );
    for (name, size, lines) in files.iter().take(10) {
        println!(
            "  │  {:<22} {:>6} events  {}",
            name.bright_cyan(),
            lines.to_string().dimmed(),
            format_bytes(*size).dimmed()
        );
    }
    println!(
        "{}",
        "  ├────────────────────────────────────────────────────".dimmed()
    );
    println!("  │  Lifecycle: 30 days active  |  12 months archived");
    println!(
        "  │  Run {} to compress old logs",
        "core events archive".bright_cyan()
    );
    println!(
        "{}",
        "  ╰────────────────────────────────────────────────────".dimmed()
    );
    println!();
    Ok(())
}

pub fn archive(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("events", &[Capability::FilesystemReadHome])?;
    let home = std::env::var("HOME").unwrap_or_default();
    let events_dir = std::path::PathBuf::from(&home).join("0-core/runtime/events");
    let archive_dir = events_dir.join("archive");
    if !events_dir.exists() {
        println!("  {} No event log directory yet", "○".dimmed());
        return Ok(());
    }
    std::fs::create_dir_all(&archive_dir).ok();
    let now = chrono::Local::now();
    let mut archived = 0usize;
    let mut deleted = 0usize;
    let mut kept = 0usize;
    if let Ok(entries) = std::fs::read_dir(&events_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".jsonl") {
                continue;
            }
            let date_str = name.trim_end_matches(".jsonl");
            let age = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map(|d| {
                    let dt = chrono::DateTime::<chrono::Local>::from_naive_utc_and_offset(
                        d.and_hms_opt(0, 0, 0)
                            .unwrap_or_default()
                            .and_utc()
                            .naive_utc(),
                        *now.offset(),
                    );
                    now.signed_duration_since(dt).num_days()
                })
                .unwrap_or(0);
            if age > 365 {
                std::fs::remove_file(&path).ok();
                deleted += 1;
            } else if age > 30 {
                let gz = archive_dir.join(format!("{}.jsonl.gz", date_str));
                if compress_file(&path, &gz) {
                    std::fs::remove_file(&path).ok();
                    archived += 1;
                }
            } else {
                kept += 1;
            }
        }
    }
    println!();
    println!(
        "{}",
        "  ╭─ 📦 Event Log Archive ─────────────────────────────".bright_cyan()
    );
    println!(
        "  │  {} files kept (last 30 days)",
        kept.to_string().bright_white()
    );
    println!(
        "  │  {} files archived",
        archived.to_string().bright_yellow()
    );
    println!(
        "  │  {} files deleted (>12 months)",
        deleted.to_string().dimmed()
    );
    println!(
        "{}",
        "  ╰────────────────────────────────────────────────────".dimmed()
    );
    println!();
    Ok(())
}

fn compress_file(src: &std::path::PathBuf, dst: &std::path::PathBuf) -> bool {
    use std::io::{Read, Write};
    let Ok(mut input) = std::fs::File::open(src) else {
        return false;
    };
    let Ok(output) = std::fs::File::create(dst) else {
        return false;
    };
    let mut enc = flate2::write::GzEncoder::new(output, flate2::Compression::default());
    let mut buf = Vec::new();
    if input.read_to_end(&mut buf).is_err() {
        return false;
    }
    enc.write_all(&buf).is_ok() && enc.finish().is_ok()
}
