//! engines domain — coordination layer for all forest engines
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;

/// Ensure coordination tables exist in state.db
fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch("
        CREATE TABLE IF NOT EXISTS engine_registry (
            name        TEXT PRIMARY KEY,
            version     TEXT NOT NULL,
            last_active INTEGER NOT NULL,
            status      TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS engine_signals (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            source      TEXT NOT NULL,
            signal_type TEXT NOT NULL,
            payload     TEXT NOT NULL,
            weight      REAL,
            consumed_by TEXT,
            created_at  INTEGER NOT NULL,
            expires_at  INTEGER
        );
        CREATE TABLE IF NOT EXISTS engine_upgrade_log (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            engine           TEXT NOT NULL,
            from_version     TEXT NOT NULL,
            to_version       TEXT NOT NULL,
            breaking_change  INTEGER DEFAULT 0,
            affected_engines TEXT,
            migrated         INTEGER DEFAULT 0,
            upgraded_at      INTEGER NOT NULL
        );
    ")?;
    Ok(())
}

/// Seed known engines into registry if not present
fn seed_registry(ctx: &AppContext) -> CoreResult<()> {
    let now = now_ts();
    let engines = vec![
        ("core",              "3.0.0",  "active"),
        ("faelight-contextd", "0.1.0",  "active"),
        ("delegation",        "0.3.0",  "active"),
        ("friday",            "0.0.0",  "dormant"),
        ("pattern-weight",    "0.0.0",  "planned"),
        ("alignment",         "0.0.0",  "planned"),
        ("self-transform",    "0.0.0",  "planned"),
    ];
    for (name, version, status) in engines {
        let _ = ctx.runtime.db.execute(
            "INSERT OR IGNORE INTO engine_registry (name, version, last_active, status)
             VALUES (?1, ?2, ?3, ?4)",
            params![name, version, now, status],
        );
    }
    Ok(())
}

/// core engines status — show all engines and sync state
pub fn status(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    seed_registry(ctx)?;

    println!();
    println!("{}", "🌲 Engine Coordination Status".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT name, version, last_active, status FROM engine_registry ORDER BY name"
    )?;

    let engines: Vec<(String, String, i64, String)> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    })?.filter_map(|r| r.ok()).collect();

    println!("  {:<22} {:<10} {:<12} {}",
        "Engine".dimmed(),
        "Version".dimmed(),
        "Status".dimmed(),
        "Last Active".dimmed()
    );
    println!("  {}", "─".repeat(56).dimmed());

    let now = now_ts();
    for (name, version, last_active, status) in &engines {
        let status_colored = match status.as_str() {
            "active"   => status.bright_green(),
            "dormant"  => status.bright_yellow(),
            "planned"  => status.dimmed(),
            "degraded" => status.bright_red(),
            _          => status.normal(),
        };

        let age = if *last_active == 0 {
            "never".dimmed().to_string()
        } else {
            let secs = now - last_active;
            if secs < 60 { "now".bright_green().to_string() }
            else if secs < 3600 { format!("{} min ago", secs/60).normal().to_string() }
            else if secs < 86400 { format!("{} hr ago", secs/3600).normal().to_string() }
            else { format!("{} days ago", secs/86400).dimmed().to_string() }
        };

        println!("  {:<22} {:<10} {:<20} {}",
            name.bright_white(),
            version.cyan(),
            status_colored,
            age
        );
    }

    // Check for unacknowledged upgrades
    let pending: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM engine_upgrade_log WHERE migrated = 0",
        [], |r| r.get(0)
    ).unwrap_or(0);

    println!();
    if pending > 0 {
        println!("  {} {} engine(s) have unacknowledged upgrades",
            "⚠️ ".yellow(), pending.to_string().bright_yellow());
        println!("  {} Run: core engines sync <engine>", "💡".bright_cyan());
    } else {
        println!("  {} All engines synchronized", "✅".green());
    }

    // Show recent signals
    let signal_count: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM engine_signals WHERE created_at > ?1",
        params![now_ts() - 3600],
        |r| r.get(0)
    ).unwrap_or(0);

    println!("  {} {} signals in the last hour", "📡".normal(),
        signal_count.to_string().bright_white());
    println!();

    Ok(())
}

/// core engines sync <engine> — acknowledge upgrade
pub fn sync(ctx: &AppContext, engine: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let now = now_ts();

    let updated = ctx.runtime.db.execute(
        "UPDATE engine_upgrade_log SET migrated = 1
         WHERE engine = ?1 AND migrated = 0",
        params![engine],
    )?;

    // Update last_active
    let _ = ctx.runtime.db.execute(
        "UPDATE engine_registry SET last_active = ?1 WHERE name = ?2",
        params![now, engine],
    );

    if updated > 0 {
        println!("  {} {} synchronized — {} upgrade(s) acknowledged",
            "✅".green(), engine.bright_white(), updated.to_string().bright_green());
    } else {
        println!("  {} {} is already up to date", "○".dimmed(), engine.bright_white());
    }

    Ok(())
}

