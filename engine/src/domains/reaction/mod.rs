// INT-140 Core v10 — Reaction Domain
// Phase 1: Event Bus extension + Rule engine foundation
//
// Rules are evaluated against live state.db data.
// All reactions propose — never execute.
// Discipline enforced via cooldown table.

use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;

// ── DB init ──────────────────────────────────────────────────────────────────

pub fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(
        "CREATE TABLE IF NOT EXISTS reaction_log (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            rule_id     TEXT    NOT NULL,
            triggered_at INTEGER NOT NULL,
            message     TEXT,
            context     TEXT
        );
        CREATE TABLE IF NOT EXISTS reaction_cooldowns (
            rule_id     TEXT    PRIMARY KEY,
            last_fired  INTEGER NOT NULL
        );",
    )?;
    Ok(())
}

// ── Rule definitions ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ReactionRule {
    pub id:          &'static str,
    pub description: &'static str,
    pub priority:    u8,         // 1=high 2=medium 3=low
    pub cooldown_s:  i64,        // seconds before re-firing
}

pub fn rules() -> Vec<ReactionRule> {
    vec![
        ReactionRule {
            id:          "health.advisory",
            description: "Health below 95% in 2+ recent doctor runs",
            priority:    1,
            cooldown_s:  1800, // 30 min
        },
        ReactionRule {
            id:          "health.stale",
            description: "No doctor run in 24+ hours — health drift risk",
            priority:    1,
            cooldown_s:  3600, // 1 hour
        },
        ReactionRule {
            id:          "security.aging",
            description: "Security scan older than 7 days",
            priority:    2,
            cooldown_s:  86400, // 1 day
        },
        ReactionRule {
            id:          "checkpoint.stale",
            description: "No checkpoint taken in 7+ days",
            priority:    3,
            cooldown_s:  86400, // 1 day
        },
        ReactionRule {
            id:          "intent.overflow",
            description: "3+ intents in-progress simultaneously",
            priority:    2,
            cooldown_s:  7200, // 2 hours
        },
        ReactionRule {
            id:          "forecast.declining",
            description: "Health forecast trend below -1.5",
            priority:    1,
            cooldown_s:  3600, // 1 hour
        },
    ]
}

// ── Discipline: cooldown check ────────────────────────────────────────────────

fn is_on_cooldown(ctx: &AppContext, rule_id: &str, cooldown_s: i64) -> bool {
    let now = chrono::Local::now().timestamp();
    ctx.runtime.db
        .query_row(
            "SELECT last_fired FROM reaction_cooldowns WHERE rule_id = ?1",
            params![rule_id],
            |r| r.get::<_, i64>(0),
        )
        .ok()
        .map(|last| now - last < cooldown_s)
        .unwrap_or(false)
}

fn record_fired(ctx: &AppContext, rule_id: &str, message: &str, context: &str) -> CoreResult<()> {
    let now = chrono::Local::now().timestamp();
    ctx.runtime.db.execute(
        "INSERT OR REPLACE INTO reaction_cooldowns (rule_id, last_fired) VALUES (?1, ?2)",
        params![rule_id, now],
    )?;
    ctx.runtime.db.execute(
        "INSERT INTO reaction_log (rule_id, triggered_at, message, context)
         VALUES (?1, ?2, ?3, ?4)",
        params![rule_id, now, message, context],
    )?;
    Ok(())
}

// ── Rule evaluators ───────────────────────────────────────────────────────────

struct Reaction {
    rule_id: String,
    message: String,
    context: String,
    priority: u8,
    action:  String,
}

fn eval_health_advisory(ctx: &AppContext) -> Option<Reaction> {
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT payload FROM events WHERE domain='doctor' ORDER BY id DESC LIMIT 5"
    ).ok()?;
    let recent: Vec<i64> = stmt.query_map([], |r| {
        let p: Option<String> = r.get(0)?;
        Ok(p.as_deref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .and_then(|v| v["detail"]["health"].as_i64())
            .unwrap_or(95))
    }).ok()?
    .filter_map(|r| r.ok())
    .collect();

    let below = recent.iter().filter(|&&h| h < 95).count();
    if recent.len() >= 2 && below >= 2 {
        let avg = recent.iter().sum::<i64>() / recent.len() as i64;
        Some(Reaction {
            rule_id:  "health.advisory".into(),
            message:  format!("Health below 95% in {}/{} recent runs (avg: {}%)", below, recent.len(), avg),
            context:  format!("recent_health:{:?}", recent),
            priority: 1,
            action:   "Run: d  — investigate warnings".into(),
        })
    } else {
        None
    }
}

fn eval_health_stale(ctx: &AppContext) -> Option<Reaction> {
    let now = chrono::Local::now().timestamp();
    let last: Option<i64> = ctx.runtime.db.query_row(
        "SELECT MAX(timestamp) FROM events WHERE domain='doctor'",
        [], |r| r.get(0),
    ).ok().flatten();
    if let Some(ts) = last {
        let hours = (now - ts) / 3600;
        if hours > 24 {
            return Some(Reaction {
                rule_id:  "health.stale".into(),
                message:  format!("No doctor run in {}h — health drift risk elevated", hours),
                context:  format!("last_doctor:{}h_ago", hours),
                priority: 1,
                action:   "Run: d".into(),
            });
        }
    }
    None
}

