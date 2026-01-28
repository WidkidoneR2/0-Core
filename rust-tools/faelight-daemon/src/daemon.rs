//! The faelight daemon server
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::path::Path;
use crate::protocol::{Message, MessagePayload, Command, Response, Entry};

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
        
        // Bind to Unix socket
        let listener = UnixListener::bind(&self.socket_path)?;
        println!("🌲 faelight-daemon listening on {}", self.socket_path);
        
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream).await {
                            eprintln!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                }
            }
        }
    }
}

async fn handle_client(stream: UnixStream) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    
    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        
        if n == 0 {
            // Connection closed
            break;
        }
        
        // Parse message
        let message: Message = match serde_json::from_str(&line) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Failed to parse message: {}", e);
                continue;
            }
        };
        
        // Process command
        let response = match message.payload {
            MessagePayload::Command(cmd) => process_command(cmd).await,
            _ => Response::Error { 
                message: "Expected command".to_string() 
            },
        };
        
        // Send response
        let response_msg = Message {
            id: message.id,
            payload: MessagePayload::Response(response),
        };
        
        let json = serde_json::to_string(&response_msg)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }
    
    Ok(())
}

async fn process_command(cmd: Command) -> Response {
    match cmd {
        Command::Ping => Response::Pong,
        
        Command::GetEntries { path } => {
            match read_directory(&path).await {
                Ok(entries) => Response::Entries { entries },
                Err(e) => Response::Error { 
                    message: format!("Failed to read directory: {}", e) 
                },
            }
        }
        
        Command::Preview { path } => {
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => Response::Preview { content },
                Err(e) => Response::Error {
                    message: format!("Failed to read file: {}", e)
                },
            }
        }
        
        Command::Search { query } => {
            // TODO: Implement search
            Response::Error { 
                message: "Search not implemented yet".to_string() 
            }
        }
        
        Command::GitStatus { path } => {
            // TODO: Implement git status
            Response::GitStatus { 
                status: "Git status not implemented yet".to_string() 
            }
        }
        
        Command::Shutdown => {
            println!("Shutdown requested");
            std::process::exit(0);
        }
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
