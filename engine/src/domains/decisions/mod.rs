// Core v6 — Phase 1: Decision Ledger
// INT-116
//
// "The best advisor isn't the one who knows the most facts.
//  It's the one who remembers what happened last time."

use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use crate::runtime::EventWriter;
use colored::*;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// ── Schema ────────────────────────────────────────────────────────────────────

pub fn ensure_schema(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(
        "CREATE TABLE IF NOT EXISTS decisions (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            dec_id          TEXT NOT NULL UNIQUE,
            timestamp       INTEGER NOT NULL,
            context_hash    TEXT NOT NULL,
            domain          TEXT NOT NULL DEFAULT 'general',
            description     TEXT NOT NULL,
            intent_id       TEXT,
            risk_score      REAL,
            confidence      TEXT DEFAULT 'medium',
            expected_outcome TEXT,
            outcome         TEXT DEFAULT 'pending',
            outcome_notes   TEXT,
            outcome_ts      INTEGER,
            half_life_days  INTEGER DEFAULT 90
        );",
    )?;
    Ok(())
}

// ── Context Snapshot ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct DecisionContext {
    pub health_score: u8,
    pub active_intent_count: u8,
    pub git_churn_level: u8,
    pub recent_error_count: u8,
    pub update_recency_days: u8,
    pub security_scan_age_days: u8,
}

impl DecisionContext {
    pub fn capture(ctx: &AppContext) -> Self {
        // Read health from cache
        let health_score = std::fs::read_to_string(
            dirs::home_dir()
                .unwrap_or_default()
                .join("0-core/runtime/cache/health_score"),
        )
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(95);

        // Count active intents from ledger
        let active_intent_count = count_active_intents(ctx);

        // Git churn — files changed in last 7 days
        let git_churn_level = git_churn_score();

        Self {
            health_score,
            active_intent_count,
            git_churn_level,
            recent_error_count: 0,
            update_recency_days: 7,
            security_scan_age_days: 1,
        }
    }

    /// Generate a 4-char hex context fingerprint
    /// Similar system states produce similar hashes
    pub fn fingerprint(&self) -> String {
        let normalized = format!(
            "h{}i{}c{}",
            (self.health_score / 10) * 10,   // round to 10s
            self.active_intent_count.min(5), // cap at 5
            self.git_churn_level.min(3),     // low/med/high/critical
        );
        let hash = simple_hash(&normalized);
        format!("CTX-{:04X}", hash & 0xFFFF)
    }

    /// Calculate risk score 0.0-1.0
    pub fn risk_score(&self) -> f64 {
        let mut risk = 0.0f64;
        if self.health_score < 95 {
            risk += 0.2;
        }
        if self.health_score < 90 {
            risk += 0.2;
        }
        if self.active_intent_count > 2 {
            risk += 0.15;
        }
        if self.active_intent_count > 4 {
            risk += 0.15;
        }
        if self.git_churn_level > 1 {
            risk += 0.15;
        }
        if self.git_churn_level > 2 {
            risk += 0.15;
        }
        if self.security_scan_age_days > 7 {
            risk += 0.1;
        }
        risk.min(1.0)
    }

    pub fn risk_label(&self) -> &'static str {
        let r = self.risk_score();
        if r < 0.3 {
            "low"
        } else if r < 0.6 {
            "moderate"
        } else {
            "high"
        }
    }
}

fn simple_hash(s: &str) -> u32 {
    s.bytes().fold(5381u32, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(b as u32)
    })
}

fn git_churn_score() -> u8 {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD~7..HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
        .unwrap_or(0);
    match output {
        0..=3 => 0,
        4..=10 => 1,
        11..=20 => 2,
        _ => 3,
    }
}

fn count_active_intents(_ctx: &AppContext) -> u8 {
    let intents_dir = dirs::home_dir()
        .unwrap_or_default()
        .join("0-core/intents/future");
    std::fs::read_dir(&intents_dir)
        .map(|rd| rd.filter_map(|e| e.ok()).count() as u8)
        .unwrap_or(0)
}

// ── Commands ──────────────────────────────────────────────────────────────────

