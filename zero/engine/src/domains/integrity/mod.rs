// INT-184 — Doctor Integrity Engine
// Phase 0: Core infrastructure — traits, types, pipeline, logging
//
// Architecture (non-negotiable):
// Phase A — Scan    (pure, no mutation)
// Phase B — Plan    (classify issues)
// Phase C — Apply   (safe auto-fixes only)
// Phase D — Re-scan (affected domains only)
// Phase E — Report  (proposals + alerts)

use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::path::PathBuf;

// ── Core Types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    AutoFix, // safe, idempotent, no destructive behavior
    Propose, // requires human confirmation, persisted until resolved
    Alert,   // requires human intervention, no auto action
}

#[derive(Debug, Clone, PartialEq)]
pub enum Category {
    Intent,
    Registry,
    Database,
    Documentation,
    Shell,
    Temporal,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Intent => "intent",
            Category::Registry => "registry",
            Category::Database => "database",
            Category::Documentation => "documentation",
            Category::Shell => "shell",
            Category::Temporal => "temporal",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FixAction {
    MoveFile {
        from: PathBuf,
        to: PathBuf,
    },
    UpdateRegistryVersion {
        tool: String,
        version: String,
    },
    UpdateRegistryField {
        tool: String,
        field: String,
        value: String,
    },
    InsertDbRow {
        table: String,
        sql: String,
    },
    VacuumDb,
    SyncDocs,
    RebuildJarvisScore,
    UpdateFile {
        path: PathBuf,
        old: String,
        new: String,
    },
}

#[derive(Debug, Clone)]
pub struct IntegrityIssue {
    pub category: Category,
    pub check: &'static str,
    pub severity: Severity,
    pub description: String,
    pub fix: Option<FixAction>,
    pub weight: u8, // 1=trivial 2=minor 3=moderate 4=significant 5=critical
}

impl IntegrityIssue {
    pub fn auto_fix(
        category: Category,
        check: &'static str,
        description: &str,
        fix: FixAction,
        weight: u8,
    ) -> Self {
        Self {
            category,
            check,
            severity: Severity::AutoFix,
            description: description.to_string(),
            fix: Some(fix),
            weight,
        }
    }
    pub fn propose(
        category: Category,
        check: &'static str,
        description: &str,
        fix: FixAction,
        weight: u8,
    ) -> Self {
        Self {
            category,
            check,
            severity: Severity::Propose,
            description: description.to_string(),
            fix: Some(fix),
            weight,
        }
    }
    pub fn alert(category: Category, check: &'static str, description: &str, weight: u8) -> Self {
        Self {
            category,
            check,
            severity: Severity::Alert,
            description: description.to_string(),
            fix: None,
            weight,
        }
    }
}

// ── IntegrityCheck Trait ──────────────────────────────────────────────────────

pub struct IntegrityContext<'a> {
    pub ctx: &'a AppContext,
    pub core_root: PathBuf,
}

impl<'a> IntegrityContext<'a> {
    pub fn new(ctx: &'a AppContext) -> Self {
        Self {
            core_root: PathBuf::from(&ctx.core_root),
            ctx,
        }
    }
}

#[allow(dead_code)]
pub trait IntegrityCheck {
    fn name(&self) -> &'static str;
    fn category(&self) -> Category;
    fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue>;
}

// ── DB Init ───────────────────────────────────────────────────────────────────

pub fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(
        "CREATE TABLE IF NOT EXISTS integrity_log (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            category    TEXT    NOT NULL,
            check_name  TEXT    NOT NULL,
            severity    TEXT    NOT NULL,
            description TEXT    NOT NULL,
            weight      INTEGER NOT NULL DEFAULT 1,
            fixed       INTEGER NOT NULL DEFAULT 0,
            fixed_at    INTEGER,
            detected_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS pending_fixes (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            category    TEXT    NOT NULL,
            check_name  TEXT    NOT NULL,
            action_type TEXT    NOT NULL,
            action_data TEXT    NOT NULL,
            description TEXT    NOT NULL,
            created_at  INTEGER NOT NULL,
            applied_at  INTEGER
        );",
    )?;
    // INT-275: a proposal has an identity and can retire without being applied. Additive
    // columns on the existing table, added once; no row is rewritten except pending drift rows,
    // whose subject is the tool already stored as data before the tab.
    for (column, decl) in [
        ("subject", "TEXT"),
        ("retired_at", "INTEGER"),
        ("retired_reason", "TEXT"),
    ] {
        let present: i64 = ctx.runtime.db.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('pending_fixes') WHERE name = ?1",
            [column],
            |r| r.get(0),
        )?;
        if present == 0 {
            ctx.runtime.db.execute_batch(&format!(
                "ALTER TABLE pending_fixes ADD COLUMN {} {};",
                column, decl
            ))?;
        }
    }
    ctx.runtime.db.execute(
        "UPDATE pending_fixes SET subject = substr(action_data, 1, instr(action_data, char(9)) - 1)
         WHERE subject IS NULL AND applied_at IS NULL
           AND action_type = 'UpdateRegistryVersion' AND instr(action_data, char(9)) > 0",
        [],
    )?;
    Ok(())
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ── Pipeline Execution ────────────────────────────────────────────────────────

pub struct PipelineResult {
    pub auto_fixed: usize,
    pub proposed: usize,
    pub alerts: usize,
    pub total_weight: u32,
    pub issue_weight: u32,
}

impl PipelineResult {
    pub fn integrity_pct(&self) -> u32 {
        if self.total_weight == 0 {
            return 100;
        }

        100u32.saturating_sub((self.issue_weight * 100) / self.total_weight)
    }
}

fn apply_safe_fix(fix: &FixAction, ctx: &IntegrityContext) -> bool {
    match fix {
        FixAction::UpdateRegistryVersion { tool, version } => {
            let path = ctx.ctx.fpath("registry/tools.toml");
            if let Ok(content) = std::fs::read_to_string(&path) {
                let old = format!("name = \"{}\"\nversion = \"", tool);
                if let Some(idx) = content.find(&old) {
                    let ver_start = idx + old.len();
                    if let Some(ver_end) = content[ver_start..].find('"') {
                        let new_content = format!(
                            "{}{}{}",
                            &content[..ver_start],
                            version,
                            &content[ver_start + ver_end..]
                        );
                        return std::fs::write(&path, new_content).is_ok();
                    }
                }
            }
            false
        }
        FixAction::InsertDbRow { table: _, sql } => ctx.ctx.runtime.db.execute_batch(sql).is_ok(),
        FixAction::SyncDocs => std::process::Command::new("zero-docs")
            .arg("sync")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false),
        FixAction::RebuildJarvisScore => {
            // Trigger friday readiness recomputation by running it
            true // score rebuilds on next friday-readiness call
        }
        _ => false, // MoveFile, VacuumDb, config rewrites = not safe
    }
}

fn log_issue(ctx: &IntegrityContext, issue: &IntegrityIssue, fixed: bool) {
    let severity_str = match issue.severity {
        Severity::AutoFix => "auto-fix",
        Severity::Propose => "propose",
        Severity::Alert => "alert",
    };
    // Only insert if no identical unfixed record already exists
    let existing: i64 = ctx.ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM integrity_log WHERE check_name = ?1 AND description = ?2 AND fixed = 0",
        rusqlite::params![issue.check, issue.description],
        |r| r.get(0),
    ).unwrap_or(0);
    if existing == 0 {
        ctx.ctx.runtime.db.execute(
            "INSERT INTO integrity_log (category, check_name, severity, description, weight, fixed, fixed_at, detected_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                issue.category.as_str(),
                issue.check,
                severity_str,
                issue.description,
                issue.weight,
                if fixed { 1 } else { 0 },
                if fixed { Some(now_ts()) } else { None::<i64> },
                now_ts()
            ],
        ).ok();
    }
}

