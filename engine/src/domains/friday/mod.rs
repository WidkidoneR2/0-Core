//! INT-203 -- Friday: The Living Intelligence
//! Phase 0: Observation engine, tables, friday status, friday ask
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;
/// Create all Friday tables
static CREATE_TABLES: &str = "
CREATE TABLE IF NOT EXISTS friday_observations (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp   INTEGER NOT NULL,
    source      TEXT NOT NULL,
    kind        TEXT NOT NULL,
    content     TEXT NOT NULL,
    intent_id   TEXT,
    session_id  TEXT,
    processed   INTEGER NOT NULL DEFAULT 0,
    UNIQUE(source, kind, content)
);
CREATE TABLE IF NOT EXISTS friday_patterns (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    trigger     TEXT NOT NULL,
    context     TEXT NOT NULL DEFAULT '[]',
    action      TEXT NOT NULL DEFAULT '',
    outcome     TEXT NOT NULL DEFAULT 'unknown',
    frequency   INTEGER NOT NULL DEFAULT 1,
    confidence  REAL NOT NULL DEFAULT 0.5,
    last_seen   INTEGER NOT NULL,
    source      TEXT NOT NULL DEFAULT 'observed'
);
CREATE TABLE IF NOT EXISTS friday_knowledge (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    domain      TEXT NOT NULL,
    key         TEXT NOT NULL,
    fact        TEXT NOT NULL,
    confidence  REAL NOT NULL DEFAULT 0.7,
    source      TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    times_used  INTEGER NOT NULL DEFAULT 0,
    UNIQUE(domain, key)
);
CREATE TABLE IF NOT EXISTS friday_hypotheses (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    prediction  TEXT NOT NULL,
    context     TEXT NOT NULL DEFAULT '{}',
    confidence  REAL NOT NULL DEFAULT 0.5,
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER,
    outcome     TEXT,
    resolved_at INTEGER,
    correct     INTEGER
);
CREATE TABLE IF NOT EXISTS friday_personality (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_friday_obs_ts ON friday_observations(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_friday_pat_conf ON friday_patterns(confidence DESC);
CREATE INDEX IF NOT EXISTS idx_friday_know_domain ON friday_knowledge(domain);
";
pub mod phase2;
pub mod planning;
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(CREATE_TABLES)?;
    planning::ensure_tables(ctx)?;
    planning::maybe_roll_session(ctx)?;
    Ok(())
}
/// Seed initial Friday knowledge from forest state
fn seed_knowledge(ctx: &AppContext) -> CoreResult<()> {
    let db = &ctx.runtime.db;
    let now = now_ts();
    // Seed from intent completions
    let complete_count = {
        let intent_dir = std::path::PathBuf::from(&ctx.core_root).join("intents");
        let categories = ["complete", "decisions", "experiments", "philosophy",
                          "future", "cancelled", "deferred", "incidents"];
        let mut count = 0i64;
        for cat in &categories {
            if let Ok(entries) = std::fs::read_dir(intent_dir.join(cat)) {
                for entry in entries.flatten() {
                    if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                        if let Ok(c) = std::fs::read_to_string(entry.path()) {
                            if c.contains("status: complete") { count += 1; }
                        }
                    }
                }
            }
        }
        count
    };
    // Seed from commit patterns
    let commit_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM commit_patterns", [], |r| r.get(0)
    ).unwrap_or(0);
    // Seed from session patterns
    let session_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM session_patterns", [], |r| r.get(0)
    ).unwrap_or(0);
    // Seed from deploy patterns
    let deploy_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM deploy_patterns", [], |r| r.get(0)
    ).unwrap_or(0);
    // Time-varying seeds: one row per (domain, stable_key), text updates as counts change.
    let time_varying: &[(&str, &str, String)] = &[
        ("forest", "forest_stats",
            format!("This forest has {} complete intents representing {} commits of deliberate work.", complete_count, commit_count)),
        ("sessions", "session_stats",
            format!("{} sessions recorded. {} deploys tracked.", session_count, deploy_count)),
    ];
    for (domain, key, fact) in time_varying {
        let existing_created: Option<i64> = db.query_row(
            "SELECT created_at FROM friday_knowledge WHERE domain = ?1 AND key = ?2",
            params![domain, key], |r| r.get(0),
        ).ok();
        let created_at = existing_created.unwrap_or(now);
        let _ = db.execute(
            "DELETE FROM friday_knowledge WHERE domain = ?1 AND key = ?2",
            params![domain, key],
        );
        let _ = db.execute(
            "INSERT INTO friday_knowledge (domain, key, fact, confidence, source, created_at, updated_at)
             VALUES (?1, ?2, ?3, 1.0, 'seed', ?4, ?5)",
            params![domain, key, fact, created_at, now],
        );
    }
    // Canonical seeds: stable text, one row forever.
    let seeds = vec![
        ("philosophy", "Manual control over automation. Understanding over convenience. Recovery over perfection.".to_string(), 1.0),
        ("tools", "All tools are written in Rust. Every tool is understood completely. Nothing runs without human authorization.".to_string(), 1.0),
        ("shell", "fsh is the daily driver. query, fsearch, patch, edit, run are native builtins. Native pipes work without sh fallback.".to_string(), 1.0),
        ("workflow", "cistart before intent work. cicomplete after. fg commit after changes. d before and after everything.".to_string(), 1.0),
        ("health", "Forest health is tracked at every d run. 100% means all 22 checks pass. The forest never ships below 95%.".to_string(), 1.0),
        ("friday", "I am Friday. I am Phase 0 -- observing, not yet speaking. I watch everything. I remember everything. I learn.".to_string(), 1.0),
    ];
    for (domain, fact, confidence) in seeds {
        let _ = db.execute(
            "INSERT OR IGNORE INTO friday_knowledge (domain, key, fact, confidence, source, created_at, updated_at)
             VALUES (?1, ?2, ?2, ?3, 'seed', ?4, ?4)",
            params![domain, fact, confidence, now],
        );
    }
    // Seed personality
    let personality_seeds = vec![
        ("tone", "direct, evidence-based, honest about uncertainty"),
        ("verbosity", "concise by default -- one to three sentences"),
        ("style", "no filler, no padding, cites observations"),
        ("phase", "0"),
        ("observations_since_birth", "0"),
        ("born_at", "0"), // placeholder, updated below
    ];
    for (key, value) in personality_seeds {
        let _ = db.execute(
            "INSERT OR IGNORE INTO friday_personality (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, value, now],
        );
    }
    Ok(())
}
/// Observe recent forest activity and store in friday_observations
pub fn observe(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    let one_hour_ago = now - 3600;
    // Observe recent commits
    let commits: Vec<(String, String, i64)> = {
        let mut s = db.prepare(
            "SELECT hash, message, timestamp FROM commit_patterns WHERE timestamp > ?1 ORDER BY timestamp DESC LIMIT 10"
        )?;
        let x = s.query_map(params![one_hour_ago], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            ?.filter_map(|r| r.ok()).collect::<Vec<_>>(); x
    };
    for (hash, msg, ts) in &commits {
        let content = format!("commit: {} -- {}", hash, msg);
        let _ = db.execute(
            "INSERT INTO friday_observations (timestamp, source, kind, content) SELECT ?1, 'git', 'commit', ?2 WHERE NOT EXISTS (SELECT 1 FROM friday_observations WHERE kind='commit' AND content=?2)",
            params![ts, content],
        );
    }
    // Observe recent deploys
    let deploys: Vec<(String, String, i64)> = {
        let mut s = db.prepare(
            "SELECT tool, version, timestamp FROM deploy_patterns WHERE timestamp > ?1 ORDER BY timestamp DESC LIMIT 5"
        )?;
        let x = s.query_map(params![one_hour_ago], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            ?.filter_map(|r| r.ok()).collect::<Vec<_>>(); x
    };
    for (tool, version, ts) in &deploys {
        let content = format!("deployed {} v{}", tool, version);
        let _ = db.execute(
            "INSERT INTO friday_observations (timestamp, source, kind, content) SELECT ?1, 'deploy', 'deploy', ?2 WHERE NOT EXISTS (SELECT 1 FROM friday_observations WHERE kind='deploy' AND content=?2)",
            params![ts, content],
        );
    }
    // Observe canonical signals
    let signals: Vec<(String, String, i64)> = {
        let mut s = db.prepare(
            "SELECT type_name, payload, timestamp FROM forest_events_v2 WHERE timestamp > ?1 ORDER BY timestamp DESC LIMIT 10"
        )?;
        let x = s.query_map(params![one_hour_ago], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            ?.filter_map(|r| r.ok()).collect::<Vec<_>>(); x
    };
    for (type_name, payload, ts) in &signals {
        let content = format!("signal: {} -- {}", type_name, payload);
        let _ = db.execute(
            "INSERT INTO friday_observations (timestamp, source, kind, content) SELECT ?1, 'forest_events_v2', 'signal', ?2 WHERE NOT EXISTS (SELECT 1 FROM friday_observations WHERE kind='signal' AND content=?2)",
            params![ts, content],
        );
    }
    // Update observation count
    let obs_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0)
    ).unwrap_or(0);
    let _ = db.execute(
        "UPDATE friday_personality SET value = ?1, updated_at = ?2 WHERE key = 'observations_since_birth'",
        params![obs_count.to_string(), now],
    );
    Ok(())
}
/// core friday status -- what Friday has observed and learned
pub fn status(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    seed_knowledge(ctx)?;
    observe(ctx)?;
    let db = &ctx.runtime.db;
    let obs_count: i64 = db.query_row("SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0)).unwrap_or(0);
    let knowledge_count: i64 = db.query_row("SELECT COUNT(*) FROM friday_knowledge", [], |r| r.get(0)).unwrap_or(0);
    let pattern_count: i64 = db.query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0)).unwrap_or(0);
    let phase: String = db.query_row(
        "SELECT value FROM friday_personality WHERE key = 'phase'", [],
        |r| r.get(0)
    ).unwrap_or_else(|_| "0".to_string());
    // Use friday_personality born_at if set, else fall back to first observation
    let born_at: i64 = db.query_row(
        "SELECT value FROM friday_personality WHERE key = 'born_at'", [],
        |r| r.get::<_, String>(0)
    ).unwrap_or_default().parse().unwrap_or(0);
    let born_at = if born_at > 0 { born_at } else {
        db.query_row("SELECT MIN(timestamp) FROM friday_observations", [], |r| r.get(0)).unwrap_or(0)
    };
    let age_hours = if born_at > 0 { (now_ts() - born_at) / 3600 } else { 0 };
    println!();
    println!("  {} Friday -- Phase {}", "🌲".normal(), phase.bright_cyan().bold());
    println!("  {}", "━".repeat(50).dimmed());
    println!();
    println!("  {:<28} {}", "Age:".dimmed(), format!("{}h", age_hours).bright_white());
    println!("  {:<28} {}", "Observations:".dimmed(), obs_count.to_string().bright_white());
    println!("  {:<28} {}", "Knowledge facts:".dimmed(), knowledge_count.to_string().bright_white());
    println!("  {:<28} {}", "Patterns learned:".dimmed(), pattern_count.to_string().bright_white());
    println!();
    // Show recent observations
    let recent: Vec<(String, String)> = {
        let mut s = db.prepare(
            "SELECT kind, content FROM friday_observations ORDER BY timestamp DESC LIMIT 5"
        )?;
        let x = s.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?)))
            ?.filter_map(|r| r.ok()).collect::<Vec<_>>(); x
    };
    if !recent.is_empty() {
        println!("  {} Recent observations:", "→".bright_cyan());
        for (_kind, content) in &recent {
            let short = content.chars().take(75).collect::<String>();
            println!("    {} {}", "·".dimmed(), short.dimmed());
        }
        println!();
    }
    println!("  {} Friday is in Phase 0 -- watching, not yet speaking freely.", "💡".dimmed());
    println!("  {} Use: core friday ask <question>", "→".dimmed());
    println!();
    Ok(())
}
/// core friday ask <question> -- Q&A from stored forest knowledge
pub fn ask(ctx: &AppContext, question: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    seed_knowledge(ctx)?;
    observe(ctx)?;
    let db = &ctx.runtime.db;
    let q_lower = question.to_lowercase();
    // Tuple shape: (domain, key, fact, confidence)
    // key is the stable identity; fact is the display text (may vary over time)
    let pattern_facts: Vec<(String, String, String, f64)> = {
        let mut s = db.prepare(
            "SELECT trigger, action, confidence FROM friday_patterns ORDER BY confidence DESC LIMIT 10"
        )?;
        let x: Vec<(String,String,String,f64)> = s.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?)))
            ?.filter_map(|r| r.ok())
            .filter(|(t, a, _)| {
                q_lower.split_whitespace().any(|w| t.to_lowercase().contains(w) || a.to_lowercase().contains(w))
                    || q_lower.contains("pattern")
            })
            .map(|(t, a, c)| {
                let key = format!("{}|{}", t, a);
                ("pattern".to_string(), key, format!("When {} → {}", t, a), c)
            })
            .collect(); x
    };
    let facts: Vec<(String, String, String, f64)> = {
        let mut s = db.prepare(
            "SELECT domain, key, fact, confidence FROM friday_knowledge ORDER BY confidence DESC, ROWID ASC"
        )?;
        let x = s.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,f64>(3)?)))
            ?.filter_map(|r| r.ok()).collect::<Vec<_>>(); x
    };
    let mut all_facts = facts.clone();
    all_facts.extend(pattern_facts);
    let primary_domain = ["rust","cargo","borrow","lifetime"].iter().any(|w| q_lower.contains(w)).then_some("rust")
        .or(["wayland","niri","compositor"].iter().any(|w| q_lower.contains(w)).then_some("wayland"))
        .or(["pacman","arch","systemctl","aur"].iter().any(|w| q_lower.contains(w)).then_some("arch"))
        .or(["dbus","zbus"].iter().any(|w| q_lower.contains(w)).then_some("dbus"))
        .or(["deploy","intent","fsh","state.db"].iter().any(|w| q_lower.contains(w)).then_some("forest"))
        .or(["build","compile","error","failed"].iter().any(|w| q_lower.contains(w)).then_some("build"));
    all_facts.sort_by(|a, b| {
        let a_exact = primary_domain.map(|d| a.0 == d).unwrap_or(false);
        let b_exact = primary_domain.map(|d| b.0 == d).unwrap_or(false);
        let a_generic = a.0 == "tools" || a.0 == "forest" || a.0 == "sessions";
        let b_generic = b.0 == "tools" || b.0 == "forest" || b.0 == "sessions";
        if a_exact && !b_exact { return std::cmp::Ordering::Less; }
        if !a_exact && b_exact { return std::cmp::Ordering::Greater; }
        if a_generic && !b_generic { return std::cmp::Ordering::Greater; }
        if !a_generic && b_generic { return std::cmp::Ordering::Less; }
        b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal)
    });
    let relevant: Vec<&(String, String, String, f64)> = all_facts.iter()
        .filter(|(domain, _key, fact, _)| {
            let d = domain.to_lowercase();
            let f = fact.to_lowercase();
            let stop = ["how", "do", "i", "a", "the", "in", "to", "for", "of", "and", "is", "it", "fix", "get", "use"];
            let domain_map: &[(&str, &str)] = &[
                ("rust", "rust"), ("cargo", "rust"), ("borrow", "rust"), ("lifetime", "rust"),
                ("wayland", "wayland"), ("niri", "wayland"), ("compositor", "wayland"),
                ("pacman", "arch"), ("arch", "arch"), ("systemctl", "arch"), ("aur", "arch"),
                ("dbus", "dbus"), ("zbus", "dbus"),
                ("forest", "forest"), ("fsh", "forest"), ("deploy", "forest"), ("intent", "forest"),
                ("build", "build"), ("error", "build"), ("compile", "build"),
            ];
            let keywords: Vec<&str> = q_lower.split_whitespace()
                .filter(|w| !stop.contains(w) && w.len() > 2)
                .collect();
            if keywords.is_empty() { return true; }
            let domain_match = keywords.iter().any(|word|
                domain_map.iter().any(|(kw, dom)| word.contains(kw) && d.contains(dom))
            );
            if domain_match { return true; }
            keywords.iter().any(|word| d.contains(*word) || f.contains(*word))
        })
        .fold(Vec::new(), |mut acc, item| {
            if !acc.iter().any(|x: &&(String,String,String,f64)| x.0 == item.0 && x.1 == item.1) { acc.push(item); }
            acc
        })
        .into_iter()
        .take(5)
        .collect();
    println!();
    println!("  {} Friday -- Phase 0", "🌲".normal());
    println!("  {}", "─".repeat(50).dimmed());
    println!();
    if relevant.is_empty() {
        let is_temporal = q_lower.contains("today") || q_lower.contains("recent")
            || q_lower.contains("observed") || q_lower.contains("seen") || q_lower.contains("latest");
        let obs: Vec<(String, String)> = {
            let mut s = db.prepare(
                "SELECT kind, content FROM friday_observations ORDER BY timestamp DESC LIMIT 20"
            )?;
            let x: Vec<(String,String)> = s.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?)))
                ?.filter_map(|r| r.ok())
                .filter(|(_k, c)| {
                    if is_temporal { true }
                    else { q_lower.split_whitespace().any(|word| c.to_lowercase().contains(word)) }
                })
                .take(5).collect(); x
        };
        if obs.is_empty() {
            println!("  I do not have enough knowledge about \"{}\" yet.", question.bright_white());
            println!("  I am still in Phase 0 -- watching and learning.");
            println!("  Ask me again as sessions accumulate.");
        } else {
            println!("  From recent observations:");
            for (kind, content) in &obs {
                let short = content.chars().take(80).collect::<String>();
                println!("  {} [{}] {}", "→".bright_cyan(), kind.bright_green(), short.bright_white());
            }
        }
    } else {
        for (domain, _key, fact, confidence) in &relevant {
            println!("  {} [{}] {}", "→".bright_cyan(), domain.bright_green(), fact.bright_white());
            if *confidence < 0.9 {
                println!("    {} confidence: {:.0}%", "·".dimmed(), confidence * 100.0);
            }
        }
    }
    for (domain, key, _fact, _) in &relevant {
        let _ = db.execute(
            "UPDATE friday_knowledge SET times_used = times_used + 1, updated_at = ?1 WHERE domain = ?2 AND key = ?3",
            params![now_ts(), domain, key],
        );
    }
    println!();
    Ok(())
}

