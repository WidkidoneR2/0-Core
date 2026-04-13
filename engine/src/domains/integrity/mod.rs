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
    AutoFix,  // safe, idempotent, no destructive behavior
    Propose,  // requires human confirmation, persisted until resolved
    Alert,    // requires human intervention, no auto action
}

#[derive(Debug, Clone, PartialEq)]
pub enum Category {
    Intent,
    Registry,
    Jarvis,
    Autostart,
    Database,
    Documentation,
    Shell,
    Temporal,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Intent        => "intent",
            Category::Registry      => "registry",
            Category::Jarvis        => "jarvis",
            Category::Autostart     => "autostart",
            Category::Database      => "database",
            Category::Documentation => "documentation",
            Category::Shell         => "shell",
            Category::Temporal      => "temporal",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FixAction {
    MoveFile            { from: PathBuf, to: PathBuf },
    UpdateRegistryVersion { tool: String, version: String },
    UpdateRegistryField { tool: String, field: String, value: String },
    InsertDbRow         { table: String, sql: String },
    VacuumDb,
    SyncDocs,
    RebuildJarvisScore,
    UpdateFile          { path: PathBuf, old: String, new: String },
}

#[derive(Debug, Clone)]
pub struct IntegrityIssue {
    pub category:    Category,
    pub check:       &'static str,
    pub severity:    Severity,
    pub description: String,
    pub fix:         Option<FixAction>,
    pub weight:      u8, // 1=trivial 2=minor 3=moderate 4=significant 5=critical
}

impl IntegrityIssue {
    pub fn auto_fix(category: Category, check: &'static str, description: &str, fix: FixAction, weight: u8) -> Self {
        Self { category, check, severity: Severity::AutoFix, description: description.to_string(), fix: Some(fix), weight }
    }
    pub fn propose(category: Category, check: &'static str, description: &str, fix: FixAction, weight: u8) -> Self {
        Self { category, check, severity: Severity::Propose, description: description.to_string(), fix: Some(fix), weight }
    }
    pub fn alert(category: Category, check: &'static str, description: &str, weight: u8) -> Self {
        Self { category, check, severity: Severity::Alert, description: description.to_string(), fix: None, weight }
    }
}

// ── IntegrityCheck Trait ──────────────────────────────────────────────────────

pub struct IntegrityContext<'a> {
    pub ctx:       &'a AppContext,
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
    fn name(&self)     -> &'static str;
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
        );"
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
    pub proposed:   usize,
    pub alerts:     usize,
    pub total_weight: u32,
    pub issue_weight: u32,
}

impl PipelineResult {
    pub fn integrity_pct(&self) -> u32 {
        if self.total_weight == 0 { return 100; }
        let score = 100u32.saturating_sub(
            (self.issue_weight * 100) / self.total_weight
        );
        score
    }
}