// ── INT-275: a proposal's identity, and the rows a reader may show ────────────────
//
// A proposal is shown, counted and applied only while a fresh run of its check still finds it.
// Identity is (check_name, subject). The description is display text and carries versions, so it
// is never a key. Retired is not applied: a retired row stays as history, with when and why.

/// The one test of "still pending": not applied and not retired.
const PENDING: &str = "applied_at IS NULL AND retired_at IS NULL";

/// What a fix acts on: the file for a move or an edit, the tool for drift. With the check name
/// this is the proposal's identity. Exhaustive on purpose: a new FixAction must name its subject.
fn fix_subject(fix: &FixAction) -> Option<String> {
    match fix {
        FixAction::MoveFile { from, .. } => Some(from.display().to_string()),
        FixAction::UpdateRegistryVersion { tool, .. } => Some(tool.clone()),
        FixAction::UpdateRegistryField { tool, field, .. } => Some(format!("{}.{}", tool, field)),
        FixAction::InsertDbRow { table, .. } => Some(table.clone()),
        FixAction::VacuumDb => Some("state.db".to_string()),
        FixAction::SyncDocs => Some("docs".to_string()),
        FixAction::RebuildJarvisScore => Some("jarvis-score".to_string()),
        FixAction::UpdateFile { path, .. } => Some(path.display().to_string()),
    }
}

/// The fix as data, so apply carries it out without a re-scan (INT-267 batch 1b; moves, INT-275).
fn fix_data(issue: &IntegrityIssue) -> String {
    match &issue.fix {
        Some(FixAction::UpdateRegistryVersion { tool, version }) => {
            format!("{}\t{}", tool, version)
        }
        Some(FixAction::MoveFile { from, to }) => format!("{}\t{}", from.display(), to.display()),
        _ => issue.description.clone(),
    }
}

/// INT-275: the one reader of an intent's status -- its frontmatter line, exactly, within the
/// first twenty lines. A body that mentions a status is not a status.
pub fn frontmatter_status(content: &str) -> String {
    content
        .lines()
        .take(20)
        .find(|l| l.trim().starts_with("status:"))
        .unwrap_or("")
        .trim()
        .to_string()
}

fn persist_proposal(ctx: &IntegrityContext, issue: &IntegrityIssue) {
    let action_type = match &issue.fix {
        Some(FixAction::MoveFile { .. }) => "MoveFile",
        Some(FixAction::UpdateRegistryVersion { .. }) => "UpdateRegistryVersion",
        Some(FixAction::VacuumDb) => "VacuumDb",
        Some(FixAction::SyncDocs) => "SyncDocs",
        Some(FixAction::UpdateFile { .. }) => "UpdateFile",
        _ => "Unknown",
    };
    let action_data = fix_data(issue);
    let subject = issue.fix.as_ref().and_then(fix_subject);
    // INT-275: the same check, subject and fix already pending is the same proposal. A new fix
    // for the same identity is a new row; the reconcile retires the older one as superseded.
    let exists: i64 = ctx
        .ctx
        .runtime
        .db
        .query_row(
            &format!(
                "SELECT COUNT(*) FROM pending_fixes
                 WHERE check_name = ?1 AND subject IS ?2 AND action_data = ?3 AND {}",
                PENDING
            ),
            rusqlite::params![issue.check, subject, &action_data],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if exists == 0 {
        ctx.ctx.runtime.db.execute(
            "INSERT INTO pending_fixes (category, check_name, action_type, action_data, description, created_at, subject)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                issue.category.as_str(),
                issue.check,
                action_type,
                &action_data,
                &issue.description,
                now_ts(),
                subject
            ],
        ).ok();
    }
}

/// INT-275: after a run, every pending row is still found or retired with its reason. First
/// match wins: check removed, pre-275 row, superseded by #N, resolved. A check that did not run
/// in this pipeline (a partial run) says nothing about its rows; they are left alone.
fn reconcile(ctx: &IntegrityContext, ran: &[&'static str], found: &[&IntegrityIssue]) {
    let db = &ctx.ctx.runtime.db;
    let suite: Vec<&'static str> = build_check_suite().iter().map(|c| c.name()).collect();
    let rows: Vec<(i64, String, Option<String>)> = match db.prepare(&format!(
        "SELECT id, check_name, subject FROM pending_fixes WHERE {} ORDER BY id DESC",
        PENDING
    )) {
        Ok(mut stmt) => {
            let x: Vec<(i64, String, Option<String>)> = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .map(|it| it.filter_map(|r| r.ok()).collect())
                .unwrap_or_default();
            x
        }
        Err(_) => return,
    };
    // Newest first, so the first row seen for an identity is the one that stays.
    let mut newest: std::collections::HashMap<(String, String), i64> =
        std::collections::HashMap::new();
    for (id, check, subject) in &rows {
        let ran_now = ran.iter().any(|c| *c == check.as_str());
        let reason = if !suite.iter().any(|c| *c == check.as_str()) {
            Some("check removed".to_string())
        } else if subject.is_none() {
            Some("pre-275 row".to_string())
        } else if !ran_now {
            None
        } else {
            let subject = subject.clone().unwrap_or_default();
            let key = (check.clone(), subject.clone());
            if let Some(newer) = newest.get(&key) {
                Some(format!("superseded by #{}", newer))
            } else {
                newest.insert(key, *id);
                let still_found = found.iter().any(|i| {
                    i.check == check.as_str()
                        && i.fix.as_ref().and_then(fix_subject).as_deref() == Some(subject.as_str())
                });
                if still_found {
                    None
                } else {
                    Some("resolved".to_string())
                }
            }
        };
        if let Some(reason) = reason {
            db.execute(
                &format!(
                    "UPDATE pending_fixes SET retired_at = ?1, retired_reason = ?2 WHERE id = ?3 AND {}",
                    PENDING
                ),
                rusqlite::params![now_ts(), reason, id],
            )
            .ok();
        }
    }
}

/// Run the full integrity pipeline with all provided checks.
/// Phase A → B → C → D → E
pub fn run_pipeline(
    ctx: &IntegrityContext,
    checks: &[Box<dyn IntegrityCheck>],
    safe_only: bool,
) -> PipelineResult {
    // Phase A — Scan (pure, no mutation)
    let mut all_issues: Vec<IntegrityIssue> = vec![];
    for check in checks {
        let issues = check.run(ctx);
        all_issues.extend(issues);
    }

    // Compute total possible weight (all checks at max weight 3 avg)
    let total_weight: u32 = all_issues
        .iter()
        .map(|i| i.weight as u32)
        .sum::<u32>()
        .max(1)
        * 3; // baseline

    // Phase B — Plan (classify)
    let auto_fixable: Vec<&IntegrityIssue> = all_issues
        .iter()
        .filter(|i| i.severity == Severity::AutoFix && i.fix.is_some())
        .collect();
    let proposals: Vec<&IntegrityIssue> = all_issues
        .iter()
        .filter(|i| i.severity == Severity::Propose)
        .collect();
    let alerts: Vec<&IntegrityIssue> = all_issues
        .iter()
        .filter(|i| i.severity == Severity::Alert)
        .collect();

    // Phase C — Apply safe auto-fixes
    let mut auto_fixed = 0;
    let mut post_fix_issues = all_issues.clone();

    if !safe_only {
        // Full engine — apply all auto-fixes
    }

    for issue in &auto_fixable {
        if let Some(fix) = &issue.fix {
            // INT-241: skip if this check was already fixed in last 5 minutes (dedup)
            let recently_fixed: i64 = ctx.ctx.runtime.db.query_row(
                "SELECT COUNT(*) FROM integrity_log WHERE check_name = ?1 AND fixed = 1 AND detected_at > ?2",
                rusqlite::params![issue.check, now_ts() - 300],
                |r| r.get(0)
            ).unwrap_or(0);
            if recently_fixed > 0 {
                continue; // already fixed this session -- skip
            }
            let fixed = apply_safe_fix(fix, ctx);
            log_issue(ctx, issue, fixed);
            if fixed {
                auto_fixed += 1;
                post_fix_issues.retain(|i| i.check != issue.check);
            }
        }
    }

    // Persist proposals
    for issue in &proposals {
        persist_proposal(ctx, issue);
        log_issue(ctx, issue, false);
    }

    // INT-275: every pending row is still found, or retired with its reason.
    let ran: Vec<&'static str> = checks.iter().map(|c| c.name()).collect();
    reconcile(ctx, &ran, &proposals);

    // Log alerts
    for issue in &alerts {
        log_issue(ctx, issue, false);
    }

    // Phase D — Re-scan (simplified: use remaining issues)
    // Full re-scan would re-run affected checks — deferred to Phase 6

    // Phase E — compute result
    let remaining_weight: u32 = post_fix_issues
        .iter()
        .filter(|i| i.severity != Severity::AutoFix || auto_fixed == 0)
        .map(|i| i.weight as u32)
        .sum();

    PipelineResult {
        auto_fixed,
        proposed: proposals.len(),
        alerts: alerts.len(),
        total_weight,
        issue_weight: remaining_weight,
    }
}

// ── Commands ──────────────────────────────────────────────────────────────────

pub fn cmd_run(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let ictx = IntegrityContext::new(ctx);

    println!();
    println!("  {} Integrity Scan", "🔍".normal());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();

    // Load all checks in deterministic order
    let checks = build_check_suite();
    let result = run_pipeline(&ictx, &checks, false);

    let pct = result.integrity_pct();
    let pct_str = if pct >= 95 {
        format!("{}%", pct).bright_green().to_string()
    } else if pct >= 75 {
        format!("{}%", pct).yellow().to_string()
    } else {
        format!("{}%", pct).bright_red().to_string()
    };

    println!("  {} Integrity: {}", "▶".bright_cyan(), pct_str);
    println!();
    if result.auto_fixed > 0 {
        println!(
            "  {} Auto-fixed: {} issues",
            "✅".normal(),
            result.auto_fixed.to_string().bright_green()
        );
    }
    if result.proposed > 0 {
        println!(
            "  {} Proposed:   {} issues (run: core integrity fix)",
            "⚠️ ".normal(),
            result.proposed.to_string().yellow()
        );
    }
    if result.alerts > 0 {
        println!(
            "  {} Alerts:     {} issues requiring attention",
            "❌".normal(),
            result.alerts.to_string().bright_red()
        );
    }
    if result.auto_fixed == 0 && result.proposed == 0 && result.alerts == 0 {
        println!("  {} No integrity issues detected", "✅".normal());
    }
    println!();
    Ok(())
}

pub fn cmd_status(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;

    // Count pending fixes
    let pending: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM pending_fixes WHERE applied_at IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Count recent auto-fixes (last 24h)
    let recent_fixed: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM integrity_log WHERE fixed=1 AND detected_at > ?1",
            rusqlite::params![now_ts() - 86400],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Count active alerts
    let active_alerts: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM integrity_log WHERE severity='alert' AND fixed=0 AND detected_at > ?1",
        rusqlite::params![now_ts() - 86400], |r| r.get(0)
    ).unwrap_or(0);

    println!();
    println!("  {} Integrity Status", "📊".normal());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!(
        "  {} Pending proposals: {}",
        "→".dimmed(),
        pending.to_string().yellow()
    );
    println!(
        "  {} Auto-fixed (24h):  {}",
        "→".dimmed(),
        recent_fixed.to_string().bright_green()
    );
    println!(
        "  {} Active alerts:     {}",
        "→".dimmed(),
        active_alerts.to_string().bright_red()
    );
    println!();
    println!("  {} Run: core integrity run — full scan", "hint:".dimmed());
    println!(
        "  {} Run: core integrity fix — apply proposals",
        "hint:".dimmed()
    );
    println!();
    Ok(())
}

