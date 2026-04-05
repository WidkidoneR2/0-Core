use crate::app::context::AppContext;
use crate::cli::commands::PartnerCommand;
use crate::errors::CoreResult;
use colored::*;
// Jarvis gate for v14 partnership
const V14_JARVIS_GATE: i32 = 98;
fn get_jarvis_score(ctx: &AppContext) -> i32 {
    ctx.runtime.db.query_row(
        "SELECT score FROM jarvis_readiness_log ORDER BY recorded_at DESC LIMIT 1",
        [], |r| r.get(0)
    ).unwrap_or(0)
}
fn gate_check(ctx: &AppContext) -> Option<String> {
    let score = get_jarvis_score(ctx);
    if score < V14_JARVIS_GATE {
        Some(format!(
            "  {} v14 Partnership requires Jarvis {}/100 — current: {}/100\n  {} Build continues. Activation gates at {}.\n",
            "⏳".normal(),
            V14_JARVIS_GATE,
            score,
            "→".dimmed(),
            V14_JARVIS_GATE
        ))
    } else {
        None
    }
}
fn print_gate_warning(ctx: &AppContext) -> bool {
    if let Some(msg) = gate_check(ctx) {
        println!("{}", msg);
        true
    } else {
        false
    }
}
pub fn dispatch(cmd: PartnerCommand, ctx: &AppContext) -> CoreResult<()> {
    match cmd {
        PartnerCommand::Status => status(ctx),
        PartnerCommand::Propose => propose(ctx),
        PartnerCommand::Discuss { intent_id } => discuss(ctx, &intent_id),
        PartnerCommand::Disagree { intent_id } => disagree(ctx, &intent_id),
        PartnerCommand::Consult { question } => consult(ctx, &question),
        PartnerCommand::Reflect => reflect(ctx),
        PartnerCommand::Pattern => pattern(ctx),
        PartnerCommand::Growth => growth(ctx),
        PartnerCommand::Pushback => pushback(ctx),
        PartnerCommand::Roadmap => roadmap(ctx),
        PartnerCommand::RoadmapWhy => roadmap_why(ctx),
        PartnerCommand::RoadmapDiff => roadmap_diff(ctx),
    }
}
fn ensure_tables(ctx: &AppContext) {
    let _ = ctx.runtime.db.execute_batch("
        CREATE TABLE IF NOT EXISTS partner_proposals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            reasoning TEXT NOT NULL,
            accepted INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS partner_discussions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            intent_id TEXT NOT NULL,
            opinion TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS partner_disagreements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            intent_id TEXT NOT NULL,
            reason TEXT NOT NULL,
            outcome TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS partner_reflections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pattern TEXT NOT NULL,
            detail TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS co_authored_roadmap (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            phase TEXT NOT NULL,
            intent_ref TEXT,
            reasoning TEXT NOT NULL,
            priority INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL
        );
    ");
}
fn status(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);
    let score = get_jarvis_score(ctx);
    let gate_pct = (score as f64 / V14_JARVIS_GATE as f64 * 100.0) as i32;
    println!();
    println!("  {} Core v14 Partnership", "🤝".normal());
    println!("  {}", "─".repeat(48).dimmed());
    println!("  {:<24} {}/100", "Jarvis Score:".dimmed(), 
        if score >= V14_JARVIS_GATE { score.to_string().bright_green() } 
        else { score.to_string().bright_yellow() });
    println!("  {:<24} {}/100 (gate: {})", "v14 Gate:".dimmed(),
        gate_pct.to_string().bright_white(), V14_JARVIS_GATE);
    
    let activated = score >= V14_JARVIS_GATE;
    println!("  {:<24} {}", "Status:".dimmed(),
        if activated { "ACTIVE — partnership engaged".bright_green() }
        else { "DORMANT — building toward activation".bright_yellow() });
    
    // Count activity
    let proposals: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM partner_proposals", [], |r| r.get(0)
    ).unwrap_or(0);
    let disagreements: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM partner_disagreements", [], |r| r.get(0)
    ).unwrap_or(0);
    
    println!("  {}", "─".repeat(48).dimmed());
    println!("  {:<24} {}", "Proposals made:".dimmed(), proposals.to_string().bright_white());
    println!("  {:<24} {}", "Disagreements recorded:".dimmed(), disagreements.to_string().bright_white());
    println!();
    
    println!("  {} Phase readiness:", "▶".bright_cyan());
    let phases = [
        ("Phase 1 — Collaborative Intent", "propose / discuss / disagree"),
        ("Phase 2 — Shared Decision Making", "consult"),
        ("Phase 3 — Longitudinal Memory", "reflect / pattern / growth"),
        ("Phase 4 — Honest Disagreement", "pushback"),
        ("Phase 5 — Co-Authored Roadmap", "roadmap"),
    ];
    for (phase, cmds) in &phases {
        let status = if activated { "✅".to_string() } else { "⏳".to_string() };
        println!("  {}  {:<36} {}", status, phase, cmds.dimmed());
    }
    println!();
    Ok(())
}
// Phase 1 — Collaborative Intent Creation
fn propose(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);
    print_gate_warning(ctx);
    
    // Analyze patterns to generate proposals
    let now = chrono::Utc::now().timestamp();
    let window_7d = now - 604800;
    
    // Find domains with high failure rate
    let failed: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM forest_events WHERE kind='CommandFailed' AND timestamp > ?1",
        rusqlite::params![window_7d], |r| r.get(0)
    ).unwrap_or(0);
    
    // Count in-progress intents
    let root = std::path::PathBuf::from(&ctx.core_root);
    let future_dir = root.join("intents/future");
    let in_progress = std::fs::read_dir(&future_dir)
        .map(|d| d.flatten().filter(|e| {
            std::fs::read_to_string(e.path())
                .map(|c| c.contains("status: in-progress"))
                .unwrap_or(false)
        }).count())
        .unwrap_or(0);
    
    let complete_count: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM forest_events WHERE kind='CommandSucceeded' AND timestamp > ?1",
        rusqlite::params![window_7d], |r| r.get(0)
    ).unwrap_or(0);
    
    println!();
    println!("  {} Partner Proposals", "🤝".normal());
    println!("  {}", "─".repeat(48).dimmed());
    println!("  {} Analyzing forest patterns...", "→".dimmed());
    println!();
    
    let mut proposals: Vec<(String, String)> = Vec::new();
    
    if in_progress >= 5 {
        proposals.push((
            "Intent Focus Sprint".to_string(),
            format!("{} intents in-progress — coherence risk. Propose closing 2 before starting new work.", in_progress)
        ));
    }
    
    if failed >= 10 {
        proposals.push((
            "Failure Pattern Audit".to_string(),
            format!("{} command failures in 7 days — systematic issue likely. Propose a dedicated debugging session.", failed)
        ));
    }
    
    if complete_count > 50 {
        proposals.push((
            "Prediction Accuracy Review".to_string(),
            "High command volume this week — good time to verify prediction accuracy and close feedback loop.".to_string()
        ));
    }
    
    // Always propose roadmap review
    proposals.push((
        "Roadmap Alignment Check".to_string(),
        "Regular proposal: verify current work aligns with v14 Partnership path and May 3 gate.".to_string()
    ));
    
    if proposals.is_empty() {
        println!("  {} No proposals at this time — forest patterns look healthy", "○".dimmed());
    } else {
        for (i, (title, reason)) in proposals.iter().enumerate() {
            println!("  {} Proposal {}: {}", "💡".normal(), i+1, title.bright_white().bold());
            println!("     {}", reason.dimmed());
            // Store proposal
            let _ = ctx.runtime.db.execute(
                "INSERT INTO partner_proposals (title, reasoning, created_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![title, reason, now],
            );
            println!();
        }
    }
    println!("  {} The forest proposes. You decide.", "→".dimmed().italic());
    println!();
    Ok(())
}
fn discuss(ctx: &AppContext, intent_id: &str) -> CoreResult<()> {
    ensure_tables(ctx);
    print_gate_warning(ctx);
    let now = chrono::Utc::now().timestamp();
    
    // Find intent file
    let root = std::path::PathBuf::from(&ctx.core_root);
    let mut intent_content = None;
    for dir in &["intents/future", "intents/complete"] {
        if let Ok(entries) = std::fs::read_dir(root.join(dir)) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(intent_id) {
                    intent_content = std::fs::read_to_string(entry.path()).ok();
                    break;
                }
            }
        }
    }
    
    let opinion = match &intent_content {
        None => format!("Intent {} not found in ledger — verify the ID is correct.", intent_id),
        Some(content) => {
            let title = content.lines()
                .find(|l| l.starts_with("title:"))
                .map(|l| l.trim_start_matches("title:").trim().trim_matches('"'))
                .unwrap_or("unknown")
                .to_string();
            let gates_total = content.matches("⬜").count() + content.matches("✅").count();
            let gates_done = content.matches("✅").count();
            format!(
                "INT-{} ({}): {}/{} gates complete. {}",
                intent_id, title, gates_done, gates_total,
                if gates_done == gates_total { "All gates satisfied — ready to cicomplete." }
                else { "Gates remaining — continue building." }
            )
        }
    };
    
    println!();
    println!("  {} Discussion: INT-{}", "🤝".normal(), intent_id);
    println!("  {}", "─".repeat(48).dimmed());
    println!("  {}", opinion.bright_white());
    println!();
    
    let _ = ctx.runtime.db.execute(
        "INSERT INTO partner_discussions (intent_id, opinion, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![intent_id, &opinion, now],
    );
    Ok(())
}
fn disagree(ctx: &AppContext, intent_id: &str) -> CoreResult<()> {
    ensure_tables(ctx);
    print_gate_warning(ctx);
    let now = chrono::Utc::now().timestamp();
    
    let in_progress = {
        let root = std::path::PathBuf::from(&ctx.core_root);
        std::fs::read_dir(root.join("intents/future"))
            .map(|d| d.flatten().filter(|e| {
                std::fs::read_to_string(e.path())
                    .map(|c| c.contains("status: in-progress"))
                    .unwrap_or(false)
            }).count())
            .unwrap_or(0)
    };
    
    let reason = if in_progress >= 5 {
        format!(
            "Starting INT-{} now risks focus fragmentation. {} intents already in-progress. \
             Forest recommends completing one intent before starting new work. \
             Last time focus spread this wide, velocity dropped. Proceed anyway?",
            intent_id, in_progress
        )
    } else {
        format!(
            "No strong objection to INT-{}. {} intents in-progress — within acceptable range. \
             Forest sees no pattern conflicts with current trajectory.",
            intent_id, in_progress
        )
    };
    
    println!();
    println!("  {} Disagreement: INT-{}", "🤝".normal(), intent_id);
    println!("  {}", "─".repeat(48).dimmed());
    println!("  {}", reason.bright_white());
    println!();
    
    let _ = ctx.runtime.db.execute(
        "INSERT INTO partner_disagreements (intent_id, reason, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![intent_id, &reason, now],
    );
    Ok(())
}
// Phase 2 — Shared Decision Making
fn consult(ctx: &AppContext, question: &str) -> CoreResult<()> {
    ensure_tables(ctx);
    print_gate_warning(ctx);
    
    let score = get_jarvis_score(ctx);
    let events: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM forest_events", [], |r| r.get(0)
    ).unwrap_or(0);
    
    println!();
    println!("  {} Consultation", "🤝".normal());
    println!("  {}", "─".repeat(48).dimmed());
    println!("  Question: {}", question.bright_white().italic());
    println!();
    
    // Simple pattern matching for common questions
    let q = question.to_lowercase();
    let response = if q.contains("start") || q.contains("begin") || q.contains("cistart") {
        let in_progress = {
            let root = std::path::PathBuf::from(&ctx.core_root);
            std::fs::read_dir(root.join("intents/future"))
                .map(|d| d.flatten().filter(|e| {
                    std::fs::read_to_string(e.path())
                        .map(|c| c.contains("status: in-progress"))
                        .unwrap_or(false)
                }).count())
                .unwrap_or(0)
        };
        if in_progress >= 5 {
            format!("Caution: {} intents in-progress. Starting another risks coherence. Consider closing one first.", in_progress)
        } else {
            format!("{} intents in-progress — within range. Proceeding is reasonable.", in_progress)
        }
    } else if q.contains("retire") || q.contains("remove") || q.contains("delete") {
        "Before retiring: verify 0 usage in reality-check, confirm no dependencies, create checkpoint first.".to_string()
    } else if q.contains("deploy") || q.contains("release") {
        "Before deploying: run d to verify health >= 95%, ensure no uncommitted changes, check forecast trend.".to_string()
    } else if q.contains("jarvis") || q.contains("score") || q.contains("v14") {
        format!("Current Jarvis score: {}/100. v14 gate: {}/100. Gap: {} points.", score, V14_JARVIS_GATE, V14_JARVIS_GATE - score)
    } else {
        format!("Forest has observed {} events. Based on patterns: proceed with intent, run health check after, document decisions. The forest learns from outcomes.", events)
    };
    
    println!("  {} {}", "→".bright_cyan(), response.bright_white());
    println!();
    println!("  {} The forest advises. You decide.", "·".dimmed().italic());
    println!();
    Ok(())
}
// Phase 3 — Longitudinal Memory
fn reflect(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);
    print_gate_warning(ctx);
    
    let events: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM forest_events", [], |r| r.get(0)
    ).unwrap_or(0);
    let succeeded: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM forest_events WHERE kind='CommandSucceeded'", [], |r| r.get(0)
    ).unwrap_or(0);
    let failed: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM forest_events WHERE kind='CommandFailed'", [], |r| r.get(0)
    ).unwrap_or(0);
    
    let success_rate = if events > 0 { (succeeded * 100) / events } else { 0 };
    
    println!();
    println!("  {} Partner Reflection", "🤝".normal());
    println!("  {}", "─".repeat(48).dimmed());
    println!("  What the forest has learned about your work:");
    println!();
    println!("  {:<28} {} events ({} succeeded, {} failed)",
        "Command history:".dimmed(), events, succeeded, failed);
    println!("  {:<28} {}%", "Success rate:".dimmed(), success_rate);
    println!();
    
    // Most used domain
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT domain, COUNT(*) as cnt FROM forest_events WHERE kind='CommandSucceeded' GROUP BY domain ORDER BY cnt DESC LIMIT 3"
    ).unwrap();
    let domains: Vec<(String, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap().filter_map(|r| r.ok()).collect();
    
    if !domains.is_empty() {
        println!("  {} Most active domains:", "▶".bright_cyan());
        for (domain, count) in &domains {
            println!("    · {} — {} commands", domain.bright_white(), count);
        }
    }
    println!();
    Ok(())
}
fn pattern(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);
    print_gate_warning(ctx);
    
    println!();
    println!("  {} Work Pattern Analysis", "🤝".normal());
    println!("  {}", "─".repeat(48).dimmed());
    
    // Analyze time-of-day patterns from events
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT timestamp FROM forest_events ORDER BY timestamp DESC LIMIT 100"
    ).unwrap();
    let timestamps: Vec<i64> = stmt
        .query_map([], |r| r.get(0))
        .unwrap().filter_map(|r| r.ok()).collect();
    
    if timestamps.is_empty() {
        println!("  {} Not enough data yet — patterns emerge over time", "○".dimmed());
    } else {
        let mut hour_counts = [0i32; 24];
        for ts in &timestamps {
            let hour = chrono::DateTime::from_timestamp(*ts, 0)
                .map(|t| t.format("%H").to_string().parse::<usize>().unwrap_or(0))
                .unwrap_or(0);
            if hour < 24 { hour_counts[hour] += 1; }
        }
        let peak_hour = hour_counts.iter().enumerate()
            .max_by_key(|(_, c)| *c)
            .map(|(h, _)| h)
            .unwrap_or(0);
        println!("  {:<28} {}:00", "Peak work hour:".dimmed(), peak_hour);
        println!("  {:<28} {}", "Sample size:".dimmed(), timestamps.len());
    }
    println!();
    Ok(())
}
fn growth(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);
    print_gate_warning(ctx);
    
    let root = std::path::PathBuf::from(&ctx.core_root);
    let complete_count = std::fs::read_dir(root.join("intents/complete"))
        .map(|d| d.flatten().count())
        .unwrap_or(0);
    
    let commits = {
        std::process::Command::new("git")
            .args(["-C", &ctx.core_root, "rev-list", "--count", "HEAD"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<i64>().unwrap_or(0))
            .unwrap_or(0)
    };
    
    println!();
    println!("  {} Forest Growth", "🤝".normal());
    println!("  {}", "─".repeat(48).dimmed());
    println!("  {:<28} {} intents", "Completed intents:".dimmed(), complete_count.to_string().bright_white());
    println!("  {:<28} {} commits", "Total commits:".dimmed(), commits.to_string().bright_white());
    println!("  {:<28} v9 → v10 → v11 → v12 → v13 → v14", "Intelligence arc:".dimmed());
    println!("  {:<28} {}/100", "Jarvis Score:".dimmed(), get_jarvis_score(ctx).to_string().bright_green());
    println!();
    Ok(())
}
// Phase 4 — Honest Disagreement
fn pushback(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);
    print_gate_warning(ctx);
    
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT intent_id, reason, outcome, created_at FROM partner_disagreements ORDER BY created_at DESC LIMIT 10"
    ).unwrap();
    let rows: Vec<(String, String, Option<String>, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap().filter_map(|r| r.ok()).collect();
    
    println!();
    println!("  {} Pushback History", "🤝".normal());
    println!("  {}", "─".repeat(48).dimmed());
    
    if rows.is_empty() {
        println!("  {} No disagreements recorded yet", "○".dimmed());
        println!("  {} Use: core partner disagree <INT-NNN>", "→".dimmed());
    } else {
        for (intent_id, reason, outcome, ts) in &rows {
            let time = chrono::DateTime::from_timestamp(*ts, 0)
                .map(|t| t.format("%Y-%m-%d").to_string()).unwrap_or_default();
            println!("  {} INT-{} ({})", "↩".bright_yellow(), intent_id, time.dimmed());
            println!("     {}", reason.bright_white());
            if let Some(o) = outcome {
                println!("     Outcome: {}", o.bright_green());
            }
            println!();
        }
    }
    Ok(())
}
// Phase 5 — Co-Authored Roadmap
fn roadmap(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);
    print_gate_warning(ctx);
    
    let score = get_jarvis_score(ctx);
    
    println!();
    println!("  {} Co-Authored Roadmap", "🤝".normal());
    println!("  {}", "─".repeat(48).dimmed());
    println!("  Forest view of the optimal path forward:");
    println!();
    
    // Dynamic roadmap based on current state
    let v14_note = format!("requires {}/100 Jarvis -- {} more points", V14_JARVIS_GATE, (V14_JARVIS_GATE - score).max(0));
    let steps: Vec<(&str, &str, &str)> = vec![
        ("NOW", "INT-187 Delegation Engine", "trust contracts accumulating -- keep running"),
        ("NOW", "INT-179 fsh daily driver", "30-day clock running -- May 3 gate"),
        ("SOON", "INT-193 Tool Retirement", "prune dead tools -- cleaner forest"),
        ("SOON", "INT-194 fsh v4", "shell intelligence -- prediction-aware suggestions"),
        ("SOON", "INT-195 Forest Journal", "system writes its own story"),
        ("MAY 3", "Jarvis 100/100", "shell intelligence full +10 -- complete daily driver gate"),
        ("v14", "Core v14 Partnership", &v14_note),
    ];
    
    for (when, what, why) in &steps {
        let when_colored = match *when {
            "NOW" => when.bright_green(),
            "MAY 3" => when.bright_yellow(),
            "v14" => when.bright_cyan(),
            _ => when.normal(),
        };
        println!("  {} {:<30} {}", when_colored, what.bright_white(), why.dimmed());
    }
    println!();
    Ok(())
}
fn roadmap_why(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);
    print_gate_warning(ctx);
    println!();
    println!("  {} Roadmap Reasoning", "🤝".normal());
    println!("  {}", "─".repeat(48).dimmed());
    println!("  Why the forest recommends this order:");
    println!();
    println!("  {} INT-187 first — delegation trust data needed for v14 partnership", "1.".bright_white());
    println!("  {} INT-179 clock — shell intelligence score unlocks at May 3", "2.".bright_white());
    println!("  {} Tool retirement — cleaner registry improves reality-check accuracy", "3.".bright_white());
    println!("  {} fsh v4 — prediction-aware shell closes the intelligence feedback loop", "4.".bright_white());
    println!("  {} Journal — system autobiography required for longitudinal memory (v14 Phase 3)", "5.".bright_white());
    println!();
    Ok(())
}
fn roadmap_diff(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);
    print_gate_warning(ctx);
    println!();
    println!("  {} Roadmap Diff — Forest vs Current Plan", "🤝".normal());
    println!("  {}", "─".repeat(48).dimmed());
    println!("  Where forest view diverges from ledger order:");
    println!();
    println!("  {} Forest prioritizes INT-195 (journal) before fsh v4", "→".bright_yellow());
    println!("    Reason: longitudinal memory is a v14 prerequisite, not optional");
    println!();
    println!("  {} Forest suggests deferring voice I/O (INT-142, INT-147) until after v14", "→".bright_yellow());
    println!("    Reason: partnership model should be stable before adding new I/O channels");
    println!();
    println!("  {} No other significant divergences detected", "✅".normal());
    println!();
    Ok(())
}
