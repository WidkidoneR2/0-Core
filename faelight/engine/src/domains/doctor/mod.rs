#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Status {
    Pass,
    Warn,
    Fail,
    Blocked,
    /// The check could not run (e.g. a dependency/bus/toolchain was unavailable).
    /// NOT a failure: excluded from the health denominator, rendered neutrally. (INT-148)
    Unknown,
}

/// INT-222: how much a problem in this check matters, declared per check rather than inferred.
/// The first three names are deliberately the same as RISK.toml's tiers. The fourth is NOT:
/// RISK.toml's  means nothing reads it at boot;  here means the check measures
/// something true but never renders a judgement, so it is excluded from the verdict entirely.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tier {
    Critical,
    System,
    User,
    Info,
}

#[derive(Debug)]
pub struct CheckResult {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub tier: Tier,
    pub message: String,
    pub fix: Option<String>,
}

/// INT-222: the single verdict. Derived from the checks, never stored, never weighted --
/// because a weight factor is subjective and a number built from one has to be defended
/// forever. This is the worst thing in the highest tier, and it needs no arithmetic.
///
/// GREEN means every judging check RAN and PASSED. Not "nothing shouted" -- an Unknown is
/// never green, because a tool that cannot express an undetermined outcome reports clean,
/// and that is the defect INT-192 exists to name.
///
/// Info-tier checks are excluded entirely: they measure truly but never judge, so they
/// cannot make the system unhealthy and must not be able to.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Verdict {
    Green,
    Amber,
    Red,
}

pub fn verdict(checks: &[CheckResult]) -> Verdict {
    let mut amber = false;
    for c in checks.iter().filter(|c| c.tier != Tier::Info) {
        match (c.tier, c.status) {
            (Tier::Critical, Status::Fail) => return Verdict::Red,
            (Tier::Critical, Status::Unknown) | (Tier::Critical, Status::Blocked) => {
                return Verdict::Red
            }
            (_, Status::Fail) | (_, Status::Warn) => amber = true,
            (_, Status::Unknown) | (_, Status::Blocked) => amber = true,
            (_, Status::Pass) => {}
        }
    }
    if amber {
        Verdict::Amber
    } else {
        Verdict::Green
    }
}

pub mod aliases;
pub mod bins;
mod checks;
mod cockpit;
pub mod entropy;
mod schema;

use checks::check_sandbox;
use checks::*;
pub(crate) use cockpit::render_cockpit;
use schema::check_schema_validation;

pub fn rebuild(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require(
        "doctor",
        &[crate::capabilities::Capability::FilesystemReadHome],
    )?;
    let core_root = &ctx.core_root;

    println!();
    println!(
        "{}",
        "  ╭─ 🔧 Deterministic Rebuild Plan ────────────────────".bright_cyan()
    );
    println!("  │  How to reconstruct this forest from first principles");
    println!("  │  Every step is traceable to an intent or decision.");
    println!(
        "{}",
        "  ├────────────────────────────────────────────────────".dimmed()
    );

    // ── Source 1: Registry ────────────────────────────────────────────────
    println!("  │");
    println!(
        "  │  {} {} → what should exist",
        "①".bright_white().bold(),
        "registry/tools.toml".bright_cyan()
    );

    let registry =
        std::fs::read_to_string(faelight_core::paths::tools_registry()).unwrap_or_default();
    let tools: Vec<&str> = registry
        .lines()
        .filter(|l| l.starts_with("name = "))
        .collect();
    println!(
        "  │    {} tools registered",
        tools.len().to_string().bright_white()
    );
    println!("  │    → git clone <remote>");
    println!("  │    → cargo build --release --workspace");
    println!(
        "  │    → deploy all {} binaries to ~/0-core/scripts/",
        tools.len()
    );

    // ── Source 2: Intents ─────────────────────────────────────────────────
    println!("  │");
    println!(
        "  │  {} {} → why decisions were made",
        "②".bright_white().bold(),
        "intents/complete/".bright_cyan()
    );

    let complete_dir = faelight_core::paths::intents_dir().join("complete");
    let intent_count = std::fs::read_dir(&complete_dir)
        .map(|d| d.count())
        .unwrap_or(0);
    println!(
        "  │    {} completed intents document every architectural decision",
        intent_count.to_string().bright_white()
    );
    println!("  │    → read intents/complete/ to understand WHY each tool exists");
    println!("  │    → cross-reference with CHANGELOG.md for version context");

    // ── Source 3: Interfaces ──────────────────────────────────────────────
    println!("  │");
    println!(
        "  │  {} {} → what the environment looks like",
        "③".bright_white().bold(),
        "config/".bright_cyan()
    );

    let dotfile_count = std::fs::read_dir(std::path::PathBuf::from(core_root).join("config"))
        .map(|d| d.flatten().filter(|e| e.path().is_dir()).count())
        .unwrap_or(0);
    println!(
        "  │    {} dotfile packages (deployed by home-manager)",
        dotfile_count.to_string().bright_white()
    );

    // ── Source 4: Schema ──────────────────────────────────────────────────
    println!("  │");
    println!(
        "  │  {} {} → what is valid",
        "④".bright_white().bold(),
        "schema/".bright_cyan()
    );

    let schema_dir = faelight_core::paths::schema_dir();
    let schema_count = std::fs::read_dir(&schema_dir)
        .map(|d| {
            d.flatten()
                .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
                .count()
        })
        .unwrap_or(0);
    println!(
        "  │    {} JSON schemas validate all registry files",
        schema_count.to_string().bright_white()
    );
    println!("  │    → run: core doctor run (validates schemas automatically)");

    // ── Source 5: Event Log ───────────────────────────────────────────────
    println!("  │");
    println!(
        "  │  {} {} → what happened",
        "⑤".bright_white().bold(),
        "runtime/events/".bright_cyan()
    );

    let events_dir = faelight_core::paths::events_dir();
    let event_files = std::fs::read_dir(&events_dir)
        .map(|d| {
            d.flatten()
                .filter(|e| e.file_name().to_string_lossy().ends_with(".jsonl"))
                .count()
        })
        .unwrap_or(0);
    println!(
        "  │    {} JSONL event log files",
        event_files.to_string().bright_white()
    );
    println!("  │    → replay with: core events replay --date <date>");
    println!("  │    → shows exactly what the forest was doing before any failure");

    // ── Reconstruction Steps ──────────────────────────────────────────────
    println!("  │");
    println!(
        "{}",
        "  ├────────────────────────────────────────────────────".dimmed()
    );
    println!("  │  {}", "Reconstruction Steps".bright_white().bold());
    println!("  │");
    println!("  │  {}  Install NixOS 26.05 (Yarara)", "①".bright_white());
    println!("  │");
    println!("  │  {}  Clone the forest", "②".bright_white());
    println!("  │     git clone https://github.com/WidkidoneR2/0-Core.git ~/0-core");
    println!("  │");
    println!("  │  {}  Build all tools", "③".bright_white());
    println!("  │     cd ~/0-core && cargo build --release --workspace");
    println!("  │     cp target/release/* scripts/");
    println!("  │");
    println!("  │  {}  Deploy interfaces", "④".bright_white());
    println!("  │     cd ~/0-core/config");
    println!("  │");
    println!("  │  {}  Validate", "⑤".bright_white());
    println!("  │     core doctor run  → should show 23/23 ✅");
    println!("  │     core bootstrap verify → confirms state consistency");
    println!("  │");
    println!("  │  {}  Review history", "⑥".bright_white());
    println!("  │     core narrative → understand the full story");
    println!("  │     core snapshot  → see the last known good state");
    println!("  │     intents/complete/ → understand every decision");
    println!("  │");
    println!(
        "{}",
        "  ├────────────────────────────────────────────────────".dimmed()
    );
    println!("  │  💡 NixOS reproduces state.");
    println!(
        "  │    Zero Core reproduces state {} reasoning.",
        "AND".bright_green().bold()
    );
    println!(
        "{}",
        "  ╰────────────────────────────────────────────────────".dimmed()
    );
    println!();

    // Emit event
    let payload = r#"{"actor":"core","result":"ok","detail":{"command":"doctor.rebuild"}}"#;
    // INT-251 v23: use canonical event bus
    let _ =
        crate::domains::friday::events::emit(ctx, "doctor", "health_check", payload, "core", None);

    Ok(())
}

