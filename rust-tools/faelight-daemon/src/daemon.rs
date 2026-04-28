//! The faelight daemon server - LEGENDARY EDITION
use crate::protocol::{Command, Entry, Message, MessagePayload, Response};
use colored::*;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;

/// Broadcast channel capacity — 256 events before oldest dropped
const BROADCAST_CAP: usize = 256;

/// A single event broadcast to all subscribers
#[derive(Clone, Debug)]
pub struct EventBroadcast {
    pub domain: String,
    pub action: String,
    pub payload: Option<String>,
    pub timestamp: i64,
}

pub struct Daemon {
    socket_path: String,
}

impl Daemon {
    pub fn new(socket_path: String) -> Self {
        Self { socket_path }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Remove old socket if exists
        let path = Path::new(&self.socket_path);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Event broadcast channel
        let (tx, _) = broadcast::channel::<EventBroadcast>(BROADCAST_CAP);
        let tx = Arc::new(tx);

        // Spawn SQLite polling task
        let poll_tx = tx.clone();
        let db_path = {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{}/0-core/runtime/state.db", home)
        };
        let db_path_poll = db_path.clone();
        tokio::spawn(async move {
            poll_events(poll_tx, db_path_poll).await;
        });
        // INT-196 v2 — Health watchdog (checks every 60 seconds)
        let watchdog_db = db_path.clone();
        tokio::spawn(async move {
            health_watchdog(watchdog_db).await;
        });
        // INT-196 v2 — Prediction pre-compute (every 30 seconds)
        let predict_db = db_path.clone();
        tokio::spawn(async move {
            prediction_precompute(predict_db).await;
        });
        // INT-196 v2 — Signal aggregation (every 30 seconds)
        let signal_db = db_path.clone();
        tokio::spawn(async move {
            signal_aggregation(signal_db).await;
        });
        // INT-220 Gate 6 -- Friday learning loop (runs every 30 minutes)
        tokio::spawn(async move {
            friday_learning_loop().await;
        });

        // INT-249b -- WAL checkpoint loop (every 5 minutes)
        // Prevents WAL bloat that causes morning-after-suspend SQLITE_READONLY warnings.
        let checkpoint_db = db_path.clone();
        tokio::spawn(async move {
            wal_checkpoint_loop(checkpoint_db).await;
        });

        // Bind to Unix socket
        let listener = UnixListener::bind(&self.socket_path)?;
        let mut connection_count = 0;

        let log_path = {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{}/.cache/faelight/friday.log", home)
        };
        // Ensure log dir exists
        if let Some(parent) = std::path::Path::new(&log_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    connection_count += 1;
                    let conn_id = connection_count;
                    let sub_rx = tx.subscribe();
                    let log = log_path.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream, conn_id, sub_rx).await {
                            // Only log real errors, not broken pipe (fire-and-forget clients)
                            if !e.to_string().contains("Broken pipe")
                                && !e.to_string().contains("os error 32")
                            {
                                let entry = format!(
                                    "[friday] conn#{} error: {}
",
                                    conn_id, e
                                );
                                let _ = std::fs::OpenOptions::new()
                                    .append(true)
                                    .create(true)
                                    .open(&log)
                                    .map(|mut f| {
                                        use std::io::Write;
                                        let _ = f.write_all(entry.as_bytes());
                                    });
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("{} Accept error: {}", "❌".red(), e);
                }
            }
        }
    }
}

/// Poll SQLite events table every 2s, broadcast new events to subscribers
async fn poll_events(tx: Arc<broadcast::Sender<EventBroadcast>>, db_path: String) {
    let mut last_ts: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
        - 5; // small buffer to catch events written same second

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let Ok(conn) = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) else {
            continue;
        };

        let Ok(mut stmt) = conn.prepare(
            "SELECT domain, action, payload, timestamp FROM events \
             WHERE timestamp > ? ORDER BY timestamp ASC LIMIT 50",
        ) else {
            continue;
        };

        let rows: Vec<EventBroadcast> = stmt
            .query_map(rusqlite::params![last_ts], |row| {
                Ok(EventBroadcast {
                    domain: row.get(0)?,
                    action: row.get(1)?,
                    payload: row.get(2)?,
                    timestamp: row.get(3)?,
                })
            })
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default();

        for event in rows {
            last_ts = last_ts.max(event.timestamp);
            let _ = tx.send(event);
        }
    }
}

