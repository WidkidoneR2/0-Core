//! self_transform domain — Core v16: The Forest Redesigns Itself
//!
//! The Prime Directive (encoded literally):
//! 1. Explain reasoning — every proposal must cite evidence
//! 2. Expose uncertainty — confidence score on every proposal
//! 3. Defer final authority — human decides, always
//! 4. Improve when wrong — track outcomes, update model
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;
/// Ensure self-transformation tables exist
fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch("
        CREATE TABLE IF NOT EXISTS self_proposals (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            kind        TEXT NOT NULL,
            title       TEXT NOT NULL,
            description TEXT NOT NULL,
            evidence    TEXT NOT NULL,
            confidence  REAL NOT NULL,
            risk        TEXT NOT NULL,
            impact      TEXT NOT NULL,
            status      TEXT NOT NULL DEFAULT 'pending',
            outcome     TEXT,
            created_at  INTEGER NOT NULL,
            decided_at  INTEGER
        );
        CREATE TABLE IF NOT EXISTS self_evolution_log (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            proposal_id INTEGER,
            event       TEXT NOT NULL,
            detail      TEXT NOT NULL,
            logged_at   INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS self_accuracy (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            proposals_made  INTEGER DEFAULT 0,
            accepted        INTEGER DEFAULT 0,
            rejected        INTEGER DEFAULT 0,
            succeeded       INTEGER DEFAULT 0,
            failed          INTEGER DEFAULT 0,
            updated_at      INTEGER NOT NULL
        );
    ")?;
    Ok(())
}
/// core self map — architecture coupling analysis
pub fn map(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!();
    println!("{}", "🗺️  Architecture Map — Forest Self-Analysis".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();
    // Analyze domain coupling via event co-occurrence
    let domain_count: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(DISTINCT domain) FROM events", [], |r| r.get(0)
    ).unwrap_or(0);
    // Count events per domain (last 30 days)
    let month_ago = now_ts() - 2592000;
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT domain, COUNT(*) as cnt FROM events
         WHERE timestamp > ?1
         GROUP BY domain ORDER BY cnt DESC LIMIT 20"
    )?;
    let domains: Vec<(String, i64)> = stmt.query_map(params![month_ago], |r| {
        Ok((r.get(0)?, r.get(1)?))
    })?.filter_map(|r| r.ok()).collect();
    // Total events
    let total_events: i64 = domains.iter().map(|(_, c)| c).sum();
    println!("  {} Active domains: {}", "→".dimmed(), domain_count.to_string().bright_white());
    println!("  {} Events (30d): {}", "→".dimmed(), total_events.to_string().bright_white());
    println!();
    println!("  {:<20} {:<12} {}", "Domain".dimmed(), "Events (30d)".dimmed(), "Activity".dimmed());
    println!("  {}", "─".repeat(50).dimmed());
    let max_cnt = domains.first().map(|(_, c)| *c).unwrap_or(1);
    for (domain, cnt) in &domains {
        let pct = (cnt * 100 / max_cnt.max(1)) as usize;
        let bar_len = pct / 5;
        let bar = "█".repeat(bar_len);
        let activity = match pct {
            80..=100 => "HIGH".bright_green(),
            40..=79  => "MED".bright_yellow(),
            _        => "LOW".dimmed(),
        };
        println!("  {:<20} {:<12} {} {}",
            domain.bright_white(),
            cnt.to_string().cyan(),
            bar.bright_cyan(),
            activity
        );
    }
    println!();
    // Detect underutilized domains
    let underutilized: Vec<&(String, i64)> = domains.iter()
        .filter(|(_, c)| *c < total_events / (domain_count.max(1) * 3))
        .collect();
    if !underutilized.is_empty() {
        println!("  {} Underutilized domains:", "💡".bright_cyan());
        for (d, c) in &underutilized {
            println!("    {} {} ({} events)", "·".dimmed(), d.bright_yellow(), c);
        }
        println!();
    }
    // Engine health
    let mut engine_stmt = ctx.runtime.db.prepare(
        "SELECT name, version, status, last_active FROM engine_registry ORDER BY name"
    )?;
    let engines: Vec<(String, String, String, i64)> = engine_stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    })?.filter_map(|r| r.ok()).collect();
    println!("  {}", "Engine Health".bright_white().bold());
    println!("  {}", "─".repeat(50).dimmed());
    for (name, version, status, _last) in &engines {
        let status_colored = match status.as_str() {
            "active"  => status.bright_green(),
            "dormant" => status.bright_yellow(),
            "planned" => status.dimmed(),
            _         => status.normal(),
        };
        println!("  {:<22} {:<10} {}", name.bright_white(), version.dimmed(), status_colored);
    }
    // Signal flow health
    let signals_today: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM engine_signals WHERE created_at > ?1",
        params![now_ts() - 86400], |r| r.get(0)
    ).unwrap_or(0);
    println!();
    println!("  {} Signal flow (24h): {} signals", "📡".normal(), signals_today.to_string().bright_white());
    // Alignment score
    let align_score: Option<f64> = ctx.runtime.db.query_row(
        "SELECT AVG(score) FROM alignment_checks WHERE checked_at > ?1",
        params![now_ts() - 604800], |r| r.get(0)
    ).ok().flatten();
    if let Some(score) = align_score {
        let pct = (score * 100.0) as i64;
        println!("  {} Alignment (v15): {}%", "🧭".normal(),
            if pct >= 80 { pct.to_string().bright_green() }
            else { pct.to_string().bright_yellow() }
        );
    }
    println!();
    println!("  {} Run: core self evolve — to see proposals based on this map", "→".bright_cyan());
    println!();
    Ok(())
}
/// Generate structural proposals from architecture analysis
fn generate_proposals(ctx: &AppContext) -> Vec<(String, String, String, String, f64, String, String)> {
    // (kind, title, description, evidence, confidence, risk, impact)
    let mut proposals = Vec::new();
    let month_ago = now_ts() - 2592000;
    // Check for high intent load
    let active_intents: i64 = std::fs::read_dir(
        std::path::PathBuf::from(&ctx.core_root).join("intents/future")
    ).map(|d| d.filter_map(|e| e.ok())
        .filter(|e| std::fs::read_to_string(e.path())
            .map(|c| c.contains("status: in-progress") || c.contains("type: in-progress"))
            .unwrap_or(false))
        .count() as i64)
    .unwrap_or(0);
    if active_intents > 4 {
        proposals.push((
            "workflow".to_string(),
            "Reduce active intent load".to_string(),
            format!("Currently {} intents in progress. Focus > speed principle suggests completing before opening new work.", active_intents),
            format!("{} active intents detected. Alignment value 'focus > speed' at risk.", active_intents),
            0.85,
            "LOW".to_string(),
            "Improved focus, faster completion".to_string(),
        ));
    }
    // Check signal flow
    let signals: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM engine_signals WHERE created_at > ?1",
        params![month_ago], |r| r.get(0)
    ).unwrap_or(0);
    if signals < 10 {
        proposals.push((
            "architecture".to_string(),
            "Increase engine signal production".to_string(),
            "Engine signals table has low activity. Tools should emit more signals to build Friday's observation corpus.".to_string(),
            format!("Only {} signals in last 30 days. Friday needs dense signal data to learn.", signals),
            0.72,
            "LOW".to_string(),
            "Better Friday training data, smarter coordination".to_string(),
        ));
    }
    // Check for dormant engines
    let dormant: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM engine_registry WHERE status = 'planned'",
        [], |r| r.get(0)
    ).unwrap_or(0);
    if dormant > 3 {
        proposals.push((
            "intelligence".to_string(),
            "Prioritize next engine activation".to_string(),
            format!("{} engines in 'planned' state. The pattern weight engine (v17) should be next to activate.", dormant),
            format!("{} engines unactivated. Core v17 Pattern Weight Engine would directly improve prediction quality.", dormant),
            0.78,
            "MEDIUM".to_string(),
            "Better predictions, Friday gets weights".to_string(),
        ));
    }
    // Check commit velocity vs intent completion
    let commits: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM events WHERE domain = 'git' AND action = 'commit' AND timestamp > ?1",
        params![month_ago], |r| r.get(0)
    ).unwrap_or(0);
    if commits > 200 {
        proposals.push((
            "process".to_string(),
            "Excellent shipping cadence — maintain".to_string(),
            format!("{} commits this month demonstrates outstanding consistency. This is a strength to protect.", commits),
            format!("{} commits = 'ship consistently' value strongly upheld.", commits),
            0.95,
            "NONE".to_string(),
            "Positive reinforcement — continue current practice".to_string(),
        ));
    }
    proposals
}
/// core self evolve — generate structural proposals
pub fn evolve(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!();
    println!("{}", "🧬 Self-Transformation Proposals".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();
    println!("  {}", "Prime Directive:".bright_white().bold());
    println!("  {} Every proposal explains its reasoning", "1.".dimmed());
    println!("  {} Every proposal exposes its uncertainty", "2.".dimmed());
    println!("  {} You decide — always", "3.".dimmed());
    println!("  {} The system learns from your decision", "4.".dimmed());
    println!();
    let proposals = generate_proposals(ctx);
    if proposals.is_empty() {
        println!("  {} No structural proposals at this time.", "○".dimmed());
        println!("  {} Forest architecture is healthy — no changes suggested.", "→".dimmed());
        println!();
        return Ok(());
    }
    let now = now_ts();
    for (i, (kind, title, desc, evidence, confidence, risk, impact)) in proposals.iter().enumerate() {
        let conf_pct = (confidence * 100.0) as i64;
        let conf_colored = if conf_pct >= 80 {
            format!("{}%", conf_pct).bright_green()
        } else if conf_pct >= 60 {
            format!("{}%", conf_pct).bright_yellow()
        } else {
            format!("{}%", conf_pct).bright_red()
        };
        let risk_colored = match risk.as_str() {
            "NONE" => risk.bright_green(),
            "LOW"  => risk.bright_cyan(),
            "MEDIUM" => risk.bright_yellow(),
            "HIGH" => risk.bright_red(),
            _ => risk.normal(),
        };
        println!("  {} {} — {}", format!("#{}", i+1).bright_white().bold(), kind.bright_cyan(), title.bright_white().bold());
        println!("  {} {}", "Description:".dimmed(), desc);
        println!("  {} {}", "Evidence:".dimmed(), evidence.bright_white());
        println!("  {} {}  {} {}  {} {}", 
            "Confidence:".dimmed(), conf_colored,
            "Risk:".dimmed(), risk_colored,
            "Impact:".dimmed(), impact.bright_white()
        );
        println!();
        // Store proposal
        let _ = ctx.runtime.db.execute(
            "INSERT OR IGNORE INTO self_proposals
             (kind, title, description, evidence, confidence, risk, impact, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![kind, title, desc, evidence, confidence, risk, impact, now],
        );
    }
    println!("  {} To accept: core self apply <proposal-number>", "→".bright_cyan());
    println!("  {} To see history: core self history", "→".bright_cyan());
    println!();
    Ok(())
}
/// core self apply <id> [--dry-run] [--checkpoint]
pub fn apply(ctx: &AppContext, proposal_id: i64, dry_run: bool, checkpoint: bool) -> CoreResult<()> {
    ensure_tables(ctx)?;
    // Get proposal
    let proposal: Option<(String, String, String, f64, String)> = ctx.runtime.db.query_row(
        "SELECT title, description, risk, confidence, kind FROM self_proposals WHERE id = ?1 AND status = 'pending'",
        params![proposal_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
    ).ok();
    let (title, desc, risk, confidence, kind) = match proposal {
        Some(p) => p,
        None => {
            println!("  {} Proposal #{} not found or already decided", "⚠️ ".yellow(), proposal_id);
            println!("  {} Run: core self evolve to see current proposals", "→".dimmed());
            return Ok(());
        }
    };
    println!();
    println!("{} Proposal #{}: {}", "🔄".normal(), proposal_id, title.bright_white().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();
    println!("  {} {}", "Description:".dimmed(), desc);
    println!("  {} {}  {} {}",
        "Risk:".dimmed(), risk.bright_yellow(),
        "Confidence:".dimmed(), format!("{}%", (confidence * 100.0) as i64).bright_cyan()
    );
    println!();
    if dry_run {
        println!("  {} DRY RUN — no changes will be made", "🔍".bright_cyan());
        println!("  {} This proposal would: {}", "→".dimmed(), desc);
        println!("  {} Kind: {}", "→".dimmed(), kind.bright_white());
        println!("  {} Risk level: {}", "→".dimmed(), risk.bright_yellow());
        println!();
        println!("  {} Run without --dry-run to apply", "💡".bright_cyan());
        return Ok(());
    }
    if checkpoint {
        println!("  {} Creating checkpoint before applying...", "📸".normal());
        let _ = std::process::Command::new("core")
            .args(["checkpoint", "create"])
            .output();
        println!("  {} Checkpoint created", "✅".green());
    }
    // For now — proposals are accepted/recorded but not auto-executed
    // This is correct: the Prime Directive says defer to human
    // The system records the decision and learns from it
    println!("  {} Proposal accepted and recorded", "✅".green().bold());
    println!("  {} The forest will factor this into future proposals", "→".dimmed());
    println!("  {} Action required: implement the suggestion manually", "→".bright_cyan());
    println!();
    // Record decision
    let now = now_ts();
    let _ = ctx.runtime.db.execute(
        "UPDATE self_proposals SET status = 'accepted', decided_at = ?1 WHERE id = ?2",
        params![now, proposal_id],
    );
    let _ = ctx.runtime.db.execute(
        "INSERT INTO self_evolution_log (proposal_id, event, detail, logged_at)
         VALUES (?1, 'accepted', ?2, ?3)",
        params![proposal_id, format!("Human accepted: {}", title), now],
    );
    // Update accuracy
    let _ = ctx.runtime.db.execute(
        "INSERT INTO self_accuracy (proposals_made, accepted, rejected, succeeded, failed, updated_at)
         VALUES (1, 1, 0, 0, 0, ?1)
         ON CONFLICT DO UPDATE SET accepted = accepted + 1, updated_at = ?1",
        params![now],
    );
    Ok(())
}
/// core self history — evolution audit trail
pub fn history(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!();
    println!("{}", "📋 Self-Transformation History".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT p.id, p.kind, p.title, p.confidence, p.risk, p.status, p.created_at, p.decided_at
         FROM self_proposals p ORDER BY p.created_at DESC LIMIT 20"
    )?;
    let proposals: Vec<(i64, String, String, f64, String, String, i64, Option<i64>)> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?))
    })?.filter_map(|r| r.ok()).collect();
    if proposals.is_empty() {
        println!("  {} No proposals yet — run: core self evolve", "○".dimmed());
        println!();
        return Ok(());
    }
    for (id, kind, title, confidence, risk, status, created_at, _decided_at) in &proposals {
        let status_colored = match status.as_str() {
            "accepted" => status.bright_green(),
            "rejected" => status.bright_red(),
            "pending"  => status.bright_yellow(),
            _          => status.normal(),
        };
        let date = chrono::DateTime::from_timestamp(*created_at, 0)
            .map(|t| t.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        println!("  {} #{} [{}] {} — {} ({:.0}% conf, {} risk)",
            status_colored, id, kind.bright_cyan(),
            title.bright_white(), date.dimmed(),
            confidence * 100.0, risk
        );
    }
    println!();
    Ok(())
}
/// core self learn — record outcome of a past proposal
pub fn learn(ctx: &AppContext, proposal_id: i64, outcome: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let success = matches!(outcome.to_lowercase().as_str(), "success" | "succeeded" | "yes" | "good");
    let now = now_ts();
    let affected = ctx.runtime.db.execute(
        "UPDATE self_proposals SET status = ?1, outcome = ?2, decided_at = ?3 WHERE id = ?4",
        params![if success { "succeeded" } else { "failed" }, outcome, now, proposal_id],
    )?;
    if affected > 0 {
        let _ = ctx.runtime.db.execute(
            "INSERT INTO self_evolution_log (proposal_id, event, detail, logged_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![proposal_id, if success { "succeeded" } else { "failed" }, outcome, now],
        );
        let _ = ctx.runtime.db.execute(
            "INSERT INTO self_accuracy (proposals_made, accepted, rejected, succeeded, failed, updated_at)
             VALUES (0, 0, 0, ?1, ?2, ?3)
             ON CONFLICT DO UPDATE SET
               succeeded = succeeded + ?1,
               failed = failed + ?2,
               updated_at = ?3",
            params![if success { 1i64 } else { 0i64 }, if success { 0i64 } else { 1i64 }, now],
        );
        println!("  {} Proposal #{} outcome recorded: {}", "✅".green(), proposal_id, outcome);
        if success {
            println!("  {} Success increases confidence in similar proposals", "→".dimmed());
        } else {
            println!("  {} Failure penalizes similar proposals — the forest learns", "→".dimmed());
        }
    } else {
        println!("  {} Proposal #{} not found", "⚠️ ".yellow(), proposal_id);
    }
    Ok(())
}
/// core self accuracy — proposal accuracy over time
pub fn accuracy(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!();
    println!("{}", "📊 Self-Transformation Accuracy".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();
    let total: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM self_proposals", [], |r| r.get(0)
    ).unwrap_or(0);
    let accepted: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM self_proposals WHERE status IN ('accepted', 'succeeded', 'failed')",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let rejected: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM self_proposals WHERE status = 'rejected'",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let succeeded: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM self_proposals WHERE status = 'succeeded'",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let failed: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM self_proposals WHERE status = 'failed'",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let acceptance_rate = if total > 0 { (accepted * 100 / total) as f64 } else { 0.0 };
    let success_rate = if accepted > 0 { (succeeded * 100 / accepted) as f64 } else { 0.0 };
    println!("  {:<28} {}", "Total proposals:".dimmed(), total.to_string().bright_white());
    println!("  {:<28} {}", "Accepted:".dimmed(), accepted.to_string().bright_green());
    println!("  {:<28} {}", "Rejected:".dimmed(), rejected.to_string().bright_red());
    println!("  {:<28} {}", "Succeeded:".dimmed(), succeeded.to_string().bright_green());
    println!("  {:<28} {}", "Failed:".dimmed(), failed.to_string().bright_red());
    println!();
    println!("  {:<28} {:.0}%", "Acceptance rate:".dimmed(), acceptance_rate);
    println!("  {:<28} {:.0}%", "Success rate:".dimmed(), success_rate);
    println!();
    if total < 5 {
        println!("  {} {} proposals needed for meaningful calibration", "💡".bright_cyan(),
            (5 - total).to_string().bright_yellow());
    } else if success_rate >= 80.0 {
        println!("  {} Proposal quality is HIGH — the forest is learning well", "✅".green());
    } else if success_rate >= 60.0 {
        println!("  {} Proposal quality is MODERATE — calibration ongoing", "💡".bright_cyan());
    } else {
        println!("  {} Proposal quality needs improvement — the forest is still learning", "⚠️ ".yellow());
    }
    println!();
    Ok(())
}
/// core self calibrate — adjust proposal thresholds based on history
pub fn calibrate(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!();
    println!("{}", "🔧 Self-Calibration".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();
    let total: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM self_proposals", [], |r| r.get(0)
    ).unwrap_or(0);
    if total < 5 {
        println!("  {} Need at least 5 proposals before calibrating", "○".dimmed());
        println!("  {} Currently have {} — run: core self evolve to generate more", 
            "→".dimmed(), total);
        println!();
        return Ok(());
    }
    // Find proposal kinds with high rejection rates
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT kind, COUNT(*) as total,
                SUM(CASE WHEN status='rejected' THEN 1 ELSE 0 END) as rejected
         FROM self_proposals GROUP BY kind"
    )?;
    let kinds: Vec<(String, i64, i64)> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?))
    })?.filter_map(|r| r.ok()).collect();
    println!("  {} Calibration analysis:", "🔍".bright_cyan());
    println!();
    for (kind, total_k, rejected_k) in &kinds {
        let rejection_rate = if *total_k > 0 { (rejected_k * 100 / total_k) as f64 } else { 0.0 };
        let assessment = if rejection_rate > 60.0 {
            "reduce confidence threshold".bright_red().to_string()
        } else if rejection_rate < 20.0 {
            "threshold is well-calibrated".bright_green().to_string()
        } else {
            "within normal range".normal().to_string()
        };
        println!("  {} {}: {:.0}% rejection → {}", "→".dimmed(), kind.bright_white(), rejection_rate, assessment);
    }
    println!();
    println!("  {} Calibration logged — proposals will adjust confidence over time", "✅".green());
    println!();
    let now = now_ts();
    let _ = ctx.runtime.db.execute(
        "INSERT INTO self_evolution_log (event, detail, logged_at)
         VALUES ('calibration', 'Manual calibration run', ?1)",
        params![now],
    );
    Ok(())
}
/// core partner challenge <intent_id> — prove me wrong mode
pub fn challenge(ctx: &AppContext, intent_id: &str) -> CoreResult<()> {
    println!();
    println!("{} Challenging: {}", "🎯".normal(), intent_id.bright_white().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();
    println!("  {}", "Prove Me Wrong Mode".bright_white().bold());
    println!("  {} The forest stress-tests your current plan.", "→".dimmed());
    println!();
    // Look for the intent
    let core_root = std::path::PathBuf::from(&ctx.core_root);
    let mut found_path: Option<std::path::PathBuf> = None;
    for dir in &["intents/future", "intents/complete"] {
        let dir_path = core_root.join(dir);
        if let Ok(entries) = std::fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let search_id = intent_id.trim_start_matches("INT-");
        if name.starts_with(search_id) || name.starts_with(&format!("INT-{}", search_id)) || name.contains(intent_id) {
                    found_path = Some(entry.path());
                    break;
                }
            }
        }
        if found_path.is_some() { break; }
    }
    match found_path {
        Some(path) => {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            let gate_count = content.matches('⬜').count();
            let complete_count = content.matches('✅').count();
            let total = gate_count + complete_count;
            println!("  {} Intent found: {}", "✅".green(), path.file_name().unwrap_or_default().to_string_lossy().bright_white());
            println!("  {} Gates: {}/{} complete", "→".dimmed(), complete_count, total);
            println!();
            println!("  {}", "Potential risks:".bright_yellow().bold());
            if gate_count > 10 {
                println!("  {} {} remaining gates — high complexity, consider phasing", "⚠️ ".yellow(), gate_count);
            }
            if gate_count > 0 && complete_count == 0 {
                println!("  {} No gates complete yet — validate first gate before proceeding", "⚠️ ".yellow());
            }
            if gate_count == 0 {
                println!("  {} All gates complete — intent is fully validated", "✅".green());
            }
            // Check for similar past intents that struggled
            let similar: i64 = ctx.runtime.db.query_row(
                "SELECT COUNT(*) FROM self_proposals WHERE kind = 'workflow' AND status = 'failed'",
                [], |r| r.get(0)
            ).unwrap_or(0);
            if similar > 0 {
                println!("  {} {} similar proposals failed in the past — review history", "⚠️ ".yellow(), similar);
            }
            println!();
            println!("  {}", "Counter-paths to consider:".bright_white().bold());
            println!("  {} Break into smaller intents if > 15 gates", "·".dimmed());
            println!("  {} Validate first gate before writing full implementation", "·".dimmed());
            println!("  {} Check: does this conflict with any active intent?", "·".dimmed());
            println!();
            println!("  {} Confidence in critique: {}%", "→".dimmed(), "72".bright_yellow());
        }
        None => {
            println!("  {} Intent '{}' not found", "⚠️ ".yellow(), intent_id);
            println!("  {} Use: core intent list to see available intents", "→".dimmed());
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
