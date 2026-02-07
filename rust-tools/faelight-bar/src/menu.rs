//! Application menu data loading

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub display: String,
    pub exec: String,
}

use std::fs;
use std::path::PathBuf;

pub fn get_all_items() -> Vec<MenuItem> {
    let mut items = Vec::new();
    
    let app_dirs = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ];
    
    for dir in app_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".desktop") {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if let Some(item) = parse_desktop_file(&content) {
                                items.push(item);
                            }
                        }
                    }
                }
            }
        }
    }
    
    items.sort_by(|a, b| a.display.cmp(&b.display));
    items
}

fn parse_desktop_file(content: &str) -> Option<MenuItem> {
    let mut name = None;
    let mut exec = None;
    
    for line in content.lines() {
        if line.starts_with("Name=") {
            name = Some(line[5..].to_string());
        } else if line.starts_with("Exec=") {
            exec = Some(line[5..].split_whitespace().next()?.to_string());
        }
    }
    
    Some(MenuItem {
        display: name?,
        exec: exec?,
    })
}
