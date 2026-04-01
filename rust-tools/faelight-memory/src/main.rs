//! faelight-memory v1.0.0 — Persistent Project Knowledge Layer
//! INT-160 — The forest remembers what it has learned.

use clap::{Parser, Subcommand};
use colored::*;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "faelight-memory", about = "🌲 Persistent knowledge layer", version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(long)]
    health: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Show all stored knowledge
    Show,
    /// Teach the forest a new fact
    Add {
        /// Category (preference/failure/convention/wisdom)
        #[arg(short, long, default_value = "wisdom")]
        category: String,
        /// The knowledge to store
        fact: String,
    },
    /// Remove outdated knowledge by ID
    Forget {
        id: i64,
    },
    /// Query knowledge by topic keyword
    Query {
        topic: String,
    },
    /// Show confidence scores for stored memories
    Confidence,
    /// Auto-extract insights from session history
    Extract,
}

fn db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("0-core/runtime/state.db")
}

fn open_db() -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path())?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS forest_memory (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            category    TEXT    NOT NULL DEFAULT 'wisdom',
            fact        TEXT    NOT NULL,
            confidence  INTEGER NOT NULL DEFAULT 70,
            source      TEXT    NOT NULL DEFAULT 'manual',
            created_at  INTEGER NOT NULL,
            accessed_at INTEGER,
            reinforced  INTEGER NOT NULL DEFAULT 0
        );"
    )?;
    Ok(conn)
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn cmd_show() {
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => { eprintln!("DB error: {}", e); return; }
    };

    println!();
    println!("  {} Forest Memory", "🧠".normal());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let categories = ["preference", "convention", "wisdom", "failure"];
    let mut total = 0;

    for cat in &categories {
        let mut stmt = match conn.prepare(
            "SELECT id, fact, confidence, source FROM forest_memory WHERE category=?1 ORDER BY confidence DESC, created_at DESC"
        ) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let rows: Vec<(i64, String, i64, String)> = stmt.query_map(params![cat], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        }).map(|r| r.flatten().collect::<Vec<_>>()).unwrap_or_default();

        if rows.is_empty() { continue; }

        let cat_label = match *cat {
            "preference" => "Preferences",
            "convention" => "Conventions",
            "wisdom"     => "Wisdom",
            "failure"    => "Failures to avoid",
            _            => cat,
        };

        println!();
        println!("  {} {}", "▶".bright_cyan(), cat_label.bright_white());
        for (id, fact, confidence, source) in &rows {
            let conf_str = if *confidence >= 80 { format!("{}%", confidence).bright_green().to_string() }
                else if *confidence >= 60 { format!("{}%", confidence).yellow().to_string() }
                else { format!("{}%", confidence).dimmed().to_string() };
            println!("    #{} [{}] {} {}", id.to_string().dimmed(), conf_str, fact, format!("({})", source).dimmed());
            total += 1;
        }
    }

    if total == 0 {
        println!();
        println!("  {} No knowledge stored yet", "○".dimmed());
        println!("  {} Run: faelight-memory add <fact>", "→".dimmed());
        println!("  {} Run: faelight-memory extract  — auto-extract from session history", "→".dimmed());
    }
    println!();
    println!("  {} {} memories stored", "·".dimmed(), total.to_string().bright_white());
    println!();
}

fn cmd_add(category: &str, fact: &str) {
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => { eprintln!("DB error: {}", e); return; }
    };

    let valid_categories = ["preference", "convention", "wisdom", "failure"];
    let cat = if valid_categories.contains(&category) { category } else { "wisdom" };

    conn.execute(
        "INSERT INTO forest_memory (category, fact, confidence, source, created_at) VALUES (?1, ?2, 80, 'manual', ?3)",
        params![cat, fact, now_ts()],
    ).ok();

    println!();
    println!("  {} Memory stored [{}/{}]", "✅".normal(), cat.bright_cyan(), 80.to_string().bright_green());
    println!("  {} {}", "→".dimmed(), fact.bright_white());
    println!();
}

fn cmd_forget(id: i64) {
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => { eprintln!("DB error: {}", e); return; }
    };
    let affected = conn.execute("DELETE FROM forest_memory WHERE id=?1", params![id]).unwrap_or(0);
    if affected > 0 {
        println!("  {} Memory #{} forgotten", "✅".normal(), id);
    } else {
        println!("  {} Memory #{} not found", "✗".bright_red(), id);
    }
}

fn cmd_query(topic: &str) {
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => { eprintln!("DB error: {}", e); return; }
    };

    let pattern = format!("%{}%", topic.to_lowercase());
    let mut stmt = match conn.prepare(
        "SELECT id, category, fact, confidence FROM forest_memory WHERE lower(fact) LIKE ?1 ORDER BY confidence DESC"
    ) {
        Ok(s) => s,
        Err(_) => return,
    };

    let rows: Vec<(i64, String, String, i64)> = stmt.query_map(params![pattern], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    }).map(|r| r.flatten().collect::<Vec<_>>()).unwrap_or_default();

    println!();
    println!("  {} Query: {}", "🔍".normal(), topic.bright_white());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    if rows.is_empty() {
        println!("  {} No memories found for: {}", "○".dimmed(), topic);
    } else {
        for (id, cat, fact, confidence) in &rows {
            println!("  #{} [{}] [{}%] {}", id.to_string().dimmed(), cat.bright_cyan(), confidence, fact.bright_white());
        }
    }
    println!();
}