async fn handle_client(
    stream: UnixStream,
    _conn_id: u64,
    mut sub_rx: broadcast::Receiver<EventBroadcast>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        let message: Message = match serde_json::from_str(&line) {
            Ok(msg) => msg,
            Err(_e) => {
                // Silent -- bad JSON from fire-and-forget clients is expected
                continue;
            }
        };

        // Commands logged silently -- no stdout noise

        let cmd = match message.payload {
            MessagePayload::Command(cmd) => cmd,
            _ => {
                let err = Message {
                    id: message.id,
                    payload: MessagePayload::Response(Response::Error {
                        message: "Expected command".to_string(),
                    }),
                };
                writer
                    .write_all(serde_json::to_string(&err)?.as_bytes())
                    .await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
                continue;
            }
        };

        match cmd {
            // ── Streaming commands — enter persistent loop ─────────────────
            Command::Subscribe { ref domains } => {
                let filter: Vec<String> = domains.clone();
                let confirm = Message {
                    id: message.id,
                    payload: MessagePayload::Response(Response::Subscribed {
                        domains: filter.clone(),
                    }),
                };
                writer
                    .write_all(serde_json::to_string(&confirm)?.as_bytes())
                    .await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;

                loop {
                    match sub_rx.recv().await {
                        Ok(event) => {
                            if filter.is_empty() || filter.contains(&event.domain) {
                                let msg = Message {
                                    id: 0,
                                    payload: MessagePayload::Response(Response::Event {
                                        domain: event.domain,
                                        action: event.action,
                                        payload: event.payload,
                                        timestamp: event.timestamp,
                                    }),
                                };
                                if writer
                                    .write_all(serde_json::to_string(&msg)?.as_bytes())
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                                if writer.write_all(b"\n").await.is_err() {
                                    break;
                                }
                                if writer.flush().await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            eprintln!("{} Subscriber lagged, dropped {} events", "⚠️".yellow(), n);
                        }
                        Err(_) => break,
                    }
                }
                return Ok(());
            }
            Command::EventStream => {
                let confirm = Message {
                    id: message.id,
                    payload: MessagePayload::Response(Response::Subscribed { domains: vec![] }),
                };
                writer
                    .write_all(serde_json::to_string(&confirm)?.as_bytes())
                    .await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;

                loop {
                    match sub_rx.recv().await {
                        Ok(event) => {
                            let msg = Message {
                                id: 0,
                                payload: MessagePayload::Response(Response::Event {
                                    domain: event.domain,
                                    action: event.action,
                                    payload: event.payload,
                                    timestamp: event.timestamp,
                                }),
                            };
                            if writer
                                .write_all(serde_json::to_string(&msg)?.as_bytes())
                                .await
                                .is_err()
                            {
                                break;
                            }
                            if writer.write_all(b"\n").await.is_err() {
                                break;
                            }
                            if writer.flush().await.is_err() {
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            eprintln!("{} Stream lagged, dropped {} events", "⚠️".yellow(), n);
                        }
                        Err(_) => break,
                    }
                }
                return Ok(());
            }
            // ── Request/response commands ──────────────────────────────────
            other => {
                let response = process_command(other).await;
                let response_msg = Message {
                    id: message.id,
                    payload: MessagePayload::Response(response),
                };
                let json = serde_json::to_string(&response_msg)?;
                writer.write_all(json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
            }
        }
    }
    Ok(())
}

async fn process_command(cmd: Command) -> Response {
    match cmd {
        Command::Ping => Response::Pong,
        Command::GetEntries { path } => match read_directory(&path).await {
            Ok(entries) => Response::Entries { entries },
            Err(e) => Response::Error {
                message: format!("Failed to read directory: {}", e),
            },
        },
        Command::Preview { path } => match tokio::fs::read_to_string(&path).await {
            Ok(content) => Response::Preview { content },
            Err(e) => Response::Error {
                message: format!("Failed to read file: {}", e),
            },
        },
        Command::Search { query: _ } => Response::Error {
            message: "Search not implemented yet".to_string(),
        },
        Command::GitStatus { path: _ } => Response::GitStatus {
            status: "Git status not implemented yet".to_string(),
        },
        Command::Shutdown => {
            println!("{} Shutdown requested", "🛑".red().bold());
            std::process::exit(0);
        }
        Command::GetForestContext => get_forest_context().await,
        Command::GetPrediction => get_prediction().await,
        Command::WatchdogStatus => get_watchdog_status().await,
        Command::GetEngineSignals { limit } => get_engine_signals(limit).await,
        Command::GetNeovimContext { file_path } => get_neovim_context(file_path).await,
        // INT-220 -- Friday event: record command for learning
        Command::FridayEvent {
            command,
            exit_code,
            duration_ms,
            intent,
            health,
            timestamp,
        } => friday_record_event(command, exit_code, duration_ms, intent, health, timestamp).await,
        // INT-220 Gate 11 -- Friday dismiss: negative learning
        Command::FridayDismiss { pattern_trigger } => friday_dismiss(pattern_trigger).await,

        // INT-220 -- Friday query: answer a question about the forest
        Command::FridayQuery { question, context } => friday_answer_query(question, context).await,
        // Streaming commands handled above — should not reach here
        Command::Subscribe { .. } | Command::EventStream => Response::Error {
            message: "Streaming command reached process_command — bug".to_string(),
        },
    }
}

async fn read_directory(path: &str) -> Result<Vec<Entry>, std::io::Error> {
    let mut entries = Vec::new();
    let mut dir = tokio::fs::read_dir(path).await?;
    while let Some(entry) = dir.next_entry().await? {
        let metadata = entry.metadata().await?;
        let name = entry.file_name().to_string_lossy().to_string();
        entries.push(Entry {
            name,
            path: entry.path().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
        });
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}
// ── INT-196 v2 Background Tasks ───────────────────────────────────────────────
/// Health watchdog — checks every 60 seconds, alerts on drops
async fn health_watchdog(db_path: String) {
    let mut last_health: u32 = 100;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        let health = read_health_cache();
        if health < 95 && last_health >= 95 {
            println!(
                "⚠️  WATCHDOG: Health dropped to {}% — was {}%",
                health, last_health
            );
            // Write alert to state.db
            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let now = chrono::Utc::now().timestamp();
                let _ = conn.execute(
                    "INSERT INTO engine_signals (source, signal_type, payload, weight, created_at)
                     VALUES ('watchdog', 'health_alert', ?1, ?2, ?3)",
                    rusqlite::params![
                        format!("{{\"health\":{},\"previous\":{}}}", health, last_health),
                        health as f64 / 100.0,
                        now
                    ],
                );
            }
        } else if health >= 100 && last_health < 95 {
            println!("✅ WATCHDOG: Health restored to {}%", health);
        }
        last_health = health;
    }
}
// INT-249b: periodic WAL checkpoint -- runs every 5 minutes to keep the
// shared WAL trimmed, preventing morning-after-suspend SQLITE_READONLY warnings.
async fn wal_checkpoint_loop(db_path: String) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let _ = conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE)");
        }
    }
}
/// Prediction pre-compute — runs every 30 seconds
async fn prediction_precompute(db_path: String) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        let Ok(conn) = rusqlite::Connection::open(&db_path) else {
            continue;
        };
        let now = chrono::Utc::now().timestamp();
        // Find most frequent next command in history
        let suggestion: Option<String> = conn
            .query_row(
                "SELECT next_cmd FROM (
               SELECT h2.command as next_cmd, COUNT(*) as freq
               FROM shell_history h1
               JOIN shell_history h2 ON h2.id = h1.id + 1
               WHERE h1.timestamp > ?1
               GROUP BY next_cmd ORDER BY freq DESC LIMIT 1
             )",
                rusqlite::params![now - 86400],
                |r| r.get(0),
            )
            .ok();
        if let Some(ref s) = suggestion {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('daemon_prediction', ?1)",
                rusqlite::params![s],
            );
        }
    }
}
/// Signal aggregation — summarizes engine signals every 30 seconds
async fn signal_aggregation(db_path: String) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        let Ok(conn) = rusqlite::Connection::open(&db_path) else {
            continue;
        };
        let now = chrono::Utc::now().timestamp();
        // Count signals in last hour
        let signal_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM engine_signals WHERE created_at > ?1",
                rusqlite::params![now - 3600],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if signal_count > 0 {
            // Update daemon activity log
            let _ = conn.execute(
                "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('daemon_signal_count_1h', ?1)",
                rusqlite::params![signal_count.to_string()],
            );
        }
    }
}
// ── INT-196 v2 Command Implementations ───────────────────────────────────────
fn get_db_path() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{}/0-core/runtime/state.db", home)
}
fn read_health_cache() -> u32 {
    let home = std::env::var("HOME").unwrap_or_default();
    std::fs::read_to_string(format!("{}/.cache/faelight/health-status", home))
        .unwrap_or_else(|_| "100".to_string())
        .trim()
        .parse()
        .unwrap_or(100)
}
async fn get_forest_context() -> crate::protocol::Response {
    let db_path = get_db_path();
    let Ok(conn) = rusqlite::Connection::open(&db_path) else {
        return crate::protocol::Response::Error {
            message: "Cannot open state.db".to_string(),
        };
    };
    let health = read_health_cache();
    let alignment: f64 = conn.query_row(
        "SELECT AVG(score) FROM alignment_checks WHERE checked_at > (strftime('%s','now') - 604800)",
        [], |r| r.get::<_, Option<f64>>(0)
    ).unwrap_or(None).unwrap_or(1.0);
    let friday_status: String = conn
        .query_row(
            "SELECT status FROM engine_registry WHERE name = 'friday'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "dormant".to_string());
    // Get active intent from filesystem
    let core_root = format!("{}/0-core", std::env::var("HOME").unwrap_or_default());
    let active_intent = std::fs::read_dir(format!("{}/intents/future", core_root))
        .ok()
        .and_then(|d| {
            d.filter_map(|e| e.ok())
                .find(|e| {
                    std::fs::read_to_string(e.path())
                        .map(|c| c.contains("status: in-progress"))
                        .unwrap_or(false)
                })
                .map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let num = name.split('-').next().unwrap_or("").to_string();
                    format!("INT-{}", num)
                })
        });
    // Commits today
    let _today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let commits_today: i64 = std::process::Command::new("git")
        .args(["-C", &core_root, "log", "--oneline", "--since=midnight"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count() as i64)
        .unwrap_or(0);
    // Top prediction
    let top_prediction: Option<String> = conn
        .query_row(
            "SELECT value FROM shell_state WHERE key = 'daemon_prediction'",
            [],
            |r| r.get(0),
        )
        .ok();
    crate::protocol::Response::ForestContext {
        health,
        alignment,
        active_intent,
        commits_today,
        friday_status,
        top_prediction,
    }
}
async fn get_prediction() -> crate::protocol::Response {
    let db_path = get_db_path();
    let Ok(conn) = rusqlite::Connection::open(&db_path) else {
        return crate::protocol::Response::Prediction {
            suggestion: None,
            confidence: 0.0,
            cached_at: 0,
        };
    };
    let now = chrono::Utc::now().timestamp();
    let suggestion: Option<String> = conn
        .query_row(
            "SELECT value FROM shell_state WHERE key = 'daemon_prediction'",
            [],
            |r| r.get(0),
        )
        .ok();
    crate::protocol::Response::Prediction {
        suggestion,
        confidence: 0.75,
        cached_at: now,
    }
}
async fn get_watchdog_status() -> crate::protocol::Response {
    let db_path = get_db_path();
    let Ok(conn) = rusqlite::Connection::open(&db_path) else {
        return crate::protocol::Response::Watchdog {
            last_check: 0,
            last_health: 0,
            alerts_today: 0,
        };
    };
    let now = chrono::Utc::now().timestamp();
    let alerts_today: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM engine_signals WHERE source = 'watchdog' AND created_at > ?1",
            rusqlite::params![now - 86400],
            |r| r.get(0),
        )
        .unwrap_or(0);
    crate::protocol::Response::Watchdog {
        last_check: now,
        last_health: read_health_cache(),
        alerts_today,
    }
}
async fn get_engine_signals(limit: u32) -> crate::protocol::Response {
    let db_path = get_db_path();
    let Ok(conn) = rusqlite::Connection::open(&db_path) else {
        return crate::protocol::Response::EngineSignals { signals: vec![] };
    };
    let mut stmt = match conn.prepare(
        "SELECT source, signal_type, payload, weight, created_at
         FROM engine_signals ORDER BY created_at DESC LIMIT ?1",
    ) {
        Ok(s) => s,
        Err(_) => return crate::protocol::Response::EngineSignals { signals: vec![] },
    };
    let signals: Vec<crate::protocol::SignalEntry> = stmt
        .query_map(rusqlite::params![limit], |r| {
            Ok(crate::protocol::SignalEntry {
                source: r.get(0)?,
                signal_type: r.get(1)?,
                payload: r.get(2).unwrap_or_default(),
                weight: r.get(3)?,
                created_at: r.get(4)?,
            })
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();
    crate::protocol::Response::EngineSignals { signals }
}
async fn get_neovim_context(file_path: String) -> crate::protocol::Response {
    let core_root = format!("{}/0-core", std::env::var("HOME").unwrap_or_default());
    // Find active intent
    let active = std::fs::read_dir(format!("{}/intents/future", core_root))
        .ok()
        .and_then(|d| {
            d.filter_map(|e| e.ok())
                .find(|e| {
                    std::fs::read_to_string(e.path())
                        .map(|c| c.contains("status: in-progress"))
                        .unwrap_or(false)
                })
                .map(|e| e.path())
        });
    let (active_intent, intent_title) = match active {
        Some(path) => {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let num = name.split('-').next().unwrap_or("").to_string();
            let intent_id = format!("INT-{}", num);
            // Extract title from filename
            let title = name
                .trim_end_matches(".md")
                .splitn(3, '-')
                .nth(2)
                .unwrap_or("")
                .replace('-', " ");
            (Some(intent_id), Some(title))
        }
        None => (None, None),
    };
    // Generate suggestion based on file being edited
    let suggestion = if file_path.contains("commands/mod.rs") || file_path.contains("commands.rs") {
        Some(
            "Editing commands — remember to wire through parser.rs, cli/mod.rs, dispatcher.rs"
                .to_string(),
        )
    } else if file_path.contains("main.rs") && file_path.contains("faelight-") {
        Some("Editing tool source — run deploy after building".to_string())
    } else if file_path.contains("mod.rs") && file_path.contains("domains/") {
        Some(
            "Editing domain — wire through CLI stack: commands → parser → mod → dispatcher"
                .to_string(),
        )
    } else {
        None
    };
    crate::protocol::Response::NeovimContext {
        file_path,
        active_intent,
        intent_title,
        suggestion,
    }
}

// INT-220 Friday Intelligence Functions
async fn friday_record_event(
    command: String,
    exit_code: i32,
    duration_ms: u64,
    intent: Option<String>,
    health: u32,
    timestamp: i64,
) -> crate::protocol::Response {
    use crate::protocol::Response;
    let home = std::env::var("HOME").unwrap_or_default();
    let db_path = format!("{}/0-core/runtime/state.db", home);
    let Ok(conn) = rusqlite::Connection::open(&db_path) else {
        return Response::FridaySpeak {
            message: None,
            priority: "silent".to_string(),
        };
    };
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS friday_observations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            source TEXT NOT NULL,
            kind TEXT NOT NULL,
            content TEXT NOT NULL
        );",
    );
    let obs_content = format!(
        "command: {} exit:{} {}ms intent:{} health:{}%",
        command,
        exit_code,
        duration_ms,
        intent.as_deref().unwrap_or("none"),
        health
    );
    let _ = conn.execute(
        "INSERT INTO friday_observations (timestamp, source, kind, content) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![timestamp, "fsh", "command", &obs_content],
    );
    // Gate 7 -- speak when command matches a known pattern trigger
    let speak_msg: Option<String> = {
        let cmd_base = command.split_whitespace().next().unwrap_or("").to_string();
        let cmd_base = cmd_base
            .split('/')
            .next_back()
            .unwrap_or(&cmd_base)
            .to_string();
        let mut msg: Option<String> = None;
        if let Ok(mut stmt) = conn.prepare(
            "SELECT trigger, action, confidence FROM friday_patterns WHERE confidence >= 0.75 ORDER BY confidence DESC LIMIT 10"
        ) {
            let rows = stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?)));
            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    let (trigger, action, conf) = row;
                    let trigger_base = trigger.split_whitespace().next().unwrap_or("").to_string();
                    if cmd_base == trigger_base || cmd_base == trigger {
                        // Check rate limit -- was this pattern spoken recently?
                        let last_spoken: i64 = conn.query_row(
                            "SELECT value FROM friday_context WHERE key = ?1",
                            rusqlite::params![format!("last_spoken_{}", trigger)],
                            |r| r.get(0)
                        ).unwrap_or(0);
                        let now_ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64).unwrap_or(0);
                        if now_ts - last_spoken < 300 {
                            break; // Skip -- too soon (5 min cooldown)
                        }
                        // Record this speak
                        let _ = conn.execute(
                            "INSERT OR REPLACE INTO friday_context (key, value, updated_at) VALUES (?1, ?2, ?2)",
                            rusqlite::params![format!("last_spoken_{}", trigger), now_ts],
                        );
                        msg = Some(format!("{} → {} ({:.0}%)", trigger, action, conf * 100.0));
                        break;
                    }
                }
            }
        }
        msg
    };
    // Log to friday.log for diagnostics
    if let Some(ref msg) = speak_msg {
        let home = std::env::var("HOME").unwrap_or_default();
        let log = format!("{}/.cache/faelight/friday.log", home);
        let entry = format!("[friday] speak: {}\n", msg);
        let _ = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&log)
            .map(|mut f| {
                use std::io::Write;
                let _ = f.write_all(entry.as_bytes());
            });
    }
    let priority = if speak_msg.is_some() { "low" } else { "silent" };
    Response::FridaySpeak {
        message: speak_msg,
        priority: priority.to_string(),
    }
}
/// Answer a direct question about the forest -- live data first, knowledge base fallback
async fn friday_answer_query(
    question: String,
    _context: Option<String>,
) -> crate::protocol::Response {
    use crate::protocol::Response;
    let home = std::env::var("HOME").unwrap_or_default();
    let db_path = format!("{}/0-core/runtime/state.db", home);
    let Ok(conn) = rusqlite::Connection::open(&db_path) else {
        return Response::FridayAnswer {
            answer: "Friday cannot access state.db right now.".to_string(),
            confidence: 0.0,
            sources: vec![],
        };
    };
    let q_lower = question.to_lowercase();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    // Live data queries first
    let live_answer: Option<String> = if q_lower.contains("intent")
        || q_lower.contains("complete")
        || q_lower.contains("done")
    {
        let intent_fact: Option<String> = conn.query_row(
            "SELECT fact FROM friday_knowledge WHERE domain='forest' ORDER BY updated_at DESC LIMIT 1",
            [], |r| r.get(0)
        ).ok();
        let obs: i64 = conn
            .query_row("SELECT COUNT(*) FROM friday_observations", [], |r| r.get(0))
            .unwrap_or(0);
        Some(intent_fact.unwrap_or_else(|| format!("Friday has observed {} command events.", obs)))
    } else if q_lower.contains("health") {
        let health: Option<u32> = conn
            .query_row(
                "SELECT health FROM doctor_history ORDER BY timestamp DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .ok();
        Some(format!(
            "Forest health: {}%. All systems nominal.",
            health.unwrap_or(100)
        ))
    } else if q_lower.contains("pattern") || q_lower.contains("learn") {
        let pats: i64 = conn
            .query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0))
            .unwrap_or(0);
        let top: Option<(String, String, f64)> = conn.query_row(
            "SELECT trigger, action, confidence FROM friday_patterns ORDER BY confidence DESC LIMIT 1",
            [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        ).ok();
        if let Some((trigger, action, conf)) = top {
            Some(format!(
                "Friday has {} patterns. Strongest: '{}' -> '{}' ({:.0}% confidence).",
                pats,
                trigger,
                action,
                conf * 100.0
            ))
        } else {
            Some(format!(
                "Friday has {} patterns learned from your workflow.",
                pats
            ))
        }
    } else if q_lower.contains("tool") {
        Some("The forest has 50 deployed tools, all written in Rust. Key tools: core, fsh, faelight-term, faelight-daemon, faelight-git, faelight-bar, faelight-fm, faelight-notify. Nothing runs without human authorization.".to_string())
    } else if q_lower.contains("commit") || q_lower.contains("today") {
        let commits: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM commit_patterns WHERE timestamp > ?1",
                rusqlite::params![now - 86400],
                |r| r.get(0),
            )
            .unwrap_or(0);
        Some(format!(
            "Friday has observed {} commits in the last 24 hours.",
            commits
        ))
    } else {
        None
    };
    // Knowledge base fallback
    let (answer, confidence) = if let Some(a) = live_answer {
        (a, 0.9)
    } else {
        let mut best_fact: Option<(String, f64, usize)> = None;
        if let Ok(mut stmt) = conn.prepare(
            "SELECT fact, confidence FROM friday_knowledge WHERE domain != 'forest' ORDER BY confidence DESC LIMIT 30"
        ) {
            let rows = stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,f64>(1)?)));
            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    let fact_lower = row.0.to_lowercase();
                    let words: Vec<&str> = q_lower.split_whitespace().filter(|w| w.len() > 3).collect();
                    let matches = words.iter().filter(|w| fact_lower.contains(*w)).count();
                    if matches > 0
                        && best_fact.as_ref().map(|b: &(_, _, usize)| matches > b.2).unwrap_or(true) {
                            best_fact = Some((row.0, row.1, matches));
                        }
                }
            }
        }
        if let Some((fact, conf, _)) = best_fact {
            (fact, conf)
        } else {
            (format!("Friday does not have specific knowledge about '{}' yet. Ask me about: intents, health, patterns, tools, commits.", question), 0.3)
        }
    };
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS friday_queries (
        id INTEGER PRIMARY KEY AUTOINCREMENT, timestamp INTEGER NOT NULL,
        question TEXT NOT NULL, answer TEXT NOT NULL, confidence REAL NOT NULL);",
    );
    let _ = conn.execute(
        "INSERT INTO friday_queries (timestamp, question, answer, confidence) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![now, &question, &answer, confidence],
    );
    Response::FridayAnswer {
        answer,
        confidence,
        sources: vec!["live_data".to_string()],
    }
}

