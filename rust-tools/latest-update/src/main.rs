use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let home = env::var("HOME").expect("HOME not set");
    let core_dir = PathBuf::from(&home).join("0-core");
    
    let mut latest_pkg = String::new();
    let mut latest_date = String::new();
    let mut latest_version = String::new();
    
    // Read all entries in 0-core directory
    let entries = match fs::read_dir(&core_dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        
        let dotmeta_path = path.join(".dotmeta");
        if !dotmeta_path.exists() {
            continue;
        }
        
        let content = match fs::read_to_string(&dotmeta_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        let pkg_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let mut updated = String::new();
        let mut version = String::new();
        
        for line in content.lines() {
            if line.starts_with("last_updated = ") {
                updated = extract_quoted_value(line);
            } else if line.starts_with("version = ") && version.is_empty() {
                version = extract_quoted_value(line);
            }
        }
        
        if !updated.is_empty() && (latest_date.is_empty() || updated > latest_date) {
            latest_date = updated;
            latest_pkg = pkg_name;
            latest_version = version;
        }
    }
    
    if !latest_pkg.is_empty() {
        println!("{} v{} (updated {})", latest_pkg, latest_version, latest_date);
    }
}

fn extract_quoted_value(line: &str) -> String {
    if let Some(start) = line.find('"') {
        if let Some(end) = line.rfind('"') {
            if start < end {
                return line[start + 1..end].to_string();
            }
        }
    }
    String::new()
}
