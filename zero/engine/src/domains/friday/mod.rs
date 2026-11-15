//! INT-203 -- Friday: The Living Intelligence
//! Phase 0: Observation engine, tables, friday status, friday ask
pub mod attention;
pub mod events;
pub mod reasoning;
pub mod types;
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
    -- Probe rows carry an expiry. NULL on every other fact. Existing databases
    -- gain the column in ensure_knowledge_expires_at, not from this CREATE.
    expires_at  INTEGER,
    UNIQUE(domain, key)
);
-- INT-128: native/foreign/translation metadata for friday_knowledge facts.
-- Companion table (keyed on the same domain,key) rather than ALTER-patching the
-- base table -- single source of truth, fresh-db-safe (see INT-104 schema discipline).
CREATE TABLE IF NOT EXISTS friday_knowledge_meta (
    domain        TEXT NOT NULL,
    key           TEXT NOT NULL,
    system        TEXT NOT NULL DEFAULT 'unknown',  -- 'native' (this machine) | 'zero' (project) | foreign
    kind          TEXT NOT NULL DEFAULT 'fact',      -- 'fact' | 'command' | 'translation'
    translates_to TEXT,                              -- native equivalent, for foreign rows
    PRIMARY KEY (domain, key)
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
CREATE TABLE IF NOT EXISTS friday_decisions (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp    INTEGER NOT NULL,
    what         TEXT NOT NULL,
    alternatives TEXT NOT NULL DEFAULT '',
    why          TEXT NOT NULL,
    ties_to      TEXT NOT NULL DEFAULT '',
    revisit_when TEXT NOT NULL DEFAULT '',
    session_id   TEXT,
    intent_id    TEXT
);
CREATE TABLE IF NOT EXISTS friday_map (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type  TEXT NOT NULL,
    entity_name  TEXT NOT NULL,
    version      TEXT NOT NULL DEFAULT '',
    role         TEXT NOT NULL DEFAULT '',
    status       TEXT NOT NULL DEFAULT 'active',
    depends_on   TEXT NOT NULL DEFAULT '[]',
    health       REAL,
    last_commit  TEXT NOT NULL DEFAULT '',
    last_deploy  INTEGER NOT NULL DEFAULT 0,
    updated_at   INTEGER NOT NULL,
    UNIQUE(entity_type, entity_name)
);
CREATE INDEX IF NOT EXISTS idx_friday_dec_ts ON friday_decisions(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_friday_map_type ON friday_map(entity_type);
CREATE TABLE IF NOT EXISTS friday_doc_proposals (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp    INTEGER NOT NULL,
    commit_hash  TEXT NOT NULL DEFAULT '',
    doc_file     TEXT NOT NULL,
    reason       TEXT NOT NULL,
    proposed_text TEXT NOT NULL DEFAULT '',
    status       TEXT NOT NULL DEFAULT 'pending'
);
CREATE INDEX IF NOT EXISTS idx_friday_doc_status ON friday_doc_proposals(status);

";
pub mod decisions;
pub mod doc_steward;
pub mod map;
pub mod phase2;
pub mod planning;
pub mod self_review;
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(CREATE_TABLES)?;
    ensure_knowledge_expires_at(&ctx.runtime.db)?;
    ensure_pattern_identity(&ctx.runtime.db)?;
    // INT-128: keep native/foreign metadata in sync with the knowledge base.
    let _ = sync_knowledge_meta(&ctx.runtime.db);
    planning::ensure_tables(ctx)?;
    planning::maybe_roll_session(ctx)?;
    Ok(())
}

/// INT-128: keep friday_knowledge_meta in sync with friday_knowledge.
/// Self-healing central labeler: fills a meta row for any fact row that lacks one,
/// deriving system/kind from the domain. Seed-agnostic -- reads the db, so facts from
/// ANY seed path (or runtime `core knowledge add`) get labeled automatically.
///
/// Rule: a fact row is labeled kind='fact', with no translation. pacman, Hyprland,
/// and systemd are this machine, so those keys are system='native'. Every other
/// fact is system='zero'. session_summary is excluded (a session log, not knowledge).
///
/// INSERT OR IGNORE fills gaps only. Rows for those three keys that were already
/// labeled are updated to 'native' here, so an existing state.db follows the rule.
fn sync_knowledge_meta(db: &rusqlite::Connection) -> rusqlite::Result<()> {
    // Read fact rows that lack a meta row (excluding the session log).
    let mut stmt = db.prepare(
        "SELECT k.domain, k.key FROM friday_knowledge k
         LEFT JOIN friday_knowledge_meta m ON k.domain = m.domain AND k.key = m.key
         WHERE m.domain IS NULL AND k.domain != 'session_summary'",
    )?;
    let rows: Vec<(String, String)> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .filter_map(Result::ok)
        .collect();

    for (domain, key) in rows {
        let system = if is_native_machine_key(&key) {
            "native"
        } else {
            "zero"
        };
        let (kind, translates_to): (&str, Option<&str>) = ("fact", None);
        db.execute(
            "INSERT OR IGNORE INTO friday_knowledge_meta (domain, key, system, kind, translates_to)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![domain, key, system, kind, translates_to],
        )?;
    }
    db.execute(
        "UPDATE friday_knowledge_meta SET system = 'native'
         WHERE lower(key) IN ('pacman', 'hyprland', 'systemd') AND system != 'native'",
        [],
    )?;
    Ok(())
}

fn is_native_machine_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    matches!(key.as_str(), "pacman" | "hyprland" | "systemd")
}

/// One row per (trigger, action). Existing databases are deduped before the
/// unique index is added, so a second extract replaces the row instead of
/// inserting another.
fn ensure_pattern_identity(db: &rusqlite::Connection) -> CoreResult<()> {
    db.execute_batch(
        "DELETE FROM friday_patterns
         WHERE id NOT IN (
             SELECT id FROM (
                 SELECT MIN(id) AS id FROM friday_patterns GROUP BY trigger, action
             )
         );
         CREATE UNIQUE INDEX IF NOT EXISTS idx_friday_pat_trigger_action
             ON friday_patterns(trigger, action);",
    )?;
    Ok(())
}

/// Completed intent ids. Live status complete, plus every id git recorded under
/// `zero/intents/complete/`. Same source the dashboard and strategy use.
fn completed_intent_count() -> usize {
    zero_core::ledger::completed_ids().len()
}

/// Seed initial Friday knowledge from current state
fn seed_knowledge(ctx: &AppContext) -> CoreResult<()> {
    let db = &ctx.runtime.db;
    let now = now_ts();
    // Seed from intent completions
    let complete_count = completed_intent_count() as i64;
    // Seed from commit patterns
    let commit_count: i64 = db
        .query_row("SELECT COUNT(*) FROM commit_patterns", [], |r| r.get(0))
        .unwrap_or(0);
    // Seed from session patterns
    let session_count: i64 = db
        .query_row("SELECT COUNT(*) FROM session_patterns", [], |r| r.get(0))
        .unwrap_or(0);
    // Seed from deploy patterns
    let deploy_count: i64 = db
        .query_row("SELECT COUNT(*) FROM deploy_patterns", [], |r| r.get(0))
        .unwrap_or(0);
    // Time-varying seeds: one row per (domain, stable_key), text updates as counts change.
    let time_varying: &[(&str, &str, String)] = &[
        (
            "zero",
            "project_stats",
            format!(
                "Project 0 has {} complete intents representing {} commits of deliberate work.",
                complete_count, commit_count
            ),
        ),
        (
            "sessions",
            "session_stats",
            format!(
                "{} sessions recorded. {} deploys tracked.",
                session_count, deploy_count
            ),
        ),
    ];
    for (domain, key, fact) in time_varying {
        let existing_created: Option<i64> = db
            .query_row(
                "SELECT created_at FROM friday_knowledge WHERE domain = ?1 AND key = ?2",
                params![domain, key],
                |r| r.get(0),
            )
            .ok();
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
        ("workflow", "cistart before intent work. cicomplete after. fg commit after changes. d before and after everything.".to_string(), 1.0),
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
/// Observe recent activity and store in friday_observations
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
        let x = s
            .query_map(params![one_hour_ago], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        x
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
        let x = s
            .query_map(params![one_hour_ago], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        x
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
            "SELECT type_name, payload, timestamp FROM events_v2 WHERE timestamp > ?1 ORDER BY timestamp DESC LIMIT 10"
        )?;
        let x = s
            .query_map(params![one_hour_ago], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        x
    };
    for (type_name, payload, ts) in &signals {
        let content = format!("signal: {} -- {}", type_name, payload);
        let _ = db.execute(
            "INSERT INTO friday_observations (timestamp, source, kind, content) SELECT ?1, 'events_v2', 'signal', ?2 WHERE NOT EXISTS (SELECT 1 FROM friday_observations WHERE kind='signal' AND content=?2)",
            params![ts, content],
        );
    }
    // Update observation count
    let obs_count: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0))
        .unwrap_or(0);
    let _ = db.execute(
        "UPDATE friday_personality SET value = ?1, updated_at = ?2 WHERE key = 'observations_since_birth'",
        params![obs_count.to_string(), now],
    );
    Ok(())
}
/// Closing line for `core friday status`. The counted lines above are the
/// measurement. This stays empty rather than repeat a number that was not counted.
fn status_closing_line() -> &'static str {
    ""
}

