// INT-148 Core v11 — Prediction Domain
// Phase 1: Session Pattern Engine
// Phase 2: Health Trajectory Forecasting
//
// The forest anticipates before it happens.
// Pattern recognition applied with honesty about confidence.

use crate::app::context::AppContext;
use chrono::{Datelike, Timelike};
use crate::errors::CoreResult;
use colored::*;

// ── DB init ──────────────────────────────────────────────────────────────────
pub fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(
        "CREATE TABLE IF NOT EXISTS forest_predictions (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            kind         TEXT    NOT NULL,
            prediction   TEXT    NOT NULL,
            confidence   INTEGER NOT NULL,
            evidence     TEXT,
            created_at   INTEGER NOT NULL,
            expires_at   INTEGER
        );
        CREATE TABLE IF NOT EXISTS session_patterns (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            day_of_week  INTEGER NOT NULL,
            hour_start   INTEGER NOT NULL,
            hour_end     INTEGER NOT NULL,
            commit_count INTEGER NOT NULL DEFAULT 0,
            recorded_at  INTEGER NOT NULL
        );",
    )?;
    Ok(())
}

// ── Phase 1: Session Pattern Engine ──────────────────────────────────────────
pub fn sessions(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;

    println!("{}", "🌲 Predict — Session Patterns".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    // Analyze commit timestamps for work rhythm
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='git' AND action='commit' ORDER BY timestamp ASC"
    )?;

    let commits: Vec<(String, i64)> = stmt.query_map([], |r| {
        Ok((r.get::<_, Option<String>>(0)?.unwrap_or_default(), r.get(1)?))
    })?.filter_map(|r| r.ok()).collect();

    if commits.is_empty() {
        println!("  {} No commit history to analyze yet", "○".dimmed());
        return Ok(());
    }

    // Count commits by day of week and hour
    let mut by_day = [0u32; 7];
    let mut by_hour = [0u32; 24];
    let mut session_gaps: Vec<i64> = Vec::new();
    let mut last_ts: Option<i64> = None;

    for (_, ts) in &commits {
        let dt = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|d| d.with_timezone(&chrono::Local))
            .unwrap_or_else(|| chrono::Local::now());

        let dow = dt.weekday() as usize;
        let hour = dt.hour() as usize;
        by_day[dow] += 1;
        by_hour[hour] += 1;

        if let Some(prev) = last_ts {
            let gap = ts - prev;
            if gap > 0 { session_gaps.push(gap); }
        }
        last_ts = Some(*ts);
    }

    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let max_day = *by_day.iter().max().unwrap_or(&1).max(&1);

    println!("  {} Commit activity by day ({} total)", "▶".bright_cyan(), commits.len());
    for (i, &count) in by_day.iter().enumerate() {
        let bar = "█".repeat(((count as usize * 20) / max_day as usize).max(if count > 0 { 1 } else { 0 }));
        let highlight = if count == max_day { bar.bright_green().to_string() } else { bar.green().to_string() };
        println!("    {:3}  {} {}", days[i].bright_white(), highlight, count.to_string().dimmed());
    }
    println!();

    // Peak hours
    let max_hour = *by_hour.iter().max().unwrap_or(&1).max(&1);
    let peak_hours: Vec<usize> = by_hour.iter().enumerate()
        .filter(|(_, &c)| c as f32 >= max_hour as f32 * 0.6)
        .map(|(h, _)| h)
        .collect();

    println!("  {} Peak work hours", "▶".bright_cyan());
    if peak_hours.is_empty() {
        println!("    {} Not enough data yet", "·".dimmed());
    } else {
        for h in &peak_hours {
            println!("    {} {:02}:00–{:02}:00  {} commits",
                "·".dimmed(), h, h + 1,
                by_hour[*h].to_string().bright_white());
        }
    }
    println!();

    // Average session cadence
    if session_gaps.len() > 5 {
        let avg_gap = session_gaps.iter().sum::<i64>() / session_gaps.len() as i64;
        let avg_h = avg_gap / 3600;
        let avg_m = (avg_gap % 3600) / 60;
        println!("  {} Avg time between commits: {}h {}m", "▶".bright_cyan(),
            avg_h.to_string().bright_white(),
            avg_m.to_string().bright_white());
    }

    // Prediction: when is the next likely session?
    let now = chrono::Local::now();
    let current_dow = now.weekday() as usize;
    let best_day = by_day.iter().enumerate()
        .max_by_key(|(_, &c)| c)
        .map(|(i, _)| i)
        .unwrap_or(0);

    println!();
    println!("  {} Prediction", "▶".bright_yellow());
    if by_day[current_dow] >= by_day[best_day] / 2 {
        println!("    {} Today ({}) is a typical build day for you",
            "→".bright_green(), days[current_dow].bright_white());
    } else {
        println!("    {} {} is your most active day — consider scheduling there",
            "→".bright_cyan(), days[best_day].bright_white());
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    println!("  {} Run: core predict cadence  core predict health", "hint:".dimmed());
    Ok(())
}