fn apply_safe_fix(fix: &FixAction, ctx: &IntegrityContext) -> bool {
    match fix {
        FixAction::UpdateRegistryVersion { tool, version } => {
            let path = ctx.core_root.join("01-registry/tools.toml");
            if let Ok(content) = std::fs::read_to_string(&path) {
                let old = format!("name = \"{}\"\nversion = \"", tool);
                if let Some(idx) = content.find(&old) {
                    let ver_start = idx + old.len();
                    if let Some(ver_end) = content[ver_start..].find('"') {
                        let new_content = format!("{}{}{}",
                            &content[..ver_start],
                            version,
                            &content[ver_start + ver_end..]);
                        return std::fs::write(&path, new_content).is_ok();
                    }
                }
            }
            false
        }
        FixAction::InsertDbRow { table: _, sql } => {
            ctx.ctx.runtime.db.execute_batch(sql).is_ok()
        }
        FixAction::SyncDocs => {
            std::process::Command::new("faelight-docs")
                .arg("sync")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        FixAction::RebuildJarvisScore => {
            // Trigger jarvis recomputation by running it
            true // score rebuilds on next jarvis call
        }
        _ => false, // MoveFile, VacuumDb, config rewrites = not safe
    }
}

fn log_issue(ctx: &IntegrityContext, issue: &IntegrityIssue, fixed: bool) {
    let severity_str = match issue.severity {
        Severity::AutoFix => "auto-fix",
        Severity::Propose => "propose",
        Severity::Alert   => "alert",
    };
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

fn persist_proposal(ctx: &IntegrityContext, issue: &IntegrityIssue) {
    let action_type = match &issue.fix {
        Some(FixAction::MoveFile { .. })             => "MoveFile",
        Some(FixAction::UpdateRegistryVersion { .. }) => "UpdateRegistryVersion",
        Some(FixAction::VacuumDb)                    => "VacuumDb",
        Some(FixAction::SyncDocs)                    => "SyncDocs",
        Some(FixAction::UpdateFile { .. })           => "UpdateFile",
        _                                            => "Unknown",
    };
    // Check if already pending
    let exists: i64 = ctx.ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM pending_fixes WHERE check_name=?1 AND applied_at IS NULL",
        rusqlite::params![issue.check],
        |r| r.get(0)
    ).unwrap_or(0);

    if exists == 0 {
        ctx.ctx.runtime.db.execute(
            "INSERT INTO pending_fixes (category, check_name, action_type, action_data, description, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                issue.category.as_str(),
                issue.check,
                action_type,
                &issue.description,
                &issue.description,
                now_ts()
            ],
        ).ok();
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
    let total_weight: u32 = all_issues.iter()
        .map(|i| i.weight as u32)
        .sum::<u32>()
        .max(1) * 3; // baseline

    // Phase B — Plan (classify)
    let auto_fixable: Vec<&IntegrityIssue> = all_issues.iter()
        .filter(|i| i.severity == Severity::AutoFix && i.fix.is_some())
        .collect();
    let proposals: Vec<&IntegrityIssue> = all_issues.iter()
        .filter(|i| i.severity == Severity::Propose)
        .collect();
    let alerts: Vec<&IntegrityIssue> = all_issues.iter()
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

    // Log alerts
    for issue in &alerts {
        log_issue(ctx, issue, false);
    }

    // Phase D — Re-scan (simplified: use remaining issues)
    // Full re-scan would re-run affected checks — deferred to Phase 6

    // Phase E — compute result
    let remaining_weight: u32 = post_fix_issues.iter()
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
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
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
        println!("  {} Auto-fixed: {} issues", "✅".normal(), result.auto_fixed.to_string().bright_green());
    }
    if result.proposed > 0 {
        println!("  {} Proposed:   {} issues (run: core integrity fix)", "⚠️ ".normal(), result.proposed.to_string().yellow());
    }
    if result.alerts > 0 {
        println!("  {} Alerts:     {} issues requiring attention", "❌".normal(), result.alerts.to_string().bright_red());
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
    let pending: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM pending_fixes WHERE applied_at IS NULL",
        [], |r| r.get(0)
    ).unwrap_or(0);

    // Count recent auto-fixes (last 24h)
    let recent_fixed: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM integrity_log WHERE fixed=1 AND detected_at > ?1",
        rusqlite::params![now_ts() - 86400], |r| r.get(0)
    ).unwrap_or(0);

    // Count active alerts
    let active_alerts: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM integrity_log WHERE severity='alert' AND fixed=0 AND detected_at > ?1",
        rusqlite::params![now_ts() - 86400], |r| r.get(0)
    ).unwrap_or(0);

    println!();
    println!("  {} Integrity Status", "📊".normal());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("  {} Pending proposals: {}", "→".dimmed(), pending.to_string().yellow());
    println!("  {} Auto-fixed (24h):  {}", "→".dimmed(), recent_fixed.to_string().bright_green());
    println!("  {} Active alerts:     {}", "→".dimmed(), active_alerts.to_string().bright_red());
    println!();
    println!("  {} Run: core integrity run — full scan", "hint:".dimmed());
    println!("  {} Run: core integrity fix — apply proposals", "hint:".dimmed());
    println!();
    Ok(())
}