/// core friday status -- what Friday has observed and learned
pub fn status(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    seed_knowledge(ctx)?;
    observe(ctx)?;
    let db = &ctx.runtime.db;
    let obs_count: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0))
        .unwrap_or(0);
    let knowledge_count: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_knowledge", [], |r| r.get(0))
        .unwrap_or(0);
    let pattern_count: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0))
        .unwrap_or(0);
    // Use friday_personality born_at if set, else fall back to first observation
    let born_at: i64 = db
        .query_row(
            "SELECT value FROM friday_personality WHERE key = 'born_at'",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_default()
        .parse()
        .unwrap_or(0);
    let born_at = if born_at > 0 {
        born_at
    } else {
        db.query_row("SELECT MIN(timestamp) FROM friday_observations", [], |r| {
            r.get(0)
        })
        .unwrap_or(0)
    };
    let age_hours = if born_at > 0 {
        (now_ts() - born_at) / 3600
    } else {
        0
    };
    println!();
    println!("  Friday -- Active Intelligence",);
    println!("  {}", "━".repeat(50).dimmed());
    println!();
    println!(
        "  {:<28} {}",
        "Age:".dimmed(),
        format!("{}h", age_hours).bright_white()
    );
    println!(
        "  {:<28} {}",
        "Observations:".dimmed(),
        obs_count.to_string().bright_white()
    );
    println!(
        "  {:<28} {}",
        "Knowledge facts:".dimmed(),
        knowledge_count.to_string().bright_white()
    );
    println!(
        "  {:<28} {}",
        "Patterns learned:".dimmed(),
        pattern_count.to_string().bright_white()
    );
    println!();
    // Show recent observations
    let recent: Vec<(String, String)> = {
        let mut s = db.prepare(
            "SELECT kind, content FROM friday_observations ORDER BY timestamp DESC LIMIT 5",
        )?;
        let x = s
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        x
    };
    if !recent.is_empty() {
        println!("  {} Recent observations:", "→".bright_cyan());
        for (_kind, content) in &recent {
            let short = content.chars().take(75).collect::<String>();
            println!("    {} {}", "·".dimmed(), short.dimmed());
        }
        println!();
    }
    let closing = status_closing_line();
    if !closing.is_empty() {
        println!("  {} {}", "💡".dimmed(), closing);
    }
    println!("  {} Use: core friday ask <question>", "→".dimmed());
    println!();
    Ok(())
}
/// `core friday ask <question>` -- Q&A from stored knowledge
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
        let x: Vec<(String, String, String, f64)> = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, f64>(2)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter(|(t, a, _)| {
                q_lower
                    .split_whitespace()
                    .any(|w| t.to_lowercase().contains(w) || a.to_lowercase().contains(w))
                    || q_lower.contains("pattern")
            })
            .map(|(t, a, c)| {
                let key = format!("{}|{}", t, a);
                ("pattern".to_string(), key, format!("When {} → {}", t, a), c)
            })
            .collect();
        x
    };
    let facts: Vec<(String, String, String, f64)> = {
        let mut s = db.prepare(
            "SELECT domain, key, fact, confidence FROM friday_knowledge ORDER BY confidence DESC, ROWID ASC"
        )?;
        let x = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, f64>(3)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        x
    };
    let mut all_facts = facts.clone();
    all_facts.extend(pattern_facts);
    let primary_domain = ["rust", "cargo", "borrow", "lifetime"]
        .iter()
        .any(|w| q_lower.contains(w))
        .then_some("rust")
        .or(["wayland", "compositor"]
            .iter()
            .any(|w| q_lower.contains(w))
            .then_some("wayland"))
        .or(["pacman", "arch", "systemctl", "aur"]
            .iter()
            .any(|w| q_lower.contains(w))
            .then_some("arch"))
        .or(["dbus", "zbus"]
            .iter()
            .any(|w| q_lower.contains(w))
            .then_some("dbus"))
        .or(["deploy", "intent", "nsh", "state.db"]
            .iter()
            .any(|w| q_lower.contains(w))
            .then_some("zero"))
        .or(["build", "compile", "error", "failed"]
            .iter()
            .any(|w| q_lower.contains(w))
            .then_some("build"));
    all_facts.sort_by(|a, b| {
        let a_exact = primary_domain.map(|d| a.0 == d).unwrap_or(false);
        let b_exact = primary_domain.map(|d| b.0 == d).unwrap_or(false);
        let a_generic = a.0 == "tools" || a.0 == "zero" || a.0 == "sessions";
        let b_generic = b.0 == "tools" || b.0 == "zero" || b.0 == "sessions";
        if a_exact && !b_exact {
            return std::cmp::Ordering::Less;
        }
        if !a_exact && b_exact {
            return std::cmp::Ordering::Greater;
        }
        if a_generic && !b_generic {
            return std::cmp::Ordering::Greater;
        }
        if !a_generic && b_generic {
            return std::cmp::Ordering::Less;
        }
        b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal)
    });
    let relevant: Vec<&(String, String, String, f64)> = all_facts
        .iter()
        .filter(|(domain, _key, fact, _)| {
            let d = domain.to_lowercase();
            let f = fact.to_lowercase();
            let stop = [
                "how", "do", "i", "a", "the", "in", "to", "for", "of", "and", "is", "it", "fix",
                "get", "use",
            ];
            let domain_map: &[(&str, &str)] = &[
                ("rust", "rust"),
                ("cargo", "rust"),
                ("borrow", "rust"),
                ("lifetime", "rust"),
                ("wayland", "wayland"),
                ("compositor", "wayland"),
                ("pacman", "arch"),
                ("arch", "arch"),
                ("systemctl", "arch"),
                ("aur", "arch"),
                ("dbus", "dbus"),
                ("zbus", "dbus"),
                ("zero", "zero"),
                ("nsh", "zero"),
                ("deploy", "zero"),
                ("intent", "zero"),
                ("build", "build"),
                ("error", "build"),
                ("compile", "build"),
            ];
            let keywords: Vec<&str> = q_lower
                .split_whitespace()
                .filter(|w| !stop.contains(w) && w.len() > 2)
                .collect();
            if keywords.is_empty() {
                return true;
            }
            let domain_match = keywords.iter().any(|word| {
                domain_map
                    .iter()
                    .any(|(kw, dom)| word.contains(kw) && d.contains(dom))
            });
            if domain_match {
                return true;
            }
            keywords
                .iter()
                .any(|word| d.contains(*word) || f.contains(*word))
        })
        .fold(Vec::new(), |mut acc, item| {
            if !acc
                .iter()
                .any(|x: &&(String, String, String, f64)| x.0 == item.0 && x.1 == item.1)
            {
                acc.push(item);
            }
            acc
        })
        .into_iter()
        .take(5)
        .collect();
    // Check recent events for context-aware answers
    let recent_events: Vec<(String, String)> = db.prepare(
        "SELECT domain||'.'||action, payload FROM events WHERE timestamp > ?1 ORDER BY timestamp DESC LIMIT 20"
    ).ok().and_then(|mut s| {
        s.query_map(rusqlite::params![now_ts() - 86400], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1).unwrap_or_default()))
        }).ok().map(|rows| {
            rows.filter_map(|r| r.ok())
            .filter(|(k, v)| {
                let combined = format!("{} {}", k, v).to_lowercase();
                q_lower.split_whitespace().any(|w| combined.contains(w))
            })
            .take(3).collect()
        })
    }).unwrap_or_default();
    println!();
    println!("  Friday -- Active Intelligence");
    println!("  {}", "─".repeat(50).dimmed());
    println!();
    if !recent_events.is_empty() {
        println!("  From recent activity:");
        for (kind, payload) in &recent_events {
            let short = payload.chars().take(80).collect::<String>();
            println!(
                "  {} [{}] {}",
                "→".bright_cyan(),
                kind.bright_green(),
                short.bright_white()
            );
        }
        println!();
    }
    if relevant.is_empty() {
        let is_temporal = q_lower.contains("today")
            || q_lower.contains("recent")
            || q_lower.contains("observed")
            || q_lower.contains("seen")
            || q_lower.contains("latest");
        let obs: Vec<(String, String)> = {
            let mut s = db.prepare(
                "SELECT kind, content FROM friday_observations ORDER BY timestamp DESC LIMIT 20",
            )?;
            let x: Vec<(String, String)> = s
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
                .filter_map(|r| r.ok())
                .filter(|(_k, c)| {
                    if is_temporal {
                        true
                    } else {
                        q_lower
                            .split_whitespace()
                            .any(|word| c.to_lowercase().contains(word))
                    }
                })
                .take(5)
                .collect();
            x
        };
        if obs.is_empty() {
            println!(
                "  I do not have enough knowledge about \"{}\" yet.",
                question.bright_white()
            );
            println!("  Ask me again as sessions accumulate.");
        } else {
            println!("  From recent observations:");
            for (kind, content) in &obs {
                let short = content.chars().take(80).collect::<String>();
                println!(
                    "  {} [{}] {}",
                    "→".bright_cyan(),
                    kind.bright_green(),
                    short.bright_white()
                );
            }
        }
    } else {
        for (domain, _key, fact, confidence) in &relevant {
            println!(
                "  {} [{}] {}",
                "→".bright_cyan(),
                domain.bright_green(),
                fact.bright_white()
            );
            if *confidence < 0.9 {
                println!(
                    "    {} confidence: {:.0}%",
                    "·".dimmed(),
                    confidence * 100.0
                );
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
    let mut patterns_new = 0;
    let mut patterns_updated = 0;
    // Pattern 1: deploy after cicomplete
    let deploy_after_complete: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM shell_history h1
         WHERE h1.command LIKE 'deploy%'
         AND EXISTS (
             SELECT 1 FROM shell_history h2
             WHERE h2.command LIKE 'cicomplete%'
             AND h2.timestamp < h1.timestamp
             AND h2.timestamp > h1.timestamp - 300
         )",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if deploy_after_complete > 2 {
        let exists: bool = db.query_row(
            "SELECT 1 FROM friday_patterns WHERE trigger = 'cicomplete runs' AND action = 'deploy tool'",
            [], |_| Ok(true),
        ).unwrap_or(false);
        let _ = db.execute(
            "INSERT OR REPLACE INTO friday_patterns (trigger, context, action, outcome, frequency, confidence, last_seen, source)
             VALUES ('cicomplete runs', '[\"deploy\", \"workflow\"]', 'deploy tool', 'success', ?1, ?2, ?3, 'shell_history')",
            params![deploy_after_complete, (deploy_after_complete as f64 / 10.0).min(0.95), now],
        );
        if exists {
            patterns_updated += 1;
        } else {
            patterns_new += 1;
        }
    }
    // Pattern 2: fg commit after deploy
    let commit_after_deploy: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM shell_history h1
         WHERE h1.command LIKE 'fg commit%'
         AND EXISTS (
             SELECT 1 FROM shell_history h2
             WHERE h2.command LIKE 'deploy%'
             AND h2.timestamp < h1.timestamp
             AND h2.timestamp > h1.timestamp - 300
         )",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if commit_after_deploy > 2 {
        let exists: bool = db.query_row(
            "SELECT 1 FROM friday_patterns WHERE trigger = 'deploy completes' AND action = 'fg commit'",
            [], |_| Ok(true),
        ).unwrap_or(false);
        let _ = db.execute(
            "INSERT OR REPLACE INTO friday_patterns (trigger, context, action, outcome, frequency, confidence, last_seen, source)
             VALUES ('deploy completes', '[\"commit\", \"workflow\"]', 'fg commit', 'success', ?1, ?2, ?3, 'shell_history')",
            params![commit_after_deploy, (commit_after_deploy as f64 / 10.0).min(0.95), now],
        );
        if exists {
            patterns_updated += 1;
        } else {
            patterns_new += 1;
        }
    }
    // Pattern 3: Most frequent commands
    let top_commands: Vec<(String, i64)> = {
        let mut s = db.prepare(
            "SELECT command, COUNT(*) as cnt FROM shell_history
             WHERE command NOT IN ('d', 'ls', 'pwd', 'clear')
             GROUP BY command ORDER BY cnt DESC LIMIT 10",
        )?;
        let x: Vec<(String, i64)> = s
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    for (cmd, cnt) in &top_commands {
        if *cnt > 2 {
            let exists: bool = db.query_row(
                "SELECT 1 FROM friday_patterns WHERE trigger = 'frequent command' AND action = ?1",
                params![cmd], |_| Ok(true),
            ).unwrap_or(false);
            let _ = db.execute(
                "INSERT OR REPLACE INTO friday_patterns (trigger, context, action, outcome, frequency, confidence, last_seen, source)
                 VALUES ('frequent command', '[\"habit\"]', ?1, 'observed', ?2, 0.8, ?3, 'shell_history')",
                params![cmd, cnt, now],
            );
            if exists {
                patterns_updated += 1;
            } else {
                patterns_new += 1;
            }
        }
    }
    // Update knowledge with pattern count
    let total_patterns: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0))
        .unwrap_or(0);
    let fact = format!(
        "{} behavioral patterns extracted from {} shell commands.",
        total_patterns,
        db.query_row("SELECT COUNT(*) FROM shell_history", [], |r| r
            .get::<_, i64>(0))
            .unwrap_or(0)
    );
    // Time-varying: preserve created_at if this domain/key already has a row
    let existing_created: Option<i64> = db
        .query_row(
            "SELECT created_at FROM friday_knowledge WHERE domain = 'patterns' AND key = 'global'",
            [],
            |r| r.get(0),
        )
        .ok();
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
    println!(
        "  {} Pattern extraction complete -- {} new, {} updated ({} total)",
        "✅".green(),
        patterns_new.to_string().bright_white(),
        patterns_updated.to_string().bright_cyan(),
        total_patterns.to_string().dimmed()
    );
    Ok(())
}