pub fn run(ctx: &AppContext, _preflight: bool) -> CoreResult<()> {
    ctx.capabilities.require(
        "doctor",
        &[
            Capability::OrchestratorAccess,
            Capability::FilesystemReadHome,
        ],
    )?;
    let home = std::env::var("HOME").unwrap_or_default();
    let core_root = ctx.core_root.clone();

    let version = fs::read_to_string(faelight_core::paths::version_file())
        .unwrap_or_else(|_| "unknown".into())
        .trim()
        .to_string();

    let checks = all_checks(&core_root, &home);

    let scored: Vec<_> = checks.iter().collect();
    let total = scored.len() as u32;
    let passed = scored.iter().filter(|r| r.status == Status::Pass).count() as u32;
    let warnings = scored.iter().filter(|r| r.status == Status::Warn).count() as u32;
    let failed = scored.iter().filter(|r| r.status == Status::Fail).count() as u32;
    let unknown = scored
        .iter()
        .filter(|r| r.status == Status::Unknown)
        .count() as u32;
    let blocked = scored
        .iter()
        .filter(|r| r.status == Status::Blocked)
        .count() as u32;
    // INT-342: emit health_check_failed events for each failing check
    for check in scored.iter().filter(|r| r.status == Status::Fail) {
        let _ = crate::domains::friday::events::emit(
            ctx,
            "doctor",
            "health_check_failed",
            &format!(
                r#"{{"check_id":"{}","check_name":"{}","message":"{}"}}"#,
                check.id, check.name, check.message
            ),
            "core",
            None,
        );
    }
    // INT-148: Unknown (couldn't-run) and Blocked checks are excluded from the ratio --
    // "couldn't determine" and "blocked" are not failures and must not drag health down.
    let determinable = total.saturating_sub(unknown).saturating_sub(blocked);
    let health = if determinable > 0 {
        (passed * 100) / determinable
    } else {
        0
    };

    // Run integrity quick scan (safe auto-fixes only)
    let (integrity_pct, int_fixed, int_proposed, int_alerts) =
        crate::domains::integrity::quick_scan(ctx);

    render_cockpit(
        &checks,
        &version,
        health,
        passed,
        warnings,
        failed,
        unknown,
        integrity_pct,
    );

    // INT-208: Log health pattern to state.db
    {
        let db = &ctx.runtime.db;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let _ = db.execute_batch(
            "CREATE TABLE IF NOT EXISTS health_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                health_pct INTEGER NOT NULL,
                integrity_pct INTEGER NOT NULL,
                checks_passed INTEGER NOT NULL,
                checks_warned INTEGER NOT NULL,
                checks_failed INTEGER NOT NULL,
                trigger_type TEXT NOT NULL DEFAULT 'manual'
            );",
        );
        // INT-148: add checks_unknown to pre-existing tables (CREATE IF NOT EXISTS won't
        // alter an already-created table). Ignore error if the column already exists.
        let _ = db.execute(
            "ALTER TABLE health_patterns ADD COLUMN checks_unknown INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = db.execute(
            "INSERT INTO health_patterns (timestamp, health_pct, integrity_pct, checks_passed, checks_warned, checks_failed, checks_unknown, trigger_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![ts, health as i64, integrity_pct as i64, passed as i64, warnings as i64, failed as i64, unknown as i64, "manual"],
        );
    }

    // INT-019: forest-aware notifications -- health / integrity drops
    // ⚠️ THE NOTIFICATION FIRED ON A NUMBER THAT IS NOT A VERDICT. The percentage counts every
    // non-passing check equally and its denominator shifts as checks are added or excluded, so it
    // fell twice this month while the system became strictly MORE honest -- once when a lying check
    // was deleted, once when five checks about an absent package manager stopped pretending.
    // A threshold on a trend line is an alarm that fires on arithmetic.
    //
    // ⭐ verdict() ALREADY ANSWERS THE QUESTION THIS WANTED TO ASK: Red means a critical-tier
    // failure, or a critical check that could not run. Nothing else is worth interrupting for.
    if verdict(&checks) == Verdict::Red {
        crate::domains::notify::desktop(
            "Critical check failing",
            "A critical-tier check failed or could not run -- run: d",
            true,
        );
    }
    // ⚠️ CRITICAL URGENCY MEANS THE NOTIFICATION NEVER EXPIRES -- that is the specification, and
    // the notifier was honouring it correctly. An advisory sent at the same level as a dying
    // battery stacks up permanently and covers the screen, which is how a health signal becomes
    // something to dismiss without reading.
    // ⭐ Only the Red verdict above keeps critical urgency, because that is what Red MEANS.
    if integrity_pct < 80 {
        crate::domains::notify::desktop(
            "Forest integrity below 80%",
            &format!("Integrity is {}%", integrity_pct),
            false,
        );
    }
    // Show integrity summary if issues found
    if int_fixed > 0 || int_proposed > 0 || int_alerts > 0 {
        println!();
        if int_fixed > 0 {
            println!(
                "  {} Auto-fixed {} integrity issue(s)",
                "✅".green(),
                int_fixed
            );
        }
        if int_proposed > 0 {
            println!(
                "  {} {} integrity proposal(s) pending — run: core integrity fix",
                "⚠️ ".normal(),
                int_proposed
            );
        }
        if int_alerts > 0 {
            println!(
                "  {} {} integrity alert(s) require attention — run: core integrity run",
                "❌".normal(),
                int_alerts
            );
        }
    }

    // Core v5 Phase 2 — inline forecast after doctor run
    {
        let db = &ctx.runtime.db;
        let stmt = db.prepare(
            "SELECT payload, timestamp FROM events WHERE domain='doctor' ORDER BY id DESC LIMIT 10",
        );
        if let Ok(mut stmt) = stmt {
            let points_result = stmt.query_map([], |r| {
                let payload: Option<String> = r.get(0)?;
                let ts: i64 = r.get(1)?;
                let h = payload
                    .as_deref()
                    .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
                    .and_then(|v| v["detail"]["health"].as_i64())
                    .unwrap_or(health as i64);
                Ok((h, ts))
            });
            let points: Vec<(i64, i64)> = match points_result {
                Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
                Err(_) => vec![],
            };
            // points already bound above

            if points.len() >= 3 {
                // Compute trend slope
                let n = points.len() as f64;
                let sum_h: f64 = points.iter().map(|(h, _)| *h as f64).sum();
                let _avg_h = sum_h / n;
                let recent_avg: f64 =
                    points.iter().take(3).map(|(h, _)| *h as f64).sum::<f64>() / 3.0;
                let older_avg: f64 =
                    points.iter().skip(3).map(|(h, _)| *h as f64).sum::<f64>() / (n - 3.0).max(1.0);
                let trend = recent_avg - older_avg;

                let forecast_24h = (health as f64 + trend * 0.5).round() as i64;
                let forecast_7d = (health as f64 + trend * 2.0).round() as i64;
                let forecast_24h = forecast_24h.clamp(0, 100);
                let forecast_7d = forecast_7d.clamp(0, 100);

                let trend_icon = if trend > 1.0 {
                    "📈"
                } else if trend < -1.0 {
                    "📉"
                } else {
                    "➡️ "
                };
                let trend_str = if trend > 0.5 {
                    format!("+{:.1}", trend)
                } else if trend < -0.5 {
                    format!("{:.1}", trend)
                } else {
                    "stable".to_string()
                };

                // Add active intent context to forecast
                let _core_root = std::env::var("HOME").unwrap_or_default() + "/0-core";
                let future_dir = faelight_core::paths::intents_dir().join("future");
                let active_intents: Vec<String> = std::fs::read_dir(&future_dir)
                    .map(|entries| {
                        entries
                            .flatten()
                            .filter_map(|e| {
                                let p = e.path();
                                if p.extension().map(|x| x != "md").unwrap_or(true) {
                                    return None;
                                }
                                let content = std::fs::read_to_string(&p).ok()?;
                                if !content.contains("status: in-progress") {
                                    return None;
                                }
                                let fname = p.file_stem()?.to_string_lossy().to_string();
                                let id = fname.split('-').next()?;
                                Some(format!("INT-{}", id))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let context_str = if !active_intents.is_empty() {
                    format!(" ({} in progress)", active_intents.join(", "))
                } else {
                    String::new()
                };
                println!(
                    "{}  Forecast  24h: {}%  7d: {}%  trend: {}{}",
                    trend_icon, forecast_24h, forecast_7d, trend_str, context_str,
                );
                // Predictive health advisory
                let intent_count = active_intents.len();
                let advisory: Option<(&str, String)> = if health == 100
                    && trend.abs() <= 0.5
                    && intent_count == 0
                {
                    Some(("💚", "Forest is stable — no concerns".to_string()))
                } else if trend < -1.0 && intent_count > 0 {
                    Some(("💡", format!(
                        "Health dip during active development — expected pattern ({} intent{} in progress)",
                        intent_count, if intent_count == 1 { "" } else { "s" }
                    )))
                } else if trend < -1.0 && intent_count == 0 {
                    Some((
                        "⚠️ ",
                        "Declining health with no active work — investigate".to_string(),
                    ))
                } else if forecast_7d < 90 {
                    Some(("⚠️ ", format!(
                        "7-day forecast shows potential concern ({forecast_7d}%) — review active work"
                    )))
                } else if intent_count > 4 {
                    Some(("💡", format!(
                        "High intent load ({intent_count} active) — consider completing before opening a new intent"
                    )))
                } else if health < 100 && intent_count > 0 {
                    Some(("💡", format!(
                        "Below peak health — {} intent{} in progress, recovery expected on completion",
                        intent_count, if intent_count == 1 { "" } else { "s" }
                    )))
                } else {
                    None
                };
                if let Some((icon, msg)) = advisory {
                    println!("  {}  {}", icon, msg);
                }
            }
        }
    }
    // INT-207 L1 — Alignment score inline
    {
        let align_score: Option<f64> = ctx.runtime.db.query_row(
            "SELECT AVG(score) FROM alignment_checks WHERE checked_at > (strftime('%s','now') - 604800)",
            [], |r| r.get(0)
        ).ok().flatten();
        if let Some(score) = align_score {
            let pct = (score * 100.0) as i64;
            let colored = if pct >= 80 {
                format!("{}%", pct).bright_green()
            } else if pct >= 60 {
                format!("{}%", pct).bright_yellow()
            } else {
                format!("{}%", pct).bright_red()
            };
            println!("  {}  Alignment: {}", "🧭".normal(), colored);
            {
                let iv: String = ctx.runtime.db.query_row(
            "SELECT value FROM domain_state WHERE domain = 'core' AND key = 'intelligence_version'",
            [], |r| r.get(0)
        ).unwrap_or_else(|_| "v18".to_string());
                let iname: String = ctx.runtime.db.query_row(
            "SELECT value FROM domain_state WHERE domain = 'core' AND key = 'intelligence_name'",
            [], |r| r.get(0)
        ).unwrap_or_else(|_| "Synthesis Engine".to_string());
                println!(
                    "  {}  Intelligence: {} {} {}",
                    "🧠".normal(),
                    iv.bright_cyan(),
                    "—".dimmed(),
                    iname.dimmed()
                );
            }
        }
    }
    // INT-207 L1 — Engine coordination status inline
    {
        let pending: i64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM engine_upgrade_log WHERE migrated = 0",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let degraded: i64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM engine_registry WHERE status = 'degraded'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if pending > 0 || degraded > 0 {
            println!(
                "  {}  Engines: {} unsynced, {} degraded — run: core engines check",
                "⚠️ ".yellow(),
                pending,
                degraded
            );
        }
    }
    // INT-217 -- Friday voice: surface brief when thresholds met
    {
        let pats: i64 = ctx
            .runtime
            .db
            .query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0))
            .unwrap_or(0);
        let facts: i64 = ctx
            .runtime
            .db
            .query_row("SELECT COUNT(*) FROM friday_knowledge", [], |r| r.get(0))
            .unwrap_or(0);
        // Write daily journal entry (no-op if already written today)
        let _ = crate::domains::friday::write_journal_entry(ctx);
        // INT-237 -- Friday Easter Eggs: check milestones
        if let Some(celebration) = crate::domains::friday::check_milestones(ctx) {
            println!();
            println!(
                "  {} {}",
                "🌲 Friday milestone:".bright_yellow().bold(),
                celebration.bright_white()
            );
            println!();
        }
        match crate::domains::friday::get_voice(ctx) {
            Some((brief, confidence)) => {
                println!(
                    "  🌲  Friday: {} · {} patterns · {} facts",
                    "active".bright_green(),
                    pats.to_string().bright_cyan(),
                    facts.to_string().bright_white()
                );
                println!();
                if confidence >= 0.85 {
                    println!("  🌲 Friday: {}", brief.bright_white().bold());
                } else {
                    println!(
                        "  🌲 Friday: {} (not enough signal yet -- {:.0}% confidence)",
                        brief.dimmed(),
                        confidence * 100.0
                    );
                }
            }
            None => {
                let status: String = ctx
                    .runtime
                    .db
                    .query_row(
                        "SELECT status FROM engine_registry WHERE name = 'friday'",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap_or_else(|_| "dormant".to_string());
                if pats > 0 || facts > 0 {
                    println!(
                        "  🌲  Friday: {} · {} patterns · {} facts",
                        status.dimmed(),
                        pats.to_string().bright_cyan(),
                        facts.to_string().bright_white()
                    );
                } else {
                    println!("  🌲  Friday: {}", "dormant".dimmed());
                }
            }
        }
    }
    // INT-246 -- Friday usefulness score in d output
    {
        let _ = crate::domains::friday_arch::ensure_usefulness_table(ctx);
        let total: i64 = ctx
            .runtime
            .db
            .query_row("SELECT COUNT(*) FROM friday_usefulness", [], |r| r.get(0))
            .unwrap_or(0);
        if total > 0 {
            let accepted: i64 = ctx
                .runtime
                .db
                .query_row(
                    "SELECT COUNT(*) FROM friday_usefulness WHERE accepted = 1",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            let rate = accepted as f64 / total as f64 * 100.0;
            let rate_str = if rate >= 75.0 {
                format!("{:.0}% useful ({}/{})", rate, accepted, total)
                    .bright_green()
                    .to_string()
            } else if rate >= 50.0 {
                format!("{:.0}% useful ({}/{})", rate, accepted, total)
                    .bright_yellow()
                    .to_string()
            } else {
                format!("{:.0}% useful ({}/{})", rate, accepted, total)
                    .bright_red()
                    .to_string()
            };
            let calibration = if rate >= 75.0 {
                "trust well-calibrated"
            } else if rate >= 50.0 {
                "trust building"
            } else {
                "trust needs improvement"
            };
            println!("  🌲  Friday: {} · {}", rate_str, calibration.dimmed());
        }
    }
    // INT-216 -- Friday meta-interpretation brief
    {
        let patterns = crate::domains::friday_arch::detect_patterns(ctx).unwrap_or_default();
        let contradictions =
            crate::domains::friday_arch::detect_contradictions(ctx).unwrap_or_default();
        if !contradictions.is_empty() {
            for (a, b, desc) in contradictions.iter().take(1) {
                println!(
                    "  🧠  ⚠ {} ↔ {}: {}",
                    a.bright_red(),
                    b.bright_red(),
                    desc.chars().take(55).collect::<String>().bright_yellow()
                );
            }
        } else if !patterns.is_empty() {
            if let Some(p) = patterns.first() {
                println!(
                    "  🧠  {}",
                    p.chars().take(70).collect::<String>().bright_white()
                );
            }
        }
    }
    // INT-207 L1 — Emit health signal to engine_signals
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let _ = ctx.runtime.db.execute(
            "INSERT INTO engine_signals (source, signal_type, payload, weight, created_at)
             VALUES ('doctor', 'health', ?1, ?2, ?3)",
            rusqlite::params![
                format!("{{\"health\":{}}}", health),
                health as f64 / 100.0,
                now
            ],
        );
        let _ = ctx.runtime.db.execute(
            "UPDATE engine_registry SET last_active = ?1 WHERE name = 'core'",
            rusqlite::params![now],
        );
    }
    // Auto-store health prediction on every doctor run (INT-167 feedback loop)
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let expires_at = now + (24 * 3600); // verify in 24h on next doctor run
        let prediction = format!("health will be {}% on next doctor run", health);
        let _ = ctx.runtime.db.execute(
            "INSERT OR IGNORE INTO forest_predictions (kind, prediction, confidence, evidence, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["health", prediction, 85i64, "{}", now, expires_at],
        );
        // Auto-verify previous health predictions
        let prev_preds: Vec<(i64, i64)> = ctx.runtime.db.prepare(
            "SELECT id, CAST(SUBSTR(prediction, INSTR(prediction, 'be ')+3, INSTR(prediction, '% on')-INSTR(prediction, 'be ')-3) AS INTEGER)
             FROM forest_predictions
             WHERE kind='health' AND expires_at <= ?1
             AND id NOT IN (SELECT prediction_id FROM prediction_outcomes)"
        ).map(|mut s| {
            let rows: Vec<(i64, i64)> = s.query_map(rusqlite::params![now], |r| Ok((r.get(0)?, r.get(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                .unwrap_or_default();
            rows
        }).unwrap_or_default();
        for (pred_id, predicted_health) in prev_preds {
            let correct = (predicted_health - health as i64).abs() <= 5;
            let _ = ctx.runtime.db.execute(
                "INSERT INTO prediction_outcomes (prediction_id, actual, correct, verified_at)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    pred_id,
                    format!("{}%", health),
                    if correct { 1 } else { 0 },
                    now
                ],
            );
        }
    }
    // Write health score to cache for bar/prompt/palette
    let cache_dir =
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache/faelight");
    let _ = std::fs::create_dir_all(&cache_dir);
    let _ = std::fs::write(cache_dir.join("health-status"), format!("{}", health));

    // Event Ledger
    let writer = crate::runtime::EventWriter::new(&ctx.runtime.db);
    writer.write(
        "doctor",
        "run",
        "core doctor run",
        if failed == 0 { "ok" } else { "warn" },
        Some(&format!(
            r#"{{"health":{},"passed":{},"warnings":{},"failed":{}}}"#,
            health, passed, warnings, failed
        )),
    );

    Ok(())
}

pub fn aliases(_ctx: &AppContext, subcmd: Option<&str>) -> CoreResult<()> {
    aliases::run_full_audit(subcmd)
}

pub fn entropy(_ctx: &AppContext, baseline: bool, trends: bool, json: bool) -> CoreResult<()> {
    entropy::run(baseline, trends, json)
}

pub fn bins(_ctx: &AppContext, subcmd: Option<&str>) -> CoreResult<()> {
    bins::run(subcmd, false)
}

pub fn simulate(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require(
        "doctor",
        &[
            Capability::OrchestratorAccess,
            Capability::FilesystemReadHome,
        ],
    )?;

    let home = std::env::var("HOME").unwrap_or_default();
    let core_root = ctx.core_root.clone();

    // Read current cached health
    let cached: u32 =
        fs::read_to_string(PathBuf::from(&home).join(".cache/faelight/health-status"))
            .unwrap_or_default()
            .trim()
            .parse()
            .unwrap_or(0);

    // Run all checks silently
    let checks = all_checks(&core_root, &home);

    let total = checks.len() as u32;
    let passed = checks.iter().filter(|r| r.status == Status::Pass).count() as u32;
    let warnings = checks.iter().filter(|r| r.status == Status::Warn).count() as u32;
    let failed = checks.iter().filter(|r| r.status == Status::Fail).count() as u32;
    let predicted = if total > 0 { (passed * 100) / total } else { 0 };

    println!("{}", "🔮 core simulate doctor".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    // Show only changed or failing checks
    let mut changes = 0;
    for r in &checks {
        match r.status {
            Status::Fail => {
                println!("  {} {}", "✗".bright_red(), r.name.bright_white());
                println!("    {} {}", "→".dimmed(), r.message.bright_red());
                if let Some(ref fix) = r.fix {
                    println!("    {} {}", "fix:".yellow(), fix.dimmed());
                }
                changes += 1;
            }
            Status::Warn => {
                println!("  {} {}", "⚠".yellow(), r.name.bright_white());
                println!("    {} {}", "→".dimmed(), r.message.yellow());
                changes += 1;
            }
            _ => {}
        }
    }

    if changes == 0 {
        println!("  {}", "All checks passing — no issues found.".green());
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());

    // Health diff
    let delta = predicted as i32 - cached as i32;
    let delta_str = if delta > 0 {
        format!("+{}%", delta).green().to_string()
    } else if delta < 0 {
        format!("{}%", delta).bright_red().to_string()
    } else {
        "no change".dimmed().to_string()
    };

    let predicted_colored = if predicted >= 95 {
        format!("{}%", predicted).green().to_string()
    } else if predicted >= 80 {
        format!("{}%", predicted).yellow().to_string()
    } else {
        format!("{}%", predicted).bright_red().to_string()
    };

    println!();
    println!("  current health   {}%", cached.to_string().dimmed());
    println!("  predicted health {}  ({})", predicted_colored, delta_str);
    println!(
        "  checks           {}  passed  {}  warnings  {}  failed",
        passed.to_string().green(),
        warnings.to_string().yellow(),
        if failed > 0 {
            failed.to_string().bright_red()
        } else {
            failed.to_string().dimmed()
        }
    );
    println!();
    println!("  {} No changes made to system.", "ℹ".cyan());

    Ok(())
}

// ── Phase 6: Health Forecasting ───────────────────────────────────────────────

pub fn trend(ctx: &AppContext) -> CoreResult<()> {
    let conn = &ctx.runtime.db;

    let mut stmt = conn
        .prepare(
            "SELECT payload, timestamp FROM events WHERE domain='doctor' ORDER BY timestamp ASC",
        )
        .map_err(crate::errors::CoreError::Database)?;

    struct HealthPoint {
        health: i64,
        ts: i64,
    }

    let points: Vec<HealthPoint> = stmt
        .query_map([], |row| {
            let payload: String = row.get(0)?;
            let ts: i64 = row.get(1)?;
            Ok((payload, ts))
        })
        .map_err(crate::errors::CoreError::Database)?
        .filter_map(|r| r.ok())
        .filter_map(|(p, ts)| {
            let v: serde_json::Value = serde_json::from_str(&p).ok()?;
            let health = v["detail"]["health"].as_i64()?;
            Some(HealthPoint { health, ts })
        })
        .collect();

    if points.is_empty() {
        println!("  {} No health history found", "○".dimmed());
        return Ok(());
    }

    println!("{}", "🔬 Health Trend Analysis".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    // Summary stats
    let avg = points.iter().map(|p| p.health).sum::<i64>() / points.len() as i64;
    let min = points.iter().map(|p| p.health).min().unwrap_or(0);
    let max = points.iter().map(|p| p.health).max().unwrap_or(0);
    let current = points.last().map(|p| p.health).unwrap_or(0);
    let first = points.first().map(|p| p.health).unwrap_or(0);
    let drift = current - first;

    println!(
        "  {} {} readings over {} sessions",
        "📊".cyan(),
        points.len().to_string().bright_white(),
        points.len().to_string().dimmed(),
    );
    println!(
        "  {} Current:  {}%",
        "▶".dimmed(),
        current.to_string().bright_white()
    );
    println!(
        "  {} Average:  {}%",
        "▶".dimmed(),
        avg.to_string().bright_white()
    );
    println!("  {} Range:    {}% – {}%", "▶".dimmed(), min, max);

    let drift_str = if drift > 0 {
        format!("+{}% since first run", drift).green().to_string()
    } else if drift < 0 {
        format!("{}% since first run", drift).yellow().to_string()
    } else {
        "stable since first run".dimmed().to_string()
    };
    println!("  {} Drift:    {}", "▶".dimmed(), drift_str);

    // Sparkline — last 20 readings
    println!();
    println!("  {} Recent history (oldest → newest):", "📈".cyan());
    print!("    ");
    let recent: Vec<_> = points.iter().rev().take(20).rev().collect();
    for p in &recent {
        let ch = match p.health {
            h if h >= 95 => "█".bright_green(),
            h if h >= 85 => "▇".green(),
            h if h >= 75 => "▅".yellow(),
            _ => "▂".red(),
        };
        print!("{}", ch);
    }
    println!();
    println!(
        "    {}  {}  {}  {}",
        "█ 95%+".bright_green(),
        "▇ 85%+".green(),
        "▅ 75%+".yellow(),
        "▂ <75%".red(),
    );

    // Pattern detection
    println!();
    println!("  {} Pattern:", "🔍".cyan());
    let dips: usize = points
        .windows(2)
        .filter(|w| w[1].health < w[0].health)
        .count();
    let recoveries: usize = points
        .windows(2)
        .filter(|w| w[1].health > w[0].health)
        .count();

    if dips == 0 {
        println!("    ✅ No health dips detected — stable system");
    } else {
        println!(
            "    ⚠️  {} dip(s) detected, {} recovery(s)",
            dips, recoveries
        );
    }

    let at_95 = points.iter().filter(|p| p.health >= 95).count();
    let pct_healthy = (at_95 * 100) / points.len();
    println!("    📊 {}% of readings at 95%+ health", pct_healthy);

    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn forecast(ctx: &AppContext) -> CoreResult<()> {
    let conn = &ctx.runtime.db;

    let mut stmt = conn.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='doctor' ORDER BY timestamp DESC LIMIT 10"
    ).map_err(crate::errors::CoreError::Database)?;

    struct HealthPoint {
        health: i64,
        warnings: i64,
    }

    let points: Vec<HealthPoint> = stmt
        .query_map([], |row| {
            let payload: String = row.get(0)?;
            Ok(payload)
        })
        .map_err(crate::errors::CoreError::Database)?
        .filter_map(|r| r.ok())
        .filter_map(|p| {
            let v: serde_json::Value = serde_json::from_str(&p).ok()?;
            let health = v["detail"]["health"].as_i64()?;
            let warnings = v["detail"]["warnings"].as_i64().unwrap_or(0);
            Some(HealthPoint { health, warnings })
        })
        .collect();

    if points.is_empty() {
        println!("  {} No health history to forecast from", "○".dimmed());
        return Ok(());
    }

    println!("{}", "🔮 Health Forecast".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    let current = points.first().map(|p| p.health).unwrap_or(0);
    let avg_warnings = points.iter().map(|p| p.warnings).sum::<i64>() / points.len() as i64;

    // Simple linear trend from last 5 readings
    let recent: Vec<i64> = points.iter().take(5).map(|p| p.health).rev().collect();
    let trend_delta: i64 = if recent.len() >= 2 {
        let first = recent[0];
        let last = *recent.last().unwrap();
        (last - first) / (recent.len() as i64 - 1)
    } else {
        0
    };

    let predicted = (current + trend_delta).clamp(0, 100);

    println!(
        "  {} Current health:   {}%",
        "▶".dimmed(),
        current.to_string().bright_white()
    );
    println!(
        "  {} Recent trend:     {} per run",
        "▶".dimmed(),
        if trend_delta >= 0 {
            format!("+{}", trend_delta).green().to_string()
        } else {
            format!("{}", trend_delta).yellow().to_string()
        }
    );
    println!(
        "  {} Avg warnings:     {} per run",
        "▶".dimmed(),
        avg_warnings
    );
    println!();

    // Prediction
    let pred_str = match predicted {
        p if p >= 95 => format!("{}% ✅ Excellent", p).bright_green().to_string(),
        p if p >= 85 => format!("{}% ✓ Good", p).green().to_string(),
        p if p >= 75 => format!("{}% ⚠️  Watch", p).yellow().to_string(),
        p => format!("{}% ❌ Needs attention", p).red().to_string(),
    };
    println!("  {} Next run forecast: {}", "🔮".cyan(), pred_str);

    // Risk factors
    println!();
    println!("  {} Risk factors:", "⚠️".yellow());
    let mut risks = 0;

    if avg_warnings >= 2 {
        println!("    • Persistent warnings — {} avg per run", avg_warnings);
        risks += 1;
    }
    if trend_delta < 0 {
        println!(
            "    • Declining trend — health dropping {} per run",
            trend_delta
        );
        risks += 1;
    }
    if current < 95 {
        println!("    • Below optimal — currently {}% (target 95%+)", current);
        risks += 1;
    }

    if risks == 0 {
        println!("    ✅ No risk factors detected");
    }

    // Recommendations
    println!();
    println!("  {} Recommendations:", "💡".cyan());
    if current < 95 {
        println!("    • Run 'csd' to simulate what's causing the warning");
        println!("    • Run 'cwh' to see health trajectory");
    } else {
        println!("    • System on track — maintain current practices");
    }

    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn run_quick(ctx: &AppContext) -> CoreResult<()> {
    use colored::*;
    let home = std::env::var("HOME").unwrap_or_default();
    let core_root = ctx.core_root.clone();
    // Only run critical checks — fast subset
    // INT-222: DERIVED, not copied. The old hardcoded five contained git-is-dirty and
    // scripts-executable, while boot errors and disk space were absent entirely. Deleting a
    // check also meant editing two lists, and only the compiler noticed the second.
    let checks: Vec<CheckResult> = all_checks(&core_root, &home)
        .into_iter()
        .filter(|c| c.tier == Tier::Critical)
        .collect();
    let passed = checks.iter().filter(|c| c.status == Status::Pass).count();
    println!();
    for check in &checks {
        let icon = match check.status {
            Status::Pass => "✅".to_string(),
            Status::Warn => "⚠️ ".to_string(),
            Status::Fail | Status::Blocked => "❌".to_string(),
            Status::Unknown => "❔".to_string(),
        };
        println!(
            "  {}  {:<28} {}",
            icon,
            check.name.bright_white(),
            check.message.dimmed()
        );
    }
    println!();
    // INT-222: the same verdict the full panel uses. This block said CRITICAL where the
    // panel said DEGRADED for the identical condition, and neither counted Unknown at all --
    // so a check that could not run rendered the system green.
    let health = match verdict(&checks) {
        Verdict::Red => "DEGRADED",
        Verdict::Amber => "ADVISORY",
        Verdict::Green => "HEALTHY",
    };
    // INT-222: the colour comes from the SAME verdict as the word. These were separate
    // rules, so an Unknown at critical tier gave DEGRADED in bright green -- the word said
    // one thing and the colour said another, in the same string.
    let health_color = match verdict(&checks) {
        Verdict::Red => health.bright_red().to_string(),
        Verdict::Amber => health.bright_yellow().to_string(),
        Verdict::Green => health.bright_green().to_string(),
    };
    println!(
        "  {} {}/{} checks  {}",
        "⚡ Quick check:".bright_cyan(),
        passed,
        checks.len(),
        health_color
    );
    println!();
    Ok(())
}
pub fn run_history(ctx: &AppContext) -> CoreResult<()> {
    use colored::*;
    // Read health history from state.db
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT score, captured_at FROM horizon_snapshots ORDER BY captured_at DESC LIMIT 20",
    )?;
    let rows: Vec<(i64, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();
    println!();
    println!("  {} Doctor History", "📊".normal());
    println!("  {}", "─".repeat(48).dimmed());
    if rows.is_empty() {
        // Fallback: show from horizon_snapshots
        let mut stmt2 = ctx.runtime.db.prepare(
            "SELECT health_score, captured_at FROM horizon_snapshots ORDER BY captured_at DESC LIMIT 10"
        )?;
        let rows2: Vec<(i64, i64)> = stmt2
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        if rows2.is_empty() {
            println!(
                "  {} No health history yet — run d regularly to build history",
                "○".dimmed()
            );
        } else {
            for (score, ts) in &rows2 {
                let dt = chrono::DateTime::from_timestamp(*ts, 0)
                    .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_default();
                let bar = "█".repeat((*score / 10) as usize);
                let color = if *score >= 95 {
                    bar.bright_green()
                } else if *score >= 80 {
                    bar.bright_yellow()
                } else {
                    bar.bright_red()
                };
                println!(
                    "  {} {}%  {}",
                    dt.dimmed(),
                    score.to_string().bright_white(),
                    color
                );
            }
        }
    } else {
        for (score, ts) in &rows {
            let dt = chrono::DateTime::from_timestamp(*ts, 0)
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_default();
            let bar = "█".repeat((*score / 10) as usize);
            let color = if *score >= 95 {
                bar.bright_green()
            } else if *score >= 80 {
                bar.bright_yellow()
            } else {
                bar.bright_red()
            };
            println!(
                "  {} {}%  {}",
                dt.dimmed(),
                score.to_string().bright_white(),
                color
            );
        }
    }
    println!();
    Ok(())
}

/// INT-094: forest hygiene -- orphan accumulation surfaced from faelight-deadwood --summary.
/// Summary line format: TOTAL|aliases|baks|keybinds|registry|scripts|modules
fn check_deadwood(_core_root: &str) -> CheckResult {
    let out = std::process::Command::new("faelight-deadwood")
        .arg("--summary")
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let line = String::from_utf8_lossy(&o.stdout);
            let line = line.trim();
            let parts: Vec<&str> = line.split('|').collect();
            let total: usize = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            // High-confidence structural orphans (registry+modules) are the ones worth a Warn.
            let registry: usize = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
            let modules: usize = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
            let structural = registry + modules;
            if structural > 0 {
                CheckResult {
                    tier: Tier::User,
                    id: "deadwood".into(),
                    name: "Deadwood".into(),
                    status: Status::Warn,
                    message: format!(
                        "{} orphans flagged ({} structural: {} registry, {} modules)",
                        total, structural, registry, modules
                    ),
                    fix: Some(
                        "Run: faelight-deadwood (reports only -- you decide every cut)".into(),
                    ),
                }
            } else {
                CheckResult {
                    tier: Tier::User,
                    id: "deadwood".into(),
                    name: "Deadwood".into(),
                    status: Status::Pass,
                    message: format!(
                        "{} low-priority items (stale .baks); no structural orphans",
                        total
                    ),
                    fix: None,
                }
            }
        }
        _ => CheckResult {
            tier: Tier::User,
            id: "deadwood".into(),
            name: "Deadwood".into(),
            status: Status::Pass,
            message: "faelight-deadwood not installed (run after deploy)".into(),
            fix: None,
        },
    }
}

/// INT-222: this runs every check eagerly, so run_quick filters 32 results down to 3 --
/// measured at 0.49s against the old hardcoded list's 0.04s. Deliberately NOT restructured
/// into a lazy registry of Tier plus closure pairs: run_quick has exactly one caller, the
/// hand-typed doctor quick command, so half a second buys nothing on a path nobody waits on.
/// And moving the tier onto a registry entry would take it off CheckResult, which is what
/// makes the compiler force a classification at every construction site.
/// Revisit only if this lands somewhere hot -- a prompt, a shell start, a service.
fn all_checks(core_root: &str, home: &str) -> Vec<CheckResult> {
    vec![
        check_services(),
        check_broken_symlinks(core_root, home),
        check_binaries(),
        check_git(core_root),
        check_themes(core_root),
        check_rust_docs(core_root),
        check_intents(core_root),
        check_deadwood(core_root),
        check_faelight_config(home),
        check_keybinds(core_root, home),
        check_security_hardening(),
        check_security_audit(home),
        check_alias_coverage(),
        check_rust_toolchain(),
        check_disk_space(),
        check_tool_installation(),
        check_path_resilience(core_root),
        check_schema_validation(core_root),
        check_sandbox(core_root),
        check_boot_errors(),
        check_boot_time(),
        check_reboot_needed(),
        check_update_readiness(core_root),
        check_package_cache(),
        check_orphan_packages(),
        check_friday(core_root),
        check_network(),
        check_vm_state(),
        check_compositor(),
    ]
}