pub fn cadence(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;

    println!("{}", "🌲 Predict — Commit Cadence".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    // Get commits per day over last 28 days
    let now = chrono::Local::now().timestamp();
    let window = 28 * 86400;

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT timestamp FROM events WHERE domain='git' AND action='commit' AND timestamp >= ?1 ORDER BY timestamp ASC"
    )?;

    let timestamps: Vec<i64> = stmt.query_map(
        rusqlite::params![now - window],
        |r| r.get(0)
    )?.filter_map(|r| r.ok()).collect();

    if timestamps.len() < 3 {
        println!("  {} Need more history — commit more to enable cadence prediction", "○".dimmed());
        return Ok(());
    }

    let total_days = 28;
    let per_day = timestamps.len() as f64 / total_days as f64;
    let per_week = per_day * 7.0;

    println!("  {} Last 28 days", "▶".bright_cyan());
    println!("    {} {} commits total", "·".dimmed(), timestamps.len().to_string().bright_white());
    println!("    {} {:.1} commits/day", "·".dimmed(), per_day.to_string().bright_white());
    println!("    {} {:.0} commits/week", "·".dimmed(), per_week.to_string().bright_white());
    println!();

    // Burst detection — find days with 5+ commits
    let mut day_counts: std::collections::HashMap<i64, u32> = std::collections::HashMap::new();
    for ts in &timestamps {
        let day = ts / 86400;
        *day_counts.entry(day).or_insert(0) += 1;
    }

    let burst_days = day_counts.values().filter(|&&c| c >= 5).count();
    let quiet_days = total_days - day_counts.len();

    println!("  {} Session character", "▶".bright_cyan());
    println!("    {} {} burst days (5+ commits)", "·".dimmed(), burst_days.to_string().bright_green());
    println!("    {} {} quiet days (0 commits)", "·".dimmed(), quiet_days.to_string().dimmed());
    println!();

    // Prediction: next week
    let predicted_next_week = (per_week * 1.05) as u32; // slight upward trend
    println!("  {} Prediction — next 7 days", "▶".bright_yellow());
    println!("    {} ~{} commits expected (based on current pace)",
        "→".bright_green(), predicted_next_week.to_string().bright_white());

    if burst_days > 3 {
        println!("    {} You build in bursts — expect 1-2 high-intensity sessions",
            "→".bright_cyan());
    } else {
        println!("    {} You have a steady cadence — consistent daily progress",
            "→".bright_cyan());
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

// ── Phase 2: Health Trajectory ────────────────────────────────────────────────
pub fn health(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;

    println!("{}", "🌲 Predict — Health Trajectory".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='doctor' AND action='run' ORDER BY timestamp DESC LIMIT 20"
    )?;

    let runs: Vec<(Option<String>, i64)> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?))
    })?.filter_map(|r| r.ok()).collect();

    if runs.len() < 3 {
        println!("  {} Need more doctor runs — run `d` more to enable health prediction", "○".dimmed());
        return Ok(());
    }

    // Extract health scores
    let scores: Vec<(i64, i64)> = runs.iter().filter_map(|(payload, ts)| {
        let h = payload.as_deref()
            .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
            .and_then(|v| v["detail"]["health"].as_i64())
            .unwrap_or(95);
        Some((*ts, h))
    }).collect();

    let scores: Vec<(i64, i64)> = scores.into_iter().rev().collect();

    // Show recent trend
    println!("  {} Recent health readings", "▶".bright_cyan());
    for (ts, h) in scores.iter().take(10) {
        let dt = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|d| d.with_timezone(&chrono::Local).format("%m-%d %H:%M").to_string())
            .unwrap_or_default();
        let bar = "█".repeat((h / 10) as usize);
        let colored = if *h >= 95 { bar.bright_green().to_string() }
            else if *h >= 80 { bar.yellow().to_string() }
            else { bar.bright_red().to_string() };
        println!("    {} {}  {}", dt.dimmed(), colored, format!("{}%", h).bright_white());
    }
    println!();

    // Trend calculation
    let n = scores.len() as f64;
    let sum_x: f64 = (0..scores.len()).map(|i| i as f64).sum();
    let sum_y: f64 = scores.iter().map(|(_, h)| *h as f64).sum();
    let sum_xy: f64 = scores.iter().enumerate().map(|(i, (_, h))| i as f64 * *h as f64).sum();
    let sum_x2: f64 = (0..scores.len()).map(|i| (i as f64).powi(2)).sum();
    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));

    let current = scores.last().map(|(_, h)| *h).unwrap_or(95);
    let predicted_3 = (current as f64 + slope * 3.0).clamp(0.0, 100.0) as i64;
    let predicted_7 = (current as f64 + slope * 7.0).clamp(0.0, 100.0) as i64;

    println!("  {} Trajectory forecast", "▶".bright_yellow());
    println!("    {} Current: {}%", "·".dimmed(), current.to_string().bright_white());

    let trend_str = if slope > 0.5 { "▲ improving".bright_green().to_string() }
        else if slope < -0.5 { "▼ declining".bright_red().to_string() }
        else { "→ stable".dimmed().to_string() };
    println!("    {} Trend: {}", "·".dimmed(), trend_str);
    println!("    {} In 3 runs: {}%", "·".dimmed(), predicted_3.to_string().bright_white());
    println!("    {} In 7 runs: {}%", "·".dimmed(), predicted_7.to_string().bright_white());
    println!();

    if slope < -1.0 {
        println!("  {} Health is declining — check recent changes", "⚠️ ".normal());
    } else if predicted_7 >= 100 {
        println!("  {} Forecast: 100% health within 7 doctor runs", "✅".normal());
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    println!("  {} Run: core predict decline  core predict sessions", "hint:".dimmed());
    Ok(())
}