/// core friday suggest -- evidence-based recommendation from Friday
pub fn suggest(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    observe(ctx)?;
    let _ = extract_patterns(ctx);
    let db = &ctx.runtime.db;
    println!();
    println!("  Friday suggests:");
    println!("  {}", "─".repeat(50).dimmed());
    println!();
    // Check predict next from core
    let _top_intent: Option<String> = db
        .query_row(
            "SELECT title FROM intents WHERE status = 'planned' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .ok();
    // Get top pattern
    let top_pattern: Option<(String, String, f64)> = {
        let mut s = db.prepare(
            "SELECT trigger, action, confidence FROM friday_patterns ORDER BY confidence DESC LIMIT 1"
        )?;
        let x: Vec<(String,String,f64)> = s.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?)))
            ?.filter_map(|r| r.ok()).collect(); x
    }.into_iter().next();
    // Get recent health
    // INT-265: an absent last_health row is unknown; this COALESCEd it to 100.
    let health: Option<i64> = db
        .query_row(
            "SELECT value FROM domain_state WHERE key = 'last_health' LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .ok()
        .and_then(|s| s.parse().ok());
    let mut suggestions: Vec<String> = Vec::new();
    if health.is_none_or(|h| h < 95) {
        suggestions.push(format!(
            "Health is at {} -- run d and investigate before continuing.",
            health
                .map(|h| format!("{}%", h))
                .unwrap_or_else(|| "unknown".to_string())
        ));
    }
    if let Some((trigger, action, conf)) = top_pattern {
        // Skip if pattern is about deploying and we just deployed recently
        let recently_deployed: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM deploy_patterns WHERE timestamp > ?1",
                rusqlite::params![now_ts() - 300],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let skip = action.contains("deploy") && recently_deployed > 0;
        if !skip && conf >= 0.85 {
            suggestions.push(format!(
                "Pattern detected ({:.0}% confidence): when {} \u{2192} {}",
                conf * 100.0,
                trigger,
                action
            ));
        }
    }
    let session_cmds: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM shell_history WHERE timestamp > ?1",
            rusqlite::params![now_ts() - 3600],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if session_cmds > 50 {
        suggestions.push(format!(
            "{} commands this hour \u{2014} consider committing and taking a break.",
            session_cmds
        ));
    }
    for (i, s) in suggestions.iter().enumerate() {
        let icon = if i == 0 {
            "●".bright_cyan()
        } else {
            "○".dimmed()
        };
        println!("  {} {}", icon, s.bright_white());
    }
    println!();
    Ok(())
}

/// Phase 3: Update friday_personality from interaction patterns
pub fn update_personality(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    // Count interactions
    let ask_count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM friday_knowledge WHERE times_used > 0",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let pattern_count: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0))
        .unwrap_or(0);
    let obs_count: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0))
        .unwrap_or(0);
    // Determine phase based on data richness
    let phase = if pattern_count >= 20 && obs_count >= 10 {
        "2"
    } else if obs_count >= 5 {
        "1"
    } else {
        "0"
    };
    // Update personality traits based on data
    let traits = vec![
        ("phase", phase.to_string()),
        ("observations_since_birth", obs_count.to_string()),
        ("patterns_learned", pattern_count.to_string()),
        ("knowledge_consulted", ask_count.to_string()),
        (
            "voice_confidence",
            if pattern_count >= 10 {
                "growing"
            } else {
                "nascent"
            }
            .to_string(),
        ),
        ("dominant_domain", "operations".to_string()),
        (
            "communication_style",
            "direct, evidence-based, concise".to_string(),
        ),
        ("updated_at", now.to_string()),
    ];
    for (key, value) in &traits {
        let _ = db.execute(
            "INSERT OR REPLACE INTO friday_personality (key, value, updated_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![key, value, now],
        );
    }
    println!(
        "  {} Personality updated -- phase {}, {} patterns, {} observations",
        "✅".green(),
        phase.bright_cyan(),
        pattern_count.to_string().bright_white(),
        obs_count.to_string().bright_white()
    );
    Ok(())
}
/// Phase 4: Seed Linux/Rust/Project 0 knowledge base
pub fn seed_linux_knowledge(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    let knowledge = vec![
        // Linux and systemd facts.
        ("linux", "systemctl start/stop/enable/disable/status manages services. journalctl -u <service> shows logs.", 0.95),
        // Wayland
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
        // Project 0 tools
        ("zero", "Project 0 is built from its own Rust tools, each one understood completely.", 0.95),
        ("zero", "core is the orchestrator binary. Its domains include friday, synthesis, predict and doctor.", 0.95),
        ("zero", "nsh (NovaShell) is the interactive shell; bash stays the login shell.", 0.95),
        ("zero", "ship builds the release and installs it into ~/.local/bin; d verifies health after.", 0.95),
        ("zero", "state.db is Project 0's memory. WAL mode. Never DELETE or UPDATE events_v2.", 0.95),
        ("zero", "cistart starts an intent; cicomplete closes it once every gate is proven. Commit after changes. d before everything.", 0.95),
        ("zero", "d measures health. Below 95%, investigate before shipping.", 0.95),
        // Build patterns
        ("build", "When a Rust build fails with E0432, check the use statements and module paths.", 0.90),
        ("build", "E0308 type mismatch -- check if you need .to_string(), &str vs String, or type annotation.", 0.90),
        ("build", "E0597 borrow lifetime -- use let x = ...; x pattern to extend lifetime past block.", 0.90),
        ("build", "E0716 temporary dropped -- assign to a named variable before using its reference.", 0.90),
        ("build", "dead_code warnings: prefix with _ or add #[allow(dead_code)]. Never suppress real bugs.", 0.85),
        // DBus
        ("dbus", "DBus allows IPC between processes. Session bus for user services, system bus for system.", 0.85),
        ("dbus", "zbus is the idiomatic Rust DBus library. It uses async/await with tokio.", 0.85),
    ];
    let mut added = 0;
    for (domain, fact, confidence) in &knowledge {
        let result = db.execute(
            "INSERT OR IGNORE INTO friday_knowledge (domain, key, fact, confidence, source, created_at, updated_at)
             VALUES (?1, ?2, ?2, ?3, 'seed_linux', ?4, ?4)",
            rusqlite::params![domain, fact, confidence, now],
        );
        if result.is_ok() {
            added += 1;
        }
    }
    println!(
        "  {} Linux/Rust/Project 0 knowledge seeded -- {} facts added",
        "✅".green(),
        added.to_string().bright_white()
    );
    Ok(())
}

