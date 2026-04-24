use crate::git::GitStatus;
use crate::intent::IntentStatus;
use faelight_zone::Zone;
use std::path::PathBuf;
use std::time::SystemTime;

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
    pub is_symlink: bool,
    pub zone: Zone,
    pub health: HealthStatus,
    pub intent_info: Option<IntentInfo>,
    pub git_status: GitStatus,
    // v3 additions
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub permissions: u32, // Unix mode bits
    pub is_executable: bool,
    pub is_hidden: bool,
}
impl FaelightEntry {
    pub fn permissions_str(&self) -> String {
        let m = self.permissions;
        let file_type = if self.is_dir {
            'd'
        } else if self.is_symlink {
            'l'
        } else {
            '-'
        };
        format!(
            "{}{}{}{}{}{}{}{}{}{}",
            file_type,
            if m & 0o400 != 0 { 'r' } else { '-' },
            if m & 0o200 != 0 { 'w' } else { '-' },
            if m & 0o100 != 0 { 'x' } else { '-' },
            if m & 0o040 != 0 { 'r' } else { '-' },
            if m & 0o020 != 0 { 'w' } else { '-' },
            if m & 0o010 != 0 { 'x' } else { '-' },
            if m & 0o004 != 0 { 'r' } else { '-' },
            if m & 0o002 != 0 { 'w' } else { '-' },
            if m & 0o001 != 0 { 'x' } else { '-' },
        )
    }
    pub fn octal_str(&self) -> String {
        format!("{:04o}", self.permissions & 0o7777)
    }
    pub fn size_str(&self) -> String {
        if self.is_dir {
            return "—".to_string();
        }
        match self.size {
            0 => "0 B".to_string(),
            1..=1023 => format!("{} B", self.size),
            1024..=1048575 => format!("{:.1} K", self.size as f64 / 1024.0),
            1048576..=1073741823 => format!("{:.1} M", self.size as f64 / 1048576.0),
            _ => format!("{:.1} G", self.size as f64 / 1073741824.0),
        }
    }
    pub fn modified_str(&self) -> String {
        use std::time::UNIX_EPOCH;
        if let Some(mt) = self.modified {
            if let Ok(secs) = mt.duration_since(UNIX_EPOCH) {
                let s = secs.as_secs();
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let diff = now.saturating_sub(s);
                return match diff {
                    0..=59 => "just now".to_string(),
                    60..=3599 => format!("{}m ago", diff / 60),
                    3600..=86399 => format!("{}h ago", diff / 3600),
                    86400..=604799 => format!("{}d ago", diff / 86400),
                    _ => format!("{}w ago", diff / 604800),
                };
            }
        }
        "—".to_string()
    }
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
        if self.is_symlink {
            "🔗" // Symlink icon!
        } else if self.is_dir {
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