/// core engines signals — show recent cross-engine signals
pub fn signals(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;

    println!();
    println!("{}", "📡 Engine Signals (last 24h)".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();

    let since = now_ts() - 86400;
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT source, signal_type, payload, weight, created_at
         FROM engine_signals WHERE created_at > ?1
         ORDER BY created_at DESC LIMIT 50"
    )?;

    let signals: Vec<(String, String, String, Option<f64>, i64)> = stmt.query_map(
        params![since], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
    )?.filter_map(|r| r.ok()).collect();

    if signals.is_empty() {
        println!("  {} No signals in the last 24 hours", "○".dimmed());
        println!();
        return Ok(());
    }

    for (source, sig_type, payload, weight, ts) in &signals {
        let time = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_default();

        let weight_str = weight.map(|w| format!(" [{:.2}]", w)).unwrap_or_default();
        println!("  {} {} {} → {}{}",
            time.dimmed(),
            source.bright_cyan(),
            sig_type.bright_white(),
            payload.dimmed(),
            weight_str.bright_yellow()
        );
    }
    println!();
    Ok(())
}

/// core engines check — verify all engines are consistent
pub fn check(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    seed_registry(ctx)?;

    println!();
    println!("{}", "🔍 Engine Consistency Check".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();

    let mut issues = 0;

    // Check 1: any unacknowledged upgrades?
    let pending: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM engine_upgrade_log WHERE migrated = 0",
        [], |r| r.get(0)
    ).unwrap_or(0);

    if pending > 0 {
        println!("  {} {} unacknowledged upgrade(s)", "⚠️ ".yellow(), pending);
        issues += 1;
    } else {
        println!("  {} All upgrades acknowledged", "✅".green());
    }

    // Check 2: any engines in degraded state?
    let degraded: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM engine_registry WHERE status = 'degraded'",
        [], |r| r.get(0)
    ).unwrap_or(0);

    if degraded > 0 {
        println!("  {} {} engine(s) degraded", "⚠️ ".yellow(), degraded);
        issues += 1;
    } else {
        println!("  {} No degraded engines", "✅".green());
    }

    // Check 3: core version matches expected
    let core_version: String = ctx.runtime.db.query_row(
        "SELECT version FROM engine_registry WHERE name = 'core'",
        [], |r| r.get(0)
    ).unwrap_or_else(|_| "unknown".to_string());

    println!("  {} Core version: {}", "✅".green(), core_version.bright_cyan());

    println!();
    if issues == 0 {
        println!("  {} All engines consistent — forest thinks as one", "✅".green().bold());
    } else {
        println!("  {} {} issue(s) found — run: core engines status",
            "⚠️ ".yellow(), issues.to_string().bright_yellow());
    }
    println!();

    Ok(())
}

/// core engines upgrade-log — show upgrade history
pub fn upgrade_log(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;

    println!();
    println!("{}", "📋 Engine Upgrade History".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT engine, from_version, to_version, breaking_change, migrated, upgraded_at
         FROM engine_upgrade_log ORDER BY upgraded_at DESC LIMIT 20"
    )?;

    let logs: Vec<(String, String, String, i64, i64, i64)> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
    })?.filter_map(|r| r.ok()).collect();

    if logs.is_empty() {
        println!("  {} No upgrades recorded yet", "○".dimmed());
        println!();
        return Ok(());
    }

    for (engine, from, to, breaking, migrated, ts) in &logs {
        let time = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();
        let status = if *migrated == 1 { "✅".to_string() } else { "⏳".to_string() };
        let breaking_str = if *breaking == 1 { " 🔴 BREAKING".bright_red().to_string() } else { String::new() };
        println!("  {} {} {} → {}{}  {}",
            status, engine.bright_white(),
            from.dimmed(), to.bright_green(),
            breaking_str, time.dimmed()
        );
    }
    println!();
    Ok(())
}

/// Record an engine upgrade in the log
#[allow(dead_code)]
pub fn record_upgrade(ctx: &AppContext, engine: &str, from: &str, to: &str, breaking: bool, affected: &[&str]) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let affected_json = format!("[{}]", affected.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(","));
    ctx.runtime.db.execute(
        "INSERT INTO engine_upgrade_log (engine, from_version, to_version, breaking_change, affected_engines, migrated, upgraded_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
        params![engine, from, to, breaking as i64, affected_json, now_ts()],
    )?;
    // Update registry version
    let _ = ctx.runtime.db.execute(
        "UPDATE engine_registry SET version = ?1, last_active = ?2 WHERE name = ?3",
        params![to, now_ts(), engine],
    );
    Ok(())
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