pub fn decide(ctx: &AppContext, description: &str, intent_id: Option<&str>) -> CoreResult<()> {
    ctx.capabilities.require(
        "decisions",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;
    ensure_schema(ctx)?;

    let context = DecisionContext::capture(ctx);
    let fingerprint = context.fingerprint();
    let risk = context.risk_score();
    let risk_label = context.risk_label();

    // Generate decision ID
    let dec_id = next_decision_id(ctx)?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    // Warn on risk signals
    println!();
    println!("{}", "🧭 Decision Risk Assessment".bright_cyan().bold());
    println!("{}", "━".repeat(48).dimmed());
    println!(
        "  {}  {}",
        "Decision:".dimmed(),
        description.bright_white().bold()
    );
    println!("  {}  {}", "ID:".dimmed(), dec_id.bright_cyan());
    println!("  {}  {}", "Context:".dimmed(), fingerprint.bright_yellow());
    println!();
    println!(
        "  {} {}",
        "Health:".dimmed(),
        format!("{}%", context.health_score).bright_green()
    );
    println!(
        "  {} {}",
        "Active intents:".dimmed(),
        context.active_intent_count.to_string().yellow()
    );
    println!(
        "  {} {}",
        "Git churn:".dimmed(),
        churn_label(context.git_churn_level)
    );
    println!();

    // Risk signals
    let mut signals: Vec<&str> = vec![];
    if context.health_score < 95 {
        signals.push("health below 95%");
    }
    if context.active_intent_count > 2 {
        signals.push("multiple active intents");
    }
    if context.git_churn_level > 1 {
        signals.push("elevated git churn");
    }
    if context.security_scan_age_days > 7 {
        signals.push("security scan outdated");
    }

    if signals.is_empty() {
        println!("  {} No risk signals detected", "✅".green());
    } else {
        for signal in &signals {
            println!("  {} {}", "⚠".yellow(), signal.yellow());
        }
    }

    let risk_color = match risk_label {
        "low" => format!("Risk score: {:.2} ({})", risk, risk_label)
            .green()
            .to_string(),
        "moderate" => format!("Risk score: {:.2} ({})", risk, risk_label)
            .yellow()
            .to_string(),
        _ => format!("Risk score: {:.2} ({})", risk, risk_label)
            .red()
            .to_string(),
    };
    println!();
    println!("  {}", risk_color);
    println!();

    // Store in state.db
    let _context_json = serde_json::to_string(&context).unwrap_or_default();
    ctx.runtime.db.execute(
        "INSERT INTO decisions (dec_id, timestamp, context_hash, description, intent_id, risk_score, confidence)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![dec_id, ts, fingerprint, description, intent_id, risk, "medium"],
    )?;

    // Emit event to ledger
    let ew = EventWriter::new(&ctx.runtime.db);
    ew.write(
        "decisions",
        "decision.created",
        "core",
        "ok",
        Some(&format!(
            r#"{{"dec_id":"{}","description":"{}","context_hash":"{}","risk":{:.2}}}"#,
            dec_id, description, fingerprint, risk
        )),
    );

    println!(
        "  {} Decision recorded: {}",
        "🌲".green(),
        dec_id.bright_cyan()
    );
    println!("{}", "━".repeat(48).dimmed());
    println!();

    Ok(())
}

pub fn outcome(
    ctx: &AppContext,
    dec_id: &str,
    result: &str,
    notes: Option<&str>,
) -> CoreResult<()> {
    ctx.capabilities.require(
        "decisions",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;
    ensure_schema(ctx)?;

    let valid = ["success", "partial", "failure", "unknown"];
    if !valid.contains(&result) {
        println!(
            "  {} Invalid outcome. Use: success, partial, failure, unknown",
            "✗".red()
        );
        return Ok(());
    }

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let rows = ctx.runtime.db.execute(
        "UPDATE decisions SET outcome=?1, outcome_notes=?2, outcome_ts=?3 WHERE dec_id=?4",
        params![result, notes, ts, dec_id],
    )?;

    if rows == 0 {
        println!("  {} Decision {} not found", "✗".red(), dec_id);
        return Ok(());
    }

    let outcome_color = match result {
        "success" => result.bright_green().to_string(),
        "partial" => result.yellow().to_string(),
        "failure" => result.bright_red().to_string(),
        _ => result.dimmed().to_string(),
    };

    println!();
    println!(
        "  {} {} → {}",
        "📝".cyan(),
        dec_id.bright_cyan(),
        outcome_color
    );
    if let Some(n) = notes {
        println!("  {}  {}", "Note:".dimmed(), n);
    }
    println!("  {} Outcome recorded", "🌲".green());
    println!();

    let ew = EventWriter::new(&ctx.runtime.db);
    ew.write(
        "decisions",
        "decision.outcome",
        "core",
        "ok",
        Some(&format!(
            r#"{{"dec_id":"{}","outcome":"{}"}}"#,
            dec_id, result
        )),
    );

    Ok(())
}

pub fn list(ctx: &AppContext, open_only: bool) -> CoreResult<()> {
    ctx.capabilities
        .require("decisions", &[Capability::FilesystemReadHome])?;
    ensure_schema(ctx)?;

    let query = if open_only {
        "SELECT dec_id, timestamp, context_hash, description, risk_score, outcome 
         FROM decisions WHERE outcome = 'pending' ORDER BY timestamp DESC LIMIT 20"
    } else {
        "SELECT dec_id, timestamp, context_hash, description, risk_score, outcome 
         FROM decisions ORDER BY timestamp DESC LIMIT 20"
    };

    let mut stmt = ctx.runtime.db.prepare(query)?;
    let rows: Vec<(String, i64, String, String, f64, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get::<_, f64>(4).unwrap_or(0.0),
                row.get(5)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    println!();
    println!("{}", "📜 Decision Ledger".bright_cyan().bold());
    println!("{}", "━".repeat(60).dimmed());

    if rows.is_empty() {
        println!("  {} No decisions recorded yet.", "○".dimmed());
        println!(
            "  Use {} to record a decision.",
            "core decide \"description\"".bright_cyan()
        );
    } else {
        for (dec_id, _ts, ctx_hash, desc, risk, outcome) in &rows {
            let outcome_str = match outcome.as_str() {
                "success" => outcome.bright_green().to_string(),
                "partial" => outcome.yellow().to_string(),
                "failure" => outcome.bright_red().to_string(),
                "pending" => outcome.dimmed().to_string(),
                _ => outcome.dimmed().to_string(),
            };
            let risk_str = if *risk < 0.3 {
                format!("{:.2}", risk).green().to_string()
            } else if *risk < 0.6 {
                format!("{:.2}", risk).yellow().to_string()
            } else {
                format!("{:.2}", risk).red().to_string()
            };
            println!(
                "  {} {}  {}  risk:{}  {}",
                dec_id.bright_cyan(),
                ctx_hash.yellow(),
                outcome_str,
                risk_str,
                desc.dimmed(),
            );
        }
    }
    println!("{}", "━".repeat(60).dimmed());
    println!();
    Ok(())
}

pub fn hindsight(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("decisions", &[Capability::FilesystemReadHome])?;
    ensure_schema(ctx)?;

    let total: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM decisions", [], |r| r.get(0))
        .unwrap_or(0);

    let success: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome='success'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let partial: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome='partial'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let failure: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome='failure'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let pending: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome='pending'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    println!();
    println!("{}", "📜 Hindsight".bright_cyan().bold());
    println!("{}", "━".repeat(48).dimmed());
    println!(
        "  Total decisions recorded:  {}",
        total.to_string().bright_white()
    );
    println!();
    println!(
        "  {}  {}",
        "✅ Success:".green(),
        success.to_string().bright_green()
    );
    println!(
        "  {}  {}",
        "⚡ Partial:".yellow(),
        partial.to_string().yellow()
    );
    println!(
        "  {}  {}",
        "✖ Failure:".red(),
        failure.to_string().bright_red()
    );
    println!(
        "  {}  {}",
        "○ Pending:".dimmed(),
        pending.to_string().dimmed()
    );

    if total > 0 && (success + partial + failure) > 0 {
        let resolved = success + partial + failure;
        let rate = (success as f64 / resolved as f64) * 100.0;
        println!();
        println!(
            "  Success rate: {}",
            format!("{:.0}%", rate).bright_green().bold()
        );
    }

    if total == 0 {
        println!();
        println!("  {} No decisions recorded yet.", "○".dimmed());
        println!(
            "  Start with: {}",
            "core decide \"your decision\"".bright_cyan()
        );
    }

    println!("{}", "━".repeat(48).dimmed());
    println!();
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn next_decision_id(ctx: &AppContext) -> CoreResult<String> {
    let count: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM decisions", [], |r| r.get(0))
        .unwrap_or(0);
    Ok(format!("DEC-{:03}", count + 1))
}

fn churn_label(level: u8) -> colored::ColoredString {
    match level {
        0 => "clean".green(),
        1 => "low".bright_green(),
        2 => "elevated".yellow(),
        _ => "high".red(),
    }
}

// ── Phase 2: Outcome Correlation ──────────────────────────────────────────────

pub fn show(ctx: &AppContext, dec_id: &str) -> CoreResult<()> {
    ctx.capabilities
        .require("decisions", &[Capability::FilesystemReadHome])?;
    ensure_schema(ctx)?;

    let result = ctx.runtime.db.query_row(
        "SELECT dec_id, timestamp, context_hash, domain, description,
                intent_id, risk_score, confidence, outcome, outcome_notes, outcome_ts
         FROM decisions WHERE dec_id = ?1",
        params![dec_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, f64>(6).unwrap_or(0.0),
                row.get::<_, String>(7).unwrap_or_else(|_| "medium".into()),
                row.get::<_, String>(8).unwrap_or_else(|_| "pending".into()),
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<i64>>(10)?,
            ))
        },
    );

    match result {
        Err(_) => {
            println!("  {} Decision {} not found", "✗".red(), dec_id);
            return Ok(());
        }
        Ok((
            id,
            ts,
            ctx_hash,
            domain,
            desc,
            intent,
            risk,
            confidence,
            outcome,
            notes,
            outcome_ts,
        )) => {
            let outcome_color = match outcome.as_str() {
                "success" => outcome.bright_green().to_string(),
                "partial" => outcome.yellow().to_string(),
                "failure" => outcome.bright_red().to_string(),
                _ => outcome.dimmed().to_string(),
            };

            println!();
            println!("{}", "📋 Decision Detail".bright_cyan().bold());
            println!("{}", "━".repeat(52).dimmed());
            println!("  {}  {}", "ID:".dimmed(), id.bright_cyan().bold());
            println!("  {}  {}", "Description:".dimmed(), desc.bright_white());
            println!("  {}  {}", "Domain:".dimmed(), domain.yellow());
            println!("  {}  {}", "Context:".dimmed(), ctx_hash.bright_yellow());
            println!("  {}  {:.2}", "Risk score:".dimmed(), risk);
            println!("  {}  {}", "Confidence:".dimmed(), confidence.dimmed());
            if let Some(intent_id) = intent {
                println!("  {}  {}", "Intent:".dimmed(), intent_id.bright_blue());
            }
            println!();
            println!("  {}  {}", "Outcome:".dimmed(), outcome_color);
            if let Some(n) = notes {
                println!("  {}  {}", "Notes:".dimmed(), n.dimmed());
            }
            if let Some(ots) = outcome_ts {
                let decision_age = ts;
                let outcome_age = ots;
                let delta_secs = outcome_age - decision_age;
                let delta_mins = delta_secs / 60;
                println!(
                    "  {}  {} mins after decision",
                    "Resolved in:".dimmed(),
                    delta_mins
                );
            }

            // Find similar context decisions
            let prefix = &ctx_hash[..6]; // CTX-64 — first 6 chars
            let mut stmt = ctx.runtime.db.prepare(
                "SELECT dec_id, description, outcome FROM decisions
                 WHERE context_hash LIKE ?1 AND dec_id != ?2
                 ORDER BY timestamp DESC LIMIT 5",
            )?;
            let similar: Vec<(String, String, String)> = stmt
                .query_map(params![format!("{}%", prefix), &id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })?
                .filter_map(|r| r.ok())
                .collect();

            if !similar.is_empty() {
                println!();
                println!("  {} Similar context decisions:", "◈".bright_yellow());
                for (sid, sdesc, sout) in &similar {
                    println!(
                        "    {} {} — {}",
                        sid.bright_cyan(),
                        sout.dimmed(),
                        sdesc.dimmed()
                    );
                }
            }

            println!("{}", "━".repeat(52).dimmed());
            println!();
        }
    }
    Ok(())
}

pub fn stats(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("decisions", &[Capability::FilesystemReadHome])?;
    ensure_schema(ctx)?;

    let total: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM decisions", [], |r| r.get(0))
        .unwrap_or(0);

    if total == 0 {
        println!("\n  {} No decisions to correlate yet.\n", "○".dimmed());
        return Ok(());
    }

    let success: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome='success'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let partial: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome='partial'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let failure: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome='failure'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let pending: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome='pending'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Risk correlation — do high risk decisions fail more?
    let high_risk_failure: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE risk_score > 0.5 AND outcome='failure'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let high_risk_total: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE risk_score > 0.5",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let low_risk_success: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE risk_score <= 0.3 AND outcome='success'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let low_risk_total: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE risk_score <= 0.3",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Most common context hash
    let top_context: Option<(String, i64)> = ctx
        .runtime
        .db
        .query_row(
            "SELECT context_hash, COUNT(*) as cnt FROM decisions
             GROUP BY context_hash ORDER BY cnt DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();

    println!();
    println!("{}", "📊 Decision Correlation Stats".bright_cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!("  Total decisions:  {}", total.to_string().bright_white());
    println!(
        "  Resolved:         {}",
        (success + partial + failure).to_string().bright_white()
    );
    println!("  Pending:          {}", pending.to_string().dimmed());
    println!();
    println!(
        "  {} Success:  {}",
        "✅".green(),
        success.to_string().bright_green()
    );
    println!(
        "  {} Partial:  {}",
        "⚡".yellow(),
        partial.to_string().yellow()
    );
    println!(
        "  {} Failure:  {}",
        "✖".red(),
        failure.to_string().bright_red()
    );

    let resolved = success + partial + failure;
    if resolved > 0 {
        let rate = (success as f64 / resolved as f64) * 100.0;
        println!();
        println!(
            "  Success rate:  {}",
            format!("{:.0}%", rate).bright_green().bold()
        );
    }

    // Risk correlation insight
    if high_risk_total > 0 {
        println!();
        println!("  {} Risk correlation:", "◈".bright_yellow());
        println!(
            "    High risk decisions → failure: {}/{}",
            high_risk_failure.to_string().yellow(),
            high_risk_total.to_string().dimmed()
        );
    }
    if low_risk_total > 0 {
        println!(
            "    Low risk decisions → success:  {}/{}",
            low_risk_success.to_string().green(),
            low_risk_total.to_string().dimmed()
        );
    }

    if let Some((hash, count)) = top_context {
        println!();
        println!(
            "  {} Most frequent context: {} ({} decisions)",
            "◈".bright_yellow(),
            hash.bright_yellow(),
            count.to_string().dimmed()
        );
    }

    println!("{}", "━".repeat(52).dimmed());
    println!();
    Ok(())
}

/// Find decisions made in similar context — used by Phase 3 advise
pub fn find_similar_context(
    ctx: &AppContext,
    context_hash: &str,
    limit: usize,
) -> Vec<(String, String, String, f64)> {
    let prefix = if context_hash.len() >= 6 {
        &context_hash[..6]
    } else {
        context_hash
    };

    let query = format!(
        "SELECT dec_id, description, outcome, risk_score
         FROM decisions
         WHERE context_hash LIKE '{}%' AND outcome != 'pending'
         ORDER BY timestamp DESC LIMIT {}",
        prefix, limit
    );

    ctx.runtime
        .db
        .prepare(&query)
        .ok()
        .map(|mut stmt| {
            stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, f64>(3).unwrap_or(0.0),
                ))
            })
            .ok()
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
        })
        .unwrap_or_default()
}

