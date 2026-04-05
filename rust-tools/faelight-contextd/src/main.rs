use colored::*;
use rusqlite::{Connection, params};
enum Cmd { Start, Status, Insights, Log }
impl Cmd {
    fn from_args() -> Self {
        match std::env::args().nth(1).as_deref() {
            Some("start") => Cmd::Start,
            Some("insights") => Cmd::Insights,
            Some("log") | Some("signal-log") => Cmd::Log,
            _ => Cmd::Status,
        }
    }
}
fn db_path() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{}/0-core/runtime/state.db", home)
}
fn open_db() -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path())?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    Ok(conn)
}
fn show_insights(conn: &Connection) {
    let mut stmt = conn.prepare(
        "SELECT signal, detail, importance, confidence, created_at FROM forest_insights
         WHERE shown = 0 ORDER BY importance DESC, created_at DESC LIMIT 10"
    ).unwrap();
    let rows: Vec<(String, String, f64, f64, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
        .unwrap().filter_map(|r| r.ok()).collect();
    if rows.is_empty() {
        println!("  {} No pending insights", "○".dimmed());
        return;
    }
    println!("
  {} Contextd Insights
  {}", "💡", "─".repeat(48).dimmed());
    for (signal, detail, importance, confidence, ts) in &rows {
        let time = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|t| t.format("%H:%M").to_string()).unwrap_or_default();
        println!("  💡 {} {}", signal.bright_yellow().bold(), time.dimmed());
        println!("     {}", detail.bright_white());
        println!("     Importance: {:.0}%  Confidence: {:.0}%", importance * 100.0, confidence * 100.0);
    }
    println!();
}
fn insert_insight(conn: &Connection, signal: &str, detail: &str, importance: f64, confidence: f64, now: i64) {
    if importance < 0.70 || confidence < 0.60 { return; }
    let cooldown: i64 = conn.query_row(
        "SELECT COUNT(*) FROM forest_insights WHERE signal = ?1 AND created_at > ?2",
        params![signal, now - 3600], |r| r.get(0)
    ).unwrap_or(0);
    if cooldown > 0 { return; }
    let expires = now + 3600;
    let _ = conn.execute(
        "INSERT INTO forest_insights (signal, detail, importance, confidence, expires_at, shown, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
        params![signal, detail, importance, confidence, expires, now],
    );
}
fn detect_signals(conn: &Connection) {
    let now = chrono::Utc::now().timestamp();
    let window_5m = now - 300;
    let window_2h = now - 7200;
    let window_10m = now - 600;
    let window_1h = now - 3600;
    // Signal 1: failure loop — forest notices when you are stuck
    let recent_failures: i64 = conn.query_row(
        "SELECT COUNT(*) FROM forest_events WHERE kind = 'CommandFailed' AND timestamp > ?1",
        params![window_5m], |r| r.get(0)
    ).unwrap_or(0);
    if recent_failures >= 4 {
        insert_insight(conn, "failure-loop",
            &format!("{} failures in 5 minutes — something is broken. Check the error above carefully.", recent_failures),
            0.90, 0.85, now);
    } else if recent_failures >= 2 {
        insert_insight(conn, "failure-pattern-forming",
            &format!("{} failures in 5 minutes — pattern forming. What changed recently?", recent_failures),
            0.72, 0.65, now);
    }
    // Signal 2: deploy without health check
    let last_deploy: Option<i64> = conn.query_row(
        "SELECT timestamp FROM forest_events WHERE kind = 'CommandSucceeded' AND domain = 'deploy' ORDER BY timestamp DESC LIMIT 1",
        [], |r| r.get(0)
    ).ok();
    if let Some(deploy_ts) = last_deploy {
        if deploy_ts > window_2h {
            let health_after: i64 = conn.query_row(
                "SELECT COUNT(*) FROM forest_events WHERE kind = 'CommandSucceeded' AND domain = 'doctor' AND timestamp > ?1",
                params![deploy_ts], |r| r.get(0)
            ).unwrap_or(0);
            if health_after == 0 {
                insert_insight(conn, "deploy-unchecked",
                    "You deployed something — health not checked yet. Run d to verify nothing broke.",
                    0.78, 0.72, now);
            }
        }
    }
    // Signal 3: focus fragmentation
    let mut stmt = conn.prepare(
        "SELECT DISTINCT domain FROM forest_events WHERE timestamp > ?1"
    ).unwrap();
    let domains: Vec<String> = stmt
        .query_map(params![window_10m], |r| r.get(0))
        .unwrap().filter_map(|r| r.ok()).collect();
    if domains.len() >= 5 {
        insert_insight(conn, "focus-fragmentation",
            &format!("Touching {} domains in 10 minutes — deep focus builds better than wide. Pick one thread.", domains.len()),
            0.70, 0.65, now);
    }
    // Signal 4: long build session — suggest checkpoint
    let commit_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM forest_events WHERE kind = 'CommandSucceeded' AND domain = 'git' AND timestamp > ?1",
        params![window_1h], |r| r.get(0)
    ).unwrap_or(0);
    let deploy_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM forest_events WHERE kind = 'CommandSucceeded' AND domain = 'deploy' AND timestamp > ?1",
        params![window_1h], |r| r.get(0)
    ).unwrap_or(0);
    if commit_count >= 3 && deploy_count >= 3 {
        insert_insight(conn, "high-velocity-session",
            &format!("{} commits and {} deploys this hour — strong momentum. Consider a checkpoint snapshot.", commit_count, deploy_count),
            0.68, 0.70, now);
    }
    // Signal 5: uncommitted work sitting too long
    let last_commit: Option<i64> = conn.query_row(
        "SELECT timestamp FROM forest_events WHERE domain = 'git' AND kind = 'CommandSucceeded' ORDER BY timestamp DESC LIMIT 1",
        [], |r| r.get(0)
    ).ok();
    let last_deploy2: Option<i64> = conn.query_row(
        "SELECT timestamp FROM forest_events WHERE domain = 'deploy' AND kind = 'CommandSucceeded' ORDER BY timestamp DESC LIMIT 1",
        [], |r| r.get(0)
    ).ok();
    if let (Some(last_d), Some(last_c)) = (last_deploy2, last_commit) {
        if last_d > last_c + 1800 {
            // Deploy happened 30+ min after last commit
            insert_insight(conn, "work-uncommitted",
                "Deployed work that hasn't been committed yet — fg commit to save your progress.",
                0.75, 0.70, now);
        }
    }
}
fn auto_verify_predictions(conn: &Connection) {
    // Auto-verify predictions: if predicted command ran within 10 min, mark correct
    let now = chrono::Utc::now().timestamp();
    let window_10m = now - 600;
    
    // Get unverified predictions
    let preds: Vec<(i64, String, String)> = {
        let mut stmt = conn.prepare(
            "SELECT id, kind, prediction FROM forest_predictions
             WHERE id NOT IN (SELECT prediction_id FROM prediction_outcomes)
             AND created_at > ?1"
        ).unwrap();
        stmt.query_map(params![now - 86400], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap().filter_map(|r| r.ok()).collect()
    };
    
    for (id, kind, _prediction) in preds {
        // Check if predicted domain had activity in last 10 min
        let domain = match kind.as_str() {
            "sessions" => "doctor",
            "cadence" => "intent", 
            _ => continue,
        };
        let activity: i64 = conn.query_row(
            "SELECT COUNT(*) FROM forest_events WHERE domain = ?1 AND timestamp > ?2",
            params![domain, window_10m], |r| r.get(0)
        ).unwrap_or(0);
        
        if activity > 0 {
            let _ = conn.execute(
                "INSERT INTO prediction_outcomes (prediction_id, actual, correct, verified_at)
                 VALUES (?1, 'auto-verified', 1, ?2)",
                params![id, now],
            );
        }
    }
}
fn run_once(conn: &Connection) {
    detect_signals(conn);
    auto_verify_predictions(conn);
    let now = chrono::Utc::now().timestamp();
    let _ = conn.execute(
        "UPDATE forest_insights SET shown = 1 WHERE expires_at < ?1",
        params![now],
    );
}
fn main() {
    match Cmd::from_args() {
        Cmd::Status => {
            let conn = open_db().expect("db error");
            let event_count: i64 = conn.query_row("SELECT COUNT(*) FROM forest_events", [], |r| r.get(0)).unwrap_or(0);
            let insight_count: i64 = conn.query_row("SELECT COUNT(*) FROM forest_insights WHERE shown = 0", [], |r| r.get(0)).unwrap_or(0);
            println!("
  🧠 faelight-contextd");
            println!("  {}", "─".repeat(40).dimmed());
            println!("  {:<20} {}", "events observed:".dimmed(), event_count.to_string().bright_white());
            println!("  {:<20} {}", "pending insights:".dimmed(), insight_count.to_string().bright_cyan());
            println!("  {:<20} {}", "db:".dimmed(), db_path().dimmed());
            println!();
        }
        Cmd::Insights => {
            let conn = open_db().expect("db error");
            show_insights(&conn);
        }
        Cmd::Start => {
            println!("  🧠 faelight-contextd starting (30s poll interval)...");
            let conn = open_db().expect("db error");
            loop {
                run_once(&conn);
                std::thread::sleep(std::time::Duration::from_secs(30));
            }
        }
        Cmd::Log => {
            let conn = open_db().expect("db error");
            let mut stmt = conn.prepare(
                "SELECT kind, domain, detail, timestamp FROM forest_events ORDER BY timestamp DESC LIMIT 20"
            ).unwrap();
            let rows: Vec<(String, String, String, i64)> = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
                .unwrap().filter_map(|r| r.ok()).collect();
            println!("
  📋 Recent forest events
  {}", "─".repeat(48).dimmed());
            for (kind, domain, detail, ts) in &rows {
                let time = chrono::DateTime::from_timestamp(*ts, 0)
                    .map(|t| t.format("%H:%M:%S").to_string()).unwrap_or_default();
                let k = if kind.contains("Succeeded") { kind.bright_green() } else { kind.bright_red() };
                println!("  {} {} {} {}", time.dimmed(), k, domain.bright_cyan(), detail.dimmed());
            }
            println!();
        }
    }
}