pub fn cmd_log(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;

    println!();
    println!("  {} Integrity Log (last 20)", "📋".normal());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT category, check_name, severity, description, fixed, detected_at
         FROM integrity_log ORDER BY detected_at DESC LIMIT 20",
    )?;

    let rows: Vec<(String, String, String, String, i64)> = stmt
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if rows.is_empty() {
        println!(
            "  {} No integrity history yet — run: core integrity run",
            "○".dimmed()
        );
    } else {
        for (cat, check, severity, desc, fixed) in &rows {
            let icon = match severity.as_str() {
                "auto-fix" => {
                    if *fixed == 1 {
                        "✅".to_string()
                    } else {
                        "🔧".to_string()
                    }
                }
                "propose" => "⚠️ ".to_string(),
                "alert" => "❌".to_string(),
                _ => "○".to_string(),
            };
            println!(
                "  {} [{}] {} — {}",
                icon,
                cat.bright_cyan(),
                check.bright_white(),
                desc.dimmed()
            );
        }
    }
    println!();
    Ok(())
}

pub fn cmd_fix(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    // INT-275: list what a fresh run finds, not what the table remembers. This is the run
    // quick_scan makes on every d; the reconcile at its end retires what is no longer true.
    run_pipeline(&IntegrityContext::new(ctx), &build_check_suite(), true);

    let mut stmt = ctx.runtime.db.prepare(&format!(
        "SELECT id, category, check_name, description FROM pending_fixes WHERE {} ORDER BY created_at ASC",
        PENDING
    ))?;

    let rows: Vec<(i64, String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .filter_map(|r| r.ok())
        .collect();

    if rows.is_empty() {
        println!("  {} No pending proposals", "✅".normal());
        print_retired_history(ctx);
        return Ok(());
    }

    println!();
    println!("  {} Pending Proposals", "📋".normal());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();

    for (id, cat, check, desc) in &rows {
        println!(
            "  {} #{} [{}] {}",
            "⚠️ ".normal(),
            id,
            cat.bright_cyan(),
            desc.bright_white()
        );
        println!("     {} {}", "check:".dimmed(), check.dimmed());
        println!(
            "     {} Apply this fix? (run: core integrity apply {})",
            "→".dimmed(),
            id
        );
        println!();
    }
    print_retired_history(ctx);
    Ok(())
}

/// INT-275: one line for what fix no longer shows, so a retired row is visible without being
/// pending. Superseded rows are counted together; each row keeps its own reason in the table.
fn print_retired_history(ctx: &AppContext) {
    let counts: Vec<(String, i64)> = match ctx.runtime.db.prepare(
        "SELECT CASE WHEN retired_reason LIKE 'superseded%' THEN 'superseded' ELSE retired_reason END AS why,
                COUNT(*) FROM pending_fixes WHERE retired_at IS NOT NULL GROUP BY why ORDER BY why",
    ) {
        Ok(mut stmt) => {
            let x: Vec<(String, i64)> = stmt
                .query_map([], |r| {
                    Ok((
                        r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                        r.get::<_, i64>(1)?,
                    ))
                })
                .map(|it| it.filter_map(|r| r.ok()).collect())
                .unwrap_or_default();
            x
        }
        Err(_) => return,
    };
    let total: i64 = counts.iter().map(|c| c.1).sum();
    if total == 0 {
        return;
    }
    let parts: Vec<String> = counts
        .iter()
        .map(|(why, n)| format!("{} {}", why, n))
        .collect();
    println!(
        "  {} {} retired, kept as history, not applied: {}",
        "·".dimmed(),
        total,
        parts.join(", ").dimmed()
    );
}

// ── Check Suite ───────────────────────────────────────────────────────────────
// Deterministic order: Intent → Registry → Jarvis → Autostart → DB → Docs → Shell → Temporal

pub fn build_check_suite() -> Vec<Box<dyn IntegrityCheck>> {
    vec![
        // Phase 1: Intent Ledger
        Box::new(checks::IntentStatusDirectoryCheck),
        Box::new(checks::IntentDuplicateIdCheck),
        Box::new(checks::IntentInProgressCountCheck),
        // Phase 2: Registry
        Box::new(checks::RegistryVersionDriftCheck),
        Box::new(checks::RegistryDeployableExistsCheck),
        // Phase 3: Jarvis
        // Phase 5: Database
        Box::new(checks::DbWalModeCheck),
        Box::new(checks::DbIntegrityCheck),
        // Phase 6: Documentation
        Box::new(checks::DocsCountConsistencyCheck),
        // Phase 7: Shell
        Box::new(checks::ShellStaleReferenceCheck),
        // Phase 8: Temporal
        Box::new(checks::TemporalDoctorFreshnessCheck),
        Box::new(checks::TemporalClockSanityCheck),
    ]
}

// ── Check Implementations ─────────────────────────────────────────────────────

pub mod checks {
    use super::*;

    // ── Intent Checks ─────────────────────────────────────────────────────────

    pub struct IntentStatusDirectoryCheck;
    impl IntegrityCheck for IntentStatusDirectoryCheck {
        fn name(&self) -> &'static str {
            "intent_status_directory"
        }
        fn category(&self) -> Category {
            Category::Intent
        }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let dirs = [
                ("complete", ctx.ctx.fpath("intents/complete")),
                ("future", ctx.ctx.fpath("intents/future")),
            ];
            for (dir_type, dir_path) in &dirs {
                let expected_status = match *dir_type {
                    "complete" => "complete",
                    "future" => "planned",
                    _ => continue,
                };
                if let Ok(entries) = std::fs::read_dir(dir_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        // Skip arch-era/ -- archived Arch intents are not active intents
                        if path.to_string_lossy().contains("arch-era") {
                            continue;
                        }
                        if path.extension().map(|e| e != "md").unwrap_or(true) {
                            continue;
                        }
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            // Check for status mismatch
                            // Only check frontmatter status field — exact line match
                            let status_line = frontmatter_status(&content);
                            let has_complete = status_line == "status: complete";
                            let has_planned = status_line == "status: planned";
                            let has_deferred = status_line == "status: deferred";
                            let has_inprog = status_line == "status: in-progress";

                            if *dir_type == "future" && has_complete {
                                let fname = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                let dest = ctx.ctx.fpath("intents/complete").join(&fname);
                                issues.push(IntegrityIssue::propose(
                                    Category::Intent,
                                    "intent_status_directory",
                                    &format!("{} has status: complete but lives in future/", fname),
                                    FixAction::MoveFile {
                                        from: path.clone(),
                                        to: dest,
                                    },
                                    3,
                                ));
                            }
                            // INT-332: detect in-progress intent in future/ directory
                            if *dir_type == "future" && has_inprog {
                                let fname = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                let dest = ctx.ctx.fpath("intents/in-progress").join(&fname);
                                issues.push(IntegrityIssue::propose(
                                    Category::Intent,
                                    "intent_status_directory",
                                    &format!("{} has status: in-progress but lives in future/ -- move to in-progress/", fname),
                                    FixAction::MoveFile {
                                        from: path.clone(),
                                        to: dest,
                                    },
                                    4,
                                ));
                            }
                            if *dir_type == "complete"
                                && (has_planned || has_deferred || has_inprog)
                            {
                                let fname = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                issues.push(IntegrityIssue::alert(
                                    Category::Intent,
                                    "intent_status_directory",
                                    &format!("{} in complete/ but status is not complete", fname),
                                    3,
                                ));
                            }
                            let _ = expected_status;
                        }
                    }
                }
            }
            issues
        }
    }

    pub struct IntentDuplicateIdCheck;
    impl IntegrityCheck for IntentDuplicateIdCheck {
        fn name(&self) -> &'static str {
            "intent_duplicate_id"
        }
        fn category(&self) -> Category {
            Category::Intent
        }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let mut seen: std::collections::HashMap<String, Vec<String>> =
                std::collections::HashMap::new();
            let intent_root = ctx.ctx.fpath("intents");
            // Only check main intent spaces -- decisions/incidents/philosophy/experiments have own numbering
            let subdirs = ["complete", "future", "in-progress"];
            for sub in &subdirs {
                let dir = intent_root.join(sub);
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if !name.ends_with(".md") {
                            continue;
                        }
                        let id = name.split('-').next().unwrap_or("").to_string();
                        if id.chars().all(|c| c.is_ascii_digit()) && !id.is_empty() {
                            seen.entry(id)
                                .or_default()
                                .push(format!("{}/{}", sub, name));
                        }
                    }
                }
            }
            for (id, paths) in &seen {
                if paths.len() > 1 {
                    issues.push(IntegrityIssue::alert(
                        Category::Intent,
                        "intent_duplicate_id",
                        &format!(
                            "INT-{} appears {} times: {}",
                            id,
                            paths.len(),
                            paths.join(", ")
                        ),
                        5,
                    ));
                }
            }
            issues
        }
    }

    pub struct IntentInProgressCountCheck;
    impl IntegrityCheck for IntentInProgressCountCheck {
        fn name(&self) -> &'static str {
            "intent_inprogress_count"
        }
        fn category(&self) -> Category {
            Category::Intent
        }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let future_dir = ctx.ctx.fpath("intents/future");
            let mut in_progress = vec![];
            if let Ok(entries) = std::fs::read_dir(&future_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e != "md").unwrap_or(true) {
                        continue;
                    }
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.contains("status: in-progress") {
                            in_progress.push(
                                path.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string(),
                            );
                        }
                    }
                }
            }
            if in_progress.len() > 7 {
                issues.push(IntegrityIssue::alert(
                    Category::Intent,
                    "intent_inprogress_count",
                    &format!(
                        "{} intents marked in-progress (expected ≤7): {}",
                        in_progress.len(),
                        in_progress.join(", ")
                    ),
                    2,
                ));
            }
            issues
        }
    }

    // ── Registry Checks ───────────────────────────────────────────────────────

    pub struct RegistryVersionDriftCheck;
    impl IntegrityCheck for RegistryVersionDriftCheck {
        fn name(&self) -> &'static str {
            "registry_version_drift"
        }
        fn category(&self) -> Category {
            Category::Registry
        }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let registry_path = ctx.ctx.fpath("registry/tools.toml");
            let registry = match std::fs::read_to_string(&registry_path) {
                Ok(r) => r,
                Err(_) => return issues,
            };

            // Parse registry name→version pairs
            let mut reg_versions: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            let mut current_name = String::new();
            for line in registry.lines() {
                let line = line.trim();
                if let Some(v) = line.strip_prefix("name = \"") {
                    current_name = v.trim_end_matches('"').to_string();
                } else if let Some(v) = line.strip_prefix("version = \"") {
                    let ver = v.trim_end_matches('"').to_string();
                    if !current_name.is_empty() {
                        reg_versions.insert(current_name.clone(), ver);
                    }
                }
            }

            // Each registered crate's Cargo.toml, found through the owner (INT-267 L2).
            // Registry name "core" is the engine, whose directory is named for its role.
            for (name, reg_ver) in &reg_versions {
                let dir = if name == "core" {
                    "engine"
                } else {
                    name.as_str()
                };
                let cargo_path = match zero_core::paths::crate_rel_dir_in(&ctx.core_root, dir) {
                    Some(rel) => ctx.core_root.join(rel).join("Cargo.toml"),
                    None => continue,
                };
                if let Ok(cargo) = std::fs::read_to_string(&cargo_path) {
                    if let Some(line) = cargo.lines().find(|l| l.starts_with("version = \"")) {
                        let cargo_ver = line
                            .trim_start_matches("version = \"")
                            .trim_end_matches('"');
                        if cargo_ver != reg_ver {
                            issues.push(IntegrityIssue::propose(
                                Category::Registry,
                                "registry_version_drift",
                                &format!("{}: registry={} cargo={}", name, reg_ver, cargo_ver),
                                FixAction::UpdateRegistryVersion {
                                    tool: name.clone(),
                                    version: cargo_ver.to_string(),
                                },
                                2,
                            ));
                        }
                    }
                }
            }
            issues
        }
    }

    pub struct RegistryDeployableExistsCheck;
    impl IntegrityCheck for RegistryDeployableExistsCheck {
        fn name(&self) -> &'static str {
            "registry_deployable_exists"
        }
        fn category(&self) -> Category {
            Category::Registry
        }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let registry_path = ctx.ctx.fpath("registry/tools.toml");
            // paths::bin_dir() is the single owner of where a deployed binary lives.
            let bin_dir = zero_core::paths::bin_dir();
            let registry = match std::fs::read_to_string(&registry_path) {
                Ok(r) => r,
                Err(_) => return issues,
            };

            let mut name = String::new();
            let mut deployable = false;
            let mut retired = false;

            for line in registry.lines() {
                let line = line.trim();
                if line == "[[tool]]" {
                    name.clear();
                    deployable = false;
                    retired = false;
                } else if let Some(v) = line.strip_prefix("name = \"") {
                    name = v.trim_end_matches('"').to_string();
                } else if line == "deployable = true" {
                    deployable = true;
                } else if line == "retired = true" {
                    retired = true;
                } else if line.starts_with("[[")
                    && !name.is_empty()
                    && deployable
                    && !retired
                    && !bin_dir.join(&name).exists()
                {
                    issues.push(IntegrityIssue::alert(
                        Category::Registry,
                        "registry_deployable_exists",
                        &format!(
                            "{} is deployable but not installed — run: deploy {}",
                            name, name
                        ),
                        4,
                    ));
                }
            }
            // Check last tool
            if deployable && !retired && !name.is_empty() && !bin_dir.join(&name).exists() {
                issues.push(IntegrityIssue::alert(
                    Category::Registry,
                    "registry_deployable_exists",
                    &format!("{} is deployable but not installed", name),
                    4,
                ));
            }
            issues
        }
    }

    // ── Jarvis Checks -- RETIRED: Jarvis replaced by Friday entirely
    // friday_readiness_log table no longer exists -- was causing 67% integrity drift

    // ── Database Checks ───────────────────────────────────────────────────────

    pub struct DbWalModeCheck;
    impl IntegrityCheck for DbWalModeCheck {
        fn name(&self) -> &'static str {
            "db_wal_mode"
        }
        fn category(&self) -> Category {
            Category::Database
        }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let mode: String = ctx
                .ctx
                .runtime
                .db
                .query_row("PRAGMA journal_mode", [], |r| r.get(0))
                .unwrap_or_default();

            if mode.to_lowercase() != "wal" {
                issues.push(IntegrityIssue::auto_fix(
                    Category::Database,
                    "db_wal_mode",
                    &format!("state.db journal_mode is '{}' — expected WAL", mode),
                    FixAction::InsertDbRow {
                        table: "pragma".to_string(),
                        sql: "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;".to_string(),
                    },
                    3,
                ));
            }
            issues
        }
    }

    pub struct DbIntegrityCheck;
    impl IntegrityCheck for DbIntegrityCheck {
        fn name(&self) -> &'static str {
            "db_integrity"
        }
        fn category(&self) -> Category {
            Category::Database
        }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let result: String = ctx
                .ctx
                .runtime
                .db
                .query_row("PRAGMA integrity_check", [], |r| r.get(0))
                .unwrap_or_else(|_| "error".to_string());

            if result != "ok" {
                issues.push(IntegrityIssue::alert(
                    Category::Database,
                    "db_integrity",
                    &format!(
                        "state.db integrity_check failed: {} — run: core db restore",
                        result
                    ),
                    5,
                ));
            }
            issues
        }
    }

    // ── Documentation Checks ──────────────────────────────────────────────────

    pub struct DocsCountConsistencyCheck;
    impl IntegrityCheck for DocsCountConsistencyCheck {
        fn name(&self) -> &'static str {
            "docs_count_consistency"
        }
        fn category(&self) -> Category {
            Category::Documentation
        }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let readme_path = ctx.core_root.join("README.md");
            let registry_path = ctx.ctx.fpath("registry/tools.toml");

            // Count tools in registry
            let registry_tools: usize = std::fs::read_to_string(&registry_path)
                .map(|r| {
                    r.lines()
                        .filter(|l| l.trim().starts_with("name = \""))
                        .count()
                })
                .unwrap_or(0);

            // Count tools mentioned in README -- only match specific patterns
            if let Ok(readme) = std::fs::read_to_string(&readme_path) {
                for line in readme.lines() {
                    // Only match lines like "50 tools" or "tools: 50" or "**50 tools**"
                    let lower = line.to_lowercase();
                    let is_tool_count_line = (lower.contains("tools deployed")
                        || lower.contains("tools installed")
                        || lower.contains("key tools")
                        || lower.contains("· tools:"))
                        && !lower.contains("install")
                        && !lower.contains("pipeline");
                    if is_tool_count_line {
                        if let Some(n) = line.split_whitespace().find_map(|w| {
                            w.trim_matches(|c: char| !c.is_ascii_digit())
                                .parse::<usize>()
                                .ok()
                        }) {
                            if n != registry_tools && (n as i64 - registry_tools as i64).abs() > 2 {
                                issues.push(IntegrityIssue::propose(
                                    Category::Documentation,
                                    "docs_count_consistency",
                                    &format!(
                                        "README shows {} tools but registry has {}",
                                        n, registry_tools
                                    ),
                                    FixAction::SyncDocs,
                                    1,
                                ));
                                break;
                            }
                        }
                    }
                }
            }
            issues
        }
    }

    // ── Shell Checks ──────────────────────────────────────────────────────────

    pub struct ShellStaleReferenceCheck;
    impl IntegrityCheck for ShellStaleReferenceCheck {
        fn name(&self) -> &'static str {
            "shell_stale_reference"
        }
        fn category(&self) -> Category {
            Category::Shell
        }
        fn run(&self, _ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let stale_refs = ["swaymsg", "hyprctl", "sway-", "hyprland"];
            let config_files = [
                std::env::var("HOME")
                    .map(|h| PathBuf::from(h).join(".zshrc"))
                    .unwrap_or_default(),
                zero_core::paths::shell_config(),
            ];

            for config_path in &config_files {
                if !config_path.exists() {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(config_path) {
                    for stale in &stale_refs {
                        if content.contains(stale) {
                            issues.push(IntegrityIssue::alert(
                                Category::Shell,
                                "shell_stale_reference",
                                &format!(
                                    "Stale reference to '{}' found in {}",
                                    stale,
                                    config_path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                ),
                                2,
                            ));
                        }
                    }
                }
            }
            issues
        }
    }

    // ── Temporal Checks ───────────────────────────────────────────────────────

    pub struct TemporalDoctorFreshnessCheck;
    impl IntegrityCheck for TemporalDoctorFreshnessCheck {
        fn name(&self) -> &'static str {
            "temporal_doctor_freshness"
        }
        fn category(&self) -> Category {
            Category::Temporal
        }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let last_run: Option<i64> = ctx
                .ctx
                .runtime
                .db
                .query_row(
                    "SELECT MAX(timestamp) FROM events WHERE domain='doctor' AND action='run'",
                    [],
                    |r| r.get(0),
                )
                .ok()
                .flatten();

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            if last_run.map(|t| now - t > 86400 * 7).unwrap_or(false) {
                issues.push(IntegrityIssue::alert(
                    Category::Temporal,
                    "temporal_doctor_freshness",
                    "No doctor run recorded in last 7 days — system may be unmonitored",
                    2,
                ));
            }
            issues
        }
    }

    pub struct TemporalClockSanityCheck;
    impl IntegrityCheck for TemporalClockSanityCheck {
        fn name(&self) -> &'static str {
            "temporal_clock_sanity"
        }
        fn category(&self) -> Category {
            Category::Temporal
        }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            // Check if any intent has a future completion date
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let future_completions: i64 = ctx
                .ctx
                .runtime
                .db
                .query_row(
                    "SELECT COUNT(*) FROM integrity_log WHERE detected_at > ?1",
                    rusqlite::params![now + 3600],
                    |r| r.get(0),
                )
                .unwrap_or(0);

            if future_completions > 0 {
                issues.push(IntegrityIssue::alert(
                    Category::Temporal,
                    "temporal_clock_sanity",
                    &format!(
                        "{} integrity log entries with future timestamps — possible clock drift",
                        future_completions
                    ),
                    4,
                ));
            }
            issues
        }
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Run a safe-only integrity scan for use in doctor
/// Returns (integrity_pct, auto_fixed, proposed, alerts)
#[allow(dead_code)]
pub fn quick_scan(ctx: &AppContext) -> (u32, usize, usize, usize) {
    if ensure_tables(ctx).is_err() {
        return (100, 0, 0, 0);
    }
    let ictx = IntegrityContext::new(ctx);
    let checks = build_check_suite();
    let result = run_pipeline(&ictx, &checks, true);
    (
        result.integrity_pct(),
        result.auto_fixed,
        result.proposed,
        result.alerts,
    )
}