/// Phase 5: Name an abstraction -- confirm a candidate and record it in friday_language
pub fn name_abstraction(ctx: &AppContext, name: &str, description: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = now_ts();
    // Create friday_language table if not exists
    let _ = db.execute_batch(
        "
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
    ",
    );
    // Check if already exists
    let exists: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM friday_language WHERE name = ?1",
            rusqlite::params![name],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if exists > 0 {
        println!(
            "  {} '{}' already in Friday's vocabulary",
            "⚠️ ".yellow(),
            name.bright_white()
        );
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
    println!("  Friday's vocabulary grows:");
    println!(
        "  {} '{}' named and recorded",
        "✅".green(),
        name.bright_cyan().bold()
    );
    println!("  {} {}", "→".dimmed(), description.bright_white());
    println!();
    println!("  {} This is how a language is born.", "💡".dimmed());
    Ok(())
}
/// List all named abstractions in Friday's vocabulary
pub fn list_vocabulary(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let _ = db.execute_batch(
        "
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
    ",
    );
    let rows: Vec<(String, String, f64)> = {
        let mut s = db.prepare(
            "SELECT name, description, confidence FROM friday_language ORDER BY created_at ASC",
        )?;
        let x = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, f64>(2)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        x
    };
    println!();
    println!("  Friday's Vocabulary");
    println!("  {}", "─".repeat(50).dimmed());
    if rows.is_empty() {
        println!("  {} No abstractions named yet.", "→".dimmed());
        println!(
            "  {} Use: core friday name-abstraction <name> <description>",
            "💡".dimmed()
        );
    } else {
        for (name, desc, conf) in &rows {
            println!(
                "  {} {} -- {} ({:.0}%)",
                "→".bright_green(),
                name.bright_cyan().bold(),
                desc.white(),
                conf * 100.0
            );
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
    // Gather signals to ground the proposal
    let pattern_count: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0))
        .unwrap_or(0);
    let obs_count: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0))
        .unwrap_or(0);
    let top_pattern: Option<(String, String, f64, i64)> = db.query_row(
        "SELECT trigger, action, confidence, frequency FROM friday_patterns ORDER BY frequency DESC LIMIT 1",
        [], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?, r.get::<_,i64>(3)?))
    ).ok();
    let top_gap: Option<String> = db
        .query_row(
            "SELECT command FROM shell_history WHERE command LIKE 'python3 /tmp/%'
         GROUP BY command ORDER BY COUNT(*) DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .ok();
    let vocab_count: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_language", [], |r| r.get(0))
        .unwrap_or(0);
    // Determine what to propose based on what Friday sees
    let (title, tags, rationale, proposal) = if vocab_count < 3 {
        (
            "Friday Vocabulary Expansion -- Name the Patterns Project 0 Repeats",
            "friday, vocabulary, abstraction, language, patterns",
            format!(
                "Friday has observed {} patterns and named {} abstractions. \
                Project 0 repeats behaviors that deserve names. \
                Naming them gives Friday vocabulary to reason with.",
                pattern_count, vocab_count
            ),
            format!(
                "Friday has identified {} patterns from {} observations. \
                The top pattern occurs {} times. \
                This intent establishes the vocabulary expansion practice: \
                weekly review of abstraction candidates, confirmation of names, \
                and recording in friday_language. \
                A language built from what Project 0 actually does.",
                pattern_count,
                obs_count,
                top_pattern.as_ref().map(|p| p.3).unwrap_or(0)
            ),
        )
    } else if top_gap.is_some() {
        (
            "nsh Script Mode -- Eliminate python3 /tmp/ Workflow Completely",
            "nsh, shell, workflow, scripting, friction",
            "Friday has observed hundreds of python3 /tmp/ invocations. \
                Every one is friction. The run builtin exists but the habit persists. \
                Project 0 needs a first-class script mode that makes python3 /tmp/ obsolete."
                .to_string(),
            "Friday observes that the single largest workflow friction point is the \
                python3 /tmp/script.py pattern. The run builtin handles .py files but \
                lacks the ergonomics needed to replace the habit: no argument passing, \
                no stdin support, no script templates. \
                This intent builds fsh script mode: fsh new script.py (template), \
                run script.py arg1 arg2 (with args), run - (from stdin). \
                Friday proposes this because it has watched the pattern 845 times."
                .to_string(),
        )
    } else {
        (
            "Friday Session Intelligence -- Know the Session Before It Starts",
            "friday, intelligence, session, context, awareness",
            "Friday has observed patterns across sessions but cannot yet orient itself \
                at session start. Project 0 deserves a morning briefing."
                .to_string(),
            format!(
                "Friday has {} facts and {} patterns. \
                At session start, Friday should synthesize: what was left incomplete, \
                what the predictions say comes next, what the health trajectory shows, \
                and what the dominant pattern suggests. \
                This intent builds core friday morning-brief -- called automatically \
                at fsh startup, showing a 3-line situational summary. \
                Friday already has the data. It just needs to speak first.",
                pattern_count + obs_count,
                pattern_count
            ),
        )
    };
    // Generate intent ID suggestion
    let max_id: i64 = 230; // approximate next available
    let intent_id = max_id + 1;
    let slug = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let filename = format!("{}-{}.md", intent_id, &slug[..slug.len().min(60)]);
    let intent_content = format!(
        r#"---
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
    println!("  Friday proposes:");
    println!("  {}", "─".repeat(55).dimmed());
    println!("  {} {}", "Title:".dimmed(), title.bright_cyan().bold());
    println!("  {} {}", "Tags: ".dimmed(), tags.dimmed());
    println!();
    println!("  {} Rationale:", "→".bright_green());
    for line in rationale.lines() {
        println!("    {}", line.white());
    }
    println!();
    println!(
        "  {} File: intents/open/{}",
        "📄".normal(),
        filename.bright_white()
    );
    println!();
    print!("  Save this intent? (y/n): ");
    use std::io::{self, BufRead, Write};
    io::stdout().flush().ok();
    let stdin = io::stdin();
    let line = stdin
        .lock()
        .lines()
        .next()
        .and_then(|l| l.ok())
        .unwrap_or_default();
    if line.trim().to_lowercase() == "y" {
        let path = format!("intents/open/{}", filename);
        std::fs::write(&path, &intent_content)?;
        // Record in observations
        let _ = db.execute(
            "INSERT INTO friday_observations (timestamp, source, kind, content)
             VALUES (?1, 'friday', 'proposal', ?2)",
            rusqlite::params![now, format!("proposed intent: {}", title)],
        );
        println!("  {} Intent saved: {}", "✅".green(), path.bright_white());
        println!("  Friday co-authored its first intent.");
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
             WHERE confidence >= 0.6 ORDER BY confidence DESC LIMIT 5",
        )?;
        let x: Vec<(i64, String, String, f64)> = s
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, f64>(3)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    let mut hypotheses_created = 0;
    for (pat_id, trigger, action, confidence) in &patterns {
        let prediction = format!("When '{}' occurs, '{}' will follow", trigger, action);
        let exists: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM friday_hypotheses WHERE prediction = ?1 AND outcome IS NULL",
                rusqlite::params![prediction],
                |r| r.get(0),
            )
            .unwrap_or(0);
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
             WHERE outcome IS NULL AND expires_at < ?1 LIMIT 10",
        )?;
        let x: Vec<(i64, String, f64)> = s
            .query_map(rusqlite::params![now], |r| {
                Ok((r.get(0)?, r.get::<_, String>(1)?, r.get::<_, f64>(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    let mut validated = 0;
    let mut reinforced = 0;
    let mut penalized = 0;
    for (hyp_id, prediction, confidence) in &pending {
        // Simple validation: check if the predicted action appeared in shell_history
        // after the hypothesis was created
        let hyp_created: i64 = db
            .query_row(
                "SELECT created_at FROM friday_hypotheses WHERE id = ?1",
                rusqlite::params![hyp_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        // Extract action from prediction text (handles both quoted and unquoted formats)
        let action_part = if prediction.contains("'") {
            prediction.split("'").nth(3).unwrap_or("").to_string()
        } else {
            // "When X occurs, Y will follow" -- extract after last comma
            prediction
                .split(", ")
                .last()
                .unwrap_or("")
                .trim_end_matches(" will follow")
                .to_string()
        };
        let action_part = action_part.trim();
        let found: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM shell_history WHERE command LIKE ?1 AND timestamp > ?2",
                rusqlite::params![format!("%{}%", action_part), hyp_created],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let (outcome, delta) = if found > 0 {
            ("correct", 0.05f64) // reinforce
        } else {
            ("incorrect", -0.10f64) // penalize
        };
        // Update hypothesis
        let _ = db.execute(
            "UPDATE friday_hypotheses SET outcome = ?1, resolved_at = ?2, correct = ?3
             WHERE id = ?4",
            rusqlite::params![
                outcome,
                now,
                if outcome == "correct" { 1 } else { 0 },
                hyp_id
            ],
        );
        // Update pattern confidence (reinforce or decay)
        if !action_part.is_empty() {
            let new_conf = (confidence + delta).clamp(0.1, 0.99);
            let _ = db.execute(
                "UPDATE friday_patterns SET confidence = ?1 WHERE action LIKE ?2",
                rusqlite::params![new_conf, format!("%{}%", action_part)],
            );
            if delta > 0.0 {
                reinforced += 1;
            } else {
                penalized += 1;
            }
        }
        validated += 1;
    }
    // Step 3: Generate abstraction candidates from high-frequency patterns
    let high_freq: Vec<(String, String, i64)> = {
        let mut s = db.prepare(
            "SELECT trigger, action, frequency FROM friday_patterns
             WHERE frequency >= 5 AND confidence >= 0.7 ORDER BY frequency DESC LIMIT 3",
        )?;
        let x: Vec<(String, String, i64)> = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        x
    };
    let mut abstractions = 0;
    for (trigger, action, freq) in &high_freq {
        let name_candidate = format!(
            "{}_then_{}",
            trigger.split_whitespace().last().unwrap_or("event"),
            action.split_whitespace().last().unwrap_or("action")
        );
        let fact = format!(
            "abstraction_candidate: '{}' pattern (trigger: {}, action: {}, freq: {})",
            name_candidate, trigger, action, freq
        );
        let existing_created: Option<i64> = db
            .query_row(
                "SELECT created_at FROM friday_knowledge WHERE domain = 'abstraction' AND key = ?1",
                rusqlite::params![name_candidate],
                |r| r.get(0),
            )
            .ok();
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
    println!("  Learning loop complete:");
    println!(
        "    {} hypotheses created",
        hypotheses_created.to_string().bright_white()
    );
    println!(
        "    {} hypotheses validated ({} reinforced, {} penalized)",
        validated.to_string().bright_white(),
        reinforced.to_string().bright_green(),
        penalized.to_string().bright_yellow()
    );
    println!(
        "    {} abstraction candidates generated",
        abstractions.to_string().bright_cyan()
    );
    Ok(())
}
/// core friday observe -- manually trigger observation cycle
pub fn run_observe(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    observe(ctx)?;
    let count: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0))
        .unwrap_or(0);
    println!(
        "  {} Friday observed. Total observations: {}",
        "✅".green(),
        count.to_string().bright_white()
    );
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// INT-217 -- Core v19: Friday Finds Her Voice
// ═══════════════════════════════════════════════════════════
/// How long ago Friday spoke, while the 30-minute cooldown is why she is quiet.
pub fn cooldown_line(seconds_since_spoken: i64) -> String {
    let minutes = seconds_since_spoken.max(0) / 60;
    format!("spoke {minutes}m ago")
}

/// What `d` should print. Cooldown is a reason, not the same as having nothing to say.
pub enum Voice {
    Speak(String, f64),
    Cooldown(i64),
    Quiet,
}

/// Check all thresholds and return Friday's voice if she has earned the right to speak.
/// Also handles dormant → observing → active engine_registry transition.
pub fn voice(ctx: &AppContext) -> Voice {
    let db = &ctx.runtime.db;
    // Gate 1: synthesis snapshot exists with confidence >= 0.7
    let confidence: Option<f64> = db
        .query_row(
            "SELECT brief_confidence FROM synthesis_snapshots ORDER BY timestamp DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();
    let confidence = match confidence {
        Some(confidence) if confidence >= 0.7 => confidence,
        _ => {
            let _ = set_friday_status(ctx, "dormant");
            return Voice::Quiet;
        }
    };
    // Gate 2: at least 3 days of pattern data (259200 seconds)
    // DEC: lowered from 7 days -- system has 10k+ history entries, 4 days is real signal
    let oldest_pattern: i64 = db
        .query_row("SELECT MIN(last_seen) FROM friday_patterns", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    if now - oldest_pattern < 259200 {
        let _ = set_friday_status(ctx, "observing");
        return Voice::Quiet;
    }
    // Gate 3: at least 50 shell_history entries
    let history_count: i64 = db
        .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))
        .unwrap_or(0);
    if history_count < 50 {
        let _ = set_friday_status(ctx, "observing");
        return Voice::Quiet;
    }
    // Gate 4: rate limit -- max once per 30 minutes
    let last_spoken: i64 = db
        .query_row(
            "SELECT value FROM friday_personality WHERE key = 'last_spoken_ts'",
            [],
            |r| r.get::<_, String>(0).map(|s| s.parse::<i64>().unwrap_or(0)),
        )
        .unwrap_or(0);
    if now - last_spoken < 1800 {
        // Rate limited -- still active. The caller can say how long ago she spoke.
        let _ = set_friday_status(ctx, "active");
        return Voice::Cooldown(now - last_spoken);
    }
    // All gates passed -- Friday is active and ready to speak
    let _ = set_friday_status(ctx, "active");
    // Update last_spoken timestamp
    let _ = db.execute(
        "INSERT OR REPLACE INTO friday_personality (key, value, updated_at) VALUES ('last_spoken_ts', ?1, ?2)",
        rusqlite::params![now.to_string(), now],
    );
    // Log first speech event if this is the first time
    let speech_count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM friday_personality WHERE key = 'has_spoken'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if speech_count == 0 {
        let _ = db.execute(
            "INSERT OR IGNORE INTO friday_personality (key, value, updated_at) VALUES ('has_spoken', 'true', ?1)",
            rusqlite::params![now],
        );
        let payload = format!(
            "{{\"note\":\"Friday spoke for the first time\",\"confidence\":{confidence:.2}}}"
        );
        if let Err(err) = db.execute(
            "INSERT INTO events_v2 (timestamp, source, kind, type_name, payload, confidence)
             VALUES (?1, 'friday', 'milestone', 'friday_first_speech', ?2, ?3)",
            rusqlite::params![now, payload, confidence],
        ) {
            eprintln!("friday: first speech was not recorded in events_v2: {err}");
        }
    }
    let brief = crate::domains::friday_arch::room::sentence_for(ctx);
    Voice::Speak(brief, confidence)
}

/// Some((brief, confidence)) when Friday may speak. Cooldown and silence are both None.
/// Tests use this door. `d` calls `voice` so a cooldown is not the same as silence.
#[cfg(test)]
pub fn get_voice(ctx: &AppContext) -> Option<(String, f64)> {
    match voice(ctx) {
        Voice::Speak(brief, confidence) => Some((brief, confidence)),
        Voice::Cooldown(_) | Voice::Quiet => None,
    }
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
    let pattern_count: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0))
        .unwrap_or(0);
    let complete_count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM friday_observations WHERE kind = 'deploy'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    // Build a 1-sentence observation from what Friday knows
    let observation = if pattern_count > 20 && complete_count > 10 {
        format!(
            "{} complete -- Project 0 grows stronger. {} patterns observed. Momentum continues.",
            intent_title, pattern_count
        )
    } else {
        format!(
            "{} complete -- Friday is watching and learning.",
            intent_title
        )
    };
    println!("  Friday: {}", observation.bright_white());
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
/// Friday writes a daily journal entry -- her own perspective on Project 0.
pub fn write_journal_entry(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    // Check if already written today (86400 seconds = 1 day)
    let last_journal: i64 = db
        .query_row(
            "SELECT value FROM friday_personality WHERE key = 'last_journal_ts'",
            [],
            |r| r.get::<_, String>(0).map(|s| s.parse::<i64>().unwrap_or(0)),
        )
        .unwrap_or(0);
    if now - last_journal < 86400 {
        return Ok(());
    }
    // Gather data for the entry
    let pattern_count: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0))
        .unwrap_or(0);
    let obs_count: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0))
        .unwrap_or(0);
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
    // Write to journal via the journal system
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
    // Ensure the milestone row and the knowledge fact both have a table.
    let _ = db.execute_batch(
        "CREATE TABLE IF NOT EXISTS friday_milestones (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            milestone TEXT NOT NULL UNIQUE,
            triggered_at INTEGER NOT NULL,
            message TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS friday_knowledge (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain TEXT NOT NULL,
            key TEXT NOT NULL,
            fact TEXT NOT NULL,
            confidence REAL NOT NULL DEFAULT 0.7,
            source TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            times_used INTEGER NOT NULL DEFAULT 0,
            UNIQUE(domain, key)
        );",
    );
    // Get live stats. Commits are git's count. Intents are the ledger's.
    // A repo git cannot read does not become zero, and it does not become 186.
    let commit_count =
        crate::domains::friday_arch::room::git_commit_count(std::path::Path::new(&ctx.core_root));
    let complete_intents = completed_intent_count() as i64;
    let commit_words = match commit_count {
        Some(n) => n.to_string(),
        None => "unknown".to_string(),
    };
    let _patterns: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0))
        .unwrap_or(0);
    // Define milestones
    let milestones: Vec<(&str, i64, &str, String)> = vec![
        ("commits_500",  500,  "commit",  "500 commits. Project 0 is no longer an experiment. It is a system.".to_string()),
        ("commits_1000", 1000, "commit",  "1000 commits. Every one intentional. Every one part of something real.".to_string()),
        ("commits_1500", 1500, "commit",  "1500 commits. Project 0 has grown from nothing into a living system with its own intelligence.".to_string()),
        ("commits_2000", 2000, "commit",  format!("2000 commits. {} intents complete. Project 0 has a voice now. That was the plan all along.", complete_intents)),
        ("commits_2500", 2500, "commit",  "2500 commits. Project 0 keeps building. Friday is watching every one.".to_string()),
        ("intents_50",   50,   "intent",  "50 complete intents. Project 0 stopped being a prototype a long time ago.".to_string()),
        ("intents_100",  100,  "intent",  "100 complete intents. More custom Rust tools than most teams build in a year.".to_string()),
        ("intents_150",  150,  "intent",  "150 complete intents. Project 0 rebuilt itself from a catastrophic failure and kept growing.".to_string()),
        ("intents_200",  200,  "intent",  format!("200 complete intents. {} commits. Friday is active. Project 0 built its own intelligence.", commit_words)),
    ];
    // Every crossed, unrecorded milestone is recorded. Only the highest is announced.
    // A commit count git could not read is skipped, never replaced with a fallback.
    let mut crossed: Vec<(&str, i64, &str, &str)> = Vec::new();
    for (key, threshold, kind, message) in &milestones {
        let count = match (*kind, commit_count) {
            ("commit", Some(n)) => n,
            ("commit", None) => continue,
            _ => complete_intents,
        };
        if count < *threshold {
            continue;
        }
        let already: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM friday_milestones WHERE milestone = ?1",
                rusqlite::params![key],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if already == 0 {
            crossed.push((key, *threshold, kind, message.as_str()));
        }
    }
    if crossed.is_empty() {
        return None;
    }
    for (key, _threshold, _kind, message) in &crossed {
        let _ = db.execute(
            "INSERT OR IGNORE INTO friday_milestones (milestone, triggered_at, message) VALUES (?1, ?2, ?3)",
            rusqlite::params![key, now, message],
        );
        let fact = format!("milestone: {key} -- {message}");
        let _ = db.execute(
            "INSERT OR IGNORE INTO friday_knowledge (domain, key, fact, confidence, source, created_at, updated_at) VALUES ('milestone', ?1, ?2, 1.0, 'easter_egg', ?3, ?3)",
            rusqlite::params![key, fact, now],
        );
    }
    let &(_, threshold, kind, message) = crossed
        .iter()
        .max_by_key(|item| item.1)
        .expect("crossed is non-empty");
    if kind == "commit" {
        let count = commit_count.expect("a commit milestone was crossed, so git answered");
        let head = format!(
            "{} commits. Past the {} mark.",
            with_thousands(count),
            with_thousands(threshold)
        );
        // The stored celebration starts with the mark. The line already said it.
        let prefix = format!("{threshold} commits. ");
        let rest = message.strip_prefix(&prefix).unwrap_or(message);
        if rest.is_empty() {
            Some(head)
        } else {
            Some(format!("{head} {rest}"))
        }
    } else {
        Some(message.to_string())
    }
}