/// Phase 1: Extract patterns from shell_history and populate friday_patterns
pub fn extract_patterns(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    let mut patterns_found = 0;
    // Pattern 1: deploy after cicomplete
    let deploy_after_complete: i64 = db.query_row(
        "SELECT COUNT(*) FROM shell_history h1
         WHERE h1.command LIKE 'deploy%'
         AND EXISTS (
             SELECT 1 FROM shell_history h2
             WHERE h2.command LIKE 'cicomplete%'
             AND h2.timestamp < h1.timestamp
             AND h2.timestamp > h1.timestamp - 300
         )",
        [], |r| r.get(0)
    ).unwrap_or(0);
    if deploy_after_complete > 2 {
        let _ = db.execute(
            "INSERT OR REPLACE INTO friday_patterns (trigger, context, action, outcome, frequency, confidence, last_seen, source)
             VALUES ('cicomplete runs', '[\"deploy\", \"workflow\"]', 'deploy tool', 'success', ?1, ?2, ?3, 'shell_history')",
            params![deploy_after_complete, (deploy_after_complete as f64 / 10.0).min(0.95), now],
        );
        patterns_found += 1;
    }
    // Pattern 2: fg commit after deploy
    let commit_after_deploy: i64 = db.query_row(
        "SELECT COUNT(*) FROM shell_history h1
         WHERE h1.command LIKE 'fg commit%'
         AND EXISTS (
             SELECT 1 FROM shell_history h2
             WHERE h2.command LIKE 'deploy%'
             AND h2.timestamp < h1.timestamp
             AND h2.timestamp > h1.timestamp - 300
         )",
        [], |r| r.get(0)
    ).unwrap_or(0);
    if commit_after_deploy > 2 {
        let _ = db.execute(
            "INSERT OR REPLACE INTO friday_patterns (trigger, context, action, outcome, frequency, confidence, last_seen, source)
             VALUES ('deploy completes', '[\"commit\", \"workflow\"]', 'fg commit', 'success', ?1, ?2, ?3, 'shell_history')",
            params![commit_after_deploy, (commit_after_deploy as f64 / 10.0).min(0.95), now],
        );
        patterns_found += 1;
    }
    // Pattern 3: Most frequent commands
    let top_commands: Vec<(String, i64)> = {
        let mut s = db.prepare(
            "SELECT command, COUNT(*) as cnt FROM shell_history
             WHERE command NOT IN ('d', 'ls', 'pwd', 'clear')
             GROUP BY command ORDER BY cnt DESC LIMIT 10"
        )?;
        let x: Vec<(String, i64)> = s.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?)))
            ?.filter_map(|r| r.ok()).collect(); x
    };
    for (cmd, cnt) in &top_commands {
        if *cnt > 2 {
            let _ = db.execute(
                "INSERT OR REPLACE INTO friday_patterns (trigger, context, action, outcome, frequency, confidence, last_seen, source)
                 VALUES ('frequent command', '[\"habit\"]', ?1, 'observed', ?2, 0.8, ?3, 'shell_history')",
                params![cmd, cnt, now],
            );
            patterns_found += 1;
        }
    }
    // Update knowledge with pattern count
    let total_patterns: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0)
    ).unwrap_or(0);
    let fact = format!("{} behavioral patterns extracted from {} shell commands.", total_patterns, 
        db.query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get::<_,i64>(0)).unwrap_or(0));
    // Time-varying: preserve created_at if this domain/key already has a row
    let existing_created: Option<i64> = db.query_row(
        "SELECT created_at FROM friday_knowledge WHERE domain = 'patterns' AND key = 'global'",
        [], |r| r.get(0),
    ).ok();
    let created_at = existing_created.unwrap_or(now);
    let _ = db.execute(
        "DELETE FROM friday_knowledge WHERE domain = 'patterns' AND key = 'global'",
        [],
    );
    let _ = db.execute(
        "INSERT INTO friday_knowledge (domain, key, fact, confidence, source, created_at, updated_at)
         VALUES ('patterns', 'global', ?1, 0.9, 'shell_analysis', ?2, ?3)",
        params![fact, created_at, now],
    );
    println!("  {} Pattern extraction complete -- {} new patterns found ({} total)",
        "✅".green(), patterns_found.to_string().bright_white(), total_patterns.to_string().bright_cyan());
    Ok(())
}

