//! INT-218 -- Friday Knowledge Engine
//! Situated Learning and Conflict Resolution
//! The forest has made every mistake once.
//! Friday makes sure it only needs to make each mistake once.
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
static INIT_TABLES: &str = "
CREATE TABLE IF NOT EXISTS knowledge_entries (
    id              TEXT PRIMARY KEY,
    source          TEXT NOT NULL DEFAULT 'manual',
    domain          TEXT NOT NULL,
    error_signature TEXT,
    description     TEXT NOT NULL,
    resolution      TEXT NOT NULL,
    confidence      REAL NOT NULL DEFAULT 0.5,
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    success_count   INTEGER NOT NULL DEFAULT 0,
    failure_count   INTEGER NOT NULL DEFAULT 0,
    last_seen       INTEGER NOT NULL,
    created_at      INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS knowledge_outcomes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id    TEXT NOT NULL,
    outcome     TEXT NOT NULL,
    was_correct INTEGER NOT NULL,
    recorded_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_knowledge_domain ON knowledge_entries(domain);
CREATE INDEX IF NOT EXISTS idx_knowledge_sig ON knowledge_entries(error_signature);
";
pub fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(INIT_TABLES)?;
    Ok(())
}
/// Normalize an error signature -- strip file paths, line numbers, variable names
///
/// Show full entry with resolution history
pub fn show(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let row: Option<(String, String, Option<String>, String, String, f64, i64, i64, i64)> = db.query_row(
        "SELECT id, domain, error_signature, description, resolution, confidence, occurrence_count, success_count, failure_count
         FROM knowledge_entries WHERE id = ?1",
        params![id], |r| Ok((
            r.get::<_,String>(0)?, r.get::<_,String>(1)?,
            r.get::<_,Option<String>>(2)?, r.get::<_,String>(3)?,
            r.get::<_,String>(4)?, r.get::<_,f64>(5)?,
            r.get::<_,i64>(6)?, r.get::<_,i64>(7)?, r.get::<_,i64>(8)?
        ))
    ).ok();
    match row {
        None => println!("  No entry found: {}", id),
        Some((id, domain, sig, desc, resolution, conf, count, success, failure)) => {
            println!();
            println!(
                "  {} [{}] {}",
                "🌲".normal(),
                domain.bright_cyan(),
                id.dimmed()
            );
            println!("  {}", "─".repeat(55).dimmed());
            println!("  {} {}", "Problem:".dimmed(), desc.bright_white());
            if let Some(s) = sig {
                println!("  {} {}", "Signature:".dimmed(), s.dimmed());
            }
            println!();
            println!("  {} {}", "Resolution:".bright_green(), resolution.white());
            println!();
            println!(
                "  {} {:.0}%  {} seen  {} correct  {} incorrect",
                "Confidence:".dimmed(),
                conf * 100.0,
                count.to_string().bright_white(),
                success.to_string().bright_green(),
                failure.to_string().bright_red()
            );
            // Show outcome history
            let outcomes: Vec<(String, i64, i64)> = {
                let mut s = db.prepare(
                    "SELECT outcome, was_correct, recorded_at FROM knowledge_outcomes
                     WHERE entry_id = ?1 ORDER BY recorded_at DESC LIMIT 5",
                )?;
                let x: Vec<(String, i64, i64)> = s
                    .query_map(params![&id], |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, i64>(1)?,
                            r.get::<_, i64>(2)?,
                        ))
                    })?
                    .filter_map(|r| r.ok())
                    .collect();
                x
            };
            if !outcomes.is_empty() {
                println!();
                println!("  {} Outcome history:", "→".dimmed());
                for (outcome, correct, ts) in &outcomes {
                    let icon = if *correct == 1 {
                        "✅".to_string()
                    } else {
                        "❌".to_string()
                    };
                    let time = chrono::DateTime::from_timestamp(*ts, 0)
                        .map(|t| t.format("%m/%d %H:%M").to_string())
                        .unwrap_or_default();
                    println!("    {} {} -- {}", icon, outcome.white(), time.dimmed());
                }
            }
            println!();
        }
    }
    Ok(())
}
/// Record outcome for a knowledge entry
pub fn record_outcome(ctx: &AppContext, id: &str, correct: bool) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    // Check entry exists
    let exists: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM knowledge_entries WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if exists == 0 {
        println!("  {} Entry not found: {}", "✗".bright_red(), id);
        return Ok(());
    }
    // Record outcome
    db.execute(
        "INSERT INTO knowledge_outcomes (entry_id, outcome, was_correct, recorded_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            id,
            if correct { "correct" } else { "incorrect" },
            if correct { 1 } else { 0 },
            now
        ],
    )?;
    // Update confidence and counts
    if correct {
        db.execute(
            "UPDATE knowledge_entries SET success_count = success_count + 1,
             confidence = MIN(0.99, confidence + 0.02), occurrence_count = occurrence_count + 1,
             last_seen = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        println!(
            "  {} Outcome recorded -- confidence increased",
            "✅".green()
        );
    } else {
        db.execute(
            "UPDATE knowledge_entries SET failure_count = failure_count + 1,
             confidence = MAX(0.3, confidence - 0.10), occurrence_count = occurrence_count + 1,
             last_seen = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        println!(
            "  {} Outcome recorded -- confidence reduced (learning from failure)",
            "⚠️ ".yellow()
        );
    }
    Ok(())
}
#[allow(dead_code)]
pub fn normalize_signature(error: &str) -> String {
    let mut s = error.to_string();
    // Strip file paths
    while let Some(start) = s.find("-->") {
        if let Some(end) = s[start..].find('\n') {
            s.replace_range(start..start + end, "");
        } else {
            break;
        }
    }
    // Strip line:col references
    let re_like = |s: &str| -> String {
        s.split_whitespace()
            .filter(|w| !w.contains(':') || w.starts_with("error") || w.starts_with("warning"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    // Keep error code and message, strip specifics
    if let Some(bracket_end) = s.find("]: ") {
        let code = &s[..bracket_end + 3];
        let msg = &s[bracket_end + 3..];
        // Normalize variable/type names in backticks
        let mut normalized_msg = String::new();
        let mut in_backtick = false;
        let mut placeholder_count = 0;
        for ch in msg.chars() {
            if ch == '`' {
                in_backtick = !in_backtick;
                if in_backtick {
                    placeholder_count += 1;
                    normalized_msg.push_str(&format!("`T{}`", placeholder_count));
                }
            } else if !in_backtick {
                normalized_msg.push(ch);
            }
        }
        let result = format!("{}{}", code, normalized_msg.trim());
        return result.chars().take(200).collect();
    }
    re_like(&s).chars().take(200).collect()
}
/// Seed the knowledge engine with hard-won forest lessons
pub fn seed_forest_lessons(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    let lessons: Vec<(&str, &str, Option<&str>, &str, &str, f64)> = vec![
        // (id, domain, error_sig, description, resolution, confidence)
        ("rust_e0597_lifetime",
         "rust",
         Some("error[E0597]"),
         "E0597: borrowed value does not live long enough",
         "Assign to named variable before using reference: `let x = expr; use &x` or `let x = ...; x` at end of block",
         0.99),
        ("rust_e0716_temporary",
         "rust",
         Some("error[E0716]"),
         "E0716: temporary value dropped while borrowed",
         "Assign temporary to named let binding to extend lifetime past the block",
         0.99),
        ("rust_e0277_missing_debug",
         "rust",
         Some("error[E0277]"),
         "E0277: missing Debug/Clone/Subcommand derive on enum or struct",
         "Add #[derive(Debug, Clone, Subcommand)] to enum. For clap: also need #[command(subcommand)] on field",
         0.99),
        ("rust_unicode_corruption",
         "rust",
         None,
         "String replacement in Rust files with Python unicode escapes causes build corruption",
         "Never use \\u{XXXX} or \\uXXXX in Python strings generating Rust code. Use literal UTF-8 characters. If corrupted: git -C ~/0-core checkout engine/src/path/to/file.rs then redeploy",
         0.99),
        ("rust_double_unwrap",
         "rust",
         None,
         "rusqlite double unwrap_or causes type inference error",
         "Use single unwrap_or_else or unwrap_or. Check: query_row returns Result<T>, not Option<T>",
         0.95),
        ("rust_clap_subcommand",
         "rust",
         None,
         "Clap derive subcommand not recognized at runtime",
         "Enum needs #[derive(Debug, Clone, Subcommand)]. Field needs #[command(subcommand)]. Both required.",
         0.99),
        ("fsh_multiline_python",
         "fsh",
         None,
         "python3 -c with multiline code fails in fsh",
         "Write script to /tmp/script.py then run python3 /tmp/script.py. Or use `run script.py` builtin.",
         0.99),
        ("fsh_heredoc_backtick",
         "fsh",
         None,
         "Heredoc with unquoted delimiter expands backticks in content",
         "Always use quoted delimiter: << 'PYEOF' not << PYEOF. Content with backticks will corrupt otherwise.",
         0.99),
        ("fsh_echo_redirect",
         "fsh",
         None,
         "echo 'text' > /tmp/file fails in fsh with redirect error",
         "Use python3 to write files: python3 -c \"open('/tmp/file','w').write('text')\". Or write script to /tmp/ first.",
         0.95),
        ("build_order_core",
         "build",
         None,
         "Core domain build order -- wrong order causes missing symbol errors",
         "Always: mod.rs → commands.rs → parser.rs → cli/mod.rs → dispatcher.rs → domains/mod.rs → deploy core",
         0.99),
        ("build_deploy_time",
         "build",
         None,
         "deploy core takes 13-15 seconds. If under 3 seconds, Rust did not recompile",
         "If deploy seems instant, check that you actually saved the file. cargo only recompiles changed files.",
         0.95),
        ("build_rspatch_double",
         "fsh",
         None,
         "rspatch double-replace: new content contains anchor text",
         "Never let --new content contain --anchor text. The anchor will match inside the new content on next run.",
         0.99),
        ("arch_pacman_conflict",
         "arch",
         None,
         "pacman update conflict: file exists in filesystem",
         "Run: sudo pacman -Syu --overwrite '*' only if you understand what is being overwritten. Usually safe for minor updates.",
         0.85),
        ("niri_config_reload",
         "niri",
         None,
         "Niri config changes not taking effect",
         "Run: niri msg action reload-config. If niri crashes, check ~/.config/niri/config.kdl syntax.",
         0.95),
        ("statedb_wal_mode",
         "database",
         None,
         "state.db WAL mode required for concurrent access",
         "Run: PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; on state.db. Already done in forest setup.",
         0.99),
        ("git_checkout_recovery",
         "git",
         None,
         "File corrupted or accidentally modified during session",
         "Run: git -C ~/0-core checkout engine/src/path/to/file.rs to restore from last commit. Then redeploy.",
         0.99),
        ("python_anchor_mismatch",
         "python",
         None,
         "Python string anchor not found when patching Rust files",
         "Check for em dash vs double dash mismatch. Use repr() to debug: print(repr(content[idx:idx+80])). Also check whitespace differences.",
         0.95),
    ];
    let mut seeded = 0;
    for (id, domain, error_sig, description, resolution, confidence) in &lessons {
        let exists: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM knowledge_entries WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if exists == 0 {
            let _ = db.execute(
                "INSERT INTO knowledge_entries
                 (id, source, domain, error_signature, description, resolution, confidence, occurrence_count, last_seen, created_at)
                 VALUES (?1, 'forest_lesson', ?2, ?3, ?4, ?5, ?6, 1, ?7, ?7)",
                params![id, domain, error_sig, description, resolution, confidence, now],
            );
            seeded += 1;
        }
    }
    println!(
        "  {} Knowledge engine seeded -- {} lessons loaded",
        "✅".green(),
        seeded
    );
    Ok(())
}
/// Search knowledge base by term
pub fn search(ctx: &AppContext, term: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let pattern = format!("%{}%", term.to_lowercase());
    let rows: Vec<(String, String, String, String, f64, i64)> = {
        let mut s = db.prepare(
            "SELECT id, domain, description, resolution, confidence, occurrence_count
             FROM knowledge_entries
             WHERE LOWER(description) LIKE ?1 OR LOWER(resolution) LIKE ?1
                OR LOWER(domain) LIKE ?1 OR LOWER(error_signature) LIKE ?1
             ORDER BY confidence DESC LIMIT 5",
        )?;
        let x: Vec<(String, String, String, String, f64, i64)> = s
            .query_map(params![pattern], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, f64>(4)?,
                    r.get::<_, i64>(5)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    println!();
    println!(
        "  {} Knowledge Search: {}",
        "🌲".normal(),
        term.bright_cyan()
    );
    println!("  {}", "─".repeat(55).dimmed());
    if rows.is_empty() {
        println!("  {} No known lessons for '{}'", "→".dimmed(), term);
        println!(
            "  {} This may be a new pattern -- consider: core knowledge add",
            "💡".dimmed()
        );
    } else {
        for (_id, domain, desc, resolution, conf, count) in &rows {
            println!();
            println!(
                "  {} [{}] {} ({:.0}% · seen {}x)",
                "→".bright_green(),
                domain.bright_cyan(),
                desc.chars().take(60).collect::<String>().bright_white(),
                conf * 100.0,
                count
            );
            println!(
                "  {} {}",
                "fix:".dimmed(),
                resolution.chars().take(120).collect::<String>().white()
            );
        }
    }
    println!();
    Ok(())
}
/// Show all patterns by domain
pub fn patterns(ctx: &AppContext, domain: Option<&str>) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let rows: Vec<(String, String, String, f64, i64)> = {
        let mut s = if let Some(d) = domain {
            db.prepare(&format!(
                "SELECT id, domain, description, confidence, occurrence_count
                 FROM knowledge_entries WHERE domain = '{}' ORDER BY confidence DESC",
                d
            ))?
        } else {
            db.prepare(
                "SELECT id, domain, description, confidence, occurrence_count
                 FROM knowledge_entries ORDER BY domain, confidence DESC",
            )?
        };
        let x: Vec<(String, String, String, f64, i64)> = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, f64>(3)?,
                    r.get::<_, i64>(4)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    println!();
    println!(
        "  {} Knowledge Patterns{}",
        "🌲".normal(),
        domain
            .map(|d| format!(" -- {}", d))
            .unwrap_or_default()
            .bright_cyan()
    );
    println!("  {}", "─".repeat(55).dimmed());
    let mut current_domain = String::new();
    for (id, dom, desc, conf, count) in &rows {
        if *dom != current_domain {
            current_domain = dom.clone();
            println!();
            println!("  {} {}", "▸".bright_green(), dom.bright_white().bold());
        }
        println!(
            "    [{}] {} ({:.0}% · {}x)",
            id.chars().take(20).collect::<String>().dimmed(),
            desc.chars().take(55).collect::<String>().white(),
            conf * 100.0,
            count
        );
    }
    println!();
    Ok(())
}
/// Show accuracy by domain
pub fn accuracy(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let rows: Vec<(String, i64, f64)> = {
        let mut s = db.prepare(
            "SELECT domain, COUNT(*) as entries, AVG(confidence) as avg_conf
             FROM knowledge_entries GROUP BY domain ORDER BY avg_conf DESC",
        )?;
        let x: Vec<(String, i64, f64)> = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, f64>(2)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    println!();
    println!("  {} Knowledge Accuracy by Domain", "🌲".normal());
    println!("  {}", "─".repeat(40).dimmed());
    for (domain, count, conf) in &rows {
        let bar_len = (*conf * 20.0) as usize;
        let bar = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);
        let colored = if *conf >= 0.9 {
            bar.bright_green().to_string()
        } else if *conf >= 0.7 {
            bar.bright_yellow().to_string()
        } else {
            bar.bright_red().to_string()
        };
        println!(
            "  {:<12} {} {:.0}% ({} entries)",
            domain.bright_white(),
            colored,
            conf * 100.0,
            count
        );
    }
    println!();
    Ok(())
}
/// Add a new lesson manually
pub fn add(ctx: &AppContext, domain: &str, description: &str, resolution: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    let id = format!("manual_{}_{}", domain, now);
    db.execute(
        "INSERT INTO knowledge_entries
         (id, source, domain, description, resolution, confidence, occurrence_count, last_seen, created_at)
         VALUES (?1, 'manual', ?2, ?3, ?4, 0.8, 1, ?5, ?5)",
        params![id, domain, description, resolution, now],
    )?;
    println!(
        "  {} Lesson recorded: [{}] {}",
        "✅".green(),
        domain.bright_cyan(),
        description.white()
    );
    Ok(())
}
/// Query knowledge for a specific error -- used by build pipeline
#[allow(dead_code)]
pub fn query_for_error(ctx: &AppContext, error: &str) -> Option<(String, String, f64)> {
    let db = &ctx.runtime.db;
    let sig = normalize_signature(error);
    let pattern = format!("%{}%", &sig[..sig.len().min(50)]);
    db.query_row(
        "SELECT description, resolution, confidence FROM knowledge_entries
         WHERE error_signature LIKE ?1 OR LOWER(description) LIKE ?2
         ORDER BY confidence DESC LIMIT 1",
        params![
            pattern,
            format!("%{}%", &error[..error.len().min(30)].to_lowercase())
        ],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, f64>(2)?,
            ))
        },
    )
    .ok()
}
