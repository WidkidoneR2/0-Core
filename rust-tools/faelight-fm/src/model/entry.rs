use std::path::PathBuf;
use faelight_zone::Zone;
use crate::intent::IntentStatus;

#[derive(Debug, Clone)]
pub struct IntentInfo {
    pub id: String,
    pub title: String,
    pub status: IntentStatus,
}

#[derive(Debug, Clone)]
pub struct FaelightEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub zone: Zone,
    pub health: HealthStatus,
    pub intent_info: Option<IntentInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Ok,
    #[allow(dead_code)]
    Warning,
    #[allow(dead_code)]
    Error,
}

impl HealthStatus {
    pub fn badge(&self) -> &'static str {
        match self {
            HealthStatus::Ok => "✔",
            HealthStatus::Warning => "⚠",
            HealthStatus::Error => "✘",
        }
    }
}

impl FaelightEntry {
    pub fn icon(&self) -> &'static str {
        if self.is_dir {
            "📁"
        } else {
            match self.path.extension().and_then(|e| e.to_str()) {
                Some("rs") => "🦀",
                Some("toml") => "🔧",
                Some("md") => "📝",
                Some("sh") => "🐚",
                _ => "📄",
            }
        }
    }
}
