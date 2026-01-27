use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentStatus {
    Complete,
    Future,
    Cancelled,
    Deferred,
}

#[derive(Debug, Clone)]
pub struct Intent {
    pub id: String,
    pub title: String,
    pub status: IntentStatus,
    #[allow(dead_code)]
    pub path: PathBuf,
}

impl Intent {
    pub fn from_file(path: &Path) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        let parts: Vec<&str> = content.split("---").collect();
        if parts.len() < 3 {
            return None;
        }
        
        let frontmatter = parts[1];
        
        let id = frontmatter
            .lines()
            .find(|line| line.starts_with("id:"))?
            .split(':')
            .nth(1)?
            .trim()
            .to_string();
        
        let title = frontmatter
            .lines()
            .find(|line| line.starts_with("title:"))?
            .split(':')
            .nth(1)?
            .trim()
            .trim_matches('"')
            .to_string();
        
        let status_str = frontmatter
            .lines()
            .find(|line| line.starts_with("status:"))?
            .split(':')
            .nth(1)?
            .trim();
        
        let status = match status_str {
            "complete" => IntentStatus::Complete,
            "future" => IntentStatus::Future,
            "cancelled" => IntentStatus::Cancelled,
            "deferred" => IntentStatus::Deferred,
            _ => return None,
        };
        
        Some(Intent {
            id,
            title,
            status,
            path: path.to_path_buf(),
        })
    }
    
    pub fn matches_path(&self, target: &Path) -> bool {
        let target_name = target
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        if !target_name.is_empty() && self.title.to_lowercase().contains(&target_name.to_lowercase()) {
            return true;
        }
        
        false
    }
}

pub fn find_intents_for_path(intent_dir: &Path, target: &Path) -> Vec<Intent> {
    let mut intents = Vec::new();
    
    for status_dir in &["complete", "future", "deferred", "cancelled"] {
        let dir = intent_dir.join(status_dir);
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map_or(false, |e| e == "md") {
                    if let Some(intent) = Intent::from_file(&entry.path()) {
                        if intent.matches_path(target) {
                            intents.push(intent);
                        }
                    }
                }
            }
        }
    }
    
    intents
}
