//! delegate domain — Trust Contracts and Safe Autonomy Simulation
//! INT-187 Phase 1: Simulation only — no real execution
//! Activation gate: simulation accuracy >= 85% over 14+ days
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;

// ── DB setup ─────────────────────────────────────────────────────────────────
/// INT-187 — Typed Capability enum (replaces string action types)
#[allow(dead_code)]
pub enum Capability {
    RestartService {
        name: String,
        allowed_services: Vec<String>,
        max_per_hour: u8,
    },
    CreateCheckpoint {
        tag: String,
        max_per_session: u8,
    },
    NotifyUser {
        message: String,
    },
    RunDiagnostic {
        check_name: String,
        read_only: bool,
    },
    ClearCache {
        target: String,
    },
    RunHealthCheck,
}
/// INT-187 — Typed RollbackAction (RunCommand eliminated)
#[allow(dead_code)]
pub enum RollbackAction {
    RestartService {
        name: String,
    },
    RestoreFile {
        path: std::path::PathBuf,
        backup: std::path::PathBuf,
    },
    RevertDb {
        checkpoint: String,
    },
    CreateCheckpoint {
        tag: String,
    },
    // RunCommand intentionally absent — too dangerous
}
/// INT-187 — Three-dimensional accuracy
pub struct DelegateAccuracy {
    pub action_match: f64,      // >= 0.85 to activate
    pub outcome_success: f64,   // >= 0.80 to activate
    pub calibration_error: f64, // <= 0.10 to activate
}
impl DelegateAccuracy {
    pub fn activation_ready(&self) -> bool {
        self.action_match >= 0.85 && self.outcome_success >= 0.80 && self.calibration_error <= 0.10
    }
    pub fn from_db(ctx: &AppContext) -> Self {
        let total: i64 = ctx
            .runtime
            .db
            .query_row("SELECT COUNT(*) FROM delegate_counterfactuals", [], |r| {
                r.get(0)
            })
            .unwrap_or(0);
        if total == 0 {
            return Self {
                action_match: 0.0,
                outcome_success: 0.0,
                calibration_error: 1.0,
            };
        }
        let matches: i64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM delegate_counterfactuals WHERE action_match = 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let successes: i64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM delegate_counterfactuals WHERE outcome_matched = 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        // calibration_error = avg |predicted_confidence - actual_success_rate|
        let calib: f64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT AVG(ABS(predicted_confidence - CAST(outcome_matched AS REAL)))
             FROM delegate_counterfactuals",
                [],
                |r| r.get::<_, f64>(0),
            )
            .unwrap_or(1.0);
        Self {
            action_match: matches as f64 / total as f64,
            outcome_success: successes as f64 / total as f64,
            calibration_error: calib,
        }
    }
}
/// INT-187 — Hard boundaries (enforced at execution layer, not just policy)
pub fn check_hard_boundaries(action: &str) -> Option<&'static str> {
    let action_lower = action.to_lowercase();
    if action_lower.contains("git commit") || action_lower.contains("git push") {
        return Some("git commit/push is a hard boundary — never delegated");
    }
    if action_lower.contains("rm ")
        || action_lower.contains("delete")
        || action_lower.contains("remove")
    {
        return Some("file deletion is a hard boundary — never delegated");
    }
    if action_lower.contains("state.db")
        || action_lower.contains("engine/src")
        || action_lower.contains("scripts/core")
    {
        return Some("protected path modification is a hard boundary — never delegated");
    }
    if action_lower.contains("chmod") || action_lower.contains("chown") {
        return Some("permission changes are a hard boundary — never delegated");
    }
    None
}
fn ensure_tables(ctx: &AppContext) {
    ctx.runtime
        .db
        .execute_batch(
            "
        CREATE TABLE IF NOT EXISTS delegate_contracts (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            action_type  TEXT NOT NULL UNIQUE,
            risk_level   TEXT NOT NULL,
            confidence_gate REAL NOT NULL,
            requires_rollback INTEGER NOT NULL,
            rollback_action TEXT,
            max_frequency TEXT NOT NULL,
            human_notify TEXT NOT NULL,
            active       INTEGER NOT NULL DEFAULT 0,
            created      INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS delegate_simulations (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            action_type  TEXT NOT NULL,
            description  TEXT NOT NULL,
            confidence   REAL NOT NULL,
            risk_level   TEXT NOT NULL,
            would_execute INTEGER NOT NULL,
            reason       TEXT NOT NULL,
            rollback     TEXT,
            outcome      TEXT NOT NULL DEFAULT 'NOT_EXECUTED',
            verified     INTEGER,
            timestamp    INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS delegate_counterfactuals (
            id                   INTEGER PRIMARY KEY AUTOINCREMENT,
            simulation_id        INTEGER,
            proposed_action      TEXT NOT NULL,
            human_action         TEXT NOT NULL,
            action_match         INTEGER NOT NULL,
            predicted_confidence REAL NOT NULL,
            outcome_matched      INTEGER,
            notes                TEXT,
            logged_at            INTEGER NOT NULL
        );
    ",
        )
        .ok();
}

fn seed_contracts(ctx: &AppContext) {
    let now = chrono::Utc::now().timestamp();
    let contracts = vec![
        (
            "auto-checkpoint",
            "LOW",
            0.85,
            1,
            "core checkpoint restore {id}",
            "once_per_session",
            "on-failure",
        ),
        (
            "restart-service",
            "MEDIUM",
            0.80,
            1,
            "systemctl --user start {name}",
            "once_per_hour",
            "always",
        ),
        (
            "clear-cache",
            "LOW",
            0.85,
            0,
            "",
            "once_per_day",
            "on-failure",
        ),
        ("run-health-check", "LOW", 0.75, 0, "", "unlimited", "never"),
        ("git-commit", "HIGH", 1.00, 0, "", "never", "always"),
        ("delete-file", "CRITICAL", 1.00, 0, "", "never", "always"),
        ("modify-config", "CRITICAL", 1.00, 0, "", "never", "always"),
    ];
    for (action, risk, gate, req_rb, rollback, freq, notify) in contracts {
        ctx.runtime.db.execute(
            "INSERT OR IGNORE INTO delegate_contracts
             (action_type, risk_level, confidence_gate, requires_rollback, rollback_action, max_frequency, human_notify, active, created)
             VALUES (?1,?2,?3,?4,?5,?6,?7,0,?8)",
            rusqlite::params![action, risk, gate, req_rb, rollback, freq, notify, now],
        ).ok();
    }
}

fn risk_color(risk: &str) -> colored::ColoredString {
    match risk {
        "LOW" => risk.bright_green(),
        "MEDIUM" => risk.yellow(),
        "HIGH" => risk.bright_red(),
        "CRITICAL" => risk.bright_red().bold(),
        _ => risk.normal(),
    }
}

// ── simulate ──────────────────────────────────────────────────────────────────
pub fn simulate(ctx: &AppContext, action: &str) -> CoreResult<()> {
    ensure_tables(ctx);
    seed_contracts(ctx);

    // INT-187 — Hard boundary check (execution layer, not policy)
    if let Some(boundary_reason) = check_hard_boundaries(action) {
        println!();
        println!("{}", "🛑 Hard Boundary Violation".bright_red().bold());
        println!("{}", "━".repeat(52).dimmed());
        println!("  {} {}", "Action:".dimmed(), action.bright_white());
        println!("  {} {}", "Reason:".dimmed(), boundary_reason.bright_red());
        println!("  {} This action can never be delegated.", "→".dimmed());
        println!();
        return Ok(());
    }
    // Classify action type
    let action_type = classify_action(action);

    // Load contract
    let contract = ctx.runtime.db.query_row(
        "SELECT action_type, risk_level, confidence_gate, requires_rollback, rollback_action, active
         FROM delegate_contracts WHERE action_type = ?1",
        rusqlite::params![&action_type],
        |r: &rusqlite::Row| Ok((
            r.get::<_,String>(0)?,
            r.get::<_,String>(1)?,
            r.get::<_,f64>(2)?,
            r.get::<_,i64>(3)?,
            r.get::<_,String>(4).unwrap_or_default(),
            r.get::<_,i64>(5)?,
        )),
    );

    let (risk_level, confidence_gate, _requires_rollback, rollback_action) = match contract {
        Ok((_, risk, gate, req_rb, rollback, _)) => (risk, gate, req_rb, rollback),
        Err(_) => ("HIGH".to_string(), 1.0, 0, String::new()),
    };

    // Estimate confidence from history
    let confidence = estimate_confidence(ctx, &action_type, action);
    let would_execute =
        confidence >= confidence_gate && risk_level != "HIGH" && risk_level != "CRITICAL";

    let reason = build_reason(ctx, &action_type, confidence, confidence_gate, &risk_level);
    let rollback_display = if rollback_action.is_empty() {
        "none".to_string()
    } else {
        rollback_action.clone()
    };

    // Record simulation
    let now = chrono::Utc::now().timestamp();
    ctx.runtime.db.execute(
        "INSERT INTO delegate_simulations
         (action_type, description, confidence, risk_level, would_execute, reason, rollback, outcome, timestamp)
         VALUES (?1,?2,?3,?4,?5,?6,?7,'NOT_EXECUTED',?8)",
        rusqlite::params![
            &action_type, action, confidence, &risk_level,
            would_execute as i64, &reason, &rollback_display, now
        ],
    ).ok();

    // Display
    println!();
    println!("{}", "🔮 Delegation Simulation".bright_cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!("  {}  {}", "Action:".dimmed(), action.bright_white().bold());
    println!("  {}  {}", "Type:".dimmed(), action_type.bright_yellow());
    println!("  {}  {}", "Risk:".dimmed(), risk_color(&risk_level));
    println!();
    println!(
        "  {}  {:.0}%  {}  {:.0}% required",
        "Confidence:".dimmed(),
        confidence * 100.0,
        "│".dimmed(),
        confidence_gate * 100.0
    );
    println!("  {}  {}", "Reason:".dimmed(), reason.dimmed());
    println!();
    if !rollback_display.is_empty() && rollback_display != "none" {
        println!("  {}  {}", "Rollback:".dimmed(), rollback_display.dimmed());
    }
    println!();
    if would_execute {
        println!(
            "  {} {}",
            "Would execute?".bright_white(),
            "YES".bright_green().bold()
        );
    } else {
        println!(
            "  {} {}",
            "Would execute?".bright_white(),
            "NO".bright_red().bold()
        );
    }
    println!(
        "  {} {}",
        "Outcome:".bright_white(),
        "NOT EXECUTED (simulation only)".yellow().bold()
    );
    println!();

    Ok(())
}

fn classify_action(action: &str) -> String {
    let a = action.to_lowercase();
    if a.contains("checkpoint") {
        "auto-checkpoint".into()
    } else if a.contains("restart") || a.contains("start") {
        "restart-service".into()
    } else if a.contains("cache") || a.contains("clear") {
        "clear-cache".into()
    } else if a.contains("health") || a.contains("doctor") {
        "run-health-check".into()
    } else if a.contains("commit") || a.contains("push") {
        "git-commit".into()
    } else if a.contains("delete") || a.contains("remove") {
        "delete-file".into()
    } else if a.contains("config") || a.contains("modify") {
        "modify-config".into()
    } else {
        "run-health-check".into()
    }
}

fn estimate_confidence(ctx: &AppContext, action_type: &str, _action: &str) -> f64 {
    // Check past simulation accuracy for this action type
    let total: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM delegate_simulations WHERE action_type = ?1 AND verified IS NOT NULL",
        rusqlite::params![action_type],
        |r: &rusqlite::Row| r.get(0),
    ).unwrap_or(0);

    if total == 0 {
        // No history — use conservative base confidence by risk type
        return match action_type {
            "run-health-check" => 0.88,
            "auto-checkpoint" => 0.86,
            "clear-cache" => 0.82,
            "restart-service" => 0.72,
            _ => 0.50,
        };
    }

    let correct: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM delegate_simulations WHERE action_type = ?1 AND verified = 1",
            rusqlite::params![action_type],
            |r: &rusqlite::Row| r.get(0),
        )
        .unwrap_or(0);

    correct as f64 / total as f64
}

fn build_reason(
    ctx: &AppContext,
    action_type: &str,
    confidence: f64,
    gate: f64,
    risk: &str,
) -> String {
    if risk == "CRITICAL" {
        return "CRITICAL risk — never auto-executed, human required".into();
    }
    if risk == "HIGH" {
        return "HIGH risk — always requires human confirmation".into();
    }
    if confidence < gate {
        return format!(
            "confidence {:.0}% below gate {:.0}% — more history needed",
            confidence * 100.0,
            gate * 100.0
        );
    }
    let sims: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM delegate_simulations WHERE action_type = ?1",
            rusqlite::params![action_type],
            |r: &rusqlite::Row| r.get(0),
        )
        .unwrap_or(0);
    format!(
        "confidence {:.0}% meets gate, {} prior simulations observed",
        confidence * 100.0,
        sims
    )
}

