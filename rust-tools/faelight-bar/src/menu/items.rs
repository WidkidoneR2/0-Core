//! Menu item sources (applications, commands, etc.)

use std::process::Command;
use std::fs;

/// Get shell commands from PATH
pub fn get_shell_commands() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("compgen -c | sort -u | head -n 500")
        .output()?;
    
    let commands = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(|s| s.to_string())
        .collect();
    
    Ok(commands)
}

/// Get desktop applications
pub fn get_desktop_apps() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut apps = Vec::new();
    let app_dirs = [
        "/usr/share/applications",
        "/usr/local/share/applications",
    ];
    
    for dir in &app_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("desktop") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Some(name) = parse_desktop_name(&content) {
                            apps.push(name);
                        }
                    }
                }
            }
        }
    }
    
    apps.sort();
    apps.dedup();
    Ok(apps)
}

fn parse_desktop_name(content: &str) -> Option<String> {
    content
        .lines()
        .find(|line| line.starts_with("Name="))
        .and_then(|line| line.strip_prefix("Name="))
        .map(|s| s.to_string())
}

/// Get all available items (apps + commands)
pub fn get_all_items() -> Vec<String> {
    let mut items = Vec::new();
    
    // Desktop apps first (higher quality)
    if let Ok(apps) = get_desktop_apps() {
        items.extend(apps);
    }
    
    // Shell commands as fallback
    if let Ok(cmds) = get_shell_commands() {
        items.extend(cmds);
    }
    
    items.sort();
    items.dedup();
    items
}
