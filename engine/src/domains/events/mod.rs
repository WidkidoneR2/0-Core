//! events domain — query the event ledger
use crate::app::context::AppContext;
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