// ── Phase 3: Judgment Assist ──────────────────────────────────────────────────

pub fn advise(ctx: &AppContext, planned_decision: Option<&str>) -> CoreResult<()> {
    ctx.capabilities
        .require("decisions", &[Capability::FilesystemReadHome])?;
    ensure_schema(ctx)?;

    // Capture current context
    let context = DecisionContext::capture(ctx);
    let fingerprint = context.fingerprint();
    let risk = context.risk_score();
    let risk_label = context.risk_label();

    println!();
    println!("{}", "🧭 Judgment Advisory".bright_cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    // Show current context
    println!("  {}", "Current State:".bright_white().bold());
    println!(
        "    Health:          {}",
        format!("{}%", context.health_score).bright_green()
    );
    println!(
        "    Active intents:  {}",
        context.active_intent_count.to_string().yellow()
    );
    println!(
        "    Git churn:       {}",
        churn_label(context.git_churn_level)
    );
    println!("    Context hash:    {}", fingerprint.bright_yellow());

    if let Some(decision) = planned_decision {
        println!();
        println!(
            "  {}  {}",
            "Evaluating:".dimmed(),
            decision.bright_white().bold()
        );
    }

    // Risk signals
    println!();
    let mut signals: Vec<(&str, &str)> = vec![];
    if context.health_score < 95 {
        signals.push(("⚠", "health below 95% — consider running doctor first"));
    }
    if context.health_score < 90 {
        signals.push(("⚠", "health significantly degraded — high regression risk"));
    }
    if context.active_intent_count > 2 {
        signals.push(("⚠", "multiple active intents — context switching risk"));
    }
    if context.active_intent_count > 4 {
        signals.push((
            "⚠",
            "too many concurrent intents — consider completing one first",
        ));
    }
    if context.git_churn_level > 1 {
        signals.push(("⚠", "elevated git churn — instability window"));
    }
    if context.git_churn_level > 2 {
        signals.push(("⚠", "high git churn — risky time for large changes"));
    }
    if context.security_scan_age_days > 7 {
        signals.push(("⚠", "security scan outdated — run core security scan"));
    }

    // Audit signal — check stale tools from audit_scores
    let stale_tool_count: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(DISTINCT tool_name) FROM audit_scores WHERE score < 70 AND timestamp = (SELECT MAX(timestamp) FROM audit_scores)",
        [],
        |r| r.get(0),
    ).unwrap_or(0);
    if stale_tool_count >= 3 {
        signals.push((
            "⚠",
            "3+ tools below health threshold — run: core audit stale",
        ));
    }

    if signals.is_empty() {
        println!(
            "  {} No risk signals — conditions are favorable",
            "✅".green()
        );
    } else {
        println!("  {}", "Risk Signals:".bright_white().bold());
        for (icon, msg) in &signals {
            println!("    {} {}", icon.yellow(), msg.yellow());
        }
    }

    // Historical pattern matching
    let similar = find_similar_context(ctx, &fingerprint, 10);

    println!();
    if similar.is_empty() {
        println!(
            "  {} No historical decisions in similar context yet.",
            "◈".dimmed()
        );
        println!(
            "    {} decisions recorded so far.",
            count_total_decisions(ctx)
        );
    } else {
        let total_similar = similar.len();
        let successes = similar.iter().filter(|(_, _, o, _)| o == "success").count();
        let partials = similar.iter().filter(|(_, _, o, _)| o == "partial").count();
        let failures = similar.iter().filter(|(_, _, o, _)| o == "failure").count();
        let success_rate = (successes as f64 / total_similar as f64) * 100.0;

        println!(
            "  {}",
            "Historical Pattern (similar context):"
                .bright_white()
                .bold()
        );
        println!(
            "    Found {} decisions in context {}xx",
            total_similar.to_string().bright_white(),
            &fingerprint[..6].bright_yellow()
        );
        println!(
            "    {} success  {} partial  {} failure",
            successes.to_string().bright_green(),
            partials.to_string().yellow(),
            failures.to_string().bright_red()
        );
        println!(
            "    Success rate: {}",
            format!("{:.0}%", success_rate).bright_green().bold()
        );

        // Show recent similar decisions
        println!();
        println!("  {}", "Recent similar decisions:".dimmed());
        for (dec_id, desc, outcome, _risk) in similar.iter().take(3) {
            let outcome_str = match outcome.as_str() {
                "success" => outcome.bright_green().to_string(),
                "partial" => outcome.yellow().to_string(),
                "failure" => outcome.bright_red().to_string(),
                _ => outcome.dimmed().to_string(),
            };
            let short_desc = if desc.len() > 40 {
                format!("{}...", &desc[..40])
            } else {
                desc.clone()
            };
            println!(
                "    {} {} — {}",
                dec_id.bright_cyan(),
                outcome_str,
                short_desc.dimmed()
            );
        }
    }

    // Overall advisory
    println!();
    println!("  {}", "Advisory:".bright_white().bold());

    let risk_color = match risk_label {
        "low" => format!("Risk: {:.2} ({})", risk, risk_label)
            .green()
            .to_string(),
        "moderate" => format!("Risk: {:.2} ({})", risk, risk_label)
            .yellow()
            .to_string(),
        _ => format!("Risk: {:.2} ({})", risk, risk_label)
            .red()
            .to_string(),
    };
    println!("    {}", risk_color);

    // Specific suggestions based on signals
    if context.health_score < 95 {
        println!(
            "    {} Run {} before proceeding",
            "→".dimmed(),
            "d".bright_cyan()
        );
    }
    if context.git_churn_level > 1 {
        println!(
            "    {} Consider {} before large changes",
            "→".dimmed(),
            "cpc".bright_cyan()
        );
    }
    if signals.is_empty() && similar.is_empty() {
        println!(
            "    {} Conditions favorable — proceed with confidence",
            "→".dimmed().to_string().green()
        );
    }
    if !signals.is_empty() && context.git_churn_level > 1 {
        println!(
            "    {} Create a checkpoint first: {}",
            "→".dimmed(),
            "cpc pre-decision".bright_cyan()
        );
    }
    if stale_tool_count >= 3 {
        println!(
            "    {} {} tools need attention: {}",
            "→".dimmed(),
            stale_tool_count.to_string().yellow(),
            "core audit stale".bright_cyan()
        );
    }

    println!();
    println!("  {}", "The forest advises. You decide.".dimmed().italic());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    Ok(())
}

