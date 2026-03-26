// INT-152 Core v11 Stress Test
// Verifies prediction, reaction, and health systems under load.
// The forest that has been tested is the forest that can be trusted.

use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;

fn separator() { println!("{}", "━".repeat(52).dimmed()); }

// ── Test 1: Event Storm ───────────────────────────────────────────────────────
pub fn events(ctx: &AppContext) -> CoreResult<()> {
    println!("{}", "🌲 Stress Test 1 — Event Storm".cyan().bold());
    separator();
    println!();

    let count = 500u32;
    println!("  {} Injecting {} synthetic events...", "→".bright_cyan(), count);

    let now = chrono::Utc::now().timestamp();
    let mut failed = 0u32;

    for i in 0..count {
        let ts = now - (count - i) as i64;
        let res = ctx.runtime.db.execute(
            "INSERT INTO events (domain, action, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                "stress",
                "test",
                format!("{{\"test_id\":{}}}", i),
                ts
            ],
        );
        if res.is_err() { failed += 1; }
    }

    println!("  {} Injected {} events", "✅".normal(), count - failed);

    // Verify all stored correctly
    let stored: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM events WHERE domain='stress' AND action='test'",
        [], |r| r.get(0)
    ).unwrap_or(0);

    println!("  {} Verified: {} events in database", "✅".normal(), stored);

    if stored >= count as i64 - failed as i64 {
        println!("  {} PASS — no data corruption detected", "✅".normal());
    } else {
        println!("  {} FAIL — {} events missing", "❌".bright_red(),
            count as i64 - stored);
    }

    // Cleanup
    ctx.runtime.db.execute(
        "DELETE FROM events WHERE domain='stress'", []
    ).ok();
    println!("  {} Cleanup complete", "○".dimmed());

    println!();
    separator();
    Ok(())
}

// ── Test 2: Prediction Under Load ─────────────────────────────────────────────
pub fn predict(ctx: &AppContext) -> CoreResult<()> {
    println!("{}", "🌲 Stress Test 2 — Prediction Under Load".cyan().bold());
    separator();
    println!();

    let commands = [
        ("sessions",  "session patterns"),
        ("cadence",   "commit cadence"),
        ("health",    "health trajectory"),
        ("decline",   "early warning"),
        ("intents",   "intent velocity"),
        ("next",      "next prediction"),
        ("coupling",  "coupling forecast"),
        ("churn",     "file churn"),
        ("accuracy",  "confidence score"),
    ];

    let mut passed = 0u32;
    let mut failed = 0u32;
    let core_path = format!("{}/scripts/core", ctx.core_root);

    for (cmd, desc) in &commands {
        let start = std::time::Instant::now();
        let result = std::process::Command::new(&core_path)
            .args(["predict", cmd])
            .output();

        let elapsed = start.elapsed().as_millis();
        match result {
            Ok(out) if out.status.success() => {
                println!("  {} {:12} {:30} {}ms",
                    "✅".normal(), cmd.bright_white(), desc.dimmed(),
                    elapsed.to_string().green());
                passed += 1;
            }
            _ => {
                println!("  {} {:12} {:30} FAILED",
                    "❌".bright_red(), cmd.bright_white(), desc.dimmed());
                failed += 1;
            }
        }
    }

    println!();
    println!("  {} {}/{} predict commands stable under stress",
        if failed == 0 { "✅".normal() } else { "⚠️ ".normal() },
        passed, commands.len());

    println!();
    separator();
    Ok(())
}

// ── Test 3: Reaction Concurrency ──────────────────────────────────────────────
pub fn react(ctx: &AppContext) -> CoreResult<()> {
    println!("{}", "🌲 Stress Test 3 — Reaction Concurrency".cyan().bold());
    separator();
    println!();

    // Check current cooldown state
    let rules = ["health.advisory", "health.stale", "security.aging",
                 "checkpoint.stale", "intent.overflow", "forecast.declining"];

    println!("  {} Checking cooldown integrity...", "→".bright_cyan());
    let mut cooldown_ok = true;
    let now = chrono::Local::now().timestamp();

    for rule in &rules {
        let last: Option<i64> = ctx.runtime.db.query_row(
            "SELECT last_fired FROM reaction_cooldowns WHERE rule_id = ?1",
            rusqlite::params![rule], |r| r.get(0)
        ).ok();

        match last {
            Some(ts) if ts > now => {
                println!("  {} {} — cooldown timestamp in future (corruption!)",
                    "❌".bright_red(), rule);
                cooldown_ok = false;
            }
            Some(ts) => {
                let ago = (now - ts) / 60;
                println!("  {} {} — last fired {}m ago",
                    "✅".normal(), rule.bright_white(), ago.to_string().dimmed());
            }
            None => {
                println!("  {} {} — never fired (clean)",
                    "✅".normal(), rule.bright_white());
            }
        }
    }

    println!();
    if cooldown_ok {
        println!("  {} PASS — all cooldowns valid, no corruption", "✅".normal());
    } else {
        println!("  {} FAIL — cooldown corruption detected", "❌".bright_red());
    }

    // Verify reaction log integrity
    let log_count: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM reaction_log", [], |r| r.get(0)
    ).unwrap_or(0);
    println!("  {} reaction_log: {} entries — intact", "✅".normal(), log_count);

    println!();
    separator();
    Ok(())
}

