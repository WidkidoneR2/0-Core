//! INT-246 Pillar 1 -- Friday typed structs
//! Forward declarations -- will be used by simulation engine and event bus
#![allow(dead_code)]


use crate::domains::events::signal::SignalKind;

/// The confidence tier that determines Friday's voice level
#[derive(Debug, Clone, PartialEq)]
pub enum ConfidenceTier {
    /// 0.0-0.4: collect data, say nothing
    Observe,
    /// 0.4-0.7: surface insight, no interruption
    Suggest,
    /// 0.7-0.9: interrupt with specific suggestion
    Recommend,
    /// 0.9+: block and require explicit approval
    Challenge,
}

impl ConfidenceTier {
    pub fn from_confidence(c: f64) -> Self {
        if c >= 0.9 { Self::Challenge }
        else if c >= 0.7 { Self::Recommend }
        else if c >= 0.4 { Self::Suggest }
        else { Self::Observe }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Observe    => "OBSERVE",
            Self::Suggest    => "SUGGEST",
            Self::Recommend  => "RECOMMEND",
            Self::Challenge  => "CHALLENGE",
        }
    }

    pub fn should_speak(&self) -> bool {
        !matches!(self, Self::Observe)
    }

    pub fn should_block(&self) -> bool {
        matches!(self, Self::Challenge)
    }
}

/// What Friday receives -- a snapshot of forest state after each command
#[derive(Debug, Clone)]
pub struct FridayInput {
    /// The shell command that just ran (if any)
    pub command: Option<String>,
    /// Exit code of the command
    pub exit_code: Option<i32>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Currently active intent ID
    pub intent_id: Option<String>,
    /// Current forest health percentage
    pub health: Option<u32>,
    /// Unix timestamp
    pub timestamp: i64,
    /// Current working directory
    pub cwd: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
}

impl FridayInput {
    pub fn now(command: Option<String>, exit_code: Option<i32>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self {
            command,
            exit_code,
            duration_ms: None,
            intent_id: None,
            health: None,
            timestamp,
            cwd: std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
            session_id: std::env::var("FSH_SESSION_ID").ok(),
        }
    }

    pub fn signal_kind(&self) -> SignalKind {
        match self.exit_code {
            Some(0) | None => SignalKind::Observation,
            Some(_) => SignalKind::Judgment,
        }
    }
}

/// A silent observation -- stored in state.db, never shown
#[derive(Debug, Clone)]
pub struct Observation {
    pub key: String,
    pub value: String,
    pub confidence: f64,
    pub source: String,
}

/// A suggestion shown inline -- low friction, no interruption
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub confidence: f64,
    pub tier: ConfidenceTier,
    /// Which pattern triggered this suggestion
    pub source_pattern: Option<String>,
    /// Unique key for deduplication within a session
    pub dedup_key: String,
}

impl Suggestion {
    pub fn new(message: impl Into<String>, confidence: f64) -> Self {
        let message = message.into();
        let dedup_key = message.to_lowercase().replace(' ', "_").chars().take(40).collect();
        Self {
            message,
            confidence,
            tier: ConfidenceTier::from_confidence(confidence),
            source_pattern: None,
            dedup_key,
        }
    }

    pub fn format_for_display(&self) -> String {
        format!(
            "{} ({:.0}% · {})",
            self.message,
            self.confidence * 100.0,
            self.tier.as_str()
        )
    }
}

/// A multi-step plan requiring human approval
#[derive(Debug, Clone)]
pub struct Plan {
    pub title: String,
    pub steps: Vec<String>,
    pub confidence: f64,
    pub rationale: String,
    /// ID in friday_proposals table once created
    pub proposal_id: Option<i64>,
}

/// A warning -- high confidence signal about a potentially harmful action
#[derive(Debug, Clone)]
pub struct Warning {
    pub message: String,
    pub affected_systems: Vec<String>,
    pub confidence: f64,
    /// Optional simulation output
    pub simulation_preview: Option<String>,
}

impl Warning {
    pub fn should_block(&self) -> bool {
        ConfidenceTier::from_confidence(self.confidence).should_block()
    }
}

/// What Friday produces -- one of four output types
#[derive(Debug, Clone)]
pub enum FridayOutput {
    /// Silent observation, stored only
    Observe(Observation),
    /// Inline suggestion, shown to user
    Suggest(Suggestion),
    /// Multi-step plan requiring approval
    Plan(Plan),
    /// Warning, may block execution
    Warn(Warning),
}

impl FridayOutput {
    pub fn confidence(&self) -> f64 {
        match self {
            Self::Observe(o) => o.confidence,
            Self::Suggest(s) => s.confidence,
            Self::Plan(p) => p.confidence,
            Self::Warn(w) => w.confidence,
        }
    }

    pub fn tier(&self) -> ConfidenceTier {
        ConfidenceTier::from_confidence(self.confidence())
    }

    pub fn should_speak(&self) -> bool {
        !matches!(self, Self::Observe(_))
    }
}