fn count_total_decisions(ctx: &AppContext) -> i64 {
    ctx.runtime
        .db
        .query_row("SELECT COUNT(*) FROM decisions", [], |r| r.get(0))
        .unwrap_or(0)
}

// ── Phase 4: Heuristics Engine ────────────────────────────────────────────────

pub fn heuristics(ctx: &AppContext, domain_filter: Option<&str>) -> CoreResult<()> {
    ctx.capabilities
        .require("decisions", &[Capability::FilesystemReadHome])?;
    ensure_schema(ctx)?;

    // Auto-derive heuristics from decision corpus
    let derived = derive_heuristics(ctx);

    let filtered: Vec<&DerivedHeuristic> = if let Some(domain) = domain_filter {
        derived.iter().filter(|h| h.domain == domain).collect()
    } else {
        derived.iter().collect()
    };

    println!();
    println!("{}", "🌿 Heuristics Engine".bright_cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    if filtered.is_empty() {
        println!("  {} Not enough observations yet.", "○".dimmed());
        println!(
            "  Heuristics require {} decisions with outcomes.",
            "3+".bright_cyan()
        );
        println!(
            "  Currently: {} decisions recorded.",
            count_total_decisions(ctx)
        );
    } else {
        for h in &filtered {
            let confidence_color = if h.confidence >= 0.8 {
                format!("{:.0}%", h.confidence * 100.0).bright_green()
            } else if h.confidence >= 0.6 {
                format!("{:.0}%", h.confidence * 100.0).yellow()
            } else {
                format!("{:.0}%", h.confidence * 100.0).dimmed()
            };

            println!(
                "  {} [{}] {}",
                "◆".bright_yellow(),
                h.domain.bright_cyan(),
                h.description.bright_white()
            );
            println!(
                "    Confidence: {}  Observations: {}",
                confidence_color,
                h.observations.to_string().dimmed()
            );
            println!();
        }
    }

    println!("{}", "━".repeat(52).dimmed());
    println!();
    Ok(())
}

pub fn lessons(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("decisions", &[Capability::FilesystemReadHome])?;
    ensure_schema(ctx)?;

    let derived = derive_heuristics(ctx);
    let total: i64 = count_total_decisions(ctx);
    let resolved = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome != 'pending'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0);

    println!();
    println!("{}", "📖 What the Forest Has Learned".bright_cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!("  Based on {} decisions ({} resolved)", total, resolved);
    println!();

    if derived.is_empty() {
        println!("  {} The forest is still learning.", "○".dimmed());
        println!("  Keep recording decisions and outcomes.");
        println!("  Lessons emerge after 3+ observations per pattern.");
    } else {
        println!("  {}", "Learned Lessons:".bright_white().bold());
        for h in &derived {
            println!("  {} {}", "→".bright_green(), h.description.bright_white());
        }
    }

    // Always-available structural lessons
    println!();
    println!("  {}", "Structural Observations:".bright_white().bold());

    let low_risk_success_rate = {
        let s: i64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM decisions WHERE risk_score <= 0.3 AND outcome='success'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let t: i64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM decisions WHERE risk_score <= 0.3 AND outcome != 'pending'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if t > 0 {
            (s as f64 / t as f64) * 100.0
        } else {
            0.0
        }
    };

    let high_risk_failure_rate = {
        let f: i64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM decisions WHERE risk_score > 0.5 AND outcome='failure'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let t: i64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM decisions WHERE risk_score > 0.5 AND outcome != 'pending'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if t > 0 {
            (f as f64 / t as f64) * 100.0
        } else {
            0.0
        }
    };

    if low_risk_success_rate > 0.0 {
        println!(
            "  → Low risk decisions succeed {:.0}% of the time",
            low_risk_success_rate
        );
    }
    if high_risk_failure_rate > 0.0 {
        println!(
            "  → High risk decisions fail {:.0}% of the time",
            high_risk_failure_rate
        );
    }
    if total > 0 {
        println!("  → {} decisions recorded since Core v6 Phase 1", total);
    }

    println!();
    println!(
        "  {}",
        "The forest remembers. You decide.".dimmed().italic()
    );
    println!("{}", "━".repeat(52).dimmed());
    println!();
    Ok(())
}

pub fn story(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("decisions", &[Capability::FilesystemReadHome])?;
    ensure_schema(ctx)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let thirty_days_ago = now - (30 * 24 * 3600);

    // Events in last 30 days
    let event_count: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM events WHERE timestamp > ?1",
            params![thirty_days_ago],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Decisions in last 30 days
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT dec_id, description, outcome, timestamp, risk_score
         FROM decisions WHERE timestamp > ?1
         ORDER BY timestamp ASC",
    )?;
    let decisions: Vec<(String, String, String, i64, f64)> = stmt
        .query_map(params![thirty_days_ago], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get::<_, f64>(4).unwrap_or(0.0),
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let decision_count = decisions.len();
    let success_count = decisions
        .iter()
        .filter(|(_, _, o, _, _)| o == "success")
        .count();
    let failure_count = decisions
        .iter()
        .filter(|(_, _, o, _, _)| o == "failure")
        .count();
    let pending_count = decisions
        .iter()
        .filter(|(_, _, o, _, _)| o == "pending")
        .count();

    // Doctor runs
    let doctor_runs: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM events WHERE domain='doctor' AND action='run' AND timestamp > ?1",
            params![thirty_days_ago],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Git commits
    let git_commits: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM events WHERE domain='git' AND action='commit' AND timestamp > ?1",
            params![thirty_days_ago],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Window events
    let window_opens: i64 = ctx.runtime.db
        .query_row(
            "SELECT COUNT(*) FROM events WHERE domain='compositor' AND action='window.open' AND timestamp > ?1",
            params![thirty_days_ago],
            |r| r.get(0)
        ).unwrap_or(0);

    println!();
    println!(
        "{}",
        "📜 The Forest's Story — Last 30 Days".bright_cyan().bold()
    );
    println!("{}", "━".repeat(52).dimmed());
    println!();

    // Narrative
    println!("  In the last 30 days, the forest was active.");
    println!();
    println!(
        "  {} events flowed through the ledger.",
        event_count.to_string().bright_white().bold()
    );
    println!(
        "  {} health checks run.",
        doctor_runs.to_string().bright_green()
    );
    println!(
        "  {} commits made to the forest.",
        git_commits.to_string().bright_cyan()
    );
    if window_opens > 0 {
        println!(
            "  {} windows opened in the compositor.",
            window_opens.to_string().yellow()
        );
    }
    println!();

    if decision_count > 0 {
        println!(
            "  {} decisions were recorded:",
            decision_count.to_string().bright_white().bold()
        );
        println!("    {} succeeded", success_count.to_string().bright_green());
        if failure_count > 0 {
            println!("    {} failed", failure_count.to_string().bright_red());
        }
        if pending_count > 0 {
            println!("    {} still pending", pending_count.to_string().dimmed());
        }
        println!();

        // Show the decision arc
        println!("  {}", "Decision arc:".dimmed());
        for (dec_id, desc, outcome, _ts, _risk) in &decisions {
            let outcome_icon = match outcome.as_str() {
                "success" => "✅",
                "partial" => "⚡",
                "failure" => "✖",
                _ => "○",
            };
            let short = if desc.len() > 45 {
                format!("{}...", &desc[..45])
            } else {
                desc.clone()
            };
            println!(
                "    {} {} — {}",
                outcome_icon,
                dec_id.bright_cyan(),
                short.dimmed()
            );
        }
    } else {
        println!("  No decisions recorded in this period.");
        println!(
            "  Use {} to start building the forest's memory.",
            "core decide".bright_cyan()
        );
    }

    println!();
    println!(
        "  {}",
        "The forest remembers the path that led here."
            .dimmed()
            .italic()
    );
    println!("{}", "━".repeat(52).dimmed());
    println!();
    Ok(())
}

// ── Heuristic Derivation ─────────────────────────────────────────────────────

#[derive(Debug)]
struct DerivedHeuristic {
    domain: String,
    description: String,
    confidence: f64,
    observations: usize,
}

fn derive_heuristics(ctx: &AppContext) -> Vec<DerivedHeuristic> {
    let mut heuristics = Vec::new();
    let min_observations = 3;

    // Heuristic: Low risk → success correlation
    let low_risk_success: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE risk_score <= 0.3 AND outcome='success'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let low_risk_total: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE risk_score <= 0.3 AND outcome != 'pending'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if low_risk_total >= min_observations {
        let confidence = low_risk_success as f64 / low_risk_total as f64;
        heuristics.push(DerivedHeuristic {
            domain: "general".into(),
            description: format!(
                "Low risk decisions succeed {:.0}% of the time ({} observations)",
                confidence * 100.0,
                low_risk_total
            ),
            confidence,
            observations: low_risk_total as usize,
        });
    }

    // Heuristic: High risk → failure correlation
    let high_risk_failure: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE risk_score > 0.5 AND outcome='failure'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let high_risk_total: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE risk_score > 0.5 AND outcome != 'pending'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if high_risk_total >= min_observations {
        let confidence = high_risk_failure as f64 / high_risk_total as f64;
        heuristics.push(DerivedHeuristic {
            domain: "general".into(),
            description: format!(
                "High risk decisions fail {:.0}% of the time ({} observations)",
                confidence * 100.0,
                high_risk_total
            ),
            confidence,
            observations: high_risk_total as usize,
        });
    }

    // Heuristic: Git domain decisions
    let git_success: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE domain='git' AND outcome='success'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let git_total: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE domain='git' AND outcome != 'pending'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if git_total >= min_observations {
        let confidence = git_success as f64 / git_total as f64;
        heuristics.push(DerivedHeuristic {
            domain: "git".into(),
            description: format!(
                "Git decisions succeed {:.0}% of the time",
                confidence * 100.0
            ),
            confidence,
            observations: git_total as usize,
        });
    }

    heuristics
}

// ── Core v8 Phase 3 — Decision Pattern Intelligence ───────────────────────────

/// patterns — find repeating decision types and domains
/// Pure observation. No suggestions. Data only.
pub fn patterns(ctx: &AppContext) -> CoreResult<()> {
    use colored::*;

    println!();
    println!("{}", "🔍  Decision Patterns".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {}  {}",
        "Source:".dimmed(),
        "decisions table".bright_white()
    );
    println!();

    // Count by domain
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT domain, COUNT(*) as cnt FROM decisions GROUP BY domain ORDER BY cnt DESC",
    )?;
    let domain_rows: Vec<(String, i64)> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    // Count by outcome
    let total: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM decisions", [], |r| r.get(0))
        .unwrap_or(0);
    let success: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome='success'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let pending: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome='pending'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let failure: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome='failure'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    println!(
        "  {}  {}",
        "Total decisions:".bright_white().bold(),
        total.to_string().bright_green()
    );
    println!(
        "  {}  {}  {}  {}",
        "Outcomes:".dimmed(),
        format!("{} success", success).bright_green(),
        format!("{} pending", pending).yellow(),
        format!("{} failure", failure).bright_red(),
    );
    println!();

    if domain_rows.is_empty() {
        println!("  {}", "No decisions recorded yet.".dimmed());
        println!(
            "  {} {}",
            "Record one with:".dimmed(),
            "core decide \"description\"".bright_cyan()
        );
    } else {
        println!("  {}", "By Domain:".bright_white().bold());
        println!("  {:<20} {:>8}", "Domain".dimmed(), "Count".dimmed());
        println!("  {}", "─".repeat(30).dimmed());
        for (domain, count) in &domain_rows {
            println!(
                "  {:<20} {:>8}",
                domain.bright_white(),
                count.to_string().bright_green()
            );
        }
    }

    println!();
    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {}",
        "Data collected. No suggestions. The forest observes."
            .dimmed()
            .italic()
    );
    println!();
    Ok(())
}

