//! deploy domain -- INT-222 Deploy Intelligence v2
//! `core deploy check <tool>` -- pre-deploy health gate + dependency warning
//! `core deploy record <tool> <version> <outcome> <duration_ms>` -- log pattern + emit signal
//! core deploy log            -- recent deploy history
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
static CREATE_TABLE: &str = "
CREATE TABLE IF NOT EXISTS deploy_patterns (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp       INTEGER NOT NULL,
    tool            TEXT NOT NULL,
    version         TEXT NOT NULL DEFAULT '',
    outcome         TEXT NOT NULL DEFAULT 'success',
    duration_ms     INTEGER NOT NULL DEFAULT 0,
    health_before   INTEGER NOT NULL DEFAULT 100,
    active_intents  TEXT NOT NULL DEFAULT '',
    git_commit      TEXT NOT NULL DEFAULT ''
);";
/// `core deploy check <tool>` -- pre-deploy gate
pub fn check(ctx: &AppContext, tool: &str) -> CoreResult<()> {
    let db = &ctx.runtime.db;
    // Ensure table exists
    db.execute_batch(CREATE_TABLE)?;
    // Read health from cache
    let health_cache = std::fs::read_to_string(
        std::path::Path::new(&std::env::var("HOME").unwrap_or_default())
            .join(".cache/faelight/health-status"),
    )
    .unwrap_or_else(|_| "100".to_string());
    let health: i64 = health_cache.trim().parse().unwrap_or(100);
    println!();
    println!(
        "  {} pre-deploy check: {}",
        "🔍".normal(),
        tool.bright_cyan()
    );
    // Health gate
    if health < 95 {
        println!(
            "  {} health: {}% -- below 95% threshold",
            "⚠️ ".yellow(),
            health.to_string().bright_red()
        );
        println!("  {} run d to check before deploying", "→".dimmed());
    } else {
        println!(
            "  {} health: {}%",
            "✅".normal(),
            health.to_string().bright_green()
        );
    }
    // Check for uncommitted changes
    let git_status = std::process::Command::new("git")
        .args(["-C", &ctx.core_root, "status", "--porcelain"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    if !git_status.is_empty() {
        println!(
            "  {} uncommitted changes -- commit before deploying for clean version label",
            "⚠️ ".yellow()
        );
    } else {
        println!("  {} working tree clean", "✅".normal());
    }
    // Recent failure check
    let recent_failures: i64 = db.query_row(
        "SELECT COUNT(*) FROM deploy_patterns WHERE tool = ?1 AND outcome != 'success' AND timestamp > ?2",
        rusqlite::params![tool, chrono::Utc::now().timestamp() - 86400],
        |r: &rusqlite::Row| r.get::<_, i64>(0),
    ).unwrap_or(0);
    if recent_failures >= 2 {
        println!(
            "  {} {} failures in last 24h for {} -- proceed with care",
            "⚠️ ".yellow(),
            recent_failures,
            tool.bright_cyan()
        );
    }
    // Dependency awareness
    let deps = tool_dependencies(tool);
    if !deps.is_empty() {
        println!(
            "  {} downstream: {}",
            "→".bright_cyan(),
            deps.join(", ").dimmed()
        );
    }
    println!();
    Ok(())
}
/// `core deploy record <tool> <version> <outcome> <duration_ms>` -- log to state.db
pub fn record(
    ctx: &AppContext,
    tool: &str,
    version: &str,
    outcome: &str,
    duration_ms: i64,
) -> CoreResult<()> {
    let _intent = std::env::var("DEPLOY_INTENT").unwrap_or_default();
    let db = &ctx.runtime.db;
    db.execute_batch(CREATE_TABLE)?;
    let now = chrono::Utc::now().timestamp();
    // Read health
    let health: i64 = std::fs::read_to_string(
        std::path::Path::new(&std::env::var("HOME").unwrap_or_default())
            .join(".cache/faelight/health-status"),
    )
    .unwrap_or_else(|_| "100".to_string())
    .trim()
    .parse()
    .unwrap_or(100);
    // Read active intents from db
    let active_intents: String = db
        .query_row(
            "SELECT GROUP_CONCAT(id) FROM intents WHERE status = 'in-progress'",
            [],
            |r: &rusqlite::Row| r.get::<_, Option<String>>(0),
        )
        .unwrap_or(None)
        .unwrap_or_default();

    // Read git hash
    let commit = std::process::Command::new("git")
        .args(["-C", &ctx.core_root, "rev-parse", "--short", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    db.execute(
        "INSERT INTO deploy_patterns (timestamp, tool, version, outcome, duration_ms, health_before, active_intents, git_commit)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![now, tool, version, outcome, duration_ms, health, active_intents, commit],
    )?;
    // Emit engine signal
    let weight: f64 = match outcome {
        "success" => 1.0,
        "failed" => 0.3,
        _ => 0.0,
    };
    let payload = format!(
        r#"{{"tool":"{}","version":"{}","outcome":"{}","health":{}}}"#,
        tool, version, outcome, health
    );
    let _ = db.execute(
        "INSERT INTO engine_signals (source, signal_type, payload, weight, created_at) VALUES ('deploy', 'deploy', ?1, ?2, ?3)",
        rusqlite::params![payload, weight, now],
    );
    // INT-251 v23: emit to unified event bus
    let event_kind = if outcome == "success" {
        "deploy_completed"
    } else {
        "deploy_failed"
    };
    let _ = crate::domains::friday::events::emit(ctx, "deploy", event_kind, &payload, "core", None);

    let icon = if outcome == "success" { "✅" } else { "❌" };
    println!(
        "  {} deploy recorded: {} {} ({}) {}ms",
        icon,
        tool.bright_cyan(),
        version.dimmed(),
        outcome,
        duration_ms
    );
    // INT-218: on failure, Friday checks knowledge engine for known patterns
    if outcome != "success" {
        let _ = crate::domains::friday_arch::speak_on_error(ctx, &format!("build failed {}", tool));
    }
    Ok(())
}
/// core deploy log -- recent deploy history
pub fn log(ctx: &AppContext) -> CoreResult<()> {
    let db = &ctx.runtime.db;
    db.execute_batch(CREATE_TABLE)?;
    let mut stmt: rusqlite::Statement = match db.prepare(
        "SELECT tool, version, outcome, duration_ms, health_before, timestamp
         FROM deploy_patterns ORDER BY timestamp DESC LIMIT 20",
    ) {
        Ok(s) => s,
        Err(_) => {
            println!("  {} No deploy history yet", "○".dimmed());
            return Ok(());
        }
    };
    let rows: Vec<(String, String, String, i64, i64, i64)> = stmt
        .query_map([], |r: &rusqlite::Row| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
            ))
        })
        .map(|rows: rusqlite::MappedRows<_>| rows.filter_map(|x| x.ok()).collect::<Vec<_>>())
        .unwrap_or_default();
    if rows.is_empty() {
        println!("  {} No deploy history yet", "○".dimmed());
        return Ok(());
    }
    println!();
    println!("  {} Recent Deploys", "📦".normal());
    println!(
        "  {}",
        "─────────────────────────────────────────────".dimmed()
    );
    for (tool, version, outcome, duration_ms, health, ts) in rows {
        let icon = if outcome == "success" {
            "✅".to_string()
        } else {
            "❌".to_string()
        };
        let time = chrono::DateTime::from_timestamp(ts, 0)
            .map(|t| t.format("%m/%d %H:%M").to_string())
            .unwrap_or_default();
        println!(
            "  {} {:<20} {:<12} {} {}ms  health:{}%",
            icon,
            tool.bright_cyan(),
            version.dimmed(),
            time.dimmed(),
            duration_ms,
            health
        );
    }
    println!();
    Ok(())
}
/// `core deploy rollback [tool] [--dry-run]` -- restore previous version
pub fn rollback(ctx: &AppContext, tool: Option<&str>, dry_run: bool) -> CoreResult<()> {
    use colored::*;

    // REWRITTEN 2026-08-27. The old version was unusable on two counts.
    //
    // It read bin/{name}@{version} and wrote into scripts/ -- one directory was
    // empty until ship started filling it today, the other was deleted in
    // e733287d. And with no tool named it took the two most recent rows from
    // deploy_patterns ACROSS ALL TOOLS, called them current and previous, then
    // looked for a backup of one tool selected by another tool timestamp.
    //
    // VERSION IS NOT AN IDENTIFIER HERE. bin/ currently holds three backups of
    // core and all three are 3.2.10, because the version does not change
    // between builds. The deploy log records what HAPPENED; the directory
    // records what can be RESTORED, and only the second question matters to
    // rollback. So the directory is the source of truth and the key is the
    // mtime ship stamps into the filename.
    let bin_dir = faelight_core::paths::bin_dir();
    let backup_dir = std::path::PathBuf::from(&ctx.core_root).join("bin");

    let entries = match std::fs::read_dir(&backup_dir) {
        Ok(e) => e,
        Err(e) => {
            println!();
            println!(
                "  {} no backups at {} -- {}",
                "○".dimmed(),
                backup_dir.display(),
                e
            );
            println!(
                "  {} backups are written by ship on each install",
                "→".dimmed()
            );
            println!();
            return Ok(());
        }
    };

    // name@<digits>. Anything else is skipped rather than sorted wrongly --
    // older backups carry a version suffix (core@3.2.10) from before ship
    // stamped mtimes, and parsing those as numbers would order them arbitrarily.
    let mut found: Vec<(String, u64, std::path::PathBuf)> = Vec::new();
    for e in entries.filter_map(|e| e.ok()) {
        let fname = e.file_name().to_string_lossy().to_string();
        let Some((name, stamp)) = fname.rsplit_once('@') else {
            continue;
        };
        let Ok(stamp) = stamp.parse::<u64>() else {
            continue;
        };
        if let Some(t) = tool {
            if name != t {
                continue;
            }
        }
        found.push((name.to_string(), stamp, e.path()));
    }

    if found.is_empty() {
        println!();
        match tool {
            Some(t) => println!(
                "  {} no restorable backup for {}",
                "○".dimmed(),
                t.bright_cyan()
            ),
            None => println!(
                "  {} no restorable backups in {}",
                "○".dimmed(),
                backup_dir.display()
            ),
        }
        println!();
        return Ok(());
    }

    found.sort_by(|a, b| b.1.cmp(&a.1));

    if tool.is_none() {
        // No tool named: SHOW what could be restored, restore nothing. Guessing
        // which binary to replace is not a thing to do on the user behalf.
        println!();
        println!("  {} Restorable backups", "🔄".normal());
        let mut seen: Vec<&str> = Vec::new();
        for (name, stamp, _) in &found {
            if seen.contains(&name.as_str()) {
                continue;
            }
            seen.push(name.as_str());
            let when = chrono::DateTime::from_timestamp(*stamp as i64, 0)
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| stamp.to_string());
            let count = found.iter().filter(|f| &f.0 == name).count();
            println!(
                "    {:<24} {} ({} kept)",
                name.bright_cyan(),
                when.dimmed(),
                count
            );
        }
        println!();
        println!("  {} core deploy rollback <tool>", "→".dimmed());
        println!();
        return Ok(());
    }

    let (name, stamp, src) = &found[0];
    let dest = bin_dir.join(name);
    let when = chrono::DateTime::from_timestamp(*stamp as i64, 0)
        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| stamp.to_string());

    println!();
    println!("  {} Rollback: {}", "🔄".normal(), name.bright_cyan());
    println!("  from:  {}", src.display().to_string().dimmed());
    println!("  to:    {}", dest.display().to_string().dimmed());
    println!("  saved: {}", when.bright_green());
    if found.len() > 1 {
        println!(
            "  {} {} older backups also kept",
            "○".dimmed(),
            found.len() - 1
        );
    }

    if dry_run {
        println!("  {} dry-run -- no changes made", "○".dimmed());
        println!();
        return Ok(());
    }

    // Same install mechanism ship uses: stage beside the destination, then
    // rename. A copy onto a live executable gives Text file busy; the rename
    // leaves any running process on its unlinked inode.
    let tmp = bin_dir.join(format!(".{}.rollback-tmp", name));
    std::fs::copy(src, &tmp)?;
    let mut perm = std::fs::metadata(&tmp)?.permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o755);
    std::fs::set_permissions(&tmp, perm)?;
    std::fs::rename(&tmp, &dest)?;

    println!("  {} restored {}", "✅".normal(), name.bright_cyan());
    println!();
    Ok(())
}
/// `core deploy check-deps <tool>` -- show full dependency graph
pub fn check_deps(tool: &str) -> CoreResult<()> {
    use colored::*;
    println!();
    println!(
        "  {} Dependency graph: {}",
        "🔍".normal(),
        tool.bright_cyan()
    );
    println!("  {}", "─────────────────────────────────".dimmed());
    let deps = tool_dependencies(tool);
    if deps.is_empty() {
        println!("  {} No known downstream dependencies", "○".dimmed());
    } else {
        println!(
            "  {} Downstream (tools that depend on {}):",
            "→".bright_cyan(),
            tool
        );
        for dep in &deps {
            println!("    {} {}", "·".dimmed(), dep.bright_white());
        }
    }
    // Reverse -- what does this tool depend on?
    let all_tools = [
        "faelight-shell",
        "core",
        "faelight-git",
        "faelight-term",
        "faelight-update",
        "faelight-daemon",
    ];
    let mut upstream: Vec<&str> = Vec::new();
    for t in &all_tools {
        if tool_dependencies(t).contains(&tool) {
            upstream.push(t);
        }
    }
    if !upstream.is_empty() {
        println!("  {} Upstream ({} depends on):", "←".bright_yellow(), tool);
        for dep in &upstream {
            println!("    {} {}", "·".dimmed(), dep.bright_white());
        }
    }
    println!();
    Ok(())
}

fn tool_dependencies(tool: &str) -> Vec<&'static str> {
    match tool {
        "faelight-shell" => vec!["faelight-term"],
        "core" => vec!["all tools using core commands"],
        "faelight-git" => vec!["fg alias", "cistart/cicomplete hooks"],
        _ => vec![],
    }
}
