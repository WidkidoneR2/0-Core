//! XDG Desktop Entry Parser
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub name: String,
    pub exec: String,
    pub icon: String,
    pub categories: Vec<String>,
    pub terminal: bool,
    pub no_display: bool,
}

impl DesktopEntry {
    pub fn parse(path: &PathBuf) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        let mut entry = DesktopEntry {
            name: String::new(),
            exec: String::new(),
            icon: String::new(),
            categories: Vec::new(),
            terminal: false,
            no_display: false,
        };

        let mut in_desktop_entry = false;

        for line in content.lines() {
            let line = line.trim();

            if line == "[Desktop Entry]" {
                in_desktop_entry = true;
                continue;
            }

            if line.starts_with('[') && line != "[Desktop Entry]" {
                in_desktop_entry = false;
                continue;
            }

            if !in_desktop_entry {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "Name" => entry.name = value.trim().to_string(),
                    "Exec" => entry.exec = value.trim().to_string(),
                    "Icon" => entry.icon = value.trim().to_string(),
                    "Categories" => {
                        entry.categories = value
                            .split(';')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    "Terminal" => entry.terminal = value.trim() == "true",
                    "NoDisplay" => entry.no_display = value.trim() == "true",
                    _ => {}
                }
            }
        }

        if entry.name.is_empty() || entry.exec.is_empty() {
            return None;
        }

        Some(entry)
    }

    pub fn clean_exec(&self) -> String {
        // Remove field codes like %U, %F, etc.
        let mut exec = self.exec.clone();
        for code in &["%u", "%U", "%f", "%F", "%i", "%c", "%k"] {
            exec = exec.replace(code, "");
        }
        exec.trim().to_string()
    }
}

#[allow(dead_code)]
pub fn scan_desktop_entries() -> Vec<DesktopEntry> {
    let mut entries = Vec::new();
    let paths = vec![
        "/usr/share/applications",
        "/usr/local/share/applications",
    ];

    if let Ok(home) = std::env::var("HOME") {
        let _ = paths.iter().chain(
            std::iter::once(&format!("{}/.local/share/applications", home).as_str())
        );
    }

    for search_path in paths {
        if let Ok(walker) = WalkDir::new(search_path).into_iter().collect::<Result<Vec<_>, _>>() {
            for entry in walker {
                if let Some(ext) = entry.path().extension() {
                    if ext == "desktop" {
                        if let Some(desktop_entry) = DesktopEntry::parse(&entry.path().to_path_buf()) {
                            if !desktop_entry.no_display {
                                entries.push(desktop_entry);
                            }
                        }
                    }
                }
            }
        }
    }

    entries
}



/// Scan all XDG application directories for .desktop files
pub fn scan_applications() -> Vec<DesktopEntry> {
    let mut entries = Vec::new();
    
    let home_dir = format!("{}/.local/share/applications", 
        std::env::var("HOME").unwrap_or_default());
    
    let search_dirs = vec![
        "/usr/share/applications",
        "/usr/local/share/applications",
        home_dir.as_str(),
    ];
    
    for dir in search_dirs {
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if let Some("desktop") = entry.path().extension().and_then(|s| s.to_str()) {
                if let Some(parsed) = DesktopEntry::parse(&entry.path().to_path_buf()) {
                    if !parsed.no_display && parsed.is_user_app() {
                        entries.push(parsed);
                    }
                }
            }
        }
    }
    
    // Deduplicate by name (keep first occurrence)
    let mut seen = std::collections::HashSet::new();
    entries.retain(|e| seen.insert(e.name.clone()));
    
    // Sort alphabetically
    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    
    entries
}

impl DesktopEntry {
    /// Check if this is a user-facing application
    pub fn is_user_app(&self) -> bool {
        // Filter out system utilities and daemons
        let blacklist = [
            "avahi", "cmake", "assistant", "designer", "linguist",
            "qdbusviewer", "bssh", "bvnc", "lstopo", "qv4l2", "qvidcap",
            "cmake-gui", "htop", "btop", "nvtop",
        ];
        
        let name_lower = self.name.to_lowercase();
        let exec_lower = self.exec.to_lowercase();
        
        // Skip if name/exec contains blacklisted terms
        for term in &blacklist {
            if name_lower.contains(term) || exec_lower.contains(term) {
                return false;
            }
        }
        
        // Skip if it's marked as OnlyShowIn certain DEs we don't use
        // Keep all that don't have OnlyShowIn, or that include common values
        true
    }
}

impl DesktopEntry {
    /// Execute the application
    pub fn exec(&self) {
        use std::process::Command;
        
        let exec_clean = self.clean_exec();
        let parts: Vec<&str> = exec_clean.split_whitespace().collect();
        
        if parts.is_empty() {
            eprintln!("❌ Empty exec command");
            return;
        }
        
        let program = parts[0];
        let args = &parts[1..];
        
        if self.terminal {
            // Launch in terminal
            let _ = Command::new("kitty")
                .arg("-e")
                .arg(program)
                .args(args)
                .spawn();
        } else {
            // Launch directly
            let _ = Command::new(program)
                .args(args)
                .spawn();
        }
    }
}