pub fn cmd_apply(ctx: &AppContext, id: &str) -> CoreResult<()> {
    use colored::*;
    ensure_tables(ctx)?;
    let outcome = match id.parse::<i64>() {
        Ok(fix_id) => apply_row(ctx, fix_id).map_err(|why| format!("#{} {}", fix_id, why)),
        Err(_) => Err(format!("{} is not a proposal id", id)),
    };
    match outcome {
        Ok(()) => Ok(()),
        Err(why) => {
            // INT-275, INT-272: result first, said once, non-zero exit kept by the top level.
            println!("  {} Nothing applied: {}", "✗".bright_red(), why);
            Err(crate::errors::CoreError::Reported(format!(
                "nothing applied: {}",
                why
            )))
        }
    }
}

/// INT-275: carry out one pending row, or say why not. The row's own check runs again first, so a
/// proposal is applied only while a fresh run still finds it; then the row's stored data is
/// carried out, never a re-scan. Err is the reason, for the caller to print. Nothing is written
/// on any Err path.
fn apply_row(ctx: &AppContext, fix_id: i64) -> Result<(), String> {
    use colored::*;
    let db = &ctx.runtime.db;
    let row: Option<(String, String, String, Option<i64>, Option<String>)> = db
        .query_row(
            "SELECT check_name, action_type, description, applied_at, retired_reason
             FROM pending_fixes WHERE id = ?1",
            [fix_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .ok();
    let (check_name, action_type, description, applied, retired) = match row {
        None => return Err("is not a proposal".to_string()),
        Some(r) => r,
    };
    if applied.is_some() {
        return Err("was already applied".to_string());
    }
    if let Some(reason) = retired {
        return Err(format!("was retired: {}", reason));
    }

    // Re-run this row's check alone. A check no longer in the suite runs nothing, and the
    // reconcile retires the row as check removed.
    let own: Vec<Box<dyn IntegrityCheck>> = build_check_suite()
        .into_iter()
        .filter(|c| c.name() == check_name)
        .collect();
    let ictx = IntegrityContext::new(ctx);
    run_pipeline(&ictx, &own, true);
    let (retired, data): (Option<String>, String) = db
        .query_row(
            "SELECT retired_reason, action_data FROM pending_fixes WHERE id = ?1",
            [fix_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| format!("could not be read again: {}", e))?;
    if let Some(reason) = retired {
        return Err(format!("is no longer true: {}", reason));
    }

    println!();
    println!(
        "  {} Applying fix #{}: {}",
        "🔧".normal(),
        fix_id,
        description.bright_white()
    );
    println!(
        "  {} {} ({})",
        "check:".dimmed(),
        check_name.dimmed(),
        action_type.dimmed()
    );
    println!();
    let done = match (action_type.as_str(), data.split_once('\t')) {
        ("UpdateRegistryVersion", Some((tool, version))) => {
            println!(
                "  {} registry/tools.toml: {} version becomes {}",
                "→".dimmed(),
                tool.bright_white(),
                version.bright_white()
            );
            apply_safe_fix(
                &FixAction::UpdateRegistryVersion {
                    tool: tool.to_string(),
                    version: version.to_string(),
                },
                &ictx,
            )
        }
        ("MoveFile", Some((from, to))) => {
            let (from, to) = (PathBuf::from(from), PathBuf::from(to));
            if !from.exists() {
                return Err(format!("names {}, which is not there", from.display()));
            }
            if to.exists() {
                return Err(format!("would overwrite {}", to.display()));
            }
            println!(
                "  {} move {} -> {}",
                "→".dimmed(),
                from.display(),
                to.display()
            );
            std::fs::rename(&from, &to).is_ok()
        }
        _ => {
            return Err(format!(
                "has no stored {} fix to carry out; apply it by hand: {}",
                action_type, description
            ))
        }
    };
    if !done {
        return Err("could not be carried out; it stays pending".to_string());
    }
    db.execute(
        "UPDATE pending_fixes SET applied_at = ?1 WHERE id = ?2",
        rusqlite::params![now_ts(), fix_id],
    )
    .map_err(|e| format!("was carried out but could not be marked applied: {}", e))?;
    println!("  {} Fix applied successfully", "✅".normal());
    Ok(())
}
pub fn cmd_heal(ctx: &AppContext, dry_run: bool) -> CoreResult<()> {
    use colored::*;
    ensure_tables(ctx)?;
    println!();
    println!(
        "  {} Integrity Auto-Heal {}",
        "🌿".normal(),
        if dry_run {
            "(dry run)".bright_yellow().to_string()
        } else {
            "".to_string()
        }
    );
    println!("  {}", "─".repeat(56).dimmed());
    // Safe auto-fixes: dead aliases, orphaned state entries
    let mut healed = 0;
    let mut would_heal = 0;
    // Check 1: Dead aliases (alias points to non-existent command)
    let aliases: Vec<(i64, String, String)> = {
        let mut stmt = ctx
            .runtime
            .db
            .prepare("SELECT id, name, command FROM shell_aliases")?;
        let x = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    // Only flag aliases pointing to explicitly retired tools (from registry TOML)
    let retired_tools: Vec<String> = {
        let registry_path = ctx.fpath("registry/tools.toml");
        if let Ok(content) = std::fs::read_to_string(&registry_path) {
            content
                .lines()
                .filter(|l| l.trim() == "retired = true")
                .collect::<Vec<_>>()
                .iter()
                .enumerate()
                .filter_map(|(i, _)| {
                    // Find the name field before this retired = true line
                    let lines: Vec<&str> = content.lines().collect();
                    let abs_idx = content
                        .lines()
                        .enumerate()
                        .filter(|(_, l)| l.trim() == "retired = true")
                        .nth(i)?
                        .0;
                    // Look back for name =
                    lines[..abs_idx]
                        .iter()
                        .rev()
                        .find(|l| l.trim().starts_with("name ="))
                        .and_then(|l| l.split('"').nth(1))
                        .map(|s| s.to_string())
                })
                .collect()
        } else {
            vec![]
        }
    };
    let mut dead_aliases = vec![];
    for (id, name, cmd) in &aliases {
        let binary = cmd
            .split_whitespace()
            .next()
            .unwrap_or(cmd.as_str())
            .to_string();
        if retired_tools.contains(&binary) {
            dead_aliases.push((id, name.clone(), cmd.clone()));
        }
    }
    if !dead_aliases.is_empty() {
        println!(
            "  {} {} dead aliases found:",
            "▶".bright_cyan(),
            dead_aliases.len()
        );
        for (id, name, cmd) in &dead_aliases {
            println!(
                "    {} {} → {}",
                "·".dimmed(),
                name.bright_white(),
                cmd.dimmed()
            );
            if !dry_run {
                ctx.runtime.db.execute(
                    "DELETE FROM shell_aliases WHERE id = ?1",
                    rusqlite::params![id],
                )?;
                healed += 1;
            } else {
                would_heal += 1;
            }
        }
    }
    // INT-275: a fresh run first, so heal only carries out rows a run still finds.
    run_pipeline(&IntegrityContext::new(ctx), &build_check_suite(), true);
    // Check 2: Apply all pending safe fixes
    let pending: Vec<(i64, String)> = {
        let mut stmt = ctx.runtime.db.prepare(&format!(
            "SELECT id, check_name FROM pending_fixes WHERE {}",
            PENDING
        ))?;
        let x = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    if !pending.is_empty() {
        println!(
            "  {} {} pending integrity fixes:",
            "▶".bright_cyan(),
            pending.len()
        );
        for (id, check) in &pending {
            println!("    {} #{} {}", "·".dimmed(), id, check.bright_white());
            if !dry_run {
                match apply_row(ctx, *id) {
                    Ok(()) => healed += 1,
                    Err(why) => println!("      {} #{} not healed: {}", "·".dimmed(), id, why),
                }
            } else {
                would_heal += 1;
            }
        }
    }
    println!();
    if dry_run {
        println!(
            "  {} Would heal {} issues — run without --dry to apply",
            "→".bright_cyan(),
            would_heal
        );
    } else if healed > 0 {
        println!("  {} {} issues healed", "✅".normal(), healed);
    } else {
        println!("  {} Nothing to heal — Project 0 is clean", "✅".normal());
    }
    println!();
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    /// A stand-in repo, laid out where the owner says crates live (INT-267 L2): one tool crate
    /// and the engine, each a version ahead of the registry, over an in-memory database.
    fn stand_in(tag: &str) -> (AppContext, PathBuf) {
        let root =
            std::env::temp_dir().join(format!("integrity-int267-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let tool = root.join(zero_core::paths::CRATE_PARENTS[0]).join("alpha");
        let engine = root.join(zero_core::paths::SINGLE_CRATES[0]);
        for (dir, ver) in [(&tool, "2.0.0"), (&engine, "4.0.0")] {
            std::fs::create_dir_all(dir).unwrap();
            let cargo = format!("[package]\nversion = \"{}\"\n", ver);
            std::fs::write(dir.join("Cargo.toml"), cargo).unwrap();
        }
        let registry = root.join("zero/registry");
        std::fs::create_dir_all(&registry).unwrap();
        let tools = "[[tool]]\nname = \"alpha\"\nversion = \"1.0.0\"\n\n[[tool]]\nname = \"core\"\nversion = \"3.0.0\"\n";
        std::fs::write(registry.join("tools.toml"), tools).unwrap();
        let ctx = AppContext {
            runtime: crate::runtime::Runtime {
                root: root.clone(),
                logs: root.join("logs"),
                cache: root.join("cache"),
                snapshots: root.join("snapshots"),
                locks: root.join("locks"),
                db: rusqlite::Connection::open_in_memory().unwrap(),
            },
            capabilities: crate::capabilities::CapabilityContext::unprivileged(),
            home: root.display().to_string(),
            core_root: root.display().to_string(),
            source_root: root.join("zero").display().to_string(),
        };
        assert!(ensure_tables(&ctx).is_ok());
        (ctx, root)
    }

    fn drift_only() -> Vec<Box<dyn IntegrityCheck>> {
        vec![Box::new(checks::RegistryVersionDriftCheck)]
    }

    // ── INT-275: proposals are shown and applied only while a fresh run still finds them ──

    /// The rows a reader may show: not applied, and not retired once that column exists.
    /// Written to work on the table before and after INT-275, so the same tests go red then green.
    fn pending_ids(ctx: &AppContext) -> Vec<i64> {
        let has_retired = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('pending_fixes') WHERE name = 'retired_at'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n > 0)
            .unwrap_or(false);
        let sql = if has_retired {
            "SELECT id FROM pending_fixes WHERE applied_at IS NULL AND retired_at IS NULL ORDER BY id"
        } else {
            "SELECT id FROM pending_fixes WHERE applied_at IS NULL ORDER BY id"
        };
        let mut stmt = ctx.runtime.db.prepare(sql).unwrap();
        let ids: Vec<i64> = stmt
            .query_map([], |r| r.get::<_, i64>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        ids
    }

    /// Pending rows whose description matches, with their action_data.
    fn pending_data(ctx: &AppContext, like: &str) -> Vec<(i64, String)> {
        pending_ids(ctx)
            .into_iter()
            .filter_map(|id| {
                ctx.runtime
                    .db
                    .query_row(
                        "SELECT id, action_data FROM pending_fixes WHERE id = ?1 AND description LIKE ?2",
                        rusqlite::params![id, like],
                        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
                    )
                    .ok()
            })
            .collect()
    }

    /// Move the stand-in tool crate to a new cargo version.
    fn set_alpha(root: &std::path::Path, ver: &str) {
        let cargo = root
            .join(zero_core::paths::CRATE_PARENTS[0])
            .join("alpha")
            .join("Cargo.toml");
        std::fs::write(cargo, format!("[package]\nversion = \"{}\"\n", ver)).unwrap();
    }

    /// INT-275 G3: a row whose check is no longer in the suite is never pending again.
    #[test]
    fn a_row_of_a_removed_check_is_not_pending() {
        let (ctx, root) = stand_in("ghost");
        ctx.runtime
            .db
            .execute(
                "INSERT INTO pending_fixes (category, check_name, action_type, action_data, description, created_at)
                 VALUES ('autostart', 'autostart_retired_tool', 'UpdateFile', 'x', 'keyscan is retired', 1)",
                [],
            )
            .unwrap();
        let ghost = ctx.runtime.db.last_insert_rowid();
        run_pipeline(&IntegrityContext::new(&ctx), &drift_only(), true);
        let pending = pending_ids(&ctx);
        let _ = std::fs::remove_dir_all(&root);
        assert!(
            !pending.contains(&ghost),
            "a row of a removed check is still pending"
        );
    }

    /// INT-275 G4: the same tool drifting twice leaves one pending row, carrying the newer version.
    #[test]
    fn one_pending_row_per_identity() {
        let (ctx, root) = stand_in("identity");
        run_pipeline(&IntegrityContext::new(&ctx), &drift_only(), true);
        set_alpha(&root, "2.1.0");
        run_pipeline(&IntegrityContext::new(&ctx), &drift_only(), true);
        let alpha = pending_data(&ctx, "alpha:%");
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(
            alpha.len(),
            1,
            "one tool drifting twice left these pending: {:?}",
            alpha
        );
        assert_eq!(
            alpha[0].1, "alpha\t2.1.0",
            "the pending row does not carry the newer version"
        );
    }

    /// INT-275 G5: a drift corrected by hand retires resolved; it is not left pending or marked applied.
    #[test]
    fn a_resolved_drift_is_not_pending() {
        let (ctx, root) = stand_in("resolved");
        run_pipeline(&IntegrityContext::new(&ctx), &drift_only(), true);
        let id = pending_data(&ctx, "alpha:%").first().map(|r| r.0);
        let path = root.join("zero/registry/tools.toml");
        let corrected = std::fs::read_to_string(&path).unwrap().replace(
            "name = \"alpha\"\nversion = \"1.0.0\"",
            "name = \"alpha\"\nversion = \"2.0.0\"",
        );
        std::fs::write(&path, corrected).unwrap();
        run_pipeline(&IntegrityContext::new(&ctx), &drift_only(), true);
        let pending = pending_ids(&ctx);
        let applied: Option<i64> = id.and_then(|id| {
            ctx.runtime
                .db
                .query_row(
                    "SELECT applied_at FROM pending_fixes WHERE id = ?1",
                    [id],
                    |r| r.get::<_, Option<i64>>(0),
                )
                .ok()
                .flatten()
        });
        let _ = std::fs::remove_dir_all(&root);
        let id = id.expect("no drift row for alpha was recorded");
        assert!(
            !pending.contains(&id),
            "a drift corrected by hand is still pending"
        );
        assert_eq!(applied, None, "a resolved row was marked applied");
    }

    /// INT-275 G6: applying a superseded drift refuses as an error and writes nothing.
    #[test]
    fn applying_a_stale_drift_refuses_and_writes_nothing() {
        let (ctx, root) = stand_in("stale");
        run_pipeline(&IntegrityContext::new(&ctx), &drift_only(), true);
        let stale: Option<i64> = ctx
            .runtime
            .db
            .query_row(
                "SELECT id FROM pending_fixes WHERE action_data = ?1",
                ["alpha\t2.0.0"],
                |r| r.get::<_, i64>(0),
            )
            .ok();
        set_alpha(&root, "3.0.0");
        run_pipeline(&IntegrityContext::new(&ctx), &drift_only(), true);
        let path = root.join("zero/registry/tools.toml");
        let before = std::fs::read(&path).unwrap();
        let refused = stale.map(|id| cmd_apply(&ctx, &id.to_string()).is_err());
        let after = std::fs::read(&path).unwrap();
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(
            refused,
            Some(true),
            "applying a superseded drift did not refuse"
        );
        assert!(
            before == after,
            "applying a superseded drift wrote the registry"
        );
    }
    /// INT-275 G3: apply on a row of a removed check refuses, writes nothing, and retires it.
    #[test]
    fn applying_a_row_of_a_removed_check_refuses() {
        let (ctx, root) = stand_in("ghost-apply");
        ctx.runtime
            .db
            .execute(
                "INSERT INTO pending_fixes (category, check_name, action_type, action_data, description, created_at)
                 VALUES ('autostart', 'autostart_retired_tool', 'UpdateFile', 'x', 'keyscan is retired', 1)",
                [],
            )
            .unwrap();
        let ghost = ctx.runtime.db.last_insert_rowid();
        let refused = cmd_apply(&ctx, &ghost.to_string()).is_err();
        let reason: Option<String> = ctx
            .runtime
            .db
            .query_row(
                "SELECT retired_reason FROM pending_fixes WHERE id = ?1",
                [ghost],
                |r| r.get::<_, Option<String>>(0),
            )
            .ok()
            .flatten();
        let _ = std::fs::remove_dir_all(&root);
        assert!(refused, "apply on a row of a removed check did not refuse");
        assert_eq!(reason.as_deref(), Some("check removed"));
    }

    /// INT-275 G7: a move carries out the from and to stored on its row; only that file moves,
    /// even when another intent mentions the phrase. Red on the deployed build 2026-10-05: the
    /// old apply arm moved a planned stand-in intent whose body said status: complete.
    #[test]
    fn a_move_moves_only_the_file_its_row_names() {
        let (ctx, root) = stand_in("move");
        let future = ctx.fpath("intents/future");
        let complete = ctx.fpath("intents/complete");
        std::fs::create_dir_all(&future).unwrap();
        std::fs::create_dir_all(&complete).unwrap();
        let bystander = future.join("901-bystander.md");
        std::fs::write(
            &bystander,
            "---\nstatus: planned\n---\nThis body only mentions the phrase status: complete\n",
        )
        .unwrap();
        let done = future.join("902-done.md");
        std::fs::write(&done, "---\nstatus: complete\n---\n").unwrap();
        let suite: Vec<Box<dyn IntegrityCheck>> =
            vec![Box::new(checks::IntentStatusDirectoryCheck)];
        run_pipeline(&IntegrityContext::new(&ctx), &suite, true);
        let rows = pending_data(&ctx, "902-done.md%");
        let applied = rows
            .first()
            .map(|r| cmd_apply(&ctx, &r.0.to_string()).is_ok());
        let moved = complete.join("902-done.md").exists() && !done.exists();
        let stayed = bystander.exists();
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(
            rows.len(),
            1,
            "expected one pending move for 902-done.md: {:?}",
            rows
        );
        assert_eq!(applied, Some(true), "the move was not applied");
        assert!(moved, "the named file was not moved to complete/");
        assert!(stayed, "a file the row does not name was moved");
    }

    /// INT-275: an intent's status is its frontmatter line; a body that mentions one is not one.
    #[test]
    fn status_is_the_frontmatter_line_not_a_mention() {
        assert_eq!(
            frontmatter_status("---\nstatus: planned\n---\nsays status: complete\n"),
            "status: planned"
        );
        assert_eq!(frontmatter_status("no frontmatter at all\n"), "");
    }

    /// INT-267 L2: the check finds Cargo.toml where the owner says crates live. Seen red while
    /// it joined paths no crate has lived in since the tree moved under zero/.
    #[test]
    fn drift_check_reads_every_crate_through_the_owner() {
        let (ctx, root) = stand_in("reads");
        let found = checks::RegistryVersionDriftCheck.run(&IntegrityContext::new(&ctx));
        let _ = std::fs::remove_dir_all(&root);
        let names: Vec<&str> = found.iter().map(|i| i.description.as_str()).collect();
        assert_eq!(found.len(), 2, "both drifts found: {names:?}");
    }

    /// INT-267 batch 1b: a scan proposes drift and records every one; it never edits the registry.
    #[test]
    fn a_scan_records_every_drift_and_writes_nothing() {
        let (ctx, root) = stand_in("consent");
        let path = root.join("zero/registry/tools.toml");
        let before = std::fs::read_to_string(&path).unwrap();
        let result = run_pipeline(&IntegrityContext::new(&ctx), &drift_only(), true);
        let after = std::fs::read_to_string(&path).unwrap();
        let pending: i64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM pending_fixes WHERE applied_at IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(before, after, "a scan rewrote the registry");
        assert_eq!(result.auto_fixed, 0, "a scan applied a fix");
        assert_eq!(pending, 2, "every drift is recorded, not only the first");
    }

    /// INT-267 batch 1b: applying one drift proposal writes exactly that tool's version.
    #[test]
    fn applying_a_drift_writes_exactly_its_version() {
        let (ctx, root) = stand_in("apply");
        run_pipeline(&IntegrityContext::new(&ctx), &drift_only(), true);
        let id: Option<i64> = ctx
            .runtime
            .db
            .query_row(
                "SELECT id FROM pending_fixes WHERE description LIKE ?1 AND applied_at IS NULL",
                ["alpha:%"],
                |r| r.get(0),
            )
            .ok();
        let applied = id.map(|id| cmd_apply(&ctx, &id.to_string()).is_ok());
        let registry = std::fs::read_to_string(root.join("zero/registry/tools.toml")).unwrap();
        let done: Option<i64> = id.and_then(|id| {
            ctx.runtime
                .db
                .query_row(
                    "SELECT applied_at FROM pending_fixes WHERE id = ?1",
                    [id],
                    |r| r.get(0),
                )
                .ok()
                .flatten()
        });
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(applied, Some(true), "no drift proposal for alpha to apply");
        assert!(
            registry.contains("name = \"alpha\"\nversion = \"2.0.0\""),
            "alpha not updated:\n{registry}"
        );
        assert!(
            registry.contains("name = \"core\"\nversion = \"3.0.0\""),
            "another tool changed:\n{registry}"
        );
        assert!(done.is_some(), "the proposal is not marked applied");
    }
}
pub fn cmd_trend(ctx: &AppContext) -> CoreResult<()> {
    use colored::*;
    let _stmt = ctx.runtime.db.prepare(
        "SELECT integrity_score, timestamp FROM integrity_log
         GROUP BY date(timestamp, 'unixepoch')
         ORDER BY timestamp DESC LIMIT 14",
    );
    // Fallback: use quick_scan for current score
    let (score, _, _, _) = quick_scan(ctx);
    println!();
    println!("  {} Integrity Trend", "📈".normal());
    println!("  {}", "─".repeat(48).dimmed());
    println!(
        "  Current: {}%",
        if score >= 95 {
            score.to_string().bright_green()
        } else {
            score.to_string().bright_yellow()
        }
    );
    println!("  {} Integrity tracking builds over time", "→".dimmed());
    println!(
        "  {} Run core integrity run daily to build history",
        "→".dimmed()
    );
    println!();
    Ok(())
}
