// INT-156 — Core v13 Autonomy Foundation
// DORMANT — activates when Jarvis score >= 95/100
// The forest acts within mandates the human has defined.
// No autonomous action executes without explicit authorization.

use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;

const JARVIS_GATE: i64 = 95;

pub fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(
        "CREATE TABLE IF NOT EXISTS forest_mandates (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            rule        TEXT    NOT NULL,
            scope       TEXT    NOT NULL DEFAULT 'suggest',
            created_at  INTEGER NOT NULL,
            revoked_at  INTEGER,
            active      INTEGER NOT NULL DEFAULT 1
        );
        CREATE TABLE IF NOT EXISTS autonomy_log (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            mandate_id  INTEGER,
            action      TEXT    NOT NULL,
            reason      TEXT    NOT NULL,
            result      TEXT,
            reverted    INTEGER NOT NULL DEFAULT 0,
            executed_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS trust_scores (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            action_type TEXT    NOT NULL,
            attempts    INTEGER NOT NULL DEFAULT 0,
            correct     INTEGER NOT NULL DEFAULT 0,
            updated_at  INTEGER NOT NULL
        );",
    )?;
    Ok(())
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn get_jarvis_score(ctx: &AppContext) -> i64 {
    ctx.runtime
        .db
        .query_row(
            "SELECT score FROM jarvis_readiness_log ORDER BY recorded_at DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0)
}

fn check_gate(ctx: &AppContext) -> bool {
    let score = get_jarvis_score(ctx);
    if score < JARVIS_GATE {
        println!();
        println!("  {} v13 Autonomy is DORMANT", "🔒".normal());
        println!(
            "  {} Jarvis score: {}/100 (gate: {}/100)",
            "→".bright_cyan(),
            score.to_string().yellow(),
            JARVIS_GATE
        );
        println!(
            "  {} Complete INT-159 (context) + INT-160 (memory) to reach gate",
            "→".dimmed()
        );
        println!(
            "  {} Run: core strategy jarvis — to see current score",
            "→".dimmed()
        );
        println!();
        return false;
    }
    true
}

// ── Mandate System ────────────────────────────────────────────────────────────

pub fn mandate_list(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!();
    println!("  {} Active Mandates", "📋".normal());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT id, rule, scope, created_at FROM forest_mandates WHERE active=1 ORDER BY created_at DESC"
    )?;
    let mandates: Vec<(i64, String, String)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if mandates.is_empty() {
        println!("  {} No mandates defined", "○".dimmed());
        println!("  {} Run: core mandate set <rule>", "→".dimmed());
    } else {
        for (id, rule, scope) in &mandates {
            println!(
                "  #{} [{}] {}",
                id.to_string().dimmed(),
                scope.bright_cyan(),
                rule.bright_white()
            );
        }
    }

    let score = get_jarvis_score(ctx);
    println!();
    if score < JARVIS_GATE {
        println!(
            "  {} Autonomy DORMANT — Jarvis {}/{}",
            "🔒".normal(),
            score,
            JARVIS_GATE
        );
    } else {
        println!(
            "  {} Autonomy ACTIVE — Jarvis {}/{}",
            "✅".normal(),
            score,
            JARVIS_GATE
        );
    }
    println!();
    Ok(())
}

pub fn mandate_set(ctx: &AppContext, rule: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    ctx.runtime.db.execute(
        "INSERT INTO forest_mandates (rule, scope, created_at) VALUES (?1, 'suggest', ?2)",
        rusqlite::params![rule, now_ts()],
    )?;
    println!();
    println!(
        "  {} Mandate recorded: {}",
        "✅".normal(),
        rule.bright_white()
    );
    println!(
        "  {} Status: stored, awaiting Jarvis gate ({}/100)",
        "→".dimmed(),
        JARVIS_GATE
    );
    println!(
        "  {} The forest will act on this when score reaches gate",
        "→".dimmed()
    );
    println!();
    Ok(())
}