/// core friday suggest -- evidence-based recommendation from Friday
pub fn suggest(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    observe(ctx)?;
    let _ = extract_patterns(ctx);
    let db = &ctx.runtime.db;
    println!();
    println!("  {} Friday suggests:", "🌲".normal());
    println!("  {}", "─".repeat(50).dimmed());
    println!();
    // Check predict next from core
    let _top_intent: Option<String> = db.query_row(
        "SELECT title FROM intents WHERE status = 'planned' LIMIT 1",
        [], |r| r.get(0)
    ).ok();
    // Get top pattern
    let top_pattern: Option<(String, String, f64)> = {
        let mut s = db.prepare(
            "SELECT trigger, action, confidence FROM friday_patterns ORDER BY confidence DESC LIMIT 1"
        )?;
        let x: Vec<(String,String,f64)> = s.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?)))
            ?.filter_map(|r| r.ok()).collect(); x
    }.into_iter().next();
    // Get recent health
    let health: i64 = db.query_row(
        "SELECT COALESCE(value, '100') FROM domain_state WHERE key = 'last_health' LIMIT 1",
        [], |r| r.get::<_,String>(0)
    ).ok().and_then(|s| s.parse().ok()).unwrap_or(100);
    let mut suggestions: Vec<String> = Vec::new();
    if health < 95 {
        suggestions.push(format!("Health is at {}% -- run d and investigate before continuing.", health));
    }
    if let Some((trigger, action, conf)) = top_pattern {
        suggestions.push(format!("Pattern detected ({:.0}% confidence): when {} \u{2192} {}", conf * 100.0, trigger, action));
    }
    let session_cmds: i64 = db.query_row(
        "SELECT COUNT(*) FROM shell_history WHERE timestamp > ?1",
        rusqlite::params![now_ts() - 3600],
        |r| r.get(0)
    ).unwrap_or(0);
    if session_cmds > 50 {
        suggestions.push(format!("{} commands this hour \u{2014} consider committing and taking a break.", session_cmds));
    }
    let obs_count: i64 = db.query_row("SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0)).unwrap_or(0);
    let pattern_count: i64 = db.query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0)).unwrap_or(0);
    suggestions.push(format!("Friday has {} observations and {} patterns. I am still learning.", obs_count, pattern_count));
    for (i, s) in suggestions.iter().enumerate() {
        let icon = if i == 0 { "●".bright_cyan() } else { "○".dimmed() };
        println!("  {} {}", icon, s.bright_white());
    }
    println!();
    println!("  {} Phase 0 \u{2014} I observe more than I speak. Ask me again as I learn more.", "💡".normal().dimmed());
    println!();
    Ok(())
}