// ── contracts ─────────────────────────────────────────────────────────────────
pub fn contracts(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);
    seed_contracts(ctx);

    println!();
    println!("{}", "📋 Trust Contracts".bright_cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!(
        "  {:<22} {:<10} {:<8} {:<8}",
        "Action Type".bright_white(),
        "Risk".bright_white(),
        "Gate".bright_white(),
        "Active".bright_white(),
    );
    println!("{}", "  ─".repeat(28).dimmed());

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT action_type, risk_level, confidence_gate, active FROM delegate_contracts ORDER BY risk_level"
    )?;
    let rows = stmt.query_map([], |r: &rusqlite::Row<'_>| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, f64>(2)?,
            r.get::<_, i64>(3)?,
        ))
    })?;

    for (action, risk, gate, active) in rows.flatten() {
        let active_str = if active == 1 {
            "✅".to_string()
        } else {
            "○ sim".to_string()
        };
        println!(
            "  {:<22} {:<10} {:<8} {}",
            action.bright_yellow(),
            risk_color(&risk),
            format!("{:.0}%", gate * 100.0).dimmed(),
            active_str,
        );
    }
    println!();
    println!(
        "  {} All contracts in simulation mode — activation requires 85% accuracy over 14+ days",
        "ℹ".bright_cyan()
    );
    println!();
    Ok(())
}