fn eval_security_aging(ctx: &AppContext) -> Option<Reaction> {
    let now = chrono::Local::now().timestamp();
    let last: Option<i64> = ctx.runtime.db.query_row(
        "SELECT MAX(timestamp) FROM events WHERE domain='security'",
        [], |r| r.get(0),
    ).ok().flatten();
    if let Some(ts) = last {
        let days = (now - ts) / 86400;
        if days > 7 {
            return Some(Reaction {
                rule_id:  "security.aging".into(),
                message:  format!("Security scan {}d ago — findings may be stale", days),
                context:  format!("last_scan:{}d_ago", days),
                priority: 2,
                action:   "Run: core security scan".into(),
            });
        }
    }
    None
}

fn eval_checkpoint_stale(ctx: &AppContext) -> Option<Reaction> {
    let cp_dir = std::path::PathBuf::from(&ctx.core_root)
        .join("runtime/checkpoints");
    let latest = std::fs::read_dir(&cp_dir).ok()?.filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
        .filter_map(|e| e.metadata().ok()?.modified().ok())
        .max()?;
    let age = std::time::SystemTime::now()
        .duration_since(latest).unwrap_or_default().as_secs();
    let days = age / 86400;
    if days > 7 {
        Some(Reaction {
            rule_id:  "checkpoint.stale".into(),
            message:  format!("Last checkpoint {}d ago — consider a snapshot", days),
            context:  format!("checkpoint_age:{}d", days),
            priority: 3,
            action:   "Run: cpc <name>".into(),
        })
    } else {
        None
    }
}

fn eval_intent_overflow(ctx: &AppContext) -> Option<Reaction> {
    let intents_dir = std::path::PathBuf::from(&ctx.core_root).join("intents/future");
    let count = std::fs::read_dir(&intents_dir).ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            std::fs::read_to_string(e.path())
                .unwrap_or_default()
                .contains("status: in-progress")
        })
        .count();
    if count > 2 {
        Some(Reaction {
            rule_id:  "intent.overflow".into(),
            message:  format!("{} intents in-progress — focus narrows effectiveness", count),
            context:  format!("in_progress_count:{}", count),
            priority: 2,
            action:   "Run: intent list  — pick one to complete first".into(),
        })
    } else {
        None
    }
}

fn eval_forecast_declining(ctx: &AppContext) -> Option<Reaction> {
    // Read forecast from cache file
    let cache = std::path::PathBuf::from(&ctx.core_root)
        .join("runtime/cache/forecast.txt");
    let text = std::fs::read_to_string(&cache).ok()?;
    // Look for trend value
    let trend: f64 = text.lines()
        .find(|l| l.contains("trend:"))?
        .split(':')
        .nth(1)?
        .trim()
        .parse()
        .ok()?;
    if trend < -1.5 {
        Some(Reaction {
            rule_id:  "forecast.declining".into(),
            message:  format!("Forecast trend {:.1} — health declining before it hits", trend),
            context:  format!("trend:{}", trend),
            priority: 1,
            action:   "Run: core why suggest  — review what changed".into(),
        })
    } else {
        None
    }
}

// ── Evaluate all rules ────────────────────────────────────────────────────────

fn evaluate_all(ctx: &AppContext) -> Vec<Reaction> {
    let evaluators: Vec<fn(&AppContext) -> Option<Reaction>> = vec![
        eval_health_advisory,
        eval_health_stale,
        eval_security_aging,
        eval_checkpoint_stale,
        eval_intent_overflow,
        eval_forecast_declining,
    ];

    let all_rules = rules();
    let mut fired = vec![];

    for eval in evaluators {
        if let Some(reaction) = eval(ctx) {
            let rule = all_rules.iter().find(|r| r.id == reaction.rule_id.as_str());
            let cooldown = rule.map(|r| r.cooldown_s).unwrap_or(1800);
            if !is_on_cooldown(ctx, &reaction.rule_id, cooldown) {
                fired.push(reaction);
            }
        }
    }

    fired.sort_by_key(|r| r.priority);
    fired
}

// ── CLI commands ──────────────────────────────────────────────────────────────

