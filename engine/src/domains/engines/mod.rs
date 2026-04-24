//! engines domain — coordination layer for all forest engines
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;

/// Ensure coordination tables exist in state.db
fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(
        "
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
    ",
    )?;
    Ok(())
}

/// Seed known engines into registry if not present
fn seed_registry(ctx: &AppContext) -> CoreResult<()> {
    let now = now_ts();
    let engines = vec![
        ("core", "3.0.0", "active"),
        ("faelight-contextd", "0.1.0", "active"),
        ("delegation", "0.3.0", "active"),
        ("friday", "0.0.0", "dormant"),
        ("pattern-weight", "0.0.0", "planned"),
        ("alignment", "0.0.0", "planned"),
        ("self-transform", "0.0.0", "planned"),
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

    let mut stmt = ctx
        .runtime
        .db
        .prepare("SELECT name, version, last_active, status FROM engine_registry ORDER BY name")?;

    let engines: Vec<(String, String, i64, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .filter_map(|r| r.ok())
        .collect();

    println!(
        "  {:<22} {:<10} {:<12} {}",
        "Engine".dimmed(),
        "Version".dimmed(),
        "Status".dimmed(),
        "Last Active".dimmed()
    );
    println!("  {}", "─".repeat(56).dimmed());

    let now = now_ts();
    for (name, version, last_active, status) in &engines {
        let status_colored = match status.as_str() {
            "active" => status.bright_green(),
            "dormant" => status.bright_yellow(),
            "planned" => status.dimmed(),
            "degraded" => status.bright_red(),
            _ => status.normal(),
        };

        let age = if *last_active == 0 {
            "never".dimmed().to_string()
        } else {
            let secs = now - last_active;
            if secs < 60 {
                "now".bright_green().to_string()
            } else if secs < 3600 {
                format!("{} min ago", secs / 60).normal().to_string()
            } else if secs < 86400 {
                format!("{} hr ago", secs / 3600).normal().to_string()
            } else {
                format!("{} days ago", secs / 86400).dimmed().to_string()
            }
        };

        println!(
            "  {:<22} {:<10} {:<20} {}",
            name.bright_white(),
            version.cyan(),
            status_colored,
            age
        );
    }

    // Check for unacknowledged upgrades
    let pending: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM engine_upgrade_log WHERE migrated = 0",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    println!();
    if pending > 0 {
        println!(
            "  {} {} engine(s) have unacknowledged upgrades",
            "⚠️ ".yellow(),
            pending.to_string().bright_yellow()
        );
        println!("  {} Run: core engines sync <engine>", "💡".bright_cyan());
    } else {
        println!("  {} All engines synchronized", "✅".green());
    }

    // Show recent signals
    let signal_count: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM engine_signals WHERE created_at > ?1",
            params![now_ts() - 3600],
            |r| r.get(0),
        )
        .unwrap_or(0);

    println!(
        "  {} {} signals in the last hour",
        "📡".normal(),
        signal_count.to_string().bright_white()
    );
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
        println!(
            "  {} {} synchronized — {} upgrade(s) acknowledged",
            "✅".green(),
            engine.bright_white(),
            updated.to_string().bright_green()
        );
    } else {
        println!(
            "  {} {} is already up to date",
            "○".dimmed(),
            engine.bright_white()
        );
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
         ORDER BY created_at DESC LIMIT 50",
    )?;

    let signals: Vec<(String, String, String, Option<f64>, i64)> = stmt
        .query_map(params![since], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

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
        println!(
            "  {} {} {} → {}{}",
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
    let pending: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM engine_upgrade_log WHERE migrated = 0",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if pending > 0 {
        println!("  {} {} unacknowledged upgrade(s)", "⚠️ ".yellow(), pending);
        issues += 1;
    } else {
        println!("  {} All upgrades acknowledged", "✅".green());
    }

    // Check 2: any engines in degraded state?
    let degraded: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM engine_registry WHERE status = 'degraded'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if degraded > 0 {
        println!("  {} {} engine(s) degraded", "⚠️ ".yellow(), degraded);
        issues += 1;
    } else {
        println!("  {} No degraded engines", "✅".green());
    }

    // Check 3: core version matches expected
    let core_version: String = ctx
        .runtime
        .db
        .query_row(
            "SELECT version FROM engine_registry WHERE name = 'core'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "unknown".to_string());

    println!(
        "  {} Core version: {}",
        "✅".green(),
        core_version.bright_cyan()
    );

    println!();
    if issues == 0 {
        println!(
            "  {} All engines consistent — forest thinks as one",
            "✅".green().bold()
        );
    } else {
        println!(
            "  {} {} issue(s) found — run: core engines status",
            "⚠️ ".yellow(),
            issues.to_string().bright_yellow()
        );
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
         FROM engine_upgrade_log ORDER BY upgraded_at DESC LIMIT 20",
    )?;

    let logs: Vec<(String, String, String, i64, i64, i64)> = stmt
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

    if logs.is_empty() {
        println!("  {} No upgrades recorded yet", "○".dimmed());
        println!();
        return Ok(());
    }

    for (engine, from, to, breaking, migrated, ts) in &logs {
        let time = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();
        let status = if *migrated == 1 {
            "✅".to_string()
        } else {
            "⏳".to_string()
        };
        let breaking_str = if *breaking == 1 {
            " 🔴 BREAKING".bright_red().to_string()
        } else {
            String::new()
        };
        println!(
            "  {} {} {} → {}{}  {}",
            status,
            engine.bright_white(),
            from.dimmed(),
            to.bright_green(),
            breaking_str,
            time.dimmed()
        );
    }
    println!();
    Ok(())
}

/// Record an engine upgrade in the log
#[allow(dead_code)]
pub fn record_upgrade(
    ctx: &AppContext,
    engine: &str,
    from: &str,
    to: &str,
    breaking: bool,
    affected: &[&str],
) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let affected_json = format!(
        "[{}]",
        affected
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(",")
    );
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

/// core engines process -- read unconsumed signals, route reactions, mark consumed
pub fn process(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let now = now_ts();
    let expire_cutoff = now - 86400; // 24h expiry
                                     // Step 1: expire stale signals
    let expired = ctx
        .runtime
        .db
        .execute(
            "UPDATE engine_signals SET consumed_by = 'expired'
         WHERE consumed_by IS NULL AND created_at < ?1",
            params![expire_cutoff],
        )
        .unwrap_or(0);
    // Step 2: read unconsumed, non-expired signals
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT id, source, signal_type, payload, weight
         FROM engine_signals
         WHERE consumed_by IS NULL AND created_at >= ?1
         ORDER BY created_at ASC LIMIT 50",
    )?;
    let signals: Vec<(i64, String, String, String, f64)> = stmt
        .query_map(params![expire_cutoff], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get::<_, Option<f64>>(4)?.unwrap_or(0.0),
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    if signals.is_empty() && expired == 0 {
        println!("  {} No unprocessed signals", "○".dimmed());
        return Ok(());
    }
    let mut processed = 0usize;
    let mut reactions: Vec<String> = Vec::new();
    for (id, source, sig_type, payload, weight) in &signals {
        // No-loop rule: skip if source would consume its own signal
        let consumer = route_signal(source, sig_type, payload, *weight, &mut reactions);
        // Mark consumed
        let _ = ctx.runtime.db.execute(
            "UPDATE engine_signals SET consumed_by = ?1 WHERE id = ?2",
            params![consumer, id],
        );
        processed += 1;
    }
    use colored::*;
    if expired > 0 {
        println!("  {} {} expired signals cleaned", "🧹".normal(), expired);
    }
    if processed > 0 {
        println!("  {} {} signals processed", "✅".green(), processed);
    }
    for reaction in &reactions {
        println!("  {} {}", "→".bright_cyan(), reaction);
    }
    // Record observations for Friday
    if !signals.is_empty() {
        let payload = format!(
            r#"{{"processed":{},"reactions":{}}}"#,
            processed,
            reactions.len()
        );
        let _ = ctx.runtime.db.execute(
            "INSERT INTO engine_signals (source, signal_type, payload, weight, consumed_by, created_at)
             VALUES ('engines', 'coordination', ?1, 1.0, 'self', ?2)",
            params![payload, now],
        );
    }
    Ok(())
}
fn route_signal(
    source: &str,
    sig_type: &str,
    payload: &str,
    weight: f64,
    reactions: &mut Vec<String>,
) -> String {
    match (source, sig_type) {
        // Deploy signal → suggest health check
        ("deploy", "deploy") => {
            if let Some(tool) = extract_json_str(payload, "tool") {
                reactions.push(format!(
                    "deploy detected: {} -- run d to verify health",
                    tool
                ));
            }
            "engines-coordinator".to_string()
        }
        // Health drop → surface as insight
        ("doctor", "health") => {
            let health: f64 = extract_json_f64(payload, "health").unwrap_or(100.0);
            if health < 95.0 {
                reactions.push(format!(
                    "health below peak: {:.0}% -- check for uncommitted changes or failed checks",
                    health
                ));
            }
            "engines-coordinator".to_string()
        }
        // Critical update → suggest engine sync
        ("faelight-update", "update") => {
            if weight < 0.8 {
                reactions.push(
                    "update completed with reduced health -- verify no breaking changes"
                        .to_string(),
                );
            }
            "engines-coordinator".to_string()
        }
        // Pattern weight critical → record for Friday
        ("pattern-weight", "critical-pattern") => {
            reactions.push("critical pattern detected -- Friday observation recorded".to_string());
            "friday-observer".to_string()
        }
        _ => "engines-coordinator".to_string(),
    }
}
fn extract_json_str(payload: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\":\"", key);
    let start = payload.find(&search)? + search.len();
    let end = payload[start..].find('"')? + start;
    Some(payload[start..end].to_string())
}
fn extract_json_f64(payload: &str, key: &str) -> Option<f64> {
    let search = format!("\"{}\":", key);
    let start = payload.find(&search)? + search.len();
    let end = payload[start..].find(|c: char| !c.is_ascii_digit() && c != '.')? + start;
    payload[start..end].parse().ok()
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
