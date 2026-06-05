// faelight-fm v4.0 -- Intent plugin
// Handles: .md files in intents/, intent-linked files

use std::path::Path;
use super::{Plugin, PluginAction};

pub struct IntentPlugin;

impl Plugin for IntentPlugin {
    fn name(&self) -> &str { "intent" }

    fn handles(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        path_str.contains("/intents/") && 
            path.extension().map(|e| e == "md").unwrap_or(false)
    }

    fn preview(&self, path: &Path) -> String {
        let Ok(content) = std::fs::read_to_string(path) else {
            return "Could not read intent file".to_string();
        };
        let mut out = String::from("📋 Intent\n━━━━━━━━━\n");
        // Extract frontmatter fields
        let mut in_frontmatter = false;
        let mut past_frontmatter = false;
        let mut body_lines = 0;
        for line in content.lines() {
            if line == "---" {
                if !in_frontmatter && !past_frontmatter {
                    in_frontmatter = true;
                    continue;
                } else if in_frontmatter {
                    in_frontmatter = false;
                    past_frontmatter = true;
                    out.push('\n');
                    continue;
                }
            }
            if in_frontmatter {
                // Format key fields
                for key in &["title", "status", "tags", "priority", "date"] {
                    if line.starts_with(key) {
                        let val = line.splitn(2, ':').nth(1)
                            .unwrap_or("").trim().trim_matches('"');
                        let icon = match *key {
                            "title"    => "📌",
                            "status"   => "🔄",
                            "tags"     => "🏷",
                            "priority" => "⚡",
                            "date"     => "📅",
                            _          => "  ",
                        };
                        out.push_str(&format!("{} {}: {}\n", icon, key, val));
                    }
                }
            } else if past_frontmatter && body_lines < 20 {
                out.push_str(line);
                out.push('\n');
                body_lines += 1;
            }
        }
        out
    }

    fn actions(&self, _path: &Path) -> Vec<PluginAction> {
        vec![
            PluginAction {
                label: "open intent".to_string(),
                key: 'o',
                description: "Open in editor".to_string(),
            },
            PluginAction {
                label: "start intent".to_string(),
                key: 's',
                description: "cistart this intent".to_string(),
            },
        ]
    }

    fn execute(&self, path: &Path, action: char) -> String {
        match action {
            'o' => {
                let _ = crossterm::terminal::disable_raw_mode();
                let _ = crossterm::execute!(
                    std::io::stdout(),
                    crossterm::terminal::LeaveAlternateScreen,
                    crossterm::cursor::Show,
                );
                let _ = std::process::Command::new("hx").arg(path).status();
                let _ = crossterm::terminal::enable_raw_mode();
                let _ = crossterm::execute!(
                    std::io::stdout(),
                    crossterm::terminal::EnterAlternateScreen,
                    crossterm::cursor::Hide,
                );
                format!("Opening {}", path.display())
            },
            's' => {
                // Extract ID from filename
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let id = name.split('-').next().unwrap_or("").to_string();
                if id.is_empty() {
                    return "Could not extract intent ID".to_string();
                }
                let out = std::process::Command::new("core")
                    .args(["intent", "start", &id])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .unwrap_or_default();
                if out.is_empty() {
                    format!("Started intent {}", id)
                } else {
                    out
                }
            },
            _ => String::new(),
        }
    }
}