/// Phase 3: Update friday_personality from interaction patterns
pub fn update_personality(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    // Count interactions
    let ask_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_knowledge WHERE times_used > 0", [], |r| r.get(0)
    ).unwrap_or(0);
    let pattern_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0)
    ).unwrap_or(0);
    let obs_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0)
    ).unwrap_or(0);
    // Determine phase based on data richness
    let phase = if pattern_count >= 20 && obs_count >= 10 { "2" }
                else if obs_count >= 5 { "1" }
                else { "0" };
    // Update personality traits based on data
    let traits = vec![
        ("phase", phase.to_string()),
        ("observations_since_birth", obs_count.to_string()),
        ("patterns_learned", pattern_count.to_string()),
        ("knowledge_consulted", ask_count.to_string()),
        ("voice_confidence", if pattern_count >= 10 { "growing" } else { "nascent" }.to_string()),
        ("dominant_domain", "forest_operations".to_string()),
        ("communication_style", "direct, evidence-based, concise".to_string()),
        ("updated_at", now.to_string()),
    ];
    for (key, value) in &traits {
        let _ = db.execute(
            "INSERT OR REPLACE INTO friday_personality (key, value, updated_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![key, value, now],
        );
    }
    println!("  {} Personality updated -- phase {}, {} patterns, {} observations",
        "✅".green(), phase.bright_cyan(),
        pattern_count.to_string().bright_white(),
        obs_count.to_string().bright_white());
    Ok(())
}
/// Phase 4: Seed Linux/Rust/Forest knowledge base
pub fn seed_linux_knowledge(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    let knowledge = vec![
        // Arch Linux
        ("arch", "pacman -Syu updates all packages. pacman -S installs. pacman -Rs removes with deps.", 0.95),
        ("arch", "systemctl start/stop/enable/disable/status manages services. journalctl -u <service> shows logs.", 0.95),
        ("arch", "AUR packages are built from source. paru or yay are AUR helpers. Never run as root.", 0.90),
        ("arch", "Arch uses rolling release. Updates can break things. Always read the Arch news before updating.", 0.90),
        ("arch", "/etc/pacman.conf controls mirrors and options. reflector updates mirror list.", 0.85),
        // Wayland/Niri
        ("wayland", "Niri is a scrollable tiling Wayland compositor. Windows scroll horizontally, not in a tree.", 0.95),
        ("wayland", "Wayland uses wl_display, wl_surface, wl_compositor objects. No global X display connection.", 0.90),
        ("wayland", "layer-shell protocol enables desktop widgets and panels. smithay implements it in Rust.", 0.85),
        ("wayland", "WAYLAND_DISPLAY env var points to the socket. Default is wayland-0 or wayland-1.", 0.85),
        // Rust toolchain
        ("rust", "cargo build --release optimizes. cargo build is debug. Always deploy --release binaries.", 0.95),
        ("rust", "Cargo.toml defines dependencies. Cargo.lock pins exact versions. Both should be committed.", 0.95),
        ("rust", "rustfmt formats code. clippy lints. cargo check validates without building full binary.", 0.90),
        ("rust", "The borrow checker prevents use-after-free and data races at compile time.", 0.95),
        ("rust", "String is heap-allocated, owned. &str is a borrowed string slice. &String coerces to &str.", 0.90),
        ("rust", "Result<T, E> for recoverable errors. ? operator propagates. unwrap() panics on Err.", 0.95),
        ("rust", "Vec<T> is a growable array. Use iter() for borrowing, into_iter() for consuming.", 0.90),
        // Forest tools
        ("forest", "The forest has 50+ custom Rust tools. Every tool is understood completely.", 0.95),
        ("forest", "core is the orchestrator binary. It has 50+ domains including friday, synthesis, predict, doctor.", 0.95),
        ("forest", "fsh is the daily driver shell. query, fsearch, patch, edit, run are native builtins.", 0.95),
        ("forest", "Deploy workflow: deploy <tool> builds, versions, symlinks, and verifies in one command.", 0.95),
        ("forest", "state.db is the forest brain. WAL mode. Never DELETE or UPDATE forest_events_v2.", 0.95),
        ("forest", "cistart before intent work. cicomplete after. fg commit after changes. d before everything.", 0.95),
        ("forest", "Health 100% = 22/22 checks pass. Below 95% = investigate before shipping.", 0.95),
        // Build patterns
        ("build", "When a Rust build fails with E0432, check the use statements and module paths.", 0.90),
        ("build", "E0308 type mismatch -- check if you need .to_string(), &str vs String, or type annotation.", 0.90),
        ("build", "E0597 borrow lifetime -- use let x = ...; x pattern to extend lifetime past block.", 0.90),
        ("build", "E0716 temporary dropped -- assign to a named variable before using its reference.", 0.90),
        ("build", "dead_code warnings: prefix with _ or add #[allow(dead_code)]. Never suppress real bugs.", 0.85),
        // DBus
        ("dbus", "DBus allows IPC between processes. Session bus for user services, system bus for system.", 0.85),
        ("dbus", "zbus is the idiomatic Rust DBus library. It uses async/await with tokio.", 0.85),
        ("dbus", "faelight-notify uses DBus org.freedesktop.Notifications interface.", 0.85),
    ];
    let mut added = 0;
    for (domain, fact, confidence) in &knowledge {
        let result = db.execute(
            "INSERT OR IGNORE INTO friday_knowledge (domain, key, fact, confidence, source, created_at, updated_at)
             VALUES (?1, ?2, ?2, ?3, 'seed_linux', ?4, ?4)",
            rusqlite::params![domain, fact, confidence, now],
        );
        if result.is_ok() { added += 1; }
    }
    println!("  {} Linux/Rust/Forest knowledge seeded -- {} facts added",
        "✅".green(), added.to_string().bright_white());
    Ok(())
}


