//! Intent Ledger Parser
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Intent {
    pub id: String,
    pub title: String,
    pub status: String,
    pub category: String,
}

impl Intent {
    pub fn display_text(&self) -> String {
        let icon = match self.status.as_str() {
            "complete" => "✓",
            "in-progress" => "●",
            "planned" => "○",
            "cancelled" => "✗",
            "deferred" => "⊙",
            "resolved" => "✓",
            "decided" => "✓",
            _ => "·",
        };
        
        format!("{} {} - {}", icon, self.id, self.title)
    }
}

pub fn scan_intents() -> Vec<Intent> {
    let intent_dir = dirs::home_dir()
        .expect("No home dir")
        .join("0-core/INTENT");
    
    if !intent_dir.exists() {
        return Vec::new();
    }
    
    let mut intents = Vec::new();
    
    for entry in WalkDir::new(&intent_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
    {
        if let Some(intent) = parse_intent_file(entry.path()) {
            intents.push(intent);
        }
    }
    
    // Sort by ID
    intents.sort_by(|a, b| a.id.cmp(&b.id));
    
    intents
}

fn parse_intent_file(path: &std::path::Path) -> Option<Intent> {
    let content = fs::read_to_string(path).ok()?;
    
    // Extract YAML frontmatter
    if !content.starts_with("---") {
        return None;
    }
    
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return None;
    }
    
    let yaml = parts[1];
    
    let mut id = String::new();
    let mut title = String::new();
    let mut status = String::new();
    
    for line in yaml.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            
            match key {
                "id" => id = value.to_string(),
                "title" => title = value.to_string(),
                "status" => status = value.to_string(),
                _ => {}
            }
        }
    }
    
    // Get category from parent directory
    let category = path.parent()?
        .file_name()?
        .to_str()?
        .to_string();
    
    if id.is_empty() || title.is_empty() {
        return None;
    }
    
    Some(Intent {
        id,
        title,
        status: if status.is_empty() { "unknown".to_string() } else { status },
        category,
    })
}