pub fn mandate_revoke(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let id_num: i64 = id.parse().unwrap_or(0);
    ctx.runtime.db.execute(
        "UPDATE forest_mandates SET active=0, revoked_at=?1 WHERE id=?2",
        rusqlite::params![now_ts(), id_num],
    )?;
    println!();
    println!("  {} Mandate #{} revoked", "✅".normal(), id);
    println!("  {} Forest will not act on this mandate", "→".dimmed());
    println!();
    Ok(())
}

pub fn mandate_revoke_all(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    ctx.runtime.db.execute(
        "UPDATE forest_mandates SET active=0, revoked_at=?1",
        rusqlite::params![now_ts()],
    )?;
    println!();
    println!(
        "  {} All mandates revoked — forest returned to fully manual mode",
        "🔒".normal()
    );
    println!();
    Ok(())
}

// ── Autonomy Engine ───────────────────────────────────────────────────────────

pub fn autonomy_pending(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    if !check_gate(ctx) {
        return Ok(());
    }
    println!("  {} No autonomous actions pending", "○".dimmed());
    println!(
        "  {} Mandates define what the forest may suggest",
        "→".dimmed()
    );
    Ok(())
}

pub fn autonomy_log(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!();
    println!("  {} Autonomy Log", "📋".normal());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT action, reason, result, reverted, executed_at FROM autonomy_log ORDER BY executed_at DESC LIMIT 20"
    )?;
    let entries: Vec<(String, String, String, i64)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                r.get::<_, i64>(3)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if entries.is_empty() {
        println!("  {} No autonomous actions recorded yet", "○".dimmed());
    } else {
        for (action, reason, _result, reverted) in &entries {
            let status = if *reverted == 1 {
                "↩".yellow().to_string()
            } else {
                "✅".to_string()
            };
            println!(
                "  {} {} — {}",
                status,
                action.bright_white(),
                reason.dimmed()
            );
        }
    }
    println!();
    Ok(())
}

pub fn autonomy_run(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    if !check_gate(ctx) {
        return Ok(());
    }
    println!("  {} No pending actions to execute", "○".dimmed());
    Ok(())
}

pub fn autonomy_revert(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!("  {} No autonomous actions to revert", "○".dimmed());
    Ok(())
}

// ── Trust Calibration ─────────────────────────────────────────────────────────

pub fn trust_score(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!();
    println!("  {} Trust Score", "🎯".normal());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );

    let jarvis = get_jarvis_score(ctx);
    println!(
        "  {} Jarvis score:    {}/100",
        "→".bright_cyan(),
        jarvis.to_string().bright_white()
    );
    println!("  {} Gate required:   {}/100", "→".dimmed(), JARVIS_GATE);
    println!(
        "  {} Status:          {}",
        "→".dimmed(),
        if jarvis >= JARVIS_GATE {
            "ACTIVE".bright_green().to_string()
        } else {
            format!("DORMANT ({} points to gate)", JARVIS_GATE - jarvis)
                .yellow()
                .to_string()
        }
    );

    let mandate_count: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM forest_mandates WHERE active=1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let action_count: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM autonomy_log", [], |r| r.get(0))
        .unwrap_or(0);
    println!(
        "  {} Active mandates: {}",
        "→".dimmed(),
        mandate_count.to_string().bright_white()
    );
    println!(
        "  {} Actions taken:   {}",
        "→".dimmed(),
        action_count.to_string().bright_white()
    );
    println!();
    Ok(())
}

pub fn trust_history(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!(
        "  {} No trust history yet — autonomy is dormant",
        "○".dimmed()
    );
    Ok(())
}

pub fn trust_expand(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    if !check_gate(ctx) {
        return Ok(());
    }
    println!(
        "  {} Trust expansion available when actions have been verified",
        "○".dimmed()
    );
    Ok(())
}