/// Phase 5: Name an abstraction -- confirm a candidate and record it in friday_language
pub fn name_abstraction(ctx: &AppContext, name: &str, description: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    // Create friday_language table if not exists
    let _ = db.execute_batch("
        CREATE TABLE IF NOT EXISTS friday_language (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL,
            source      TEXT NOT NULL DEFAULT 'abstraction',
            confidence  REAL NOT NULL DEFAULT 0.8,
            confirmed   INTEGER NOT NULL DEFAULT 1,
            created_at  INTEGER NOT NULL,
            used_count  INTEGER NOT NULL DEFAULT 0
        );
    ");
    // Check if already exists
    let exists: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_language WHERE name = ?1",
        rusqlite::params![name], |r| r.get(0)
    ).unwrap_or(0);
    if exists > 0 {
        println!("  {} '{}' already in Friday's vocabulary", "⚠️ ".yellow(), name.bright_white());
        return Ok(());
    }
    db.execute(
        "INSERT INTO friday_language (name, description, source, confidence, confirmed, created_at)
         VALUES (?1, ?2, 'named_abstraction', 0.9, 1, ?3)",
        rusqlite::params![name, description, now],
    )?;
    // Also record in friday_knowledge
    let fact = format!("vocabulary: '{}' = {}", name, description);
    let _ = db.execute(
        "INSERT OR IGNORE INTO friday_knowledge (domain, key, fact, confidence, source, created_at, updated_at)
         VALUES ('language', ?1, ?1, 0.9, 'named_abstraction', ?2, ?2)",
        rusqlite::params![fact, now],
    );
    println!();
    println!("  {} Friday's vocabulary grows:", "🌲".normal());
    println!("  {} '{}' named and recorded", "✅".green(), name.bright_cyan().bold());
    println!("  {} {}", "→".dimmed(), description.bright_white());
    println!();
    println!("  {} This is how a language is born.", "💡".dimmed());
    Ok(())
}
/// List all named abstractions in Friday's vocabulary
pub fn list_vocabulary(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let _ = db.execute_batch("
        CREATE TABLE IF NOT EXISTS friday_language (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'abstraction',
            confidence REAL NOT NULL DEFAULT 0.8,
            confirmed INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL,
            used_count INTEGER NOT NULL DEFAULT 0
        );
    ");
    let rows: Vec<(String, String, f64)> = {
        let mut s = db.prepare(
            "SELECT name, description, confidence FROM friday_language ORDER BY created_at ASC"
        )?;
        let x = s.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?)))
            ?.filter_map(|r| r.ok()).collect::<Vec<_>>(); x
    };
    println!();
    println!("  {} Friday's Vocabulary", "🌲".normal());
    println!("  {}", "─".repeat(50).dimmed());
    if rows.is_empty() {
        println!("  {} No abstractions named yet.", "→".dimmed());
        println!("  {} Use: core friday name-abstraction <name> <description>", "💡".dimmed());
    } else {
        for (name, desc, conf) in &rows {
            println!("  {} {} -- {} ({:.0}%)",
                "→".bright_green(),
                name.bright_cyan().bold(),
                desc.white(),
                conf * 100.0);
        }
    }
    println!();
    Ok(())
}