// ── history ───────────────────────────────────────────────────────────────────
pub fn history(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);

    println!();
    println!("{}", "📜 Delegation History".bright_cyan().bold());
    println!("{}", "━".repeat(60).dimmed());

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT description, action_type, confidence, risk_level, would_execute, outcome, timestamp
         FROM delegate_simulations ORDER BY timestamp DESC LIMIT 20",
    )?;
    let rows = stmt.query_map([], |r: &rusqlite::Row<'_>| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, f64>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, i64>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, i64>(6)?,
        ))
    })?;

    let mut count = 0;
    for (desc, atype, conf, risk, would, outcome, ts) in rows.flatten() {
        let time = chrono::DateTime::from_timestamp(ts, 0)
            .map(|t| t.format("%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "?".to_string());
        let exec = if would == 1 {
            "YES".bright_green()
        } else {
            "NO".bright_red()
        };
        println!(
            "  {} {} {} conf:{:.0}% {}",
            time.dimmed(),
            exec,
            risk_color(&risk),
            conf * 100.0,
            desc.bright_white()
        );
        println!("       {} → {}", atype.dimmed(), outcome.dimmed());
        count += 1;
    }
    if count == 0 {
        println!(
            "  {} No simulations run yet — try: core delegate simulate restart-faelight-notify",
            "○".dimmed()
        );
    }
    println!();
    Ok(())
}

// ── accuracy ──────────────────────────────────────────────────────────────────
pub fn accuracy(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);

    println!();
    println!("{}", "📊 Delegation Accuracy".bright_cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    let total: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM delegate_simulations",
            [],
            |r: &rusqlite::Row| r.get(0),
        )
        .unwrap_or(0);
    let verified: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM delegate_simulations WHERE verified IS NOT NULL",
            [],
            |r: &rusqlite::Row| r.get(0),
        )
        .unwrap_or(0);
    let correct: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM delegate_simulations WHERE verified = 1",
            [],
            |r: &rusqlite::Row| r.get(0),
        )
        .unwrap_or(0);

    let acc = if verified > 0 {
        correct as f64 / verified as f64 * 100.0
    } else {
        0.0
    };
    let gate_met = acc >= 85.0 && verified >= 14;

    println!(
        "  {}  {}",
        "Total simulations:".dimmed(),
        total.to_string().bright_white()
    );
    println!(
        "  {}  {}",
        "Verified:".dimmed(),
        verified.to_string().bright_white()
    );
    println!(
        "  {}  {:.1}%",
        "Accuracy:".dimmed(),
        if verified > 0 { acc } else { 0.0 }
    );
    println!();
    if gate_met {
        println!(
            "  {} Activation gate MET — run: core delegate activate <contract>",
            "✅".green()
        );
    } else {
        println!(
            "  {} Activation gate: need 85% accuracy over 14+ verified simulations",
            "⬜".normal()
        );
        println!(
            "  {} Current: {:.1}% accuracy, {} verified",
            "→".dimmed(),
            acc,
            verified
        );
    }
    println!();
    Ok(())
}

