//! Menu item sources (applications, commands, etc.)
use std::process::Command;
use std::fs;

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub display: String,
    pub exec: String,
}

impl MenuItem {
    pub fn new(display: String, exec: String) -> Self {
        Self { display, exec }
    }
}

/// Get shell commands from PATH
pub fn get_shell_commands() -> Result<Vec<MenuItem>, Box<dyn std::error::Error>> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("compgen -c | sort -u | head -n 500")
        .output()?;
    
    let commands = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(|s| MenuItem::new(s.to_string(), s.to_string()))
        .collect();
    
    Ok(commands)
}

/// Get desktop applications
pub fn get_desktop_apps() -> Result<Vec<MenuItem>, Box<dyn std::error::Error>> {
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
                        if let Some(app) = parse_desktop_file(&content) {
                            apps.push(app);
                        }
                    }
                }
            }
        }
    }
    
    apps.sort_by(|a, b| a.display.cmp(&b.display));
    apps.dedup_by(|a, b| a.display == b.display);
    Ok(apps)
}

fn parse_desktop_file(content: &str) -> Option<MenuItem> {
    let mut name = None;
    let mut exec = None;
    
    for line in content.lines() {
        if line.starts_with("Name=") && name.is_none() {
            name = line.strip_prefix("Name=").map(|s| s.to_string());
        }
        if line.starts_with("Exec=") {
            exec = line.strip_prefix("Exec=").map(|s| clean_exec(s));
        }
    }
    
    match (name, exec) {
        (Some(n), Some(e)) => Some(MenuItem::new(n, e)),
        _ => None,
    }
}

fn clean_exec(exec: &str) -> String {
    // Remove desktop entry field codes: %f, %F, %u, %U, %d, %D, %n, %N, %i, %c, %k, %v, %m
    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Get all available items (apps + commands)
pub fn get_all_items() -> Vec<MenuItem> {
    let mut items = Vec::new();
    
    // Desktop apps first (higher quality)
    if let Ok(apps) = get_desktop_apps() {
        items.extend(apps);
    }
    
    // Shell commands as fallback
    if let Ok(cmds) = get_shell_commands() {
        items.extend(cmds);
    }
    
    items.sort_by(|a, b| a.display.cmp(&b.display));
    items.dedup_by(|a, b| a.display == b.display);
    items
}
