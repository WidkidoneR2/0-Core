//! INT-244 v22 -- Pillar 1: Documentation Steward
//! After commits, Friday proposes doc updates -- never auto-writes
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
/// Analyze last commit and generate doc proposals
pub fn analyze_commit(ctx: &AppContext) -> CoreResult<()> {
    super::ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    // Get last commit hash and message
    let git_output = std::process::Command::new("git")
        .args(["-C", &ctx.core_root, "log", "-1", "--format=%H|%s|%b"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    if git_output.trim().is_empty() {
        return Ok(());
    }
    let parts: Vec<&str> = git_output.trim().splitn(3, '|').collect();
    let hash = parts.get(0).unwrap_or(&"").trim();
    let subject = parts.get(1).unwrap_or(&"").trim();
    let body = parts.get(2).unwrap_or(&"").trim();
    // Skip if already analyzed this commit
    let existing: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_doc_proposals WHERE commit_hash = ?1",
        params![hash],
        |r| r.get(0)
    ).unwrap_or(0);
    if existing > 0 {
        return Ok(());
    }
    // Get diff stats
    let diff_output = std::process::Command::new("git")
        .args(["-C", &ctx.core_root, "diff", "--name-only", "HEAD~1..HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let changed_files: Vec<&str> = diff_output.lines().filter(|l| !l.is_empty()).collect();
    let ts = now_ts();
    let mut proposals = 0;
    // Rule 1: vocabulary word added -> COMMAND-GUIDE.md
    if subject.contains("vocab") || subject.contains("vocabulary") || 
       body.contains("vocab") || changed_files.iter().any(|f| f.contains("commands/mod.rs")) {
        if subject.contains("feat") || subject.contains("add") {
            db.execute(
                "INSERT INTO friday_doc_proposals (timestamp, commit_hash, doc_file, reason, status)
                 VALUES (?1, ?2, 'docs/COMMAND-GUIDE.md', ?3, 'pending')",
                params![ts, hash, format!("Commit '{}' may have added new vocabulary words or commands", subject)]
            )?;
            proposals += 1;
        }
    }
    // Rule 2: new tool or deploy change -> TOOLS.md
    if changed_files.iter().any(|f| f.contains("01-registry") || f.contains("scripts/")) {
        db.execute(
            "INSERT INTO friday_doc_proposals (timestamp, commit_hash, doc_file, reason, status)
             VALUES (?1, ?2, 'docs/TOOLS.md', ?3, 'pending')",
            params![ts, hash, format!("Registry or scripts changed in commit '{}' -- TOOLS.md may need updating", subject)]
        )?;
        proposals += 1;
    }
    // Rule 3: intent closed -> CHANGELOG.md
    if subject.contains("INT-") || subject.contains("complete") || subject.contains("close") {
        db.execute(
            "INSERT INTO friday_doc_proposals (timestamp, commit_hash, doc_file, reason, status)
             VALUES (?1, ?2, '00-meta/CHANGELOG.md', ?3, 'pending')",
            params![ts, hash, format!("Intent-related commit '{}' -- CHANGELOG.md entry may be needed", subject)]
        )?;
        proposals += 1;
    }
    // Rule 4: architecture change -> README.md
    if subject.contains("arch") || subject.contains("refactor") || subject.contains("Core v") ||
       changed_files.iter().any(|f| f.contains("engine/src/domains/")) {
        db.execute(
            "INSERT INTO friday_doc_proposals (timestamp, commit_hash, doc_file, reason, status)
             VALUES (?1, ?2, 'README.md', ?3, 'pending')",
            params![ts, hash, format!("Architecture change in commit '{}' -- README.md may need updating", subject)]
        )?;
        proposals += 1;
    }
    if proposals > 0 {
        println!("  {} {} doc update{} suggested -- run: core friday docs",
            "\u{1f4cb}".cyan(),
            proposals,
            if proposals == 1 { "" } else { "s" }
        );
    }
    Ok(())
}
/// Show pending doc proposals
pub fn show_proposals(ctx: &AppContext) -> CoreResult<()> {
    super::ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let mut stmt = db.prepare(
        "SELECT id, timestamp, commit_hash, doc_file, reason, status
         FROM friday_doc_proposals
         WHERE status = 'pending'
         ORDER BY timestamp DESC LIMIT 10"
    )?;
    let rows: Vec<(i64, i64, String, String, String, String)> = stmt.query_map(
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
    )?.filter_map(|r| r.ok()).collect();
    if rows.is_empty() {
        println!("  {} No pending doc proposals", "\u{2705}".green());
        return Ok(());
    }
    println!("  {} Pending Doc Proposals ({})", "\u{1f4cb}".cyan().bold(), rows.len());
    println!("{}", "\u{2500}".repeat(52).dimmed());
    for (id, ts, hash, doc_file, reason, _status) in &rows {
        let dt = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let short_hash = &hash[..hash.len().min(8)];
        println!("  {} [{}] {} {}", id.to_string().dimmed(), dt.dimmed(), doc_file.bright_white(), short_hash.dimmed());
        println!("    {}", reason.cyan());
        println!("    {} core friday docs --approve {}", "\u{2192}".dimmed(), id);
        println!("    {} core friday docs --dismiss {}", "\u{2192}".dimmed(), id);
        println!();
    }
    Ok(())
}
/// Approve or dismiss a proposal
pub fn resolve_proposal(ctx: &AppContext, id: i64, approve: bool) -> CoreResult<()> {
    super::ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let status = if approve { "approved" } else { "dismissed" };
    let updated = db.execute(
        "UPDATE friday_doc_proposals SET status = ?1 WHERE id = ?2",
        params![status, id]
    )?;
    if updated > 0 {
        let verb = if approve { "Approved" } else { "Dismissed" };
        println!("  {} {} proposal {}", "\u{2705}".green(), verb, id);
        if approve {
            println!("  {} Remember to update the doc manually and commit", "\u{2192}".dimmed());
        }
    } else {
        println!("  {} Proposal {} not found", "\u{26a0}\u{fe0f}".yellow(), id);
    }
    Ok(())
}
/// Check for pending proposals and surface inline hint
#[allow(dead_code)]
pub fn check_pending(ctx: &AppContext) -> Option<String> {
    let db = &ctx.runtime.db;
    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_doc_proposals WHERE status = 'pending'",
        [],
        |r| r.get(0)
    ).unwrap_or(0);
    if count > 0 {
        Some(format!("{} doc update{} pending \u{2014} core friday docs",
            count, if count == 1 { "" } else { "s" }))
    } else {
        None
    }
}
