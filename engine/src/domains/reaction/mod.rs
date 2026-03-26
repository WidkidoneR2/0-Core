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

#[derive(Debug, Clone, PartialEq)]
pub enum RuleClass {
    Stability,   // fires even when health < 80%
    Maintenance, // fires when health >= 80%
    Expansion,   // fires only when health >= 95%
}

#[derive(Debug, Clone)]
pub struct ReactionRule {
    pub id:          &'static str,
    pub description: &'static str,
    pub priority:    u8,
    pub cooldown_s:  i64,
    pub class:       RuleClass,
}

#[derive(Debug, serde::Deserialize)]
struct TomlRule {
    id:          String,
    description: Option<String>,
    priority:    Option<u8>,
    cooldown_m:  Option<i64>,
    enabled:     Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
struct TomlRuleFile {
    rule: Option<Vec<TomlRule>>,
}

fn load_toml_overrides(core_root: &str) -> Vec<TomlRule> {
    let dir = std::path::PathBuf::from(core_root).join("runtime/reactions");
    let mut all = vec![];
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    if let Ok(parsed) = toml::from_str::<TomlRuleFile>(&text) {
                        if let Some(rules) = parsed.rule {
                            all.extend(rules);
                        }
                    }
                }
            }
        }
    }
    all
}

pub fn rules() -> Vec<ReactionRule> {
    vec![
        ReactionRule {
            id:          "health.advisory",
            description: "Health below 95% in 2+ recent doctor runs",
            priority:    1,
            cooldown_s:  1800,
            class:       RuleClass::Stability,
        },
        ReactionRule {
            id:          "health.stale",
            description: "No doctor run in 24+ hours — health drift risk",
            priority:    1,
            cooldown_s:  3600,
            class:       RuleClass::Stability,
        },
        ReactionRule {
            id:          "security.aging",
            description: "Security scan older than 7 days",
            priority:    2,
            cooldown_s:  86400,
            class:       RuleClass::Maintenance,
        },
        ReactionRule {
            id:          "checkpoint.stale",
            description: "No checkpoint taken in 7+ days",
            priority:    3,
            cooldown_s:  86400,
            class:       RuleClass::Expansion,
        },
        ReactionRule {
            id:          "intent.overflow",
            description: "3+ intents in-progress simultaneously",
            priority:    2,
            cooldown_s:  7200,
            class:       RuleClass::Maintenance,
        },
        ReactionRule {
            id:          "forecast.declining",
            description: "Health forecast trend below -1.5",
            priority:    1,
            cooldown_s:  3600,
            class:       RuleClass::Stability,
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

struct GoalContext {
    id:    String,
    title: String,
}

fn active_goals(ctx: &AppContext) -> Vec<GoalContext> {
    let mut stmt = match ctx.runtime.db.prepare(
        "SELECT id, title FROM forest_goals WHERE status = 'accepted' ORDER BY id ASC"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map([], |r| Ok(GoalContext {
        id:    r.get(0)?,
        title: r.get(1)?,
    }))
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

fn goal_context_for(rule_id: &str, goals: &[GoalContext]) -> Option<String> {
    if goals.is_empty() {
        return None;
    }
    // Map rule domains to relevant goal keywords
    let keywords: &[&str] = match rule_id {
        r if r.starts_with("health.") || r == "forecast.declining" =>
            &["stability", "health", "retire", "tool", "core"],
        r if r.starts_with("security.") =>
            &["security", "audit", "hardening"],
        "intent.overflow" =>
            &["intent", "goal", "focus", "retire", "tool"],
        "checkpoint.stale" =>
            &["resilience", "backup", "checkpoint", "snapshot"],
        _ => &[],
    };
    // Find first goal whose title contains any keyword
    for goal in goals {
        let title_lower = goal.title.to_lowercase();
        if keywords.iter().any(|k| title_lower.contains(k)) {
            return Some(format!("{} — {}", goal.id, goal.title));
        }
    }
    // If no keyword match, return first accepted goal as general context
    goals.first().map(|g| format!("{} — {}", g.id, g.title))
}

fn current_health(ctx: &AppContext) -> u32 {
    std::fs::read_to_string(
        std::path::PathBuf::from(&ctx.core_root).join("runtime/cache/health.txt")
    )
    .unwrap_or_else(|_| "95".to_string())
    .trim()
    .trim_end_matches('%')
    .parse()
    .unwrap_or(95)
}

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
    let toml_overrides = load_toml_overrides(&ctx.core_root);
    let health = current_health(ctx);
    let mut fired = vec![];

    for eval in evaluators {
        if let Some(mut reaction) = eval(ctx) {
            // Boundary gate — Rule 3
            let rule = all_rules.iter().find(|r| r.id == reaction.rule_id.as_str());
            let allowed = match rule.map(|r| &r.class) {
                Some(RuleClass::Stability) => true,           // always fires
                Some(RuleClass::Maintenance) => health >= 80, // needs basic health
                Some(RuleClass::Expansion) => health >= 95,   // only when healthy
                None => health >= 80,
            };
            if !allowed {
                continue;
            }

            // Check if disabled in TOML
            let toml = toml_overrides.iter().find(|r| r.id == reaction.rule_id);
            if let Some(t) = toml {
                if t.enabled == Some(false) {
                    continue;
                }
                if let Some(p) = t.priority {
                    reaction.priority = p;
                }
            }

            // Cooldown
            let base_cooldown = rule.map(|r| r.cooldown_s).unwrap_or(1800);
            let cooldown = toml.and_then(|t| t.cooldown_m).map(|m| m * 60)
                .unwrap_or(base_cooldown);
            if !is_on_cooldown(ctx, &reaction.rule_id, cooldown) {
                fired.push(reaction);
            }
        }
    }

    // Apply decay — suppress rules that fired long ago and were ignored
    let discipline = load_discipline(&ctx.core_root);
    let fired: Vec<Reaction> = fired.into_iter().filter(|reaction| {
        let disc = discipline.iter().find(|d| d.rule_id == reaction.rule_id);
        let decay_h = disc.and_then(|d| d.decay_after_h).unwrap_or(48.0);
        !is_decayed(ctx, &reaction.rule_id, decay_h)
    }).collect();

    let mut fired = fired;
    fired.sort_by_key(|r| r.priority);
    fired
}

// ── CLI commands ──────────────────────────────────────────────────────────────


pub fn enable(ctx: &AppContext, id: &str) -> CoreResult<()> {
    toggle_rule(ctx, id, true)
}

pub fn disable(ctx: &AppContext, id: &str) -> CoreResult<()> {
    toggle_rule(ctx, id, false)
}

fn toggle_rule(_ctx: &AppContext, id: &str, enabled: bool) -> CoreResult<()> {
    let dir = std::path::PathBuf::from(
        std::env::var("HOME").unwrap_or_default()
    ).join("0-core/runtime/reactions");

    let mut found = false;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(&path) {
                if !text.contains(&format!("id = \"{}\"", id)) {
                    continue;
                }
                // Parse line by line, track which [[rule]] block we are in
                let mut out = String::new();
                let mut current_id = String::new();
                let mut patched = false;
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed == "[[rule]]" {
                        current_id = String::new();
                    } else if let Some(val) = trimmed.strip_prefix("id = \"") {
                        current_id = val.trim_end_matches('"').to_string();
                    } else if trimmed.starts_with("enabled = ") && current_id == id {
                        out.push_str(&format!("enabled = {}\n", enabled));
                        patched = true;
                        continue;
                    }
                    out.push_str(line);
                    out.push('\n');
                }
                if patched {
                    std::fs::write(&path, out).ok();
                    found = true;
                    break;
                }
            }
        }
    }

    if found {
        let state = if enabled { "enabled".bright_green() } else { "disabled".yellow() };
        println!("  {} rule {} {}", "✅".normal(), id.bright_white(), state);
    } else {
        println!("  {} rule '{}' not found in runtime/reactions/", "✗".bright_red(), id);
        println!("  Rules must exist in a TOML file to be toggled");
    }
    Ok(())
}

pub fn add(ctx: &AppContext, id: &str, description: &str, priority: u8, cooldown_m: i64) -> CoreResult<()> {
    let path = std::path::PathBuf::from(&ctx.core_root)
        .join("runtime/reactions/custom.toml");

    let entry = format!(
        "\n[[rule]]\nid = \"{}\"\ndescription = \"{}\"\npriority = {}\ncooldown_m = {}\nenabled = true\n",
        id, description, priority, cooldown_m
    );

    // Append to custom.toml
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    std::fs::write(&path, format!("{}{}", existing, entry))
        .map_err(|e| crate::errors::CoreError::Runtime(e.to_string()))?;

    println!("  {} rule added → {}", "✅".normal(), id.bright_green().bold());
    println!("  {} runtime/reactions/custom.toml", "→".dimmed());
    println!("  {} core react run  — to evaluate now", "hint:".dimmed());
    Ok(())
}

pub fn rules_list(ctx: &AppContext) -> CoreResult<()> {
    let toml_overrides = load_toml_overrides(&ctx.core_root);
    let all_rules = rules();

    println!("{}", "🌲 Reaction Rules — Full Registry".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    for rule in &all_rules {
        let toml = toml_overrides.iter().find(|r| r.id == rule.id);
        let enabled = toml.and_then(|t| t.enabled).unwrap_or(true);
        let cooldown = toml.and_then(|t| t.cooldown_m).unwrap_or(rule.cooldown_s / 60);
        let priority = toml.and_then(|t| t.priority).unwrap_or(rule.priority);

        let status = if enabled {
            "✅ enabled".green().to_string()
        } else {
            "○ disabled".dimmed().to_string()
        };

        let priority_icon = match priority {
            1 => "🔴",
            2 => "🟡",
            _ => "🟢",
        };

        println!(
            "  {} {}  {}  cooldown: {}m",
            priority_icon,
            rule.id.bright_white(),
            status,
            cooldown.to_string().dimmed(),
        );
        println!("    {}", rule.description.dimmed());
        println!();
    }

    // Show any custom rules
    let custom_path = std::path::PathBuf::from(&ctx.core_root)
        .join("runtime/reactions/custom.toml");
    if custom_path.exists() {
        if let Ok(text) = std::fs::read_to_string(&custom_path) {
            if let Ok(parsed) = toml::from_str::<TomlRuleFile>(&text) {
                if let Some(custom_rules) = parsed.rule {
                    if !custom_rules.is_empty() {
                        println!("  {}", "Custom rules:".dimmed());
                        for r in &custom_rules {
                            let enabled = r.enabled.unwrap_or(true);
                            let status = if enabled {
                                "✅ enabled".green().to_string()
                            } else {
                                "○ disabled".dimmed().to_string()
                            };
                            println!(
                                "  🔧 {}  {}",
                                r.id.bright_white(),
                                status,
                            );
                            if let Some(ref d) = r.description {
                                println!("    {}", d.dimmed());
                            }
                            println!();
                        }
                    }
                }
            }
        }
    }

    println!("{}", "━".repeat(52).dimmed());
    println!("  {} core react enable/disable <id>  — toggle a rule", "hint:".dimmed());
    Ok(())
}


pub fn bounds(ctx: &AppContext) -> CoreResult<()> {
    let health = current_health(ctx);

    println!("{}", "🌲 Reaction Boundaries".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    let gate_status = |threshold: u32, label: &str| {
        if health >= threshold {
            format!("✅ OPEN  (health {}% ≥ {}%)", health, threshold).green().to_string()
        } else {
            format!("🔒 CLOSED (health {}% < {}%)", health, threshold).yellow().to_string()
        }
    };

    println!("  {} Current health: {}%", "📊".normal(),
        if health >= 95 { health.to_string().bright_green() }
        else if health >= 80 { health.to_string().yellow() }
        else { health.to_string().bright_red() }
    );
    println!();
    println!("  {} Stability rules  (health.*, forecast.*)  {}",
        "🔴".normal(), gate_status(0, "0"));
    println!("     Always active — fire regardless of health");
    println!();
    println!("  {} Maintenance rules (security.*, intent.*)  {}",
        "🟡".normal(), gate_status(80, "80"));
    println!("     Require health ≥ 80% to fire");
    println!();
    println!("  {} Expansion rules  (checkpoint.*)  {}",
        "🟢".normal(), gate_status(95, "95"));
    println!("     Require health ≥ 95% to fire");
    println!();
    println!("{}", "━".repeat(52).dimmed());

    // Rule 1 — No reactions outside goal scope
    println!("  {} Rule 1: Reactions bounded by accepted v9 goals", "🛡️ ".normal());
    println!("  {} Rule 2: Reactions propose, never execute", "🛡️ ".normal());
    println!("  {} Rule 3: Stability gates enforced in code", "🛡️ ".normal());
    println!("  {} Rule 4: Discipline (cooldown/decay) non-optional", "🛡️ ".normal());
    println!();

    Ok(())
}

pub fn audit(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let health = current_health(ctx);
    let all_rules = rules();
    let toml_overrides = load_toml_overrides(&ctx.core_root);
    let goals = active_goals(ctx);
    let now = chrono::Local::now().timestamp();

    println!("{}", "🌲 Reaction Audit".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();
    println!("  Health: {}%  Goals: {}", health, goals.len());
    println!();

    for rule in &all_rules {
        let toml = toml_overrides.iter().find(|r| r.id == rule.id);
        let enabled = toml.and_then(|t| t.enabled).unwrap_or(true);

        let boundary_ok = match &rule.class {
            RuleClass::Stability   => true,
            RuleClass::Maintenance => health >= 80,
            RuleClass::Expansion   => health >= 95,
        };

        let cooldown_s = toml.and_then(|t| t.cooldown_m).map(|m| m * 60)
            .unwrap_or(rule.cooldown_s);
        let last_fired: Option<i64> = ctx.runtime.db.query_row(
            "SELECT last_fired FROM reaction_cooldowns WHERE rule_id = ?1",
            rusqlite::params![rule.id],
            |r| r.get(0),
        ).ok();
        let on_cooldown = last_fired.map(|ts| now - ts < cooldown_s).unwrap_or(false);

        let class_label = match &rule.class {
            RuleClass::Stability   => "stability  ",
            RuleClass::Maintenance => "maintenance",
            RuleClass::Expansion   => "expansion  ",
        };

        let state = if !enabled {
            "○ disabled".dimmed().to_string()
        } else if !boundary_ok {
            "🔒 gated  ".yellow().to_string()
        } else if on_cooldown {
            "🔵 cooldown".dimmed().to_string()
        } else {
            "✅ ready   ".green().to_string()
        };

        println!(
            "  {} {} [{}]  {}",
            state,
            rule.id.bright_white(),
            class_label.dimmed(),
            goal_context_for(rule.id, &goals)
                .map(|g| format!("→ {}", g))
                .unwrap_or_default()
                .dimmed()
                .to_string(),
        );
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}


pub fn story(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let today = chrono::Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT id, rule_id, triggered_at, message, context
         FROM reaction_log
         WHERE triggered_at >= ?1
         ORDER BY triggered_at ASC"
    )?;

    let rows: Vec<(i64, String, i64, String, String)> = stmt
        .query_map(rusqlite::params![today], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    println!("{}", "🌲 Reaction Story — Today".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    if rows.is_empty() {
        println!("  {} No reactions fired today", "○".dimmed());
        println!("  The forest has been quiet — run: core react run");
        println!();
        println!("{}", "━".repeat(52).dimmed());
        return Ok(());
    }

    let goals = active_goals(ctx);
    println!(
        "  {} reaction(s) today  •  {} active goal(s)",
        rows.len().to_string().bright_white(),
        goals.len().to_string().dimmed(),
    );
    println!();

    for (i, (id, rule_id, ts, message, _context)) in rows.iter().enumerate() {
        let time = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|d| d.with_timezone(&chrono::Local).format("%H:%M").to_string())
            .unwrap_or_default();

        let all_rules = rules();
        let rule = all_rules.iter().find(|r| r.id == rule_id.as_str());
        let priority_icon = match rule.map(|r| r.priority) {
            Some(1) => "🔴",
            Some(2) => "🟡",
            _ => "🟢",
        };

        let class_label = match rule.map(|r| &r.class) {
            Some(RuleClass::Stability)   => "stability",
            Some(RuleClass::Maintenance) => "maintenance",
            Some(RuleClass::Expansion)   => "expansion",
            None => "unknown",
        };

        // Narrative sentence
        let narrative = match rule_id.as_str() {
            "health.advisory" => "The forest noticed health declining across recent runs.",
            "health.stale"    => "The forest detected no health check in over a day.",
            "security.aging"  => "The forest flagged aging security scan data.",
            "checkpoint.stale"=> "The forest suggested a snapshot — no checkpoint recently.",
            "intent.overflow" => "The forest detected too many intents in flight simultaneously.",
            "forecast.declining" => "The forest saw the health forecast trending downward.",
            _ => "The forest detected a condition worth surfacing.",
        };

        println!(
            "  {} {}  {} #{} [{}]",
            priority_icon,
            time.dimmed(),
            rule_id.bright_white(),
            id,
            class_label.dimmed(),
        );
        println!("     {}", narrative.dimmed());
        println!("     {}", message.white());

        if let Some(goal) = goal_context_for(rule_id, &goals) {
            println!("     {} {}", "🎯".dimmed(), goal.dimmed());
        }

        if i < rows.len() - 1 {
            println!();
        }
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());

    // Summary sentence
    let high_count = rows.iter().filter(|(_, rule_id, _, _, _)| {
        rules().iter().find(|r| r.id == rule_id.as_str())
            .map(|r| r.priority == 1)
            .unwrap_or(false)
    }).count();

    if high_count > 0 {
        println!(
            "  {} {} high-priority signal(s) today — review recommended",
            "⚠️ ".yellow(),
            high_count
        );
    } else {
        println!("  {} Low-priority day — forest signalling within normal range", "✅".green());
    }
    println!();

    Ok(())
}


#[derive(Debug, serde::Deserialize)]
struct DisciplineRule {
    rule_id:      String,
    decay_after_h: Option<f64>,
    coalesce:     Option<bool>,
    escalate_if:  Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct DisciplineFile {
    discipline: Option<Vec<DisciplineRule>>,
}

fn load_discipline(core_root: &str) -> Vec<DisciplineRule> {
    let path = std::path::PathBuf::from(core_root)
        .join("runtime/reaction-discipline.toml");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| toml::from_str::<DisciplineFile>(&text).ok())
        .and_then(|f| f.discipline)
        .unwrap_or_default()
}

fn is_decayed(ctx: &AppContext, rule_id: &str, decay_after_h: f64) -> bool {
    let decay_s = (decay_after_h * 3600.0) as i64;
    let now = chrono::Local::now().timestamp();
    // Check if rule fired within decay window — if it did and we are past decay, suppress
    ctx.runtime.db.query_row(
        "SELECT last_fired FROM reaction_cooldowns WHERE rule_id = ?1",
        rusqlite::params![rule_id],
        |r| r.get::<_, i64>(0),
    )
    .ok()
    .map(|last| {
        let elapsed = now - last;
        // Decayed: fired more than decay_after_h ago — suppress silently
        elapsed > decay_s
    })
    .unwrap_or(false)
}

pub fn coalesce(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let discipline = load_discipline(&ctx.core_root);
    let all_rules = rules();
    let toml_overrides = load_toml_overrides(&ctx.core_root);
    let now = chrono::Local::now().timestamp();

    println!("{}", "🌲 Reaction Coalesce — Grouped Signals".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    let mut coalesce_groups: std::collections::HashMap<&str, Vec<String>> =
        std::collections::HashMap::new();

    for rule in &all_rules {
        let disc = discipline.iter().find(|d| d.rule_id == rule.id);
        if disc.and_then(|d| d.coalesce).unwrap_or(false) {
            // Check if this rule has a pending (fired but within cooldown) signal
            let last: Option<i64> = ctx.runtime.db.query_row(
                "SELECT last_fired FROM reaction_cooldowns WHERE rule_id = ?1",
                rusqlite::params![rule.id],
                |r| r.get(0),
            ).ok();

            if let Some(ts) = last {
                let toml = toml_overrides.iter().find(|r| r.id == rule.id);
                let cooldown = toml.and_then(|t| t.cooldown_m).map(|m| m * 60)
                    .unwrap_or(rule.cooldown_s);
                if now - ts < cooldown {
                    // Still on cooldown — coalescing candidate
                    let group = match rule.class {
                        RuleClass::Stability   => "health",
                        RuleClass::Maintenance => "maintenance",
                        RuleClass::Expansion   => "expansion",
                    };
                    coalesce_groups.entry(group).or_default().push(rule.id.to_string());
                }
            }
        }
    }

    if coalesce_groups.is_empty() {
        println!("  {} No coalescing candidates — no batched signals pending", "○".dimmed());
    } else {
        for (group, rule_ids) in &coalesce_groups {
            println!("  {} group: {}", "📦".normal(), group.bright_white());
            for id in rule_ids {
                println!("    {} {}", "→".dimmed(), id.cyan());
            }
            println!();
        }
        println!("  {} These signals would surface together in one notification", "hint:".dimmed());
    }

    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn discipline_show(ctx: &AppContext) -> CoreResult<()> {
    let discipline = load_discipline(&ctx.core_root);
    let all_rules = rules();
    let now = chrono::Local::now().timestamp();

    println!("{}", "🌲 Reaction Discipline — Config".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    for rule in &all_rules {
        let disc = discipline.iter().find(|d| d.rule_id == rule.id);

        let decay_h = disc.and_then(|d| d.decay_after_h).unwrap_or(24.0);
        let coalesce = disc.and_then(|d| d.coalesce).unwrap_or(false);
        let escalate = disc.and_then(|d| d.escalate_if.as_deref()).unwrap_or("—");

        // Check decay status
        let last: Option<i64> = ctx.runtime.db.query_row(
            "SELECT last_fired FROM reaction_cooldowns WHERE rule_id = ?1",
            rusqlite::params![rule.id],
            |r| r.get(0),
        ).ok();

        let decay_status = match last {
            Some(ts) => {
                let elapsed_h = (now - ts) as f64 / 3600.0;
                if elapsed_h > decay_h {
                    format!("⏱  decayed ({:.1}h > {:.0}h)", elapsed_h, decay_h).dimmed().to_string()
                } else {
                    format!("✅ active  ({:.1}h of {:.0}h)", elapsed_h, decay_h).green().to_string()
                }
            }
            None => "✅ never fired".green().to_string(),
        };

        println!(
            "  {}  decay: {}h  coalesce: {}  escalate: {}",
            rule.id.bright_white(),
            decay_h.to_string().cyan(),
            if coalesce { "yes".green().to_string() } else { "no".dimmed().to_string() },
            escalate.dimmed(),
        );
        println!("    {}", decay_status);
        println!();
    }

    println!("{}", "━".repeat(52).dimmed());
    println!("  {} Edit: runtime/reaction-discipline.toml", "hint:".dimmed());
    Ok(())
}

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
        let goals = active_goals(ctx);
        println!(
            "  {} reaction(s) triggered",
            fired.len().to_string().bright_white()
        );
        if !goals.is_empty() {
            println!(
                "  {} {} active goal(s) in scope",
                "🎯".normal(),
                goals.len().to_string().dimmed(),
            );
        }
        println!();

        for r in &fired {
            let icon = match r.priority {
                1 => "🔴",
                2 => "🟡",
                _ => "🟢",
            };
            println!("  {}  {}", icon, r.message.bright_white());
            println!("      {} {}", "→".dimmed(), r.action.cyan());

            // Goal context enrichment
            if let Some(goal) = goal_context_for(&r.rule_id, &goals) {
                println!("      {} {}", "🎯 goal:".dimmed(), goal.bright_white().dimmed());
            }
            println!();

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
