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
    /// INT-220 -- Friday: record a command event for learning
    FridayEvent {
        command: String,
        exit_code: i32,
        duration_ms: u64,
        intent: Option<String>,
        health: u32,
        timestamp: i64,
    },
    /// INT-220 -- Friday: dismiss last suggestion (negative learning)
    FridayDismiss {
        pattern_trigger: Option<String>,
    },
    /// INT-220 -- Friday: ask a question about the forest
    FridayQuery {
        question: String,
        context: Option<String>,
    },

    /// Shutdown daemon
    Shutdown,

    /// Subscribe to event stream for specific domains (empty = all)
    Subscribe { domains: Vec<String> },

    /// Stream all events live to terminal
    EventStream,
    /// Get full forest context (active intent, health, alignment, top prediction)
    GetForestContext,
    /// Get pre-computed next prediction for current context
    GetPrediction,
    /// Get health watchdog status
    WatchdogStatus,
    /// Get recent engine signals
    GetEngineSignals { limit: u32 },
    /// Get neovim context for a specific file path
    GetNeovimContext { file_path: String },
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
    /// INT-220 -- Friday response: optional inline message after command
    FridaySpeak {
        message: Option<String>,
        priority: String, // "silent" | "low" | "medium" | "high"
    },
    /// INT-220 -- Friday answer to a direct question
    FridayAnswer {
        answer: String,
        confidence: f64,
        sources: Vec<String>,
    },

    /// Generic success
    Ok,

    /// Error occurred
    Error { message: String },

    /// Live event pushed to subscriber
    Event {
        domain: String,
        action: String,
        payload: Option<String>,
        timestamp: i64,
    },

    /// Subscription confirmed
    Subscribed { domains: Vec<String> },
    /// Forest context snapshot
    ForestContext {
        health: u32,
        alignment: f64,
        active_intent: Option<String>,
        commits_today: i64,
        friday_status: String,
        top_prediction: Option<String>,
    },
    /// Pre-computed prediction
    Prediction {
        suggestion: Option<String>,
        confidence: f64,
        cached_at: i64,
    },
    /// Watchdog status
    Watchdog {
        last_check: i64,
        last_health: u32,
        alerts_today: i64,
    },
    /// Engine signals
    EngineSignals {
        signals: Vec<SignalEntry>,
    },
    /// Neovim context for a file
    NeovimContext {
        file_path: String,
        active_intent: Option<String>,
        intent_title: Option<String>,
        suggestion: Option<String>,
    },
}
/// Engine signal entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEntry {
    pub source: String,
    pub signal_type: String,
    pub payload: String,
    pub weight: f64,
    pub created_at: i64,
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
#[serde(untagged)]
pub enum MessagePayload {
    Command(Command),
    Response(Response),
}