// ── suspend ───────────────────────────────────────────────────────────────────
pub fn suspend(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);
    ctx.runtime
        .db
        .execute("UPDATE delegate_contracts SET active = 0", [])
        .ok();
    println!(
        "  {} All delegation suspended — simulation mode only",
        "🛑".bright_red()
    );
    Ok(())
}

// ── activate ──────────────────────────────────────────────────────────────────
pub fn activate(ctx: &AppContext, contract: &str) -> CoreResult<()> {
    ensure_tables(ctx);

    // Check activation gate
    let verified: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM delegate_simulations WHERE action_type = ?1 AND verified IS NOT NULL",
        rusqlite::params![contract], |r: &rusqlite::Row| r.get(0)
    ).unwrap_or(0);
    let correct: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM delegate_simulations WHERE action_type = ?1 AND verified = 1",
            rusqlite::params![contract],
            |r: &rusqlite::Row| r.get(0),
        )
        .unwrap_or(0);

    let acc = if verified > 0 {
        correct as f64 / verified as f64 * 100.0
    } else {
        0.0
    };

    if verified < 14 || acc < 85.0 {
        println!();
        println!(
            "  {} Activation gate NOT met for: {}",
            "❌".bright_red(),
            contract.bright_yellow()
        );
        println!(
            "  {}  Need 14+ verified simulations at 85%+ accuracy",
            "→".dimmed()
        );
        println!(
            "  {}  Current: {} verified, {:.1}% accuracy",
            "→".dimmed(),
            verified,
            acc
        );
        println!();
        return Ok(());
    }

    // Check hard boundaries
    if matches!(contract, "git-commit" | "delete-file" | "modify-config") {
        println!(
            "  {} '{}' is a hard boundary — activation permanently blocked",
            "🔒".bright_red(),
            contract
        );
        return Ok(());
    }

    ctx.runtime
        .db
        .execute(
            "UPDATE delegate_contracts SET active = 1 WHERE action_type = ?1",
            rusqlite::params![contract],
        )
        .ok();

    println!(
        "  {} Contract activated: {}",
        "✅".green(),
        contract.bright_green().bold()
    );
    println!(
        "  {} Monitor closely — suspend anytime with: core delegate suspend",
        "ℹ".bright_cyan()
    );
    Ok(())
}

