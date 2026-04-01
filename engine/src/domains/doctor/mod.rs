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
}

#[derive(Debug)]
pub struct CheckResult {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub message: String,
    pub fix: Option<String>,
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
        "01-registry/tools.toml".bright_cyan()
    );

    let registry =
        std::fs::read_to_string(std::path::PathBuf::from(core_root).join("01-registry/tools.toml"))
            .unwrap_or_default();
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

    let complete_dir = std::path::PathBuf::from(core_root).join("intents/complete");
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
        "03-interfaces/stow/".bright_cyan()
    );

    let stow_dir = std::path::PathBuf::from(core_root).join("03-interfaces/stow");
    let stow_pkgs: Vec<String> = std::fs::read_dir(&stow_dir)
        .map(|d| {
            d.flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default();
    println!(
        "  │    {} stow packages to deploy",
        stow_pkgs.len().to_string().bright_white()
    );
    for pkg in &stow_pkgs {
        println!("  │      stow {}", pkg.dimmed());
    }

    // ── Source 4: Schema ──────────────────────────────────────────────────
    println!("  │");
    println!(
        "  │  {} {} → what is valid",
        "④".bright_white().bold(),
        "04-schema/".bright_cyan()
    );

    let schema_dir = std::path::PathBuf::from(core_root).join("04-schema");
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

    let events_dir = std::path::PathBuf::from(core_root).join("runtime/events");
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
    println!("  │  {}  Install Arch Linux (vanilla)", "①".bright_white());
    println!("  │     pacman -S niri greetd rustup git stow");
    println!("  │");
    println!("  │  {}  Clone the forest", "②".bright_white());
    println!("  │     git clone https://github.com/WidkidoneR2/0-Core.git ~/0-core");
    println!("  │");
    println!("  │  {}  Build all tools", "③".bright_white());
    println!("  │     cd ~/0-core && cargo build --release --workspace");
    println!("  │     cp target/release/* scripts/");
    println!("  │");
    println!("  │  {}  Deploy interfaces", "④".bright_white());
    println!("  │     cd ~/0-core/03-interfaces/stow");
    for pkg in stow_pkgs.iter().take(4) {
        println!("  │     stow {}", pkg.dimmed());
    }
    println!("  │     stow ... ({} packages total)", stow_pkgs.len());
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
    println!("  │  {} NixOS reproduces state.", "💡".to_string());
    println!(
        "  │    Faelight Forest reproduces state {} reasoning.",
        "AND".bright_green().bold()
    );
    println!(
        "{}",
        "  ╰────────────────────────────────────────────────────".dimmed()
    );
    println!();

    // Emit event
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let payload = r#"{"actor":"core","result":"ok","detail":{"command":"doctor.rebuild"}}"#;
    ctx.runtime.db.execute(
        "INSERT INTO events (domain, action, payload, timestamp) VALUES ('doctor', 'rebuild', ?1, ?2)",
        rusqlite::params![payload, ts],
    ).ok();
    crate::runtime::write_event_log("doctor", "rebuild", payload, ts);

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

    let version = fs::read_to_string(PathBuf::from(&core_root).join("00-meta/VERSION"))
        .unwrap_or_else(|_| "unknown".into())
        .trim()
        .to_string();

    let checks: Vec<CheckResult> = vec![
        check_stow(&core_root, &home),
        check_services(),
        check_broken_symlinks(&core_root, &home),
        check_yazi_plugins(&home),
        check_binaries(),
        check_git(&core_root),
        check_themes(&core_root),
        check_scripts(&core_root),
        check_dotmeta(),
        check_intents(&core_root),
        check_profiles(&core_root, &home),
        check_faelight_config(&home),
        check_keybinds(&core_root, &home),
        check_security_hardening(),
        check_security_audit(&home),
        check_alias_coverage(),
        check_rust_toolchain(),
        check_disk_space(),
        check_tool_installation(),
        check_path_resilience(&core_root),
        check_archaeology(&core_root),
        check_core_protect(&core_root),
        check_schema_validation(&core_root),
        check_sandbox(&core_root),
    ];

    // Exclude core_protection from health % — lock state is operational, not a health issue
    let scored: Vec<_> = checks.iter().filter(|r| r.id != "core_protect").collect();
    let total = scored.len() as u32;
    let passed = scored.iter().filter(|r| r.status == Status::Pass).count() as u32;
    let warnings = scored.iter().filter(|r| r.status == Status::Warn).count() as u32;
    let failed = scored.iter().filter(|r| r.status == Status::Fail).count() as u32;
    let health = if total > 0 { (passed * 100) / total } else { 0 };

    // Run integrity quick scan (safe auto-fixes only)
    let (integrity_pct, int_fixed, int_proposed, int_alerts) =
        crate::domains::integrity::quick_scan(ctx);

    render_cockpit(&checks, &version, health, passed, warnings, failed, integrity_pct);

    // Show integrity summary if issues found
    if int_fixed > 0 || int_proposed > 0 || int_alerts > 0 {
        println!();
        if int_fixed > 0 {
            println!("  {} Auto-fixed {} integrity issue(s)", "✅".green(), int_fixed);
        }
        if int_proposed > 0 {
            println!("  {} {} integrity proposal(s) pending — run: core integrity fix", "⚠️ ".normal(), int_proposed);
        }
        if int_alerts > 0 {
            println!("  {} {} integrity alert(s) require attention — run: core integrity run", "❌".normal(), int_alerts);
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
            let points = points;

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
                let forecast_24h = forecast_24h.max(0).min(100);
                let forecast_7d = forecast_7d.max(0).min(100);

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

                println!(
                    "{}  Forecast  24h: {}%  7d: {}%  trend: {}",
                    trend_icon, forecast_24h, forecast_7d, trend_str,
                );
            }
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
    let checks: Vec<CheckResult> = vec![
        check_stow(&core_root, &home),
        check_services(),
        check_broken_symlinks(&core_root, &home),
        check_yazi_plugins(&home),
        check_binaries(),
        check_git(&core_root),
        check_themes(&core_root),
        check_scripts(&core_root),
        check_dotmeta(),
        check_intents(&core_root),
        check_profiles(&core_root, &home),
        check_faelight_config(&home),
        check_keybinds(&core_root, &home),
        check_security_hardening(),
        check_security_audit(&home),
        check_alias_coverage(),
        check_rust_toolchain(),
        check_disk_space(),
        check_tool_installation(),
        check_path_resilience(&core_root),
        check_archaeology(&core_root),
        check_core_protect(&core_root),
        check_schema_validation(&core_root),
        check_sandbox(&core_root),
    ];

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