fn with_thousands(n: i64) -> String {
    let negative = n < 0;
    let digits = n.unsigned_abs().to_string();
    let mut reversed = String::new();
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            reversed.push(',');
        }
        reversed.push(ch);
    }
    let mut out: String = reversed.chars().rev().collect();
    if negative {
        out.insert(0, '-');
    }
    out
}

/// A probe fact is current for a day. The next doctor run replaces the rows;
/// the expiry is what a reader has when that run has not happened.
const PROBE_FACT_TTL_SECS: i64 = 24 * 60 * 60;

/// Nullable column. ALTER only when the column is absent, so an existing
/// friday_knowledge (a full state.db included) is not rewritten.
fn ensure_knowledge_expires_at(db: &rusqlite::Connection) -> CoreResult<()> {
    let present: i64 = db.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('friday_knowledge') WHERE name = 'expires_at'",
        [],
        |row| row.get(0),
    )?;
    if present == 0 {
        db.execute(
            "ALTER TABLE friday_knowledge ADD COLUMN expires_at INTEGER",
            [],
        )?;
    }
    Ok(())
}

fn probe_status_word(status: crate::domains::doctor::Status) -> &'static str {
    match status {
        crate::domains::doctor::Status::Pass => "pass",
        crate::domains::doctor::Status::Warn => "warn",
        crate::domains::doctor::Status::Fail => "fail",
        crate::domains::doctor::Status::Blocked => "blocked",
        crate::domains::doctor::Status::Unknown => "unknown",
    }
}

