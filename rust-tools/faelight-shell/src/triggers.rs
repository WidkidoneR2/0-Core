// Phase 17 — Event System (Reactive Shell)
// Triggers are rules: when <condition> fires, run <action>.
// Stored in state.db shell_triggers table.
// Evaluated after every command execution.

use crate::db::ForestDb;
use colored::*;
use rusqlite::params;

// ── Schema ────────────────────────────────────────────────────────────────────

pub fn ensure_schema(db: &ForestDb) {
    db.conn
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS shell_triggers (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            trigger     TEXT NOT NULL,
            action      TEXT NOT NULL,
            enabled     INTEGER NOT NULL DEFAULT 1,
            fired_count INTEGER NOT NULL DEFAULT 0,
            created_ts  INTEGER NOT NULL
        );",
        )
        .ok();
}

// ── Trigger types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TriggerKind {
    HealthDrop(u32),     // on health_drop <threshold>
    GitCommit,           // on git_commit
    EventDomain(String), // on event <domain>
    CommandRun(String),  // on run <command>
}

#[derive(Debug, Clone)]
pub struct Trigger {
    pub id: i64,
    pub trigger: String,
    pub action: String,
    pub enabled: bool,
    pub fired_count: i64,
}

// ── CRUD ──────────────────────────────────────────────────────────────────────

pub fn add(db: &ForestDb, trigger: &str, action: &str) -> Result<(), String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    db.conn
        .execute(
            "INSERT INTO shell_triggers (trigger, action, created_ts) VALUES (?1, ?2, ?3)",
            params![trigger, action, now],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list(db: &ForestDb) -> Vec<Trigger> {
    let mut stmt = match db
        .conn
        .prepare("SELECT id, trigger, action, enabled, fired_count FROM shell_triggers ORDER BY id")
    {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map([], |r| {
        Ok(Trigger {
            id: r.get(0)?,
            trigger: r.get(1)?,
            action: r.get(2)?,
            enabled: r.get::<_, i64>(3)? == 1,
            fired_count: r.get(4)?,
        })
    })
    .ok()
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

pub fn remove(db: &ForestDb, id: i64) -> bool {
    db.conn
        .execute("DELETE FROM shell_triggers WHERE id = ?1", params![id])
        .map(|n| n > 0)
        .unwrap_or(false)
}

pub fn enable(db: &ForestDb, id: i64, enabled: bool) -> bool {
    db.conn
        .execute(
            "UPDATE shell_triggers SET enabled = ?1 WHERE id = ?2",
            params![if enabled { 1i64 } else { 0i64 }, id],
        )
        .map(|n| n > 0)
        .unwrap_or(false)
}

// ── Evaluation ────────────────────────────────────────────────────────────────

pub struct TriggerContext {
    pub last_command: String,
    pub health_score: Option<i64>,
    #[allow(dead_code)]
    pub last_domain: Option<String>,
}

pub fn evaluate(db: &ForestDb, ctx: &TriggerContext, core_root: &str) {
    let triggers = list(db);
    for t in triggers.iter().filter(|t| t.enabled) {
        let fired = match_trigger(&t.trigger, ctx, db);
        if fired {
            fire(db, t, ctx, core_root);
        }
    }
}

fn match_trigger(trigger: &str, ctx: &TriggerContext, db: &ForestDb) -> bool {
    let parts: Vec<&str> = trigger.splitn(2, ' ').collect();
    match parts.as_slice() {
        ["health_drop", threshold] => {
            if let Ok(thresh) = threshold.parse::<i64>() {
                if let Some(health) = ctx.health_score {
                    return health < thresh;
                }
            }
            false
        }
        ["git_commit"] => {
            // Check if last command was a git commit
            ctx.last_command.contains("commit") || ctx.last_command.contains("fg commit")
        }
        ["event", domain] => {
            // Check if a recent event from this domain exists
            let count: i64 = db.conn.query_row(
                &format!("SELECT COUNT(*) FROM events WHERE domain='{}' AND timestamp > strftime('%s','now') - 5", domain),
                [], |r| r.get(0)
            ).unwrap_or(0);
            count > 0
        }
        ["run", cmd] => ctx.last_command.trim() == cmd.trim(),
        _ => false,
    }
}

fn fire(db: &ForestDb, trigger: &Trigger, _ctx: &TriggerContext, core_root: &str) {
    // Increment fired count
    db.conn
        .execute(
            "UPDATE shell_triggers SET fired_count = fired_count + 1 WHERE id = ?1",
            params![trigger.id],
        )
        .ok();

    let parts: Vec<&str> = trigger.action.splitn(2, ' ').collect();
    match parts.as_slice() {
        ["run", cmd] => {
            println!();
            println!(
                "  {} Trigger {} fired → {}",
                "⚡".yellow(),
                trigger.trigger.bright_cyan(),
                cmd.bright_white()
            );
            // Execute the command
            let output = std::process::Command::new("sh")
                .args(["-c", cmd])
                .current_dir(core_root)
                .output();
            match output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    if !stdout.trim().is_empty() {
                        print!("{}", stdout);
                    }
                }
                Err(e) => eprintln!("  {} Trigger action failed: {}", "✗".bright_red(), e),
            }
        }
        ["notify", msg] => {
            println!();
            println!(
                "  {} {} — {}",
                "🔔".normal(),
                trigger.trigger.bright_cyan(),
                msg.bright_white()
            );
        }
        _ => {
            println!(
                "  {} Unknown trigger action: {}",
                "✗".bright_red(),
                trigger.action
            );
        }
    }
}

// ── Display ───────────────────────────────────────────────────────────────────

pub fn render_list(db: &ForestDb) {
    let triggers = list(db);
    println!();
    println!("{}", "⚡  Shell Triggers".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    if triggers.is_empty() {
        println!("  {} No triggers defined.", "○".dimmed());
        println!(
            "  {} {}",
            "Add one:".dimmed(),
            "on health_drop 90 notify \"health degraded\"".bright_cyan()
        );
    } else {
        for t in &triggers {
            let status = if t.enabled {
                "enabled".bright_green().to_string()
            } else {
                "disabled".dimmed().to_string()
            };
            println!(
                "  {} [{}] {} → {}",
                format!("#{}", t.id).dimmed(),
                status,
                t.trigger.bright_white(),
                t.action.bright_cyan()
            );
            if t.fired_count > 0 {
                println!(
                    "    {} fired {} time(s)",
                    "→".dimmed(),
                    t.fired_count.to_string().bright_white()
                );
            }
        }
    }
    println!("{}", "━".repeat(56).dimmed());
    println!();
}