// INT-220 Gate 6 -- Friday learning loop background task
async fn friday_learning_loop() {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1800)).await; // 30 minutes
        let home = std::env::var("HOME").unwrap_or_default();
        let core_path = format!("{}/0-core/scripts/core", home);
        let _ = tokio::process::Command::new(&core_path)
            .args(["friday", "learning-loop"])
            .output()
            .await;
    }
}

// INT-220 Gate 11 -- Negative learning: dismissal penalizes confidence by -0.3
async fn friday_dismiss(pattern_trigger: Option<String>) -> crate::protocol::Response {
    use crate::protocol::Response;
    let home = std::env::var("HOME").unwrap_or_default();
    let db_path = format!("{}/0-core/runtime/state.db", home);
    let Ok(conn) = rusqlite::Connection::open(&db_path) else {
        return Response::FridaySpeak {
            message: None,
            priority: "silent".to_string(),
        };
    };
    // Find pattern to penalize
    let target = if let Some(ref trigger) = pattern_trigger {
        conn.query_row(
            "SELECT id, trigger, confidence, dismissals FROM friday_patterns WHERE trigger = ?1",
            rusqlite::params![trigger],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, f64>(2)?,
                    r.get::<_, i64>(3).unwrap_or(0),
                ))
            },
        )
        .ok()
    } else {
        // Most recently spoken pattern
        conn.query_row(
            "SELECT id, trigger, confidence, COALESCE(dismissals, 0) FROM friday_patterns ORDER BY last_seen DESC LIMIT 1",
            [],
            |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?, r.get::<_,i64>(3)?))
        ).ok()
    };
    if let Some((id, trigger, confidence, dismissals)) = target {
        let new_conf = (confidence - 0.3).max(0.1);
        let new_dismissals = dismissals + 1;
        // Ensure dismissals column exists
        let _ = conn
            .execute_batch("ALTER TABLE friday_patterns ADD COLUMN dismissals INTEGER DEFAULT 0;");
        if new_dismissals >= 3 {
            // Archive the pattern
            let _ = conn.execute(
                "UPDATE friday_patterns SET confidence = ?1, dismissals = ?2, outcome = 'archived' WHERE id = ?3",
                rusqlite::params![new_conf, new_dismissals, id],
            );
            Response::FridaySpeak {
                message: Some(format!(
                    "Pattern '{}' archived after 3 dismissals.",
                    trigger
                )),
                priority: "low".to_string(),
            }
        } else {
            let _ = conn.execute(
                "UPDATE friday_patterns SET confidence = ?1, dismissals = ?2 WHERE id = ?3",
                rusqlite::params![new_conf, new_dismissals, id],
            );
            Response::FridaySpeak {
                message: Some(format!(
                    "Noted. '{}' confidence reduced to {:.0}%.",
                    trigger,
                    new_conf * 100.0
                )),
                priority: "low".to_string(),
            }
        }
    } else {
        Response::FridaySpeak {
            message: None,
            priority: "silent".to_string(),
        }
    }
}