/// One friday_knowledge row per doctor probe. The text is that probe's name,
/// status, and message. source is "probe". Stale probe rows are replaced.
pub fn record_probe_facts(
    ctx: &AppContext,
    checks: &[crate::domains::doctor::CheckResult],
) -> CoreResult<()> {
    let db = &ctx.runtime.db;
    db.execute_batch(CREATE_TABLES)?;
    ensure_knowledge_expires_at(db)?;
    let now = now_ts();
    let expires_at = now.saturating_add(PROBE_FACT_TTL_SECS);
    db.execute(
        "DELETE FROM friday_knowledge WHERE source = 'probe'",
        [],
    )?;
    let mut seen = std::collections::HashSet::new();
    for check in checks {
        let key = if check.id.is_empty() {
            check.name.clone()
        } else {
            check.id.clone()
        };
        if !seen.insert(key.clone()) {
            continue;
        }
        let fact = format!(
            "{}: {}. {}",
            check.name,
            probe_status_word(check.status),
            check.message
        );
        db.execute(
            "INSERT INTO friday_knowledge
                (domain, key, fact, confidence, source, created_at, updated_at, expires_at)
             VALUES ('probe', ?1, ?2, 1.0, 'probe', ?3, ?3, ?4)",
            params![key, fact, now, expires_at],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod seed_count_tests {
    #[test]
    fn a_record_does_not_count_as_a_completed_intent() {
        let dir = std::env::temp_dir().join(format!(
            "friday-seed-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let open = dir.join("0-core/zero/intents/open");
        let decisions = dir.join("0-core/zero/intents/decisions");
        std::fs::create_dir_all(&open).unwrap();
        std::fs::create_dir_all(&decisions).unwrap();
        std::fs::write(
            open.join("030-done.md"),
            "---\nid: 30\nstatus: complete\ntitle: \"Done\"\n---\n",
        )
        .unwrap();
        std::fs::write(
            decisions.join("137-x.md"),
            "---\nid: 137\nstatus: complete\ntitle: \"Record\"\n---\n",
        )
        .unwrap();
        let _guard = crate::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", &dir) };
        let n = super::completed_intent_count();
        match prev {
            Some(h) => unsafe { std::env::set_var("HOME", h) },
            None => unsafe { std::env::remove_var("HOME") },
        }
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(n, 1, "a decision is not a completed intent");
    }

    #[test]
    fn friday_intent_count_matches_ledger_done() {
        let dir = std::env::temp_dir().join(format!(
            "friday-intent-count-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let repo = dir.join("0-core");
        let open = repo.join("zero/intents/open");
        let complete = repo.join("zero/intents/complete");
        std::fs::create_dir_all(&open).unwrap();
        std::fs::create_dir_all(&complete).unwrap();
        std::fs::write(
            open.join("030-done.md"),
            "---\nid: 30\nstatus: complete\ntitle: \"Done\"\n---\n",
        )
        .unwrap();
        std::fs::write(
            complete.join("277-archived.md"),
            "---\nid: 277\nstatus: complete\ntitle: \"Archived\"\n---\n",
        )
        .unwrap();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(&repo)
                .env("GIT_AUTHOR_NAME", "test")
                .env("GIT_AUTHOR_EMAIL", "test@example.com")
                .env("GIT_COMMITTER_NAME", "test")
                .env("GIT_COMMITTER_EMAIL", "test@example.com")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        git(&["init"]);
        git(&["add", "zero/intents"]);
        git(&["commit", "-m", "archive the completed intent"]);
        std::fs::remove_file(complete.join("277-archived.md")).unwrap();
        git(&["add", "-A"]);
        git(&["commit", "-m", "leave the completed intent in history"]);
        let _guard = crate::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", &dir) };
        let friday = super::completed_intent_count();
        let done_ids = zero_core::ledger::completed_ids();
        let done = done_ids.len();
        match prev {
            Some(h) => unsafe { std::env::set_var("HOME", h) },
            None => unsafe { std::env::remove_var("HOME") },
        }
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(
            friday, done,
            "friday completed_intent_count is {friday}; ledger completed_ids is {done} ({done_ids:?})"
        );
    }
}

#[cfg(test)]
mod voice_tests {
    #[test]
    fn the_voice_does_not_replay_the_stored_brief() {
        let dir = std::env::temp_dir().join(format!(
            "friday-voice-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("0-core/zero/intents/open")).unwrap();
        let repo = dir.join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        let git = |args: &[&str]| {
            let status = std::process::Command::new("git")
                .args(args)
                .current_dir(&repo)
                .env("GIT_AUTHOR_NAME", "test")
                .env("GIT_AUTHOR_EMAIL", "test@example.com")
                .env("GIT_COMMITTER_NAME", "test")
                .env("GIT_COMMITTER_EMAIL", "test@example.com")
                .status()
                .unwrap();
            assert!(status.success());
        };
        git(&["init"]);
        std::fs::write(repo.join("f"), "x\n").unwrap();
        git(&["add", "f"]);
        git(&["commit", "-m", "today"]);
        let ctx = crate::app::context::AppContext {
            runtime: crate::runtime::Runtime {
                root: dir.clone(),
                logs: dir.join("logs"),
                cache: dir.join("cache"),
                snapshots: dir.join("snapshots"),
                locks: dir.join("locks"),
                db: rusqlite::Connection::open_in_memory().unwrap(),
            },
            capabilities: crate::capabilities::CapabilityContext::unprivileged(),
            home: dir.to_string_lossy().to_string(),
            core_root: repo.to_string_lossy().to_string(),
            source_root: dir.join("0-core/zero").to_string_lossy().to_string(),
        };
        ctx.runtime
            .db
            .execute_batch(
                "CREATE TABLE synthesis_snapshots (
                timestamp INTEGER, health INTEGER, alignment REAL, active_intent TEXT,
                session_commits INTEGER, top_pattern TEXT, friday_brief TEXT,
                brief_confidence REAL, contradiction TEXT
            );
            CREATE TABLE shell_history (id INTEGER PRIMARY KEY, command TEXT, timestamp INTEGER);
            CREATE TABLE friday_patterns (last_seen INTEGER);",
            )
            .unwrap();
        ctx.runtime.db.execute(
            "INSERT INTO synthesis_snapshots (timestamp, health, alignment, active_intent, session_commits, top_pattern, friday_brief, brief_confidence, contradiction)
             VALUES (1, 100, 1.0, 'none', 0, '', 'STALE SNAPSHOT', 0.9, '')",
            [],
        ).unwrap();
        for n in 0..50 {
            ctx.runtime
                .db
                .execute(
                    "INSERT INTO shell_history (command, timestamp) VALUES ('echo', ?1)",
                    rusqlite::params![n],
                )
                .unwrap();
        }
        ctx.runtime
            .db
            .execute("INSERT INTO friday_patterns (last_seen) VALUES (1)", [])
            .unwrap();
        let _guard = crate::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", &dir) };
        let spoken = super::get_voice(&ctx).map(|(text, _)| text);
        match prev {
            Some(h) => unsafe { std::env::set_var("HOME", h) },
            None => unsafe { std::env::remove_var("HOME") },
        }
        let _ = std::fs::remove_dir_all(&dir);
        let spoken = spoken.expect("friday should speak");
        assert!(!spoken.contains("STALE"), "{spoken}");
        assert!(spoken.contains("1 commit(s) today"), "{spoken}");
    }
}

#[cfg(test)]
mod int315 {
    use super::*;

    fn scratch(name: &str) -> (std::path::PathBuf, crate::app::context::AppContext) {
        let dir = std::env::temp_dir().join(format!(
            "int315-friday-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("0-core/zero/intents/open")).unwrap();
        let ctx = crate::app::context::AppContext {
            runtime: crate::runtime::Runtime {
                root: dir.clone(),
                logs: dir.join("logs"),
                cache: dir.join("cache"),
                snapshots: dir.join("snapshots"),
                locks: dir.join("locks"),
                db: rusqlite::Connection::open_in_memory().unwrap(),
            },
            capabilities: crate::capabilities::CapabilityContext::unprivileged(),
            home: dir.to_string_lossy().to_string(),
            core_root: dir.to_string_lossy().to_string(),
            source_root: dir.join("0-core/zero").to_string_lossy().to_string(),
        };
        (dir, ctx)
    }

    struct HomeGuard {
        prev: Option<String>,
    }

    impl HomeGuard {
        fn set(dir: &std::path::Path) -> Self {
            let prev = std::env::var("HOME").ok();
            unsafe { std::env::set_var("HOME", dir) }
            Self { prev }
        }
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            unsafe {
                match self.prev.take() {
                    Some(home) => std::env::set_var("HOME", home),
                    None => std::env::remove_var("HOME"),
                }
            }
        }
    }

    /// The milestone reads git, so the fixture is a real history. commit_patterns
    /// stays at one row: a count taken from that table cannot reach 2000.
    fn seed_commits(repo: &std::path::Path, n: u32) {
        let status = std::process::Command::new("git")
            .args(["init"])
            .current_dir(repo)
            .status()
            .unwrap();
        assert!(status.success(), "git init");
        let head = std::fs::read_to_string(repo.join(".git/HEAD")).unwrap();
        let branch = head.trim().trim_start_matches("ref: ");
        let mut script = String::new();
        for i in 1..=n {
            let from = if i == 1 {
                String::new()
            } else {
                format!("from :{}\n", i - 1)
            };
            script.push_str(&format!(
                "commit {branch}\nmark :{i}\nauthor Test <test@example.com> 1700000000 +0000\ncommitter Test <test@example.com> 1700000000 +0000\ndata 2\nx\n{from}\n"
            ));
        }
        let mut child = std::process::Command::new("git")
            .args(["fast-import", "--quiet"])
            .current_dir(repo)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        {
            use std::io::Write;
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(script.as_bytes()).unwrap();
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "fast-import: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn status_has_no_literal_counts() {
        let line = status_closing_line();
        assert!(!line.contains("892h"), "{line}");
        assert!(!line.contains("14 patterns"), "{line}");
        assert!(!line.contains("333 facts"), "{line}");
    }

    #[test]
    fn milestones_have_no_186_fallback() {
        let (dir, ctx) = scratch("miles");
        let open = dir.join("0-core/zero/intents/open");
        std::fs::write(
            open.join("010-one.md"),
            "---\nid: 10\nstatus: complete\ntitle: \"One\"\n---\n",
        )
        .unwrap();
        std::fs::write(
            open.join("011-two.md"),
            "---\nid: 11\nstatus: complete\ntitle: \"Two\"\n---\n",
        )
        .unwrap();
        ctx.runtime
            .db
            .execute_batch(
                "CREATE TABLE commit_patterns (id INTEGER PRIMARY KEY);
                 INSERT INTO commit_patterns (id) VALUES (1);
                 CREATE TABLE friday_milestones (
                     id INTEGER PRIMARY KEY AUTOINCREMENT,
                     milestone TEXT NOT NULL UNIQUE,
                     triggered_at INTEGER NOT NULL,
                     message TEXT NOT NULL
                 );
                 INSERT INTO friday_milestones (milestone, triggered_at, message) VALUES
                     ('commits_500', 1, 'already'),
                     ('commits_1000', 1, 'already'),
                     ('commits_1500', 1, 'already');",
            )
            .unwrap();
        seed_commits(&dir, 2000);
        let _lock = crate::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _home = HomeGuard::set(&dir);
        let n = completed_intent_count();
        let msg = check_milestones(&ctx).unwrap_or_default();
        assert!(
            !msg.contains("186") && msg.contains(&format!("{n} intents complete")),
            "ledger count is {n}; milestone used a fallback: {msg}"
        );
    }

    const PAST_EVERY_COMMIT_THRESHOLD: u32 = 2600;

    fn commit_milestone_line(line: &str) -> bool {
        line.contains("commits. Past the ")
    }

    fn recorded_milestones(ctx: &crate::app::context::AppContext) -> Vec<String> {
        let mut stmt = ctx
            .runtime
            .db
            .prepare("SELECT milestone FROM friday_milestones ORDER BY id")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(std::result::Result::ok)
            .collect()
    }

    #[test]
    fn milestones_announce_only_the_highest_crossed() {
        let (dir, ctx) = scratch("highest");
        seed_commits(&dir, PAST_EVERY_COMMIT_THRESHOLD);
        let _lock = crate::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _home = HomeGuard::set(&dir);
        let first = check_milestones(&ctx).unwrap_or_default();
        let recorded = recorded_milestones(&ctx);
        let lower = [
            "commits_500",
            "commits_1000",
            "commits_1500",
            "commits_2000",
        ];
        let missing: Vec<&str> = lower
            .into_iter()
            .filter(|key| !recorded.iter().any(|row| row == *key))
            .collect();
        let second = check_milestones(&ctx);
        let second_commit = second.as_deref().filter(|line| commit_milestone_line(line));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            first.contains("Past the 2,500 mark.")
                && first.contains("Project 0 keeps building.")
                && missing.is_empty()
                && second_commit.is_none(),
            "first call announced {first:?} instead of the 2,500 milestone; missing lower commit rows {missing:?} (recorded {recorded:?}); second call commit milestone: {second_commit:?}"
        );
    }

    #[test]
    fn milestone_line_names_the_measured_count() {
        let (dir, ctx) = scratch("measured");
        seed_commits(&dir, PAST_EVERY_COMMIT_THRESHOLD);
        let _lock = crate::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _home = HomeGuard::set(&dir);
        let measured = crate::domains::friday_arch::room::git_commit_count(std::path::Path::new(
            &ctx.core_root,
        ))
        .expect("measured git commit count");
        let line = check_milestones(&ctx).unwrap_or_default();
        let shown = super::with_thousands(measured);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            line.contains(&shown),
            "milestone line {line:?} does not name the measured git commit count {measured}"
        );
    }

    #[test]
    fn first_speech_insert_uses_real_events_v2_columns() {
        let (dir, ctx) = scratch("speech");
        ctx.runtime
            .db
            .execute_batch(
                "CREATE TABLE synthesis_snapshots (
                    timestamp INTEGER, health INTEGER, alignment REAL, active_intent TEXT,
                    session_commits INTEGER, top_pattern TEXT, friday_brief TEXT,
                    brief_confidence REAL, contradiction TEXT
                );
                CREATE TABLE shell_history (id INTEGER PRIMARY KEY, command TEXT, timestamp INTEGER);
                CREATE TABLE friday_patterns (last_seen INTEGER);",
            )
            .unwrap();
        ctx.runtime
            .db
            .execute_batch(crate::domains::events::signal::CREATE_TABLE)
            .unwrap();
        ctx.runtime.db.execute(
            "INSERT INTO synthesis_snapshots (timestamp, health, alignment, active_intent, session_commits, top_pattern, friday_brief, brief_confidence, contradiction)
             VALUES (1, 72, 1.0, 'none', 0, '', 'live', 0.9, '')",
            [],
        ).unwrap();
        for n in 0..50 {
            ctx.runtime
                .db
                .execute(
                    "INSERT INTO shell_history (command, timestamp) VALUES ('echo', ?1)",
                    rusqlite::params![n],
                )
                .unwrap();
        }
        ctx.runtime
            .db
            .execute("INSERT INTO friday_patterns (last_seen) VALUES (1)", [])
            .unwrap();
        let _lock = crate::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _home = HomeGuard::set(&dir);
        let spoken = get_voice(&ctx);
        assert!(
            spoken.is_some(),
            "gates should let Friday speak so the milestone insert runs"
        );
        let n: i64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM events_v2 WHERE kind = 'milestone'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(n, 1, "first speech should land one events_v2 milestone row");
    }

    #[test]
    fn cooldown_line_names_minutes_since_spoken() {
        assert_eq!(cooldown_line(720), "spoke 12m ago");
    }

    #[test]
    fn extract_patterns_twice_does_not_duplicate_rows() {
        let (_dir, ctx) = scratch("patterns");
        super::phase2::ensure_tables(&ctx).unwrap();
        ctx.runtime
            .db
            .execute_batch(
                "CREATE TABLE shell_history (
                    id INTEGER PRIMARY KEY,
                    command TEXT,
                    timestamp INTEGER
                );
                INSERT INTO shell_history (command, timestamp) VALUES
                    ('cargo test -p core', 1),
                    ('cargo test -p core', 2),
                    ('cargo test -p core', 3);",
            )
            .unwrap();
        extract_patterns(&ctx).unwrap();
        let first: i64 = ctx
            .runtime
            .db
            .query_row("SELECT COUNT(*) FROM friday_patterns", [], |row| row.get(0))
            .unwrap();
        extract_patterns(&ctx).unwrap();
        let second: i64 = ctx
            .runtime
            .db
            .query_row("SELECT COUNT(*) FROM friday_patterns", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            first, second,
            "extract_patterns duplicated friday_patterns rows: {first} then {second}"
        );
    }
}

#[cfg(test)]
mod int316 {
    use super::*;
    use crate::domains::doctor::{CheckResult, Status, Tier};

    fn scratch(name: &str) -> (std::path::PathBuf, crate::app::context::AppContext) {
        let dir = std::env::temp_dir().join(format!(
            "int316-friday-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("0-core/zero/intents/open")).unwrap();
        let ctx = crate::app::context::AppContext {
            runtime: crate::runtime::Runtime {
                root: dir.clone(),
                logs: dir.join("logs"),
                cache: dir.join("cache"),
                snapshots: dir.join("snapshots"),
                locks: dir.join("locks"),
                db: rusqlite::Connection::open_in_memory().unwrap(),
            },
            capabilities: crate::capabilities::CapabilityContext::unprivileged(),
            home: dir.to_string_lossy().to_string(),
            core_root: dir.to_string_lossy().to_string(),
            source_root: dir.join("0-core/zero").to_string_lossy().to_string(),
        };
        (dir, ctx)
    }

    struct HomeGuard {
        prev: Option<String>,
    }

    impl HomeGuard {
        fn set(dir: &std::path::Path) -> Self {
            let prev = std::env::var("HOME").ok();
            unsafe { std::env::set_var("HOME", dir) }
            Self { prev }
        }
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            unsafe {
                match self.prev.take() {
                    Some(home) => std::env::set_var("HOME", home),
                    None => std::env::remove_var("HOME"),
                }
            }
        }
    }

    fn probe(name: &str, status: Status) -> CheckResult {
        CheckResult {
            section: "System".into(),
            id: name.to_ascii_lowercase().replace(' ', "_"),
            name: name.into(),
            status,
            tier: Tier::User,
            message: name.into(),
            fix: None,
        }
    }

    fn seed_commits(repo: &std::path::Path, n: u32) {
        let status = std::process::Command::new("git")
            .args(["init"])
            .current_dir(repo)
            .status()
            .unwrap();
        assert!(status.success(), "git init");
        let head = std::fs::read_to_string(repo.join(".git/HEAD")).unwrap();
        let branch = head.trim().trim_start_matches("ref: ");
        let mut script = String::new();
        for i in 1..=n {
            let from = if i == 1 {
                String::new()
            } else {
                format!("from :{}\n", i - 1)
            };
            script.push_str(&format!(
                "commit {branch}\nmark :{i}\nauthor Test <test@example.com> 1700000000 +0000\ncommitter Test <test@example.com> 1700000000 +0000\ndata 2\nx\n{from}\n"
            ));
        }
        let mut child = std::process::Command::new("git")
            .args(["fast-import", "--quiet"])
            .current_dir(repo)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        {
            use std::io::Write;
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(script.as_bytes()).unwrap();
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "fast-import: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn mentions(line: &str, raw: &str, grouped: &str) -> usize {
        line.matches(raw).count() + line.matches(grouped).count()
    }

    #[test]
    fn g1_probe_rows_carry_source_probe_and_an_expiry() {
        let (dir, ctx) = scratch("probe-expiry");
        ctx.runtime.db.execute_batch(CREATE_TABLES).unwrap();
        let checks = vec![probe("Reboot Needed", Status::Warn)];
        record_probe_facts(&ctx, &checks).unwrap();
        let mut stmt = ctx
            .runtime
            .db
            .prepare("SELECT source, expires_at, fact FROM friday_knowledge WHERE source = 'probe'")
            .unwrap_or_else(|err| {
                panic!("probe facts need source='probe' and an expiry: {err}")
            });
        let rows: Vec<(String, i64, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        let _ = std::fs::remove_dir_all(&dir);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert!(
            !rows.is_empty(),
            "after a doctor result, friday_knowledge has no source='probe' rows"
        );
        assert!(
            rows.iter().all(|(source, expires, fact)| {
                source == "probe" && *expires > now && fact.contains("Reboot Needed")
            }),
            "probe rows need source='probe', a future expiry, and the probe result; got {rows:?}"
        );
    }

    #[test]
    fn g1_probe_facts_are_built_from_results_not_written_by_hand() {
        let (dir, ctx) = scratch("probe-built");
        ctx.runtime.db.execute_batch(CREATE_TABLES).unwrap();
        let checks = vec![
            probe("Disk Space", Status::Pass),
            probe("Reboot Needed", Status::Warn),
        ];
        record_probe_facts(&ctx, &checks).unwrap();
        let mut stmt = ctx
            .runtime
            .db
            .prepare("SELECT source, fact FROM friday_knowledge")
            .unwrap();
        let facts: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        let _ = std::fs::remove_dir_all(&dir);
        let from_probe = facts.iter().any(|(source, fact)| {
            source == "probe"
                && fact.contains("Disk Space")
                && fact.to_ascii_lowercase().contains("pass")
        }) && facts.iter().any(|(source, fact)| {
            source == "probe"
                && fact.contains("Reboot Needed")
                && fact.to_ascii_lowercase().contains("warn")
        });
        let handwritten = facts.iter().any(|(_, fact)| fact.contains("I am Friday"));
        assert!(
            from_probe && !handwritten,
            "probe facts are built from the doctor's results and are not hand-written; got {facts:?}"
        );
    }

    #[test]
    fn g2_pacman_hyprland_and_systemd_are_native() {
        let (dir, ctx) = scratch("native");
        let _lock = crate::HOME_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let _home = HomeGuard::set(&dir);
        ctx.runtime
            .db
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS friday_state (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    updated_at INTEGER NOT NULL
                );",
            )
            .unwrap();
        ctx.runtime.db.execute_batch(CREATE_TABLES).unwrap();
        seed_knowledge(&ctx).unwrap();
        seed_linux_knowledge(&ctx).unwrap();
        for (domain, key, fact) in [
            (
                "packages",
                "pacman",
                "pacman installs packages on this machine",
            ),
            ("compositor", "hyprland", "Hyprland is the compositor"),
            ("services", "systemd", "systemd manages services"),
        ] {
            ctx.runtime
                .db
                .execute(
                    "INSERT INTO friday_knowledge
                        (domain, key, fact, confidence, source, created_at, updated_at)
                     VALUES (?1, ?2, ?3, 1.0, 'seed', 1, 1)",
                    rusqlite::params![domain, key, fact],
                )
                .unwrap();
        }
        sync_knowledge_meta(&ctx.runtime.db).unwrap();
        let mut labels = Vec::new();
        for key in ["pacman", "hyprland", "systemd"] {
            let system: String = ctx
                .runtime
                .db
                .query_row(
                    "SELECT system FROM friday_knowledge_meta WHERE key = ?1",
                    [key],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| "missing".into());
            labels.push(format!("{key}={system}"));
        }
        let nix: i64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM friday_knowledge WHERE lower(fact) LIKE '%nixos%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            labels.iter().all(|label| label.ends_with("=native")) && nix == 0,
            "pacman, Hyprland, and systemd are system=native, and no fact says NixOS; got {labels:?}, nixos rows {nix}"
        );
    }

    #[test]
    fn g5_milestone_names_each_number_once() {
        let (dir, ctx) = scratch("milestone");
        seed_commits(&dir, 5415);
        let _lock = crate::HOME_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let _home = HomeGuard::set(&dir);
        let line = check_milestones(&ctx).unwrap_or_default();
        drop(_home);
        drop(_lock);
        let _ = std::fs::remove_dir_all(&dir);
        let count_hits = mentions(&line, "5415", "5,415");
        let mark_hits = mentions(&line, "2500", "2,500");
        assert!(
            line.starts_with("5,415 commits. Past the 2,500 mark.")
                && count_hits == 1
                && mark_hits == 1,
            "each number once; count mentions {count_hits}, mark mentions {mark_hits}; got {line:?}"
        );
    }
}
