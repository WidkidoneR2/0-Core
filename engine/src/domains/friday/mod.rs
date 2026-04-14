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
    fact        TEXT NOT NULL,
    confidence  REAL NOT NULL DEFAULT 0.7,
    source      TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    times_used  INTEGER NOT NULL DEFAULT 0
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
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(CREATE_TABLES)?;
    Ok(())
}
/// Seed initial Friday knowledge from forest state
fn seed_knowledge(ctx: &AppContext) -> CoreResult<()> {
    let db = &ctx.runtime.db;
    let now = now_ts();
    // Seed from intent completions
    let complete_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM intents WHERE status = 'complete'",
        [], |r| r.get(0)
    ).unwrap_or(0);
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
    let seeds = vec![
        ("forest", format!("This forest has {} complete intents representing {} commits of deliberate work.", complete_count, commit_count), 1.0),
        ("sessions", format!("{} sessions recorded. {} deploys tracked.", session_count, deploy_count), 1.0),
        ("philosophy", "Manual control over automation. Understanding over convenience. Recovery over perfection.".to_string(), 1.0),
        ("tools", "All tools are written in Rust. Every tool is understood completely. Nothing runs without human authorization.".to_string(), 1.0),
        ("shell", "fsh is the daily driver. query, fsearch, patch, edit, run are native builtins. Native pipes work without sh fallback.".to_string(), 1.0),
        ("workflow", "cistart before intent work. cicomplete after. fg commit after changes. d before and after everything.".to_string(), 1.0),
        ("health", "Forest health is tracked at every d run. 100% means all 22 checks pass. The forest never ships below 95%.".to_string(), 1.0),
        ("friday", "I am Friday. I am Phase 0 -- observing, not yet speaking. I watch everything. I remember everything. I learn.".to_string(), 1.0),
    ];
    for (domain, fact, confidence) in seeds {
        let _ = db.execute(
            "INSERT OR IGNORE INTO friday_knowledge (domain, fact, confidence, source, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'seed', ?4, ?4)",
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
    let born_at: i64 = db.query_row(
        "SELECT value FROM friday_personality WHERE key = 'born_at'", [],
        |r| r.get::<_, String>(0)
    ).unwrap_or_default().parse().unwrap_or(0);
    let age_hours = if born_at > 0 {
        (now_ts() - born_at) / 3600
    } else { 0 };
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
        for (kind, content) in &recent {
            let short = content.chars().take(65).collect::<String>();
            println!("    {} {} {}", "·".dimmed(), kind.bright_green(), short.dimmed());
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
    // Also search friday_patterns directly
    let pattern_facts: Vec<(String, String, f64)> = {
        let mut s = db.prepare(
            "SELECT trigger, action, confidence FROM friday_patterns ORDER BY confidence DESC LIMIT 10"
        )?;
        let x: Vec<(String,String,f64)> = s.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?)))
            ?.filter_map(|r| r.ok())
            .filter(|(t, a, _)| {
                q_lower.split_whitespace().any(|w| t.to_lowercase().contains(w) || a.to_lowercase().contains(w))
                    || q_lower.contains("pattern")
            })
            .map(|(t, a, c)| ("pattern".to_string(), format!("When {} → {}", t, a), c))
            .collect(); x
    };

    // Search knowledge base
    let facts: Vec<(String, String, f64)> = {
        let mut s = db.prepare(
            "SELECT domain, fact, confidence FROM friday_knowledge ORDER BY confidence DESC"
        )?;
        let x = s.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?)))
            ?.filter_map(|r| r.ok()).collect::<Vec<_>>(); x
    };
    // Find relevant facts
    let mut all_facts = facts.clone();
    all_facts.extend(pattern_facts);
    let relevant: Vec<&(String, String, f64)> = all_facts.iter()
        .filter(|(domain, fact, _)| {
            let d = domain.to_lowercase();
            let f = fact.to_lowercase();
            q_lower.split_whitespace().any(|word| d.contains(word) || f.contains(word))
        })
        .take(3)
        .collect();
    println!();
    println!("  {} Friday -- Phase 0", "🌲".normal());
    println!("  {}", "─".repeat(50).dimmed());
    println!();
    if relevant.is_empty() {
        // Check for temporal/observation queries
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
        for (domain, fact, confidence) in &relevant {
            println!("  {} [{}] {}", "→".bright_cyan(), domain.bright_green(), fact.bright_white());
            if *confidence < 0.9 {
                println!("    {} confidence: {:.0}%", "·".dimmed(), confidence * 100.0);
            }
        }
    }
    // Update knowledge usage
    for (domain, fact, _) in &relevant {
        let _ = db.execute(
            "UPDATE friday_knowledge SET times_used = times_used + 1, updated_at = ?1 WHERE domain = ?2 AND fact = ?3",
            params![now_ts(), domain, fact],
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
    let _ = db.execute(
        "INSERT OR REPLACE INTO friday_knowledge (domain, fact, confidence, source, created_at, updated_at)
         VALUES ('patterns', ?1, 0.9, 'shell_analysis', ?2, ?2)",
        params![fact, now],
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