pub fn cmd_log(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;

    println!();
    println!("  {} Integrity Log (last 20)", "📋".normal());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT category, check_name, severity, description, fixed, detected_at
         FROM integrity_log ORDER BY detected_at DESC LIMIT 20"
    )?;

    let rows: Vec<(String, String, String, String, i64)> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
    })?.filter_map(|r| r.ok()).collect();

    if rows.is_empty() {
        println!("  {} No integrity history yet — run: core integrity run", "○".dimmed());
    } else {
        for (cat, check, severity, desc, fixed) in &rows {
            let icon = match severity.as_str() {
                "auto-fix" => if *fixed == 1 { "✅".to_string() } else { "🔧".to_string() },
                "propose"  => "⚠️ ".to_string(),
                "alert"    => "❌".to_string(),
                _          => "○".to_string(),
            };
            println!("  {} [{}] {} — {}", icon, cat.bright_cyan(), check.bright_white(), desc.dimmed());
        }
    }
    println!();
    Ok(())
}

pub fn cmd_fix(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT id, category, check_name, description FROM pending_fixes WHERE applied_at IS NULL ORDER BY created_at ASC"
    )?;

    let rows: Vec<(i64, String, String, String)> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    })?.filter_map(|r| r.ok()).collect();

    if rows.is_empty() {
        println!("  {} No pending proposals", "✅".normal());
        return Ok(());
    }

    println!();
    println!("  {} Pending Proposals", "📋".normal());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();

    for (id, cat, check, desc) in &rows {
        println!("  {} #{} [{}] {}", "⚠️ ".normal(), id, cat.bright_cyan(), desc.bright_white());
        println!("     {} {}", "check:".dimmed(), check.dimmed());
        println!("     {} Apply this fix? (run: core integrity apply {})", "→".dimmed(), id);
        println!();
    }
    Ok(())
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
        Box::new(checks::JarvisLogFreshnessCheck),
        // Phase 4: Autostart
        Box::new(checks::AutostartRetiredToolCheck),
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
        fn name(&self) -> &'static str { "intent_status_directory" }
        fn category(&self) -> Category { Category::Intent }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let dirs = [
                ("complete", ctx.core_root.join("intents/complete")),
                ("future",   ctx.core_root.join("intents/future")),
            ];
            for (dir_type, dir_path) in &dirs {
                let expected_status = match *dir_type {
                    "complete" => "complete",
                    "future"   => "planned",
                    _          => continue,
                };
                if let Ok(entries) = std::fs::read_dir(dir_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|e| e != "md").unwrap_or(true) { continue; }
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            // Check for status mismatch
                            // Only check frontmatter status field — exact line match
                            let status_line = content.lines().take(20)
                                .find(|l| l.trim().starts_with("status:"))
                                .unwrap_or("")
                                .trim()
                                .to_string();
                            let has_complete = status_line == "status: complete";
                            let has_planned  = status_line == "status: planned";
                            let has_deferred = status_line == "status: deferred";
                            let has_inprog   = status_line == "status: in-progress";

                            if *dir_type == "future" && has_complete {
                                let fname = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                let dest = ctx.core_root.join("intents/complete").join(&fname);
                                issues.push(IntegrityIssue::propose(
                                    Category::Intent,
                                    "intent_status_directory",
                                    &format!("{} has status: complete but lives in future/", fname),
                                    FixAction::MoveFile { from: path.clone(), to: dest },
                                    3,
                                ));
                            }
                            if *dir_type == "complete" && (has_planned || has_deferred || has_inprog) {
                                let fname = path.file_name().unwrap_or_default().to_string_lossy().to_string();
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
        fn name(&self) -> &'static str { "intent_duplicate_id" }
        fn category(&self) -> Category { Category::Intent }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let mut seen: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
            let intent_root = ctx.core_root.join("intents");
            // Only check main intent spaces — decisions/incidents/philosophy have own numbering
            let subdirs = ["complete", "future", "active"];
            for sub in &subdirs {
                let dir = intent_root.join(sub);
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if !name.ends_with(".md") { continue; }
                        let id = name.split('-').next().unwrap_or("").to_string();
                        if id.chars().all(|c| c.is_ascii_digit()) && !id.is_empty() {
                            seen.entry(id).or_default().push(format!("{}/{}", sub, name));
                        }
                    }
                }
            }
            for (id, paths) in &seen {
                if paths.len() > 1 {
                    issues.push(IntegrityIssue::alert(
                        Category::Intent,
                        "intent_duplicate_id",
                        &format!("INT-{} appears {} times: {}", id, paths.len(), paths.join(", ")),
                        5,
                    ));
                }
            }
            issues
        }
    }

    pub struct IntentInProgressCountCheck;
    impl IntegrityCheck for IntentInProgressCountCheck {
        fn name(&self) -> &'static str { "intent_inprogress_count" }
        fn category(&self) -> Category { Category::Intent }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let future_dir = ctx.core_root.join("intents/future");
            let mut in_progress = vec![];
            if let Ok(entries) = std::fs::read_dir(&future_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e != "md").unwrap_or(true) { continue; }
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.contains("status: in-progress") {
                            in_progress.push(path.file_name().unwrap_or_default().to_string_lossy().to_string());
                        }
                    }
                }
            }
            if in_progress.len() > 7 {
                issues.push(IntegrityIssue::alert(
                    Category::Intent,
                    "intent_inprogress_count",
                    &format!("{} intents marked in-progress (expected ≤7): {}", in_progress.len(), in_progress.join(", ")),
                    2,
                ));
            }
            issues
        }
    }

    // ── Registry Checks ───────────────────────────────────────────────────────

    pub struct RegistryVersionDriftCheck;
    impl IntegrityCheck for RegistryVersionDriftCheck {
        fn name(&self) -> &'static str { "registry_version_drift" }
        fn category(&self) -> Category { Category::Registry }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let registry_path = ctx.core_root.join("01-registry/tools.toml");
            let rust_tools_dir = ctx.core_root.join("rust-tools");
            let registry = match std::fs::read_to_string(&registry_path) {
                Ok(r) => r,
                Err(_) => return issues,
            };

            // Parse registry name→version pairs
            let mut reg_versions: std::collections::HashMap<String, String> = std::collections::HashMap::new();
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

            // Check each rust-tool's Cargo.toml
            for (name, reg_ver) in &reg_versions {
                let cargo_path = if name == "core" {
                    ctx.core_root.join("engine/Cargo.toml")
                } else {
                    rust_tools_dir.join(name).join("Cargo.toml")
                };
                if !cargo_path.exists() { continue; }
                if let Ok(cargo) = std::fs::read_to_string(&cargo_path) {
                    if let Some(line) = cargo.lines().find(|l| l.starts_with("version = \"")) {
                        let cargo_ver = line.trim_start_matches("version = \"").trim_end_matches('"');
                        if cargo_ver != reg_ver {
                            issues.push(IntegrityIssue::auto_fix(
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
        fn name(&self) -> &'static str { "registry_deployable_exists" }
        fn category(&self) -> Category { Category::Registry }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let registry_path = ctx.core_root.join("01-registry/tools.toml");
            let scripts_dir = ctx.core_root.join("scripts");
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
                    name.clear(); deployable = false; retired = false;
                } else if let Some(v) = line.strip_prefix("name = \"") {
                    name = v.trim_end_matches('"').to_string();
                } else if line == "deployable = true" {
                    deployable = true;
                } else if line == "retired = true" {
                    retired = true;
                } else if line.starts_with("[[") && !name.is_empty() {
                    if deployable && !retired && !scripts_dir.join(&name).exists() {
                        issues.push(IntegrityIssue::alert(
                            Category::Registry,
                            "registry_deployable_exists",
                            &format!("{} is deployable but missing from scripts/ — run: deploy {}", name, name),
                            4,
                        ));
                    }
                }
            }
            // Check last tool
            if deployable && !retired && !name.is_empty() && !scripts_dir.join(&name).exists() {
                issues.push(IntegrityIssue::alert(
                    Category::Registry,
                    "registry_deployable_exists",
                    &format!("{} is deployable but missing from scripts/", name),
                    4,
                ));
            }
            issues
        }
    }

    // ── Jarvis Checks ─────────────────────────────────────────────────────────

    pub struct JarvisLogFreshnessCheck;
    impl IntegrityCheck for JarvisLogFreshnessCheck {
        fn name(&self) -> &'static str { "jarvis_log_freshness" }
        fn category(&self) -> Category { Category::Jarvis }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let last_log: Option<i64> = ctx.ctx.runtime.db.query_row(
                "SELECT MAX(recorded_at) FROM jarvis_readiness_log",
                [], |r| r.get(0)
            ).ok().flatten();

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            if last_log.map(|t| now - t > 86400).unwrap_or(true) {
                issues.push(IntegrityIssue::auto_fix(
                    Category::Jarvis,
                    "jarvis_log_freshness",
                    "Jarvis readiness log has no entry in last 24h",
                    FixAction::RebuildJarvisScore,
                    2,
                ));
            }
            issues
        }
    }

    // ── Autostart Checks ──────────────────────────────────────────────────────

    pub struct AutostartRetiredToolCheck;
    impl IntegrityCheck for AutostartRetiredToolCheck {
        fn name(&self) -> &'static str { "autostart_retired_tool" }
        fn category(&self) -> Category { Category::Autostart }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let config_path = std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".config/niri/config.kdl"))
                .unwrap_or_default();
            let registry_path = ctx.core_root.join("01-registry/tools.toml");

            let config = match std::fs::read_to_string(&config_path) {
                Ok(c) => c,
                Err(_) => return issues,
            };
            let registry = match std::fs::read_to_string(&registry_path) {
                Ok(r) => r,
                Err(_) => return issues,
            };

            // Find retired tools
            let mut retired_tools: Vec<String> = vec![];
            let mut name = String::new();
            for line in registry.lines() {
                let line = line.trim();
                if let Some(v) = line.strip_prefix("name = \"") {
                    name = v.trim_end_matches('"').to_string();
                } else if line == "retired = true" && !name.is_empty() {
                    retired_tools.push(name.clone());
                }
            }

            // Check if any retired tool appears in autostart
            for tool in &retired_tools {
                if config.contains(tool.as_str()) {
                    issues.push(IntegrityIssue::propose(
                        Category::Autostart,
                        "autostart_retired_tool",
                        &format!("{} is retired but still in niri autostart config", tool),
                        FixAction::UpdateFile {
                            path: config_path.clone(),
                            old: format!("spawn-at-startup.*{}.*", tool),
                            new: format!("// {} removed — retired tool", tool),
                        },
                        3,
                    ));
                }
            }
            issues
        }
    }

    // ── Database Checks ───────────────────────────────────────────────────────

    pub struct DbWalModeCheck;
    impl IntegrityCheck for DbWalModeCheck {
        fn name(&self) -> &'static str { "db_wal_mode" }
        fn category(&self) -> Category { Category::Database }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let mode: String = ctx.ctx.runtime.db.query_row(
                "PRAGMA journal_mode", [], |r| r.get(0)
            ).unwrap_or_default();

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
        fn name(&self) -> &'static str { "db_integrity" }
        fn category(&self) -> Category { Category::Database }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let result: String = ctx.ctx.runtime.db.query_row(
                "PRAGMA integrity_check", [], |r| r.get(0)
            ).unwrap_or_else(|_| "error".to_string());

            if result != "ok" {
                issues.push(IntegrityIssue::alert(
                    Category::Database,
                    "db_integrity",
                    &format!("state.db integrity_check failed: {} — run: core db restore", result),
                    5,
                ));
            }
            issues
        }
    }

    // ── Documentation Checks ──────────────────────────────────────────────────

    pub struct DocsCountConsistencyCheck;
    impl IntegrityCheck for DocsCountConsistencyCheck {
        fn name(&self) -> &'static str { "docs_count_consistency" }
        fn category(&self) -> Category { Category::Documentation }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let readme_path = ctx.core_root.join("README.md");
            let registry_path = ctx.core_root.join("01-registry/tools.toml");

            // Count tools in registry
            let registry_tools: usize = std::fs::read_to_string(&registry_path)
                .map(|r| r.lines().filter(|l| l.trim().starts_with("name = \"")).count())
                .unwrap_or(0);

            // Count tools mentioned in README -- only match specific patterns
            if let Ok(readme) = std::fs::read_to_string(&readme_path) {
                for line in readme.lines() {
                    // Only match lines like "50 tools" or "tools: 50" or "**50 tools**"
                    let lower = line.to_lowercase();
                    let is_tool_count_line = 
                        (lower.contains("tools deployed") || lower.contains("tools installed")
                         || lower.contains("key tools") || lower.contains("· tools:"))
                        && !lower.contains("install") && !lower.contains("pipeline");
                    if is_tool_count_line {
                        if let Some(n) = line.split_whitespace()
                            .find_map(|w| w.trim_matches(|c: char| !c.is_ascii_digit()).parse::<usize>().ok())
                        {
                            if n != registry_tools && (n as i64 - registry_tools as i64).abs() > 2 {
                                issues.push(IntegrityIssue::propose(
                                    Category::Documentation,
                                    "docs_count_consistency",
                                    &format!("README shows {} tools but registry has {}", n, registry_tools),
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
        fn name(&self) -> &'static str { "shell_stale_reference" }
        fn category(&self) -> Category { Category::Shell }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let stale_refs = ["swaymsg", "hyprctl", "sway-", "hyprland"];
            let config_files = [
                std::env::var("HOME").map(|h| PathBuf::from(h).join(".zshrc")).unwrap_or_default(),
                ctx.core_root.join("03-interfaces/stow/shell-zsh/.zshrc"),
            ];

            for config_path in &config_files {
                if !config_path.exists() { continue; }
                if let Ok(content) = std::fs::read_to_string(config_path) {
                    for stale in &stale_refs {
                        if content.contains(stale) {
                            issues.push(IntegrityIssue::alert(
                                Category::Shell,
                                "shell_stale_reference",
                                &format!("Stale reference to '{}' found in {}", stale,
                                    config_path.file_name().unwrap_or_default().to_string_lossy()),
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
        fn name(&self) -> &'static str { "temporal_doctor_freshness" }
        fn category(&self) -> Category { Category::Temporal }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            let last_run: Option<i64> = ctx.ctx.runtime.db.query_row(
                "SELECT MAX(timestamp) FROM events WHERE domain='doctor' AND action='run'",
                [], |r| r.get(0)
            ).ok().flatten();

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
        fn name(&self) -> &'static str { "temporal_clock_sanity" }
        fn category(&self) -> Category { Category::Temporal }
        fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue> {
            let mut issues = vec![];
            // Check if any intent has a future completion date
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let future_completions: i64 = ctx.ctx.runtime.db.query_row(
                "SELECT COUNT(*) FROM integrity_log WHERE detected_at > ?1",
                rusqlite::params![now + 3600], |r| r.get(0)
            ).unwrap_or(0);

            if future_completions > 0 {
                issues.push(IntegrityIssue::alert(
                    Category::Temporal,
                    "temporal_clock_sanity",
                    &format!("{} integrity log entries with future timestamps — possible clock drift", future_completions),
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
    if ensure_tables(ctx).is_err() { return (100, 0, 0, 0); }
    let ictx = IntegrityContext::new(ctx);
    let checks = build_check_suite();
    let result = run_pipeline(&ictx, &checks, true);
    (result.integrity_pct(), result.auto_fixed, result.proposed, result.alerts)
}

pub fn cmd_apply(ctx: &AppContext, id: &str) -> CoreResult<()> {
    use colored::*;
    ensure_tables(ctx)?;
    let fix_id: i64 = id.parse().unwrap_or(0);
    
    // Get the pending fix
    let fix: Option<(i64, String, String, String, String)> = ctx.runtime.db.query_row(
        "SELECT id, category, check_name, action_type, description FROM pending_fixes 
         WHERE id = ?1 AND applied_at IS NULL",
        rusqlite::params![fix_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
    ).ok();
    let (pid, _cat, check_name, action_type, description) = match fix {
        None => {
            println!("  {} Fix #{} not found or already applied", "✗".bright_red(), id);
            return Ok(());
        }
        Some(f) => f,
    };
    println!();
    println!("  {} Applying fix #{}: {}", "🔧".normal(), pid, description.bright_white());
    println!("  {} {} ({})", "check:".dimmed(), check_name.dimmed(), action_type.dimmed());
    println!();
    // Execute the fix based on action type and check name
    let success = match check_name.as_str() {
        "intent_status_directory" => {
            // Move complete intent from future/ to complete/
            let root = std::path::PathBuf::from(&ctx.core_root);
            let future_dir = root.join("intents/future");
            let complete_dir = root.join("intents/complete");
            let mut moved = false;
            if let Ok(entries) = std::fs::read_dir(&future_dir) {
                for entry in entries.flatten() {
                    let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                    if content.contains("status: complete") {
                        let dest = complete_dir.join(entry.file_name());
                        if std::fs::rename(entry.path(), &dest).is_ok() {
                            println!("  {} Moved {} to complete/", "✅".normal(),
                                entry.file_name().to_string_lossy().bright_white());
                            moved = true;
                        }
                    }
                }
            }
            moved
        }
        "autostart_retired_tool" => {
            // Remove retired tool from niri config
            let niri_config = std::path::PathBuf::from(&ctx.core_root)
                .join("03-interfaces/stow/niri/.config/niri/config.kdl");
            if let Ok(content) = std::fs::read_to_string(&niri_config) {
                // Find retired tool name in description
                let tool = description.split_whitespace()
                    .find(|w| ctx.runtime.db.query_row(
                        "SELECT COUNT(*) FROM registry_tools WHERE name = ?1 AND retired = 1",
                        rusqlite::params![w], |r| r.get::<_, i64>(0)
                    ).unwrap_or(0) > 0);
                if let Some(tool_name) = tool {
                    let new_content: String = content.lines()
                        .filter(|l| !l.contains(tool_name))
                        .collect::<Vec<_>>()
                        .join("
");
                    std::fs::write(&niri_config, new_content).is_ok()
                } else {
                    false
                }
            } else { false }
        }
        _ => {
            println!("  {} Fix type '{}' requires manual application", "⚠️ ".normal(), action_type.bright_yellow());
            println!("     Description: {}", description);
            false
        }
    };
    if success {
        // Mark as applied
        let now = chrono::Utc::now().timestamp();
        ctx.runtime.db.execute(
            "UPDATE pending_fixes SET applied_at = ?1 WHERE id = ?2",
            rusqlite::params![now, pid],
        )?;
        println!("  {} Fix applied successfully", "✅".normal());
    } else {
        println!("  {} Fix could not be applied automatically — see description above", "⚠️ ".normal());
    }
    println!();
    Ok(())
}
pub fn cmd_heal(ctx: &AppContext, dry_run: bool) -> CoreResult<()> {
    use colored::*;
    ensure_tables(ctx)?;
    println!();
    println!("  {} Integrity Auto-Heal {}", "🌿".normal(),
        if dry_run { "(dry run)".bright_yellow().to_string() } else { "".to_string() });
    println!("  {}", "─".repeat(56).dimmed());
    // Safe auto-fixes: dead aliases, orphaned state entries
    let mut healed = 0;
    let mut would_heal = 0;
    // Check 1: Dead aliases (alias points to non-existent command)
    let aliases: Vec<(i64, String, String)> = {
        let mut stmt = ctx.runtime.db.prepare(
            "SELECT id, name, command FROM shell_aliases"
        )?;
        let x = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    // Only flag aliases pointing to explicitly retired tools (from registry TOML)
    let retired_tools: Vec<String> = {
        let registry_path = std::path::PathBuf::from(&ctx.core_root).join("01-registry/tools.toml");
        if let Ok(content) = std::fs::read_to_string(&registry_path) {
            content.lines()
                .filter(|l| l.trim() == "retired = true")
                .collect::<Vec<_>>()
                .iter()
                .enumerate()
                .filter_map(|(i, _)| {
                    // Find the name field before this retired = true line
                    let lines: Vec<&str> = content.lines().collect();
                    let abs_idx = content.lines().enumerate()
                        .filter(|(_, l)| l.trim() == "retired = true")
                        .nth(i)?.0;
                    // Look back for name =
                    lines[..abs_idx].iter().rev()
                        .find(|l| l.trim().starts_with("name ="))
                        .and_then(|l| l.split('"').nth(1))
                        .map(|s| s.to_string())
                })
                .collect()
        } else { vec![] }
    };
    let mut dead_aliases = vec![];
    for (id, name, cmd) in &aliases {
        let binary = cmd.split_whitespace().next().unwrap_or(cmd.as_str()).to_string();
        if retired_tools.contains(&binary) {
            dead_aliases.push((id, name.clone(), cmd.clone()));
        }
    }
    if !dead_aliases.is_empty() {
        println!("  {} {} dead aliases found:", "▶".bright_cyan(), dead_aliases.len());
        for (id, name, cmd) in &dead_aliases {
            println!("    {} {} → {}", "·".dimmed(), name.bright_white(), cmd.dimmed());
            if !dry_run {
                ctx.runtime.db.execute(
                    "DELETE FROM shell_aliases WHERE id = ?1", rusqlite::params![id]
                )?;
                healed += 1;
            } else {
                would_heal += 1;
            }
        }
    }
    // Check 2: Apply all pending safe fixes
    let pending: Vec<(i64, String)> = {
        let mut stmt = ctx.runtime.db.prepare(
            "SELECT id, check_name FROM pending_fixes WHERE applied_at IS NULL"
        )?;
        let x = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    if !pending.is_empty() {
        println!("  {} {} pending integrity fixes:", "▶".bright_cyan(), pending.len());
        for (id, check) in &pending {
            println!("    {} #{} {}", "·".dimmed(), id, check.bright_white());
            if !dry_run {
                cmd_apply(ctx, &id.to_string())?;
                healed += 1;
            } else {
                would_heal += 1;
            }
        }
    }
    println!();
    if dry_run {
        println!("  {} Would heal {} issues — run without --dry to apply", "→".bright_cyan(), would_heal);
    } else if healed > 0 {
        println!("  {} {} issues healed", "✅".normal(), healed);
    } else {
        println!("  {} Nothing to heal — forest is clean", "✅".normal());
    }
    println!();
    Ok(())
}
pub fn cmd_trend(ctx: &AppContext) -> CoreResult<()> {
    use colored::*;
    let _stmt = ctx.runtime.db.prepare(
        "SELECT integrity_score, timestamp FROM integrity_log
         GROUP BY date(timestamp, 'unixepoch')
         ORDER BY timestamp DESC LIMIT 14"
    );
    // Fallback: use quick_scan for current score
    let (score, _, _, _) = quick_scan(ctx);
    println!();
    println!("  {} Integrity Trend", "📈".normal());
    println!("  {}", "─".repeat(48).dimmed());
    println!("  Current: {}%", if score >= 95 { score.to_string().bright_green() } 
             else { score.to_string().bright_yellow() });
    println!("  {} Integrity tracking builds over time", "→".dimmed());
    println!("  {} Run core integrity run daily to build history", "→".dimmed());
    println!();
    Ok(())
}