/// friction — detect decisions requiring repeated corrections
pub fn friction(ctx: &AppContext) -> CoreResult<()> {
    use colored::*;

    println!();
    println!("{}", "⚡  Decision Friction".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    println!();

    // Find decisions with failure or partial outcomes
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT dec_id, domain, description, outcome, outcome_notes
         FROM decisions
         WHERE outcome IN ('failure', 'partial')
         ORDER BY timestamp DESC",
    )?;
    let rows: Vec<(String, String, String, String, Option<String>)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if rows.is_empty() {
        println!(
            "  {} {}",
            "✅".green(),
            "No friction detected — no failures or partial outcomes recorded.".dimmed()
        );
    } else {
        println!("  {}", "Decisions with friction:".bright_white().bold());
        println!();
        for (dec_id, domain, desc, outcome, notes) in &rows {
            let outcome_colored = if outcome == "failure" {
                outcome.bright_red().to_string()
            } else {
                outcome.yellow().to_string()
            };
            println!("  {} [{}] {}", dec_id.bright_white().bold(), outcome_colored, domain.dimmed());
            println!("    {}", desc.dimmed());
            if let Some(n) = notes {
                if !n.is_empty() {
                    println!("    {} {}", "Notes:".dimmed(), n.dimmed().italic());
                }
            }
            println!();
        }
    }

    // Check for pending decisions older than 30 days
    let old_pending: i64 = ctx
        .runtime
        .db
        .query_row(
            &format!(
                "SELECT COUNT(*) FROM decisions WHERE outcome='pending' AND timestamp < {}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64
                    - 30 * 86400
            ),
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if old_pending > 0 {
        println!(
            "  {} {} {}",
            "⚠".yellow(),
            old_pending.to_string().yellow(),
            "decision(s) pending for more than 30 days — consider recording outcomes.".dimmed()
        );
        println!();
    }

    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {}",
        "Data collected. No suggestions. The forest observes."
            .dimmed()
            .italic()
    );
    println!();
    Ok(())
}

