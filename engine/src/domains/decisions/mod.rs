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
            (self.health_score / 10) * 10,  // round to 10s
            self.active_intent_count.min(5), // cap at 5
            self.git_churn_level.min(3),     // low/med/high/critical
        );
        let hash = simple_hash(&normalized);
        format!("CTX-{:04X}", hash & 0xFFFF)
    }

    /// Calculate risk score 0.0-1.0
    pub fn risk_score(&self) -> f64 {
        let mut risk = 0.0f64;
        if self.health_score < 95 { risk += 0.2; }
        if self.health_score < 90 { risk += 0.2; }
        if self.active_intent_count > 2 { risk += 0.15; }
        if self.active_intent_count > 4 { risk += 0.15; }
        if self.git_churn_level > 1 { risk += 0.15; }
        if self.git_churn_level > 2 { risk += 0.15; }
        if self.security_scan_age_days > 7 { risk += 0.1; }
        risk.min(1.0)
    }

    pub fn risk_label(&self) -> &'static str {
        let r = self.risk_score();
        if r < 0.3 { "low" }
        else if r < 0.6 { "moderate" }
        else { "high" }
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
        0..=3   => 0,
        4..=10  => 1,
        11..=20 => 2,
        _       => 3,
    }
}

fn count_active_intents(ctx: &AppContext) -> u8 {
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
    println!("  {}  {}", "Decision:".dimmed(), description.bright_white().bold());
    println!("  {}  {}", "ID:".dimmed(), dec_id.bright_cyan());
    println!("  {}  {}", "Context:".dimmed(), fingerprint.bright_yellow());
    println!();
    println!("  {} {}", "Health:".dimmed(), format!("{}%", context.health_score).bright_green());
    println!("  {} {}", "Active intents:".dimmed(), context.active_intent_count.to_string().yellow());
    println!("  {} {}", "Git churn:".dimmed(), churn_label(context.git_churn_level));
    println!();

    // Risk signals
    let mut signals: Vec<&str> = vec![];
    if context.health_score < 95 { signals.push("health below 95%"); }
    if context.active_intent_count > 2 { signals.push("multiple active intents"); }
    if context.git_churn_level > 1 { signals.push("elevated git churn"); }
    if context.security_scan_age_days > 7 { signals.push("security scan outdated"); }

    if signals.is_empty() {
        println!("  {} No risk signals detected", "✅".green());
    } else {
        for signal in &signals {
            println!("  {} {}", "⚠".yellow(), signal.yellow());
        }
    }

    let risk_color = match risk_label {
        "low"      => format!("Risk score: {:.2} ({})", risk, risk_label).green().to_string(),
        "moderate" => format!("Risk score: {:.2} ({})", risk, risk_label).yellow().to_string(),
        _          => format!("Risk score: {:.2} ({})", risk, risk_label).red().to_string(),
    };
    println!();
    println!("  {}", risk_color);
    println!();

    // Store in state.db
    let context_json = serde_json::to_string(&context).unwrap_or_default();
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
        Some(&format!(r#"{{"dec_id":"{}","description":"{}","context_hash":"{}","risk":{:.2}}}"#,
            dec_id, description, fingerprint, risk)),
    );

    println!("  {} Decision recorded: {}", "🌲".green(), dec_id.bright_cyan());
    println!("{}", "━".repeat(48).dimmed());
    println!();

    Ok(())
}

pub fn outcome(ctx: &AppContext, dec_id: &str, result: &str, notes: Option<&str>) -> CoreResult<()> {
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
        println!("  {} Invalid outcome. Use: success, partial, failure, unknown", "✗".red());
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
        _         => result.dimmed().to_string(),
    };

    println!();
    println!("  {} {} → {}", "📝".cyan(), dec_id.bright_cyan(), outcome_color);
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
        Some(&format!(r#"{{"dec_id":"{}","outcome":"{}"}}"#, dec_id, result)),
    );

    Ok(())
}

pub fn list(ctx: &AppContext, open_only: bool) -> CoreResult<()> {
    ctx.capabilities.require(
        "decisions",
        &[Capability::FilesystemReadHome],
    )?;
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
        println!("  Use {} to record a decision.", "core decide \"description\"".bright_cyan());
    } else {
        for (dec_id, _ts, ctx_hash, desc, risk, outcome) in &rows {
            let outcome_str = match outcome.as_str() {
                "success" => outcome.bright_green().to_string(),
                "partial" => outcome.yellow().to_string(),
                "failure" => outcome.bright_red().to_string(),
                "pending" => outcome.dimmed().to_string(),
                _         => outcome.dimmed().to_string(),
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
    ctx.capabilities.require(
        "decisions",
        &[Capability::FilesystemReadHome],
    )?;
    ensure_schema(ctx)?;

    let total: i64 = ctx.runtime.db
        .query_row("SELECT COUNT(*) FROM decisions", [], |r| r.get(0))
        .unwrap_or(0);

    let success: i64 = ctx.runtime.db
        .query_row("SELECT COUNT(*) FROM decisions WHERE outcome='success'", [], |r| r.get(0))
        .unwrap_or(0);

    let partial: i64 = ctx.runtime.db
        .query_row("SELECT COUNT(*) FROM decisions WHERE outcome='partial'", [], |r| r.get(0))
        .unwrap_or(0);

    let failure: i64 = ctx.runtime.db
        .query_row("SELECT COUNT(*) FROM decisions WHERE outcome='failure'", [], |r| r.get(0))
        .unwrap_or(0);

    let pending: i64 = ctx.runtime.db
        .query_row("SELECT COUNT(*) FROM decisions WHERE outcome='pending'", [], |r| r.get(0))
        .unwrap_or(0);

    println!();
    println!("{}", "📜 Hindsight".bright_cyan().bold());
    println!("{}", "━".repeat(48).dimmed());
    println!("  Total decisions recorded:  {}", total.to_string().bright_white());
    println!();
    println!("  {}  {}", "✅ Success:".green(), success.to_string().bright_green());
    println!("  {}  {}", "⚡ Partial:".yellow(), partial.to_string().yellow());
    println!("  {}  {}", "✖ Failure:".red(), failure.to_string().bright_red());
    println!("  {}  {}", "○ Pending:".dimmed(), pending.to_string().dimmed());

    if total > 0 && (success + partial + failure) > 0 {
        let resolved = success + partial + failure;
        let rate = (success as f64 / resolved as f64) * 100.0;
        println!();
        println!("  Success rate: {}", format!("{:.0}%", rate).bright_green().bold());
    }

    if total == 0 {
        println!();
        println!("  {} No decisions recorded yet.", "○".dimmed());
        println!("  Start with: {}", "core decide \"your decision\"".bright_cyan());
    }

    println!("{}", "━".repeat(48).dimmed());
    println!();
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn next_decision_id(ctx: &AppContext) -> CoreResult<String> {
    let count: i64 = ctx.runtime.db
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