/// core delegate counterfactuals — show ground truth comparison log
pub fn counterfactuals(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);
    println!();
    println!(
        "{}",
        "📊 Delegation Counterfactual Log".bright_cyan().bold()
    );
    println!("{}", "━".repeat(60).dimmed());
    println!();
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT proposed_action, human_action, action_match, predicted_confidence,
                outcome_matched, logged_at
         FROM delegate_counterfactuals ORDER BY logged_at DESC LIMIT 20",
    )?;
    let rows: Vec<(String, String, i64, f64, Option<i64>, i64)> = stmt
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    if rows.is_empty() {
        println!("  {} No counterfactuals logged yet.", "○".dimmed());
        println!(
            "  {} Run simulations and record your actual actions to build the dataset.",
            "→".dimmed()
        );
        println!();
        return Ok(());
    }
    for (proposed, human, matched, conf, outcome, ts) in &rows {
        let match_icon = if *matched == 1 {
            "✓".bright_green()
        } else {
            "✗".bright_red()
        };
        let outcome_str = match outcome {
            Some(1) => "success".bright_green().to_string(),
            Some(0) => "failure".bright_red().to_string(),
            _ => "pending".dimmed().to_string(),
        };
        let date = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|t| t.format("%m-%d %H:%M").to_string())
            .unwrap_or_default();
        println!(
            "  {} {} proposed: {} → human: {} | conf: {:.0}% | outcome: {}",
            match_icon,
            date.dimmed(),
            proposed.bright_white(),
            human.bright_yellow(),
            conf * 100.0,
            outcome_str
        );
    }
    println!();
    // Show accuracy summary
    let acc = DelegateAccuracy::from_db(ctx);
    println!(
        "  {} Action match: {:.0}%  Outcome success: {:.0}%  Calibration error: {:.2}",
        "→".dimmed(),
        acc.action_match * 100.0,
        acc.outcome_success * 100.0,
        acc.calibration_error
    );
    println!();
    Ok(())
}
/// core delegate log-counterfactual <proposed> <actual> <matched>
pub fn log_counterfactual(
    ctx: &AppContext,
    proposed: &str,
    human: &str,
    matched: bool,
    confidence: f64,
) -> CoreResult<()> {
    ensure_tables(ctx);
    let now = chrono::Utc::now().timestamp();
    ctx.runtime.db.execute(
        "INSERT INTO delegate_counterfactuals
         (proposed_action, human_action, action_match, predicted_confidence, logged_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![proposed, human, matched as i64, confidence, now],
    )?;
    println!(
        "  {} Counterfactual logged: proposed='{}' human='{}' match={}",
        "✅".green(),
        proposed,
        human,
        if matched { "yes" } else { "no" }
    );
    Ok(())
}
/// core delegate accuracy — three-dimensional accuracy report
pub fn accuracy_report(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx);
    println!();
    println!(
        "{}",
        "🎯 Delegation Accuracy — Three Dimensions"
            .bright_cyan()
            .bold()
    );
    println!("{}", "━".repeat(60).dimmed());
    println!();
    let acc = DelegateAccuracy::from_db(ctx);
    let total: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM delegate_counterfactuals", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    println!(
        "  {} {} counterfactuals logged",
        "📋".normal(),
        total.to_string().bright_white()
    );
    println!();
    let am_colored = if acc.action_match >= 0.85 {
        format!("{:.0}%", acc.action_match * 100.0).bright_green()
    } else {
        format!("{:.0}%", acc.action_match * 100.0).bright_red()
    };
    let os_colored = if acc.outcome_success >= 0.80 {
        format!("{:.0}%", acc.outcome_success * 100.0).bright_green()
    } else {
        format!("{:.0}%", acc.outcome_success * 100.0).bright_red()
    };
    let ce_colored = if acc.calibration_error <= 0.10 {
        format!("{:.3}", acc.calibration_error).bright_green()
    } else {
        format!("{:.3}", acc.calibration_error).bright_red()
    };
    println!(
        "  {:<30} {} (gate: ≥85%)",
        "Action match:".bright_white(),
        am_colored
    );
    println!(
        "  {:<30} {} (gate: ≥80%)",
        "Outcome success:".bright_white(),
        os_colored
    );
    println!(
        "  {:<30} {} (gate: ≤0.10)",
        "Calibration error:".bright_white(),
        ce_colored
    );
    println!();
    if acc.activation_ready() {
        println!(
            "  {} ALL GATES PASSED — delegation can be activated",
            "✅".green().bold()
        );
        println!(
            "  {} Run: core delegate activate <contract>",
            "→".bright_cyan()
        );
    } else {
        println!("  {} Activation gates not yet met:", "⏳".normal());
        if acc.action_match < 0.85 {
            println!(
                "    {} action_match {:.0}% < 85%",
                "✗".bright_red(),
                acc.action_match * 100.0
            );
        }
        if acc.outcome_success < 0.80 {
            println!(
                "    {} outcome_success {:.0}% < 80%",
                "✗".bright_red(),
                acc.outcome_success * 100.0
            );
        }
        if acc.calibration_error > 0.10 {
            println!(
                "    {} calibration_error {:.3} > 0.10",
                "✗".bright_red(),
                acc.calibration_error
            );
        }
        if total < 50 {
            println!(
                "    {} only {} samples — need 50+ for reliable measurement",
                "💡".bright_cyan(),
                total
            );
        }
    }
    println!();
    Ok(())
}