/// reversal — detect architectural reversals (removed then reintroduced)
pub fn reversal(ctx: &AppContext) -> CoreResult<()> {
    use colored::*;

    println!();
    println!("{}", "🔄  Decision Reversals".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    println!();

    // Look for decisions with similar keywords that were reversed
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT dec_id, domain, description, outcome, timestamp
         FROM decisions
         ORDER BY domain, timestamp ASC",
    )?;
    let rows: Vec<(String, String, String, String, i64)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Simple reversal detection: look for "remove" followed by "add" or "reintroduce"
    // for similar subjects within same domain
    let mut reversals: Vec<(String, String, String, String)> = vec![];

    for (i, (id1, dom1, desc1, out1, _)) in rows.iter().enumerate() {
        let desc1_lower = desc1.to_lowercase();
        let is_removal = desc1_lower.contains("remov")
            || desc1_lower.contains("deprecat")
            || desc1_lower.contains("replac")
            || desc1_lower.contains("drop");
        if !is_removal {
            continue;
        }

        for (id2, dom2, desc2, _, _) in rows.iter().skip(i + 1) {
            if dom1 != dom2 {
                continue;
            }
            let desc2_lower = desc2.to_lowercase();
            let is_readd = desc2_lower.contains("add")
                || desc2_lower.contains("reintroduc")
                || desc2_lower.contains("restor")
                || desc2_lower.contains("bring back");
            if !is_readd {
                continue;
            }

            // Check word overlap — simple heuristic
            let words1: Vec<&str> = desc1.split_whitespace().filter(|w| w.len() > 4).collect();
            let words2: Vec<&str> = desc2.split_whitespace().filter(|w| w.len() > 4).collect();
            let overlap = words1.iter().filter(|w| words2.contains(w)).count();

            if overlap >= 1 {
                reversals.push((id1.clone(), id2.clone(), desc1.clone(), desc2.clone()));
            }
        }
        let _ = out1;
    }

    if reversals.is_empty() {
        println!(
            "  {} {}",
            "✅".green(),
            "No reversals detected in decision history.".dimmed()
        );
    } else {
        println!(
            "  {}",
            "Potential reversals detected:".bright_white().bold()
        );
        println!();
        for (id1, id2, desc1, desc2) in &reversals {
            println!(
                "  {} → {}",
                id1.bright_white().bold(),
                id2.bright_white().bold()
            );
            println!(
                "    {} {}",
                "Removed:".dimmed(),
                desc1.bright_red().dimmed()
            );
            println!(
                "    {} {}",
                "Added:".dimmed(),
                desc2.bright_green().dimmed()
            );
            println!();
        }
    }

    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {}",
        "Data collected. No suggestions. The forest observes."
            .dimmed()
            .italic()
    );
    println!();
    Ok(())
}
