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
        tokio::spawn(async move {
            poll_events(poll_tx, db_path).await;
        });

        // Bind to Unix socket
        let listener = UnixListener::bind(&self.socket_path)?;
        let mut connection_count = 0;

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    connection_count += 1;
                    let conn_id = connection_count;
                    let sub_rx = tx.subscribe();
                    println!("{} Connection #{} established", "🔌".green(), conn_id);
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream, conn_id, sub_rx).await {
                            eprintln!("{} Connection #{} error: {}", "❌".red(), conn_id, e);
                        } else {
                            println!("{} Connection #{} closed", "✅".green(), conn_id);
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
    conn_id: u64,
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
            Err(e) => {
                eprintln!(
                    "{} [#{}] Failed to parse message: {}",
                    "⚠️".yellow(),
                    conn_id,
                    e
                );
                continue;
            }
        };

        if let MessagePayload::Command(ref cmd) = message.payload {
            println!("{} [#{}] Command: {:?}", "📨".cyan(), conn_id, cmd);
        }

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
