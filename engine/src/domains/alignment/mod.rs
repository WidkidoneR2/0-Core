//! alignment domain — Core v15, the forest stays true to what matters
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;

/// Ensure alignment tables exist
fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch("
        CREATE TABLE IF NOT EXISTS declared_values (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            statement   TEXT NOT NULL UNIQUE,
            weight      INTEGER NOT NULL DEFAULT 7,
            scope       TEXT NOT NULL DEFAULT 'all',
            declared_at INTEGER NOT NULL,
            active      INTEGER NOT NULL DEFAULT 1
        );
        CREATE TABLE IF NOT EXISTS alignment_checks (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            subject     TEXT NOT NULL,
            score       REAL NOT NULL,
            aligned     TEXT,
            conflicts   TEXT,
            checked_at  INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS drift_log (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            value_id    INTEGER NOT NULL,
            observation TEXT NOT NULL,
            severity    TEXT NOT NULL,
            logged_at   INTEGER NOT NULL
        );
    ")?;
    Ok(())
}

/// Seed default values if none exist
fn seed_values(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let count: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM declared_values", [], |r| r.get(0)
    ).unwrap_or(0);

    if count == 0 {
        let now = now_ts();
        let seeds = vec![
            ("manual control over automation",          10, "all"),
            ("understanding over convenience",          9,  "all"),
            ("nothing runs without explicit human authorization", 10, "all"),
            ("recovery over perfection",                8,  "all"),
            ("focus > speed",                           8,  "intents"),
            ("ship consistently",                       7,  "commits"),
        ];
        for (stmt, weight, scope) in seeds {
            let _ = ctx.runtime.db.execute(
                "INSERT OR IGNORE INTO declared_values (statement, weight, scope, declared_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![stmt, weight, scope, now],
            );
        }
        println!("  {} Seeded 6 core values from declared principles", "✅".green());
    }
    Ok(())
}

/// core values list
pub fn values_list(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    seed_values(ctx)?;

    println!();
    println!("{}", "🌲 Declared Values".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT id, statement, weight, scope, declared_at, active
         FROM declared_values ORDER BY weight DESC, id"
    )?;

    let values: Vec<(i64, String, i64, String, i64, i64)> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
    })?.filter_map(|r| r.ok()).collect();

    if values.is_empty() {
        println!("  {} No values declared yet", "○".dimmed());
        println!("  {} Use: core values define \"your principle\"", "💡".bright_cyan());
        println!();
        return Ok(());
    }

    for (id, stmt, weight, scope, _declared_at, active) in &values {
        let weight_bar = "█".repeat(*weight as usize / 2);
        let active_marker = if *active == 1 { "✅".to_string() } else { "○".dimmed().to_string() };
        let scope_str = if scope == "all" { scope.dimmed() } else { scope.bright_cyan() };
        println!("  {} #{} [{}] {}  {} {}",
            active_marker,
            id.to_string().dimmed(),
            weight.to_string().bright_yellow(),
            stmt.bright_white(),
            weight_bar.bright_green(),
            scope_str
        );
    }
    println!();
    println!("  {} {} values declared", "→".dimmed(), values.len().to_string().bright_white());
    println!("  {} Scope: all | intents | commits | deploys", "💡".dimmed());
    println!();
    Ok(())
}

/// core values define <statement> [--weight N] [--scope S]
pub fn values_define(ctx: &AppContext, statement: &str, weight: i64, scope: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let now = now_ts();

    let result = ctx.runtime.db.execute(
        "INSERT INTO declared_values (statement, weight, scope, declared_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![statement, weight, scope, now],
    );

    match result {
        Ok(_) => {
            println!();
            println!("  {} Value declared:", "✅".green());
            println!("  {} \"{}\"", "→".bright_cyan(), statement.bright_white());
            println!("  {} Weight: {}  Scope: {}", "→".dimmed(),
                weight.to_string().bright_yellow(), scope.bright_cyan());
            println!();
        }
        Err(_) => {
            println!("  {} Value already exists — use: core values weight <id> <N>",
                "⚠️ ".yellow());
        }
    }
    Ok(())
}

/// core values remove <id>
pub fn values_remove(ctx: &AppContext, id: i64) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let affected = ctx.runtime.db.execute(
        "UPDATE declared_values SET active = 0 WHERE id = ?1",
        params![id],
    )?;
    if affected > 0 {
        println!("  {} Value #{} deactivated", "✅".green(), id);
    } else {
        println!("  {} Value #{} not found", "⚠️ ".yellow(), id);
    }
    Ok(())
}

