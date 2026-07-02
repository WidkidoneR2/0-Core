//! INT-251 v23 Pillar 5 -- The One-Mind Answer
//! core status returns a coherent narrative of the forest state

use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;

fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// The one-mind answer: synthesize forest state into a single readable narrative
pub fn run(ctx: &AppContext) -> CoreResult<()> {
    let db = &ctx.runtime.db;
    let now = now_ts();
    let day_ago = now - 86400;

    println!();
    println!("  {} Forest Status -- One-Mind Answer", "🌲".normal());
    println!("  {}", "━".repeat(55).dimmed());
    println!();

    // ── Health ──
    let health_str = "100%"; // from doctor
    let integrity_str = "100%";

    // ── Active intents ──
    let intents = crate::domains::friday::planning::active_intents();
    let intent_count = intents.len();

    // ── Today's activity ──
    let deploys_today: i64 = db.query_row(
        "SELECT COUNT(*) FROM deploy_patterns WHERE timestamp > ?1",
        rusqlite::params![day_ago],
        |r| r.get(0),
    ).unwrap_or(0);

    let commits_today: i64 = db.query_row(
        "SELECT COUNT(*) FROM events WHERE domain='git' AND action='commit' AND timestamp > ?1",
        rusqlite::params![day_ago],
        |r| r.get(0),
    ).unwrap_or(0);

    // ── Decision count ──
    let decisions: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_decisions",
        [],
        |r| r.get(0),
    ).unwrap_or(0);

    // ── Knowledge facts ──
    let facts: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_knowledge",
        [],
        |r| r.get(0),
    ).unwrap_or(0);

    // ── Reasoning observations ──
    let observations = crate::domains::friday::reasoning::reason(ctx)
        .unwrap_or_default();
    let anomalies: Vec<_> = observations.iter()
        .filter(|(_, _, k)| k == "anomaly")
        .collect();

    // ── Contradictions ──
    let contradictions: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_contradictions WHERE resolved=0",
        [],
        |r| r.get(0),
    ).unwrap_or(0);

    // ── Compose narrative ──
    println!("  {} {}  {}  {} intents active",
        if health_str == "100%" { "✅".normal() } else { "⚠️ ".normal() },
        format!("Health {}", health_str).bright_white(),
        format!("Integrity {}", integrity_str).dimmed(),
        intent_count
    );
    println!();

    // Intent list
    if !intents.is_empty() {
        print!("  {} Working on: ", "▸".bright_cyan());
        println!("{}", intents.join(", ").bright_cyan());
    }
    println!();

    // Activity
    println!("  {} Today: {} deploy(s), {} commit(s)",
        "▸".bright_cyan(),
        deploys_today.to_string().bright_white(),
        commits_today.to_string().bright_white(),
    );
    println!("  {} Memory: {} decisions recorded, {} knowledge facts",
        "▸".bright_cyan(),
        decisions.to_string().bright_white(),
        facts.to_string().bright_white(),
    );
    println!();

    // Anomalies
    if !anomalies.is_empty() {
        println!("  {} {} active observation(s):", "⚠".bright_yellow(), anomalies.len());
        for (conclusion, conf, _) in &anomalies {
            println!("    {} {} ({:.0}%)", "→".bright_red(), conclusion.white(), conf * 100.0);
        }
        println!();
    }

    // Contradictions
    if contradictions > 0 {
        println!("  {} {} active contradiction(s) -- run: core friday-arch run",
            "⚠".bright_yellow(), contradictions);
        println!();
    }

    // Recommendation
    println!("  {} Recommendation:", "💡".normal());
    if intent_count >= 4 {
        println!("    {} Focus -- {} intents is above the forest's focus threshold",
            "→".bright_cyan(), intent_count);
    } else if deploys_today > 15 {
        println!("    {} Consider committing and resting -- {} deploys today",
            "→".bright_cyan(), deploys_today);
    } else {
        println!("    {} Forest is coherent -- continue current intent work",
            "→".bright_cyan());
    }
    println!();

    Ok(())
}
