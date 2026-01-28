//! RPC Protocol for faelight-daemon
use serde::{Deserialize, Serialize};

/// Commands sent from client to daemon
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    /// List entries in a directory
    GetEntries { path: String },
    
    /// Search for files
    Search { query: String },
    
    /// Get file preview
    Preview { path: String },
    
    /// Get git status for directory
    GitStatus { path: String },
    
    /// Ping to check daemon is alive
    Ping,
    
    /// Shutdown daemon
    Shutdown,
}

/// Responses sent from daemon to client
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    /// Success with entries
    Entries { entries: Vec<Entry> },
    
    /// Success with preview
    Preview { content: String },
    
    /// Success with git status
    GitStatus { status: String },
    
    /// Pong response
    Pong,
    
    /// Generic success
    Ok,
    
    /// Error occurred
    Error { message: String },
}

/// File entry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

/// Message envelope for JSON-RPC
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