/// core values weight <id> <weight>
pub fn values_weight(ctx: &AppContext, id: i64, weight: i64) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let affected = ctx.runtime.db.execute(
        "UPDATE declared_values SET weight = ?1 WHERE id = ?2",
        params![weight, id],
    )?;
    if affected > 0 {
        println!("  {} Value #{} weight updated to {}", "✅".green(), id, weight);
    } else {
        println!("  {} Value #{} not found", "⚠️ ".yellow(), id);
    }
    Ok(())
}

/// core align check <subject>
pub fn align_check(ctx: &AppContext, subject: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    seed_values(ctx)?;

    println!();
    println!("{}", "🧭 Alignment Check".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!("  Subject: {}", subject.bright_white().bold());
    println!();

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT id, statement, weight, scope FROM declared_values WHERE active = 1 ORDER BY weight DESC"
    )?;
    let values: Vec<(i64, String, i64, String)> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    })?.filter_map(|r| r.ok()).collect();

    // Check active intent count for focus > speed
    let active_intents: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM events WHERE domain = 'intent' AND action = 'started'
         AND timestamp > (SELECT COALESCE(MAX(timestamp), 0) FROM events WHERE domain = 'intent' AND action = 'completed')",
        [], |r| r.get(0)
    ).unwrap_or(0);

    // Count commits this week
    let week_ago = now_ts() - 604800;
    let commits_this_week: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM events WHERE domain = 'git' AND action = 'commit' AND timestamp > ?1",
        params![week_ago], |r| r.get(0)
    ).unwrap_or(0);

    let mut aligned: Vec<String> = Vec::new();
    let mut conflicts: Vec<String> = Vec::new();
    let mut total_weight = 0i64;
    let mut aligned_weight = 0i64;

    for (_id, statement, weight, _scope) in &values {
        total_weight += weight;
        let stmt_lower = statement.to_lowercase();

        if stmt_lower.contains("focus") && stmt_lower.contains("speed") {
            if active_intents > 4 {
                conflicts.push(format!("\"{}\" — {} intents active simultaneously", statement, active_intents));
            } else {
                aligned_weight += weight;
                aligned.push(format!("\"{}\" — {} intents active (within range)", statement, active_intents));
            }
        } else if stmt_lower.contains("ship consistently") || stmt_lower.contains("consistently") {
            if commits_this_week >= 10 {
                aligned_weight += weight;
                aligned.push(format!("\"{}\" — {} commits this week", statement, commits_this_week));
            } else {
                conflicts.push(format!("\"{}\" — only {} commits this week", statement, commits_this_week));
            }
        } else {
            // Default: assume aligned for non-behavioral values
            aligned_weight += weight;
            aligned.push(format!("\"{}\"", statement));
        }
    }

    let score = if total_weight > 0 {
        (aligned_weight as f64 / total_weight as f64 * 100.0) as i64
    } else { 100 };

    let score_colored = match score {
        s if s >= 80 => format!("{}%", s).bright_green(),
        s if s >= 60 => format!("{}%", s).bright_yellow(),
        _ => format!("{}%", score).bright_red(),
    };

    println!("  Alignment Score: {}", score_colored);
    println!();

    if !aligned.is_empty() {
        println!("  {} Aligned:", "✅".green());
        for a in &aligned {
            println!("    {} {}", "·".dimmed(), a.bright_white());
        }
        println!();
    }

    if !conflicts.is_empty() {
        println!("  {} Conflicts:", "⚠️ ".yellow());
        for c in &conflicts {
            println!("    {} {}", "·".bright_yellow(), c.bright_white());
        }
        println!();
        println!("  {} Recommendation: address conflicts before proceeding", "→".bright_cyan());
    }

    // Store check result
    let now = now_ts();
    let _ = ctx.runtime.db.execute(
        "INSERT INTO alignment_checks (subject, score, aligned, conflicts, checked_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            subject,
            score as f64 / 100.0,
            aligned.join("|"),
            conflicts.join("|"),
            now
        ],
    );

    println!();
    Ok(())
}