pub fn list(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let all_rules = rules();
    let now = chrono::Local::now().timestamp();

    println!("{}", "🌲 Reaction Rules".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    for rule in &all_rules {
        let last_fired: Option<i64> = ctx.runtime.db.query_row(
            "SELECT last_fired FROM reaction_cooldowns WHERE rule_id = ?1",
            params![rule.id],
            |r| r.get(0),
        ).ok();

        let status = match last_fired {
            Some(ts) if (now - ts) < rule.cooldown_s => {
                let remaining = rule.cooldown_s - (now - ts);
                format!("🔵 cooldown {}m", remaining / 60).dimmed().to_string()
            }
            Some(ts) => {
                let ago = (now - ts) / 60;
                format!("✅ ready  (last fired {}m ago)", ago).green().to_string()
            }
            None => "✅ ready  (never fired)".green().to_string(),
        };

        let priority_icon = match rule.priority {
            1 => "🔴".to_string(),
            2 => "🟡".to_string(),
            _ => "🟢".to_string(),
        };

        println!(
            "  {} {}  {}",
            priority_icon,
            rule.id.bright_white(),
            status,
        );
        println!(
            "    {}  cooldown: {}m",
            rule.description.dimmed(),
            (rule.cooldown_s / 60).to_string().dimmed(),
        );
        println!();
    }

    println!("{}", "━".repeat(52).dimmed());
    println!("  {} core react run  — evaluate all rules now", "hint:".dimmed());
    Ok(())
}

pub fn run(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!("{}", "🌲 Reaction Engine — Evaluating".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    let fired = evaluate_all(ctx);

    if fired.is_empty() {
        println!("  {} No reactions triggered — forest is stable", "✅".green());
        println!("  All rules within bounds or on cooldown");
    } else {
        println!(
            "  {} reaction(s) triggered",
            fired.len().to_string().bright_white()
        );
        println!();

        for r in &fired {
            let icon = match r.priority {
                1 => "🔴",
                2 => "🟡",
                _ => "🟢",
            };
            println!("  {}  {}", icon, r.message.bright_white());
            println!("      {} {}", "→".dimmed(), r.action.cyan());
            println!();

            // Record to log + cooldown
            let _ = record_fired(ctx, &r.rule_id, &r.message, &r.context);
        }
    }

    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn history(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT id, rule_id, triggered_at, message FROM reaction_log
         ORDER BY triggered_at DESC LIMIT 20"
    )?;

    let rows: Vec<(i64, String, i64, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .filter_map(|r| r.ok())
        .collect();

    println!("{}", "🌲 Reaction History".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    if rows.is_empty() {
        println!("  {} No reactions fired yet", "○".dimmed());
        println!("  Run: core react run");
        return Ok(());
    }

    for (id, rule_id, ts, message) in &rows {
        let time = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|d| d.with_timezone(&chrono::Local).format("%m-%d %H:%M").to_string())
            .unwrap_or_default();
        println!(
            "  {} {:6}  {}  {}",
            id.to_string().dimmed(),
            time.dimmed(),
            rule_id.bright_white(),
            message.dimmed(),
        );
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn explain(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let id_num: i64 = id.trim_start_matches('#').parse().unwrap_or(0);

    let row: Option<(String, i64, String, String)> = ctx.runtime.db.query_row(
        "SELECT rule_id, triggered_at, message, context FROM reaction_log WHERE id = ?1",
        params![id_num],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    ).ok();

    println!("{}", format!("🌲 Reaction {} — Explain", id).cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    match row {
        None => {
            println!("  {} Reaction #{} not found", "✗".bright_red(), id);
            println!("  Run: core react history  — to see valid IDs");
        }
        Some((rule_id, ts, message, context)) => {
            let time = chrono::DateTime::from_timestamp(ts, 0)
                .map(|d| d.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default();

            let all_rules = rules();
            let rule = all_rules.iter().find(|r| r.id == rule_id.as_str());

            println!("  {}     #{}", "ID:".dimmed(), id);
            println!("  {}   {}", "Rule:".dimmed(), rule_id.bright_white());
            println!("  {}   {}", "Fired:".dimmed(), time.bright_white());
            println!();
            println!("  {}  {}", "Signal:".dimmed(), message.bright_white());
            println!();

            if let Some(r) = rule {
                println!("  {}  {}", "Rule def:".dimmed(), r.description.dimmed());
                println!("  {}  {}m", "Cooldown:".dimmed(), (r.cooldown_s / 60).to_string().dimmed());
                let priority_label = match r.priority {
                    1 => "HIGH 🔴",
                    2 => "MEDIUM 🟡",
                    _ => "LOW 🟢",
                };
                println!("  {} {}", "Priority:".dimmed(), priority_label);
            }
            println!();
            println!("  {}  {}", "Context:".dimmed(), context.dimmed());
        }
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn discipline(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let now = chrono::Local::now().timestamp();
    let all_rules = rules();

    println!("{}", "🌲 Reaction Discipline — Cooldown Status".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    let mut any_active = false;
    for rule in &all_rules {
        let last: Option<i64> = ctx.runtime.db.query_row(
            "SELECT last_fired FROM reaction_cooldowns WHERE rule_id = ?1",
            params![rule.id],
            |r| r.get(0),
        ).ok();

        if let Some(ts) = last {
            let elapsed = now - ts;
            if elapsed < rule.cooldown_s {
                any_active = true;
                let remaining = rule.cooldown_s - elapsed;
                println!(
                    "  🔵 {}  {}m remaining",
                    rule.id.bright_white(),
                    (remaining / 60).to_string().cyan(),
                );
            }
        }
    }

    if !any_active {
        println!("  {} No active cooldowns — all rules ready to fire", "✅".green());
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}
