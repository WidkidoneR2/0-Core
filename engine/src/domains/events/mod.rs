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
    query_events(ctx, &format!("timestamp >= {}", chrono_today()), Some(domain))
}

fn query_events(ctx: &AppContext, where_clause: &str, domain_filter: Option<&str>) -> CoreResult<()> {
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
        let result = payload.as_deref()
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
    if s.ends_with('h') {
        s[..s.len()-1].parse::<i64>().unwrap_or(1) * 3600
    } else if s.ends_with('d') {
        s[..s.len()-1].parse::<i64>().unwrap_or(1) * 86400
    } else if s.ends_with('m') {
        s[..s.len()-1].parse::<i64>().unwrap_or(1) * 60
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
            format!("{:02}:{:02}:{:02}", secs/3600, (secs%3600)/60, secs%60)
        }
    }
}
