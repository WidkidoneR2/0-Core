//! Client for connecting to faelight-daemon
use tokio::net::UnixStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde::{Deserialize, Serialize};
use std::path::Path;

// Re-define protocol types (should share with daemon eventually)
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    GetEntries { path: String },
    Preview { path: String },
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Entries { entries: Vec<Entry> },
    Preview { content: String },
    Pong,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub payload: MessagePayload,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessagePayload {
    Command(Command),
    Response(Response),
}

pub struct DaemonClient {
    socket_path: String,
    next_id: u64,
}

impl DaemonClient {
    pub fn new() -> Self {
        Self {
            socket_path: "/tmp/faelight-daemon.sock".to_string(),
            next_id: 1,
        }
    }
    
    /// Check if daemon is available
    pub fn is_available(&self) -> bool {
        Path::new(&self.socket_path).exists()
    }
    
    /// Send a command and get response
    pub async fn send_command(&mut self, cmd: Command) -> Result<Response, Box<dyn std::error::Error>> {
        let stream = UnixStream::connect(&self.socket_path).await?;
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        
        // Send command
        let msg = Message {
            id: self.next_id,
            payload: MessagePayload::Command(cmd),
        };
        self.next_id += 1;
        
        let json = serde_json::to_string(&msg)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        
        // Read response
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        let msg: Message = serde_json::from_str(&line)?;
        
        match msg.payload {
            MessagePayload::Response(resp) => Ok(resp),
            _ => Err("Expected response".into()),
        }
    }
}