/// core align drift
pub fn align_drift(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    seed_values(ctx)?;

    println!();
    println!("{}", "📉 Behavioral Drift Report".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());

    let month_ago = now_ts() - 2592000;
    let week_ago = now_ts() - 604800;

    // Focus > speed: count multi-intent periods
    let high_intent_periods: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM alignment_checks WHERE conflicts LIKE '%focus%' AND checked_at > ?1",
        params![month_ago], |r| r.get(0)
    ).unwrap_or(0);

    // Ship consistently: commits per week avg
    let total_commits: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM events WHERE domain = 'git' AND action = 'commit' AND timestamp > ?1",
        params![month_ago], |r| r.get(0)
    ).unwrap_or(0);

    let commits_last_week: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM events WHERE domain = 'git' AND action = 'commit' AND timestamp > ?1",
        params![week_ago], |r| r.get(0)
    ).unwrap_or(0);

    // Deploy without health check
    let unchecked_deploys: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM forest_insights WHERE signal = 'deploy-unchecked' AND created_at > ?1",
        params![month_ago], |r| r.get(0)
    ).unwrap_or(0);

    println!();
    println!("  {} Last 30 days:", "→".bright_cyan());
    println!();

    if high_intent_periods > 3 {
        println!("  {} \"focus > speed\" violated {} times — consider closing intents before opening new ones",
            "⚠️ ".yellow(), high_intent_periods);
    } else {
        println!("  {} \"focus > speed\" — {} focus violations detected (healthy)",
            "✅".green(), high_intent_periods);
    }

    if total_commits >= 100 {
        println!("  {} \"ship consistently\" — {} commits this month ({} last week)",
            "✅".green(), total_commits, commits_last_week);
    } else {
        println!("  {} \"ship consistently\" — {} commits this month (below cadence)",
            "⚠️ ".yellow(), total_commits);
    }

    if unchecked_deploys > 0 {
        println!("  {} Deploy-without-health-check pattern: {} occurrences",
            "⚠️ ".yellow(), unchecked_deploys);
    } else {
        println!("  {} No deploy-without-health-check patterns detected", "✅".green());
    }

    println!();
    println!("  {} This is behavioral observation — not judgment.", "→".dimmed());
    println!("  {} Pattern recognition from your own data.", "→".dimmed());
    println!();

    Ok(())
}

/// core align report [--week N]
pub fn align_report(ctx: &AppContext, weeks_ago: i64) -> CoreResult<()> {
    ensure_tables(ctx)?;

    // weeks_ago=0 means current week (last 7 days)
    let end = now_ts() - (weeks_ago * 7 * 86400);
    let start = end - (7 * 86400);

    let start_date = chrono::DateTime::from_timestamp(start, 0)
        .map(|t| t.format("%B %d").to_string())
        .unwrap_or_default();
    let end_date = chrono::DateTime::from_timestamp(end, 0)
        .map(|t| t.format("%B %d").to_string())
        .unwrap_or_default();

    println!();
    println!("{}", "📋 Weekly Alignment Report".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!("  Week of {} – {}", start_date, end_date);
    println!();

    let commits: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM events WHERE domain = 'git' AND action = 'commit'
         AND timestamp > ?1 AND timestamp < ?2",
        params![start, end], |r| r.get(0)
    ).unwrap_or(0);

    let focus_violations: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM alignment_checks WHERE conflicts LIKE '%focus%'
         AND checked_at > ?1 AND checked_at < ?2",
        params![start, end], |r| r.get(0)
    ).unwrap_or(0);

    let deploys: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM events WHERE domain = 'deploy'
         AND timestamp > ?1 AND timestamp < ?2",
        params![start, end], |r| r.get(0)
    ).unwrap_or(0);

    println!("  {:<30} {}", "Commits:".dimmed(), commits.to_string().bright_white());
    println!("  {:<30} {}", "Deploys:".dimmed(), deploys.to_string().bright_white());
    println!("  {:<30} {}", "Focus violations:".dimmed(),
        if focus_violations > 0 {
            focus_violations.to_string().bright_yellow()
        } else {
            focus_violations.to_string().bright_green()
        });

    println!();
    if commits >= 20 {
        println!("  {} Strongest alignment: ship consistently ({} commits)", "→".bright_green(), commits);
    }
    if focus_violations > 2 {
        println!("  {} Weakest: focus discipline ({} violations)", "→".bright_yellow(), focus_violations);
    }
    println!();

    Ok(())
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