pub fn decline(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;

    println!("{}", "🌲 Predict — Early Warning".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    // Check if health has dropped in last 5 runs
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='doctor' AND action='run' ORDER BY timestamp DESC LIMIT 5"
    )?;

    let scores: Vec<i64> = stmt.query_map([], |r| {
        let p: Option<String> = r.get(0)?;
        let h = p.as_deref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .and_then(|v| v["detail"]["health"].as_i64())
            .unwrap_or(95);
        Ok(h)
    })?.filter_map(|r| r.ok()).collect();

    if scores.len() < 3 {
        println!("  {} Not enough data yet for early warning", "○".dimmed());
        return Ok(());
    }

    let recent_avg = scores.iter().sum::<i64>() / scores.len() as i64;
    let min_score = *scores.iter().min().unwrap_or(&95);
    let dropping = scores.windows(2).all(|w| w[0] <= w[1]); // reversed: latest first

    println!("  {} Last {} doctor runs", "▶".bright_cyan(), scores.len());
    for (i, h) in scores.iter().enumerate() {
        let label = if i == 0 { "latest".dimmed().to_string() } else { "".to_string() };
        let col = if *h >= 95 { h.to_string().bright_green().to_string() }
            else { h.to_string().yellow().to_string() };
        println!("    {} {}%  {}", "·".dimmed(), col, label);
    }
    println!();

    println!("  {} Analysis", "▶".bright_yellow());
    println!("    {} Avg: {}%", "·".dimmed(), recent_avg.to_string().bright_white());
    println!("    {} Min: {}%", "·".dimmed(), min_score.to_string().bright_white());

    if dropping && min_score < 95 {
        println!();
        println!("  {} Health has been declining — investigate before it drops further",
            "⚠️ ".normal());
        println!("    {} Run: d  — check what warnings exist", "→".bright_cyan());
    } else if recent_avg >= 98 {
        println!();
        println!("  {} {} Forest health is excellent — no warnings detected",
            "✅".normal(), "".normal());
    } else {
        println!();
        println!("  {} Health is stable — no decline pattern detected", "✅".normal());
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}