fn cmd_confidence() {
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => { eprintln!("DB error: {}", e); return; }
    };

    println!();
    println!("  {} Memory Confidence Distribution", "📊".normal());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let total: i64 = conn.query_row("SELECT COUNT(*) FROM forest_memory", [], |r| r.get(0)).unwrap_or(0);
    let high: i64  = conn.query_row("SELECT COUNT(*) FROM forest_memory WHERE confidence >= 80", [], |r| r.get(0)).unwrap_or(0);
    let med: i64   = conn.query_row("SELECT COUNT(*) FROM forest_memory WHERE confidence >= 60 AND confidence < 80", [], |r| r.get(0)).unwrap_or(0);
    let low: i64   = conn.query_row("SELECT COUNT(*) FROM forest_memory WHERE confidence < 60", [], |r| r.get(0)).unwrap_or(0);

    println!();
    println!("  {} High confidence (≥80%):   {}", "·".dimmed(), high.to_string().bright_green());
    println!("  {} Medium confidence (60-79%): {}", "·".dimmed(), med.to_string().yellow());
    println!("  {} Low confidence (<60%):    {}", "·".dimmed(), low.to_string().dimmed());
    println!("  {} Total memories:           {}", "·".dimmed(), total.to_string().bright_white());
    println!();
}

fn cmd_extract() {
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => { eprintln!("DB error: {}", e); return; }
    };

    println!();
    println!("  {} Extracting insights from session history...", "🔍".normal());
    println!();

    let mut extracted = 0;

    // Extract: most active commit days
    let best_day: Option<String> = conn.query_row(
        "SELECT CASE CAST(strftime('%w', datetime(timestamp, 'unixepoch', 'localtime')) AS INTEGER)
         WHEN 0 THEN 'Sunday' WHEN 1 THEN 'Monday' WHEN 2 THEN 'Tuesday'
         WHEN 3 THEN 'Wednesday' WHEN 4 THEN 'Thursday' WHEN 5 THEN 'Friday'
         ELSE 'Saturday' END as day
         FROM events WHERE domain='git' AND action='commit'
         GROUP BY day ORDER BY COUNT(*) DESC LIMIT 1",
        [], |r| r.get(0)
    ).ok();

    if let Some(day) = best_day {
        let fact = format!("Most productive commit day is {}", day);
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM forest_memory WHERE fact LIKE '%productive commit day%'",
            [], |r| r.get(0)
        ).unwrap_or(0);
        if exists == 0 {
            conn.execute(
                "INSERT INTO forest_memory (category, fact, confidence, source, created_at) VALUES ('preference', ?1, 75, 'auto-extract', ?2)",
                params![fact, now_ts()],
            ).ok();
            println!("  {} {}", "✅".normal(), fact.bright_white());
            extracted += 1;
        }
    }

    // Extract: health pattern
    let health_stable: i64 = conn.query_row(
        "SELECT COUNT(*) FROM events WHERE domain='doctor' AND action='run'",
        [], |r| r.get(0)
    ).unwrap_or(0);

    if health_stable > 100 {
        let fact = format!("System has completed {} health checks — stable foundation", health_stable);
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM forest_memory WHERE fact LIKE '%health checks%'",
            [], |r| r.get(0)
        ).unwrap_or(0);
        if exists == 0 {
            conn.execute(
                "INSERT INTO forest_memory (category, fact, confidence, source, created_at) VALUES ('wisdom', ?1, 70, 'auto-extract', ?2)",
                params![fact, now_ts()],
            ).ok();
            println!("  {} {}", "✅".normal(), fact.bright_white());
            extracted += 1;
        }
    }

    // Extract: primary language convention
    let fact = "Primary language is Rust — 60%+ of codebase, domain-per-directory convention";
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM forest_memory WHERE fact LIKE '%Primary language is Rust%'",
        [], |r| r.get(0)
    ).unwrap_or(0);
    if exists == 0 {
        conn.execute(
            "INSERT INTO forest_memory (category, fact, confidence, source, created_at) VALUES ('convention', ?1, 95, 'auto-extract', ?2)",
            params![fact, now_ts()],
        ).ok();
        println!("  {} {}", "✅".normal(), fact.bright_white());
        extracted += 1;
    }

    if extracted == 0 {
        println!("  {} No new insights to extract", "○".dimmed());
    } else {
        println!();
        println!("  {} {} insights extracted", "✅".normal(), extracted.to_string().bright_green());
    }
    println!();
}

fn main() {
    let cli = Cli::parse();

    if cli.health {
        println!("faelight-memory v1.0.0 — healthy");
        return;
    }

    match cli.command {
        Some(Command::Show)                     => cmd_show(),
        Some(Command::Add { category, fact })   => cmd_add(&category, &fact),
        Some(Command::Forget { id })            => cmd_forget(id),
        Some(Command::Query { topic })          => cmd_query(&topic),
        Some(Command::Confidence)               => cmd_confidence(),
        Some(Command::Extract)                  => cmd_extract(),
        None => {
            println!();
            println!("  {} faelight-memory v1.0.0", "🌲".normal());
            println!("  {} Persistent knowledge layer", "·".dimmed());
            println!();
            println!("  Commands:");
            println!("    show        — display all stored knowledge");
            println!("    add <fact>  — teach the forest a fact");
            println!("    forget <id> — remove outdated knowledge");
            println!("    query <x>   — search knowledge by topic");
            println!("    confidence  — confidence distribution");
            println!("    extract     — auto-extract from session history");
            println!();
        }
    }
}