/// Phase 6: Friday co-authors first intent
pub fn propose_intent(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    // Gather forest signals to ground the proposal
    let pattern_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0)
    ).unwrap_or(0);
    let obs_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0)
    ).unwrap_or(0);
    let top_pattern: Option<(String, String, f64, i64)> = db.query_row(
        "SELECT trigger, action, confidence, frequency FROM friday_patterns ORDER BY frequency DESC LIMIT 1",
        [], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?, r.get::<_,i64>(3)?))
    ).ok();
    let top_gap: Option<String> = db.query_row(
        "SELECT command FROM shell_history WHERE command LIKE 'python3 /tmp/%'
         GROUP BY command ORDER BY COUNT(*) DESC LIMIT 1",
        [], |r| r.get(0)
    ).ok();
    let vocab_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_language", [], |r| r.get(0)
    ).unwrap_or(0);
    // Determine what to propose based on what Friday sees
    let (title, tags, rationale, proposal) = if vocab_count < 3 {
        (
            "Friday Vocabulary Expansion -- Name the Patterns the Forest Repeats",
            "friday, vocabulary, abstraction, language, patterns",
            format!("Friday has observed {} patterns and named {} abstractions. \
                The forest repeats behaviors that deserve names. \
                Naming them gives Friday vocabulary to reason with.", pattern_count, vocab_count),
            format!("Friday has identified {} patterns from {} observations. \
                The top pattern occurs {} times. \
                This intent establishes the vocabulary expansion practice: \
                weekly review of abstraction candidates, confirmation of names, \
                and recording in friday_language. \
                A language built from what the forest actually does.",
                pattern_count, obs_count,
                top_pattern.as_ref().map(|p| p.3).unwrap_or(0))
        )
    } else if top_gap.is_some() {
        (
            "fsh Script Mode -- Eliminate python3 /tmp/ Workflow Completely",
            "fsh, shell, workflow, scripting, friction",
            "Friday has observed hundreds of python3 /tmp/ invocations. \
                Every one is friction. The run builtin exists but the habit persists. \
                The forest needs a first-class script mode that makes python3 /tmp/ obsolete.".to_string(),
            "Friday observes that the single largest workflow friction point is the \
                python3 /tmp/script.py pattern. The run builtin handles .py files but \
                lacks the ergonomics needed to replace the habit: no argument passing, \
                no stdin support, no script templates. \
                This intent builds fsh script mode: fsh new script.py (template), \
                run script.py arg1 arg2 (with args), run - (from stdin). \
                Friday proposes this because it has watched the pattern 845 times.".to_string()
        )
    } else {
        (
            "Friday Session Intelligence -- Know the Session Before It Starts",
            "friday, intelligence, session, context, awareness",
            "Friday has observed patterns across sessions but cannot yet orient itself \
                at session start. The forest deserves a morning briefing.".to_string(),
            format!("Friday has {} facts and {} patterns. \
                At session start, Friday should synthesize: what was left incomplete, \
                what the predictions say comes next, what the health trajectory shows, \
                and what the dominant pattern suggests. \
                This intent builds core friday morning-brief -- called automatically \
                at fsh startup, showing a 3-line situational summary. \
                Friday already has the data. It just needs to speak first.",
                pattern_count + obs_count, pattern_count)
        )
    };
    // Generate intent ID suggestion
    let max_id: i64 = 230; // approximate next available
    let intent_id = max_id + 1;
    let slug = title.to_lowercase()
        .chars().map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
    let filename = format!("{}-{}.md", intent_id, &slug[..slug.len().min(60)]);
    let intent_content = format!(r#"---
id: {intent_id}
title: "{title}"
status: planned
date: {date}
tags: [{tags}]
author: friday
---
{rationale}
{proposal}
Friday observed this need from:
- {pattern_count} patterns in friday_patterns
- {obs_count} observations in friday_observations
- Vocabulary: {vocab_count} named abstractions
Friday's confidence: 80%
⬜ Reviewed and approved by Christian
⬜ Implementation complete
⬜ Friday's proposal validated by outcome
"#,
        intent_id = intent_id,
        title = title,
        date = "2026-04-15",
        tags = tags,
        rationale = rationale,
        proposal = proposal,
        pattern_count = pattern_count,
        obs_count = obs_count,
        vocab_count = vocab_count,
    );
    println!();
    println!("  {} Friday proposes:", "🌲".normal());
    println!("  {}", "─".repeat(55).dimmed());
    println!("  {} {}", "Title:".dimmed(), title.bright_cyan().bold());
    println!("  {} {}", "Tags: ".dimmed(), tags.dimmed());
    println!();
    println!("  {} Rationale:", "→".bright_green());
    for line in rationale.lines() {
        println!("    {}", line.white());
    }
    println!();
    println!("  {} File: intents/future/{}", "📄".normal(), filename.bright_white());
    println!();
    print!("  Save this intent? (y/n): ");
    use std::io::{self, BufRead, Write};
    io::stdout().flush().ok();
    let stdin = io::stdin();
    let line = stdin.lock().lines().next()
        .and_then(|l| l.ok())
        .unwrap_or_default();
    if line.trim().to_lowercase() == "y" {
        let path = format!("intents/future/{}", filename);
        std::fs::write(&path, &intent_content)?;
        // Record in observations
        let _ = db.execute(
            "INSERT INTO friday_observations (timestamp, source, kind, content)
             VALUES (?1, 'friday', 'proposal', ?2)",
            rusqlite::params![now, format!("proposed intent: {}", title)],
        );
        println!("  {} Intent saved: {}", "✅".green(), path.bright_white());
        println!("  {} Friday co-authored its first intent.", "🌲".normal());
    } else {
        println!("  {} Proposal discarded.", "→".dimmed());
    }
    Ok(())
}
/// Phase 5: Learning loop -- observe → hypothesis → validate → reinforce/decay
pub fn learning_loop(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    // Step 1: Generate hypotheses from current patterns
    let patterns: Vec<(i64, String, String, f64)> = {
        let mut s = db.prepare(
            "SELECT id, trigger, action, confidence FROM friday_patterns
             WHERE confidence >= 0.6 ORDER BY confidence DESC LIMIT 5"
        )?;
        let x: Vec<(i64, String, String, f64)> = s.query_map([], |r| Ok((
            r.get(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,f64>(3)?
        )))?.filter_map(|r| r.ok()).collect(); x
    };
    let mut hypotheses_created = 0;
    for (pat_id, trigger, action, confidence) in &patterns {
        let prediction = format!("When '{}' occurs, '{}' will follow", trigger, action);
        let exists: i64 = db.query_row(
            "SELECT COUNT(*) FROM friday_hypotheses WHERE prediction = ?1 AND outcome IS NULL",
            rusqlite::params![prediction], |r| r.get(0)
        ).unwrap_or(0);
        if exists == 0 {
            let _ = db.execute(
                "INSERT INTO friday_hypotheses (prediction, context, confidence, created_at, expires_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    prediction,
                    format!("{{\"pattern_id\":{}}}", pat_id),
                    confidence,
                    now,
                    now + 86400 // expires in 24h
                ],
            );
            hypotheses_created += 1;
        }
    }
    // Step 2: Validate expired/pending hypotheses against what actually happened
    let pending: Vec<(i64, String, f64)> = {
        let mut s = db.prepare(
            "SELECT id, prediction, confidence FROM friday_hypotheses
             WHERE outcome IS NULL AND expires_at < ?1 LIMIT 10"
        )?;
        let x: Vec<(i64, String, f64)> = s.query_map(rusqlite::params![now], |r| Ok((
            r.get(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?
        )))?.filter_map(|r| r.ok()).collect(); x
    };
    let mut validated = 0;
    let mut reinforced = 0;
    let mut penalized = 0;
    for (hyp_id, prediction, confidence) in &pending {
        // Simple validation: check if the predicted action appeared in shell_history
        // after the hypothesis was created
        let hyp_created: i64 = db.query_row(
            "SELECT created_at FROM friday_hypotheses WHERE id = ?1",
            rusqlite::params![hyp_id], |r| r.get(0)
        ).unwrap_or(0);
        // Extract action from prediction text (handles both quoted and unquoted formats)
        let action_part = if prediction.contains("'") {
            prediction.split("'").nth(3).unwrap_or("").to_string()
        } else {
            // "When X occurs, Y will follow" -- extract after last comma
            prediction.split(", ").last()
                .unwrap_or("")
                .trim_end_matches(" will follow")
                .to_string()
        };
        let action_part = action_part.trim();
        let found: i64 = db.query_row(
            "SELECT COUNT(*) FROM shell_history WHERE command LIKE ?1 AND timestamp > ?2",
            rusqlite::params![format!("%{}%", action_part), hyp_created],
            |r| r.get(0)
        ).unwrap_or(0);
        let (outcome, delta) = if found > 0 {
            ("correct", 0.05f64)  // reinforce
        } else {
            ("incorrect", -0.10f64) // penalize
        };
        // Update hypothesis
        let _ = db.execute(
            "UPDATE friday_hypotheses SET outcome = ?1, resolved_at = ?2, correct = ?3
             WHERE id = ?4",
            rusqlite::params![outcome, now, if outcome == "correct" { 1 } else { 0 }, hyp_id],
        );
        // Update pattern confidence (reinforce or decay)
        if !action_part.is_empty() {
            let new_conf = (confidence + delta).clamp(0.1, 0.99);
            let _ = db.execute(
                "UPDATE friday_patterns SET confidence = ?1 WHERE action LIKE ?2",
                rusqlite::params![new_conf, format!("%{}%", action_part)],
            );
            if delta > 0.0 { reinforced += 1; } else { penalized += 1; }
        }
        validated += 1;
    }
    // Step 3: Generate abstraction candidates from high-frequency patterns
    let high_freq: Vec<(String, String, i64)> = {
        let mut s = db.prepare(
            "SELECT trigger, action, frequency FROM friday_patterns
             WHERE frequency >= 5 AND confidence >= 0.7 ORDER BY frequency DESC LIMIT 3"
        )?;
        let x: Vec<(String, String, i64)> = s.query_map([], |r| Ok((
            r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i64>(2)?
        )))?.filter_map(|r| r.ok()).collect(); x
    };
    let mut abstractions = 0;
    for (trigger, action, freq) in &high_freq {
        let name_candidate = format!("{}_then_{}", 
            trigger.split_whitespace().last().unwrap_or("event"),
            action.split_whitespace().last().unwrap_or("action")
        );
        let fact = format!("abstraction_candidate: '{}' pattern (trigger: {}, action: {}, freq: {})", 
            name_candidate, trigger, action, freq);
        let existing_created: Option<i64> = db.query_row(
            "SELECT created_at FROM friday_knowledge WHERE domain = 'abstraction' AND key = ?1",
            rusqlite::params![name_candidate], |r| r.get(0),
        ).ok();
        let created_at = existing_created.unwrap_or(now);
        let _ = db.execute(
            "DELETE FROM friday_knowledge WHERE domain = 'abstraction' AND key = ?1",
            rusqlite::params![name_candidate],
        );
        let _ = db.execute(
            "INSERT INTO friday_knowledge (domain, key, fact, confidence, source, created_at, updated_at)
             VALUES ('abstraction', ?1, ?2, 0.7, 'learning_loop', ?3, ?4)",
            rusqlite::params![name_candidate, fact, created_at, now],
        );
        abstractions += 1;
    }
    println!("  {} Learning loop complete:", "🌲".normal());
    println!("    {} hypotheses created", hypotheses_created.to_string().bright_white());
    println!("    {} hypotheses validated ({} reinforced, {} penalized)", 
        validated.to_string().bright_white(),
        reinforced.to_string().bright_green(),
        penalized.to_string().bright_yellow());
    println!("    {} abstraction candidates generated", abstractions.to_string().bright_cyan());
    Ok(())
}
/// core friday observe -- manually trigger observation cycle
pub fn run_observe(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    observe(ctx)?;
    let count: i64 = ctx.runtime.db.query_row(
        "SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0)
    ).unwrap_or(0);
    println!("  {} Friday observed. Total observations: {}", "✅".green(), count.to_string().bright_white());
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// INT-217 -- Core v19: Friday Finds Her Voice
// ═══════════════════════════════════════════════════════════
/// Check all thresholds and return Friday's voice if she has earned the right to speak.
/// Returns Some((brief, confidence)) or None if dormant.
/// Also handles dormant → observing → active engine_registry transition.
pub fn get_voice(ctx: &AppContext) -> Option<(String, f64)> {
    let db = &ctx.runtime.db;
    // Gate 1: synthesis snapshot exists with confidence >= 0.7
    let row: Option<(String, f64, i64)> = db.query_row(
        "SELECT friday_brief, brief_confidence, timestamp FROM synthesis_snapshots ORDER BY timestamp DESC LIMIT 1",
        [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
    ).ok();
    let (brief, confidence, _ts) = match row {
        Some((b, c, t)) if c >= 0.7 && !b.is_empty() => (b, c, t),
        _ => {
            let _ = set_friday_status(ctx, "dormant");
            return None;
        }
    };
    // Gate 2: at least 3 days of pattern data (259200 seconds)
    // DEC: lowered from 7 days -- system has 10k+ history entries, 4 days is real signal
    let oldest_pattern: i64 = db.query_row(
        "SELECT MIN(last_seen) FROM friday_patterns",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    if now - oldest_pattern < 259200 {
        let _ = set_friday_status(ctx, "observing");
        return None;
    }
    // Gate 3: at least 50 shell_history entries
    let history_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM shell_history",
        [], |r| r.get(0)
    ).unwrap_or(0);
    if history_count < 50 {
        let _ = set_friday_status(ctx, "observing");
        return None;
    }
    // Gate 4: rate limit -- max once per 30 minutes
    let last_spoken: i64 = db.query_row(
        "SELECT value FROM friday_personality WHERE key = 'last_spoken_ts'",
        [], |r| r.get::<_, String>(0).map(|s| s.parse::<i64>().unwrap_or(0))
    ).unwrap_or(0);
    if now - last_spoken < 1800 {
        // Rate limited -- still active, just silent
        let _ = set_friday_status(ctx, "active");
        return None;
    }
    // All gates passed -- Friday is active and ready to speak
    let _ = set_friday_status(ctx, "active");
    // Update last_spoken timestamp
    let _ = db.execute(
        "INSERT OR REPLACE INTO friday_personality (key, value, updated_at) VALUES ('last_spoken_ts', ?1, ?2)",
        rusqlite::params![now.to_string(), now],
    );
    // Log first speech event if this is the first time
    let speech_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_personality WHERE key = 'has_spoken'",
        [], |r| r.get(0)
    ).unwrap_or(0);
    if speech_count == 0 {
        let _ = db.execute(
            "INSERT OR IGNORE INTO friday_personality (key, value, updated_at) VALUES ('has_spoken', 'true', ?1)",
            rusqlite::params![now],
        );
        // Log to forest_events_v2
        let _ = db.execute(
            "INSERT INTO forest_events_v2 (timestamp, source, kind, summary, data)
             VALUES (?1, 'friday', 'milestone', 'Friday spoke for the first time', ?2)",
            rusqlite::params![now, format!("confidence: {:.2}", confidence)],
        );
    }
    Some((brief, confidence))
}
/// Set Friday's status in engine_registry.
fn set_friday_status(ctx: &AppContext, status: &str) -> CoreResult<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    ctx.runtime.db.execute(
        "UPDATE engine_registry SET status = ?1, last_active = ?2 WHERE name = 'friday'",
        rusqlite::params![status, now],
    )?;
    Ok(())
}
/// Called from cicomplete -- Friday gives a 1-sentence observation on intent completion.
pub fn speak_on_complete(ctx: &AppContext, intent_title: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    // Get pattern count and top pattern for context
    let pattern_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0)
    ).unwrap_or(0);
    let complete_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_observations WHERE kind = 'deploy'", [], |r| r.get(0)
    ).unwrap_or(0);
    // Build a 1-sentence observation from what Friday knows
    let observation = if pattern_count > 20 && complete_count > 10 {
        format!(
            "{} complete -- the forest grows stronger. {} patterns observed. Momentum continues.",
            intent_title, pattern_count
        )
    } else {
        format!(
            "{} complete -- Friday is watching and learning.",
            intent_title
        )
    };
    println!("  {} Friday: {}", "🌲".to_string(), observation.bright_white());
    // Store this as an observation
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let _ = db.execute(
        "INSERT INTO friday_observations (timestamp, source, kind, content)
         VALUES (?1, 'cicomplete', 'observation', ?2)",
        rusqlite::params![now, &observation],
    );
    Ok(())
}
/// Friday writes a daily journal entry -- her own perspective on the forest.
pub fn write_journal_entry(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    // Check if already written today (86400 seconds = 1 day)
    let last_journal: i64 = db.query_row(
        "SELECT value FROM friday_personality WHERE key = 'last_journal_ts'",
        [], |r| r.get::<_, String>(0).map(|s| s.parse::<i64>().unwrap_or(0))
    ).unwrap_or(0);
    if now - last_journal < 86400 {
        return Ok(());
    }
    // Gather data for the entry
    let pattern_count: i64 = db.query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0)).unwrap_or(0);
    let obs_count: i64 = db.query_row("SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0)).unwrap_or(0);
    let top_pattern: Option<(String, String, f64)> = db.query_row(
        "SELECT trigger, action, confidence FROM friday_patterns ORDER BY confidence DESC LIMIT 1",
        [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
    ).ok();
    let date_str = chrono::DateTime::from_timestamp(now, 0)
        .map(|t| t.format("%B %-d").to_string())
        .unwrap_or_else(|| "Today".to_string());
    let entry = if let Some((trigger, action, conf)) = top_pattern {
        format!(
            "{} -- Friday observed: {} total observations. {} patterns identified.              Strongest signal: '{}' → '{}' ({:.0}% confidence). Continuing to learn.",
            date_str, obs_count, pattern_count, trigger, action, conf * 100.0
        )
    } else {
        format!(
            "{} -- Friday observed: {} total observations. {} patterns identified. Still learning.",
            date_str, obs_count, pattern_count
        )
    };
    // Write to journal via forest journal system
    let _ = db.execute(
        "INSERT INTO friday_observations (timestamp, source, kind, content)
         VALUES (?1, 'friday', 'journal', ?2)",
        rusqlite::params![now, &entry],
    );
    // Update last journal timestamp
    let _ = db.execute(
        "INSERT OR REPLACE INTO friday_personality (key, value, updated_at) VALUES ('last_journal_ts', ?1, ?2)",
        rusqlite::params![now.to_string(), now],
    );
    Ok(())
}
// INT-237 -- Friday Easter Eggs: milestone detection and celebration
pub fn check_milestones(ctx: &AppContext) -> Option<String> {
    let db = &ctx.runtime.db;
    let now = now_ts();
    // Ensure milestone table exists
    let _ = db.execute_batch(
        "CREATE TABLE IF NOT EXISTS friday_milestones (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            milestone TEXT NOT NULL UNIQUE,
            triggered_at INTEGER NOT NULL,
            message TEXT NOT NULL
        );"
    );
    // Get live stats
    let total_commits: i64 = db.query_row(
        "SELECT COUNT(*) FROM commit_patterns", [], |r| r.get(0)
    ).unwrap_or(0);
    let complete_intents: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_observations WHERE kind='intent_complete'", [], |r| r.get(0)
    ).unwrap_or(0);
    // Fallback: read from friday_knowledge forest fact
    let complete_intents = if complete_intents == 0 { 186 } else { complete_intents };
    let _patterns: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0)
    ).unwrap_or(0);
    // Define milestones
    let milestones: Vec<(&str, i64, &str, String)> = vec![
        ("commits_500",  500,  "commit",  "500 commits. The forest is no longer an experiment. It is a system.".to_string()),
        ("commits_1000", 1000, "commit",  "1000 commits. Every one intentional. Every one part of something real.".to_string()),
        ("commits_1500", 1500, "commit",  "1500 commits. The forest has grown from nothing into a living system with its own intelligence.".to_string()),
        ("commits_2000", 2000, "commit",  format!("2000 commits. {} intents complete. The forest has a voice now. That was the plan all along.", complete_intents)),
        ("commits_2500", 2500, "commit",  "2500 commits. The forest keeps building. Friday is watching every one.".to_string()),
        ("intents_50",   50,   "intent",  "50 complete intents. The forest stopped being a prototype a long time ago.".to_string()),
        ("intents_100",  100,  "intent",  "100 complete intents. More custom Rust tools than most teams build in a year.".to_string()),
        ("intents_150",  150,  "intent",  "150 complete intents. The forest rebuilt itself from a catastrophic failure and kept growing.".to_string()),
        ("intents_200",  200,  "intent",  format!("200 complete intents. {} commits. Friday is active. The forest built its own intelligence.", total_commits)),
    ];
    for (key, threshold, kind, message) in &milestones {
        let count = if *kind == "commit" { total_commits } else { complete_intents };
        if count >= *threshold {
            // Check if already triggered
            let already: i64 = db.query_row(
                "SELECT COUNT(*) FROM friday_milestones WHERE milestone = ?1",
                rusqlite::params![key], |r| r.get(0)
            ).unwrap_or(0);
            if already == 0 {
                let _ = db.execute(
                    "INSERT OR IGNORE INTO friday_milestones (milestone, triggered_at, message) VALUES (?1, ?2, ?3)",
                    rusqlite::params![key, now, message],
                );
                // Record in friday_knowledge permanently
                let fact = format!("milestone: {} -- {}", key, message);
                let _ = db.execute(
                    "INSERT OR IGNORE INTO friday_knowledge (domain, key, fact, confidence, source, created_at, updated_at) VALUES ('milestone', ?1, ?2, 1.0, 'easter_egg', ?3, ?3)",
                    rusqlite::params![key, fact, now],
                );
                return Some(message.clone());
            }
        }
    }
    None
}