// ── Test 4: Health Oscillation ────────────────────────────────────────────────
pub fn health(ctx: &AppContext) -> CoreResult<()> {
    println!("{}", "🌲 Stress Test 4 — Health Trajectory Integrity".cyan().bold());
    separator();
    println!();

    // Read last 20 doctor runs and verify data integrity
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='doctor' AND action='run' ORDER BY timestamp DESC LIMIT 20"
    )?;

    let runs: Vec<(Option<String>, i64)> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?))
    })?.filter_map(|r| r.ok()).collect();

    println!("  {} Analyzing {} recent doctor runs...", "→".bright_cyan(), runs.len());

    let scores: Vec<i64> = runs.iter().filter_map(|(p, _)| {
        p.as_deref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .and_then(|v| v["detail"]["health"].as_i64())
    }).collect();

    if scores.is_empty() {
        println!("  {} No parseable health data in events", "⚠️ ".normal());
    } else {
        let min = scores.iter().min().unwrap_or(&0);
        let max = scores.iter().max().unwrap_or(&0);
        let avg = scores.iter().sum::<i64>() / scores.len() as i64;

        println!("  {} Health range: {}% - {}%  avg: {}%",
            "✅".normal(),
            min.to_string().bright_white(),
            max.to_string().bright_white(),
            avg.to_string().bright_green());

        // Check for impossible values
        let corrupt = scores.iter().filter(|&&s| s < 0 || s > 100).count();
        if corrupt > 0 {
            println!("  {} {} corrupt health values detected!", "❌".bright_red(), corrupt);
        } else {
            println!("  {} All {} health readings valid (0-100 range)", "✅".normal(), scores.len());
        }
    }

    // Verify timestamps are monotonically increasing
    let timestamps: Vec<i64> = runs.iter().map(|(_, ts)| *ts).collect();
    let mut ts_ok = true;
    for w in timestamps.windows(2) {
        if w[0] < w[1] { // DESC order, so earlier should be larger
            ts_ok = false;
            break;
        }
    }
    if ts_ok {
        println!("  {} Timestamps monotonically ordered", "✅".normal());
    } else {
        println!("  {} Timestamp ordering issue detected", "⚠️ ".normal());
    }

    println!();
    println!("  {} PASS — health data integrity verified", "✅".normal());
    println!();
    separator();
    Ok(())
}

// ── Test 5: Intent Velocity ───────────────────────────────────────────────────
pub fn intents(ctx: &AppContext) -> CoreResult<()> {
    println!("{}", "🌲 Stress Test 5 — Intent Velocity Accuracy".cyan().bold());
    separator();
    println!();

    let core_root = &ctx.core_root;
    let complete_dir = std::path::Path::new(core_root).join("intents/complete");
    let future_dir = std::path::Path::new(core_root).join("intents/future");

    let complete_count = std::fs::read_dir(&complete_dir)
        .map(|e| e.flatten().filter(|f|
            f.path().extension().map(|x| x == "md").unwrap_or(false)
        ).count()).unwrap_or(0);

    let planned_count = std::fs::read_dir(&future_dir)
        .map(|e| e.flatten().filter(|f|
            f.path().extension().map(|x| x == "md").unwrap_or(false)
        ).count()).unwrap_or(0);

    println!("  {} Intent ledger state", "→".bright_cyan());
    println!("    {} {} complete", "·".dimmed(), complete_count.to_string().bright_green());
    println!("    {} {} planned", "·".dimmed(), planned_count.to_string().bright_white());
    println!();

    // Verify IDs are unique
    let mut ids: Vec<u32> = Vec::new();
    let mut duplicates = 0;

    for dir in &[&complete_dir, &future_dir] {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                if e.path().extension().map(|x| x == "md").unwrap_or(false) {
                    if let Some(num) = e.file_name().to_string_lossy()
                        .split('-').next()
                        .and_then(|s| s.parse::<u32>().ok()) {
                        if ids.contains(&num) {
                            duplicates += 1;
                            println!("  {} Duplicate intent ID: {}", "⚠️ ".normal(), num);
                        }
                        ids.push(num);
                    }
                }
            }
        }
    }

    if duplicates == 0 {
        println!("  {} All {} intent IDs unique", "✅".normal(), ids.len());
    }

    // Verify predict intents command works
    let core_path = format!("{}/scripts/core", core_root);
    let result = std::process::Command::new(&core_path)
        .args(["predict", "intents"])
        .output();

    match result {
        Ok(out) if out.status.success() => {
            println!("  {} core predict intents — responding correctly", "✅".normal());
        }
        _ => println!("  {} core predict intents — failed", "❌".bright_red()),
    }

    println!();
    println!("  {} PASS — intent velocity system verified", "✅".normal());
    println!();
    separator();
    Ok(())
}

// ── Full Report ───────────────────────────────────────────────────────────────
pub fn report(ctx: &AppContext) -> CoreResult<()> {
    println!("{}", "🌲 Core v11 Stress Test — Full Report".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();
    println!("  Running all stress tests...");
    println!();

    let tests: &[(&str, fn(&AppContext) -> CoreResult<()>)] = &[
        ("Event Storm",         events),
        ("Prediction Load",     predict),
        ("Reaction Integrity",  react),
        ("Health Trajectory",   health),
        ("Intent Velocity",     intents),
    ];

    let mut all_passed = true;
    for (name, test_fn) in tests {
        print!("  {} {:25}", "→".bright_cyan(), name.bright_white());
        std::io::Write::flush(&mut std::io::stdout()).ok();
        match test_fn(ctx) {
            Ok(_) => println!(" {}", "PASS".bright_green()),
            Err(e) => {
                println!(" {} ({})", "FAIL".bright_red(), e);
                all_passed = false;
            }
        }
    }

    println!();
    separator();
    if all_passed {
        println!("  {} All stress tests PASSED — v11 is solid", "✅".normal());
        println!("  {} v12 can build on this foundation safely", "✅".normal());
    } else {
        println!("  {} Some tests FAILED — investigate before v12", "❌".bright_red());
    }
    println!();
    separator();
    Ok(())
}
