use regex::Regex;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: bump-version <package> <new-version> [change-note]");
        eprintln!("");
        eprintln!("Example: bump-version shell-zsh 3.2.2 'Added new aliases'");
        process::exit(1);
    }
    
    let package = &args[1];
    let new_version = &args[2];
    let change_note = args.get(3).map(|s| s.as_str());
    
    let home = env::var("HOME").expect("HOME not set");
    let dotmeta_path = PathBuf::from(&home)
        .join("0-core")
        .join(package)
        .join(".dotmeta");
    
    if !dotmeta_path.exists() {
        eprintln!("❌ No .dotmeta found for package: {}", package);
        process::exit(1);
    }
    
    let content = fs::read_to_string(&dotmeta_path)
        .expect("Failed to read .dotmeta");
    
    // Get old version
    let old_version = extract_version(&content).unwrap_or("unknown".to_string());
    
    // Update version (multiline mode)
    let version_re = Regex::new(r#"(?m)^version = ".*""#).unwrap();
    let mut new_content = version_re.replace(&content, format!(r#"version = "{}""#, new_version)).to_string();
    
    // Update last_updated
    let today = chrono_lite_today();
    let date_re = Regex::new(r#"(?m)^last_updated = .*"#).unwrap();
    new_content = date_re.replace(&new_content, format!(r#"last_updated = "{}""#, today)).to_string();
    
    // Add changelog entry if provided
    if let Some(note) = change_note {
        let changelog_re = Regex::new(r"(?m)^\[changelog\]").unwrap();
        if changelog_re.is_match(&new_content) {
            new_content = changelog_re.replace(
                &new_content, 
                format!("[changelog]\n\"{}\" = \"{}\"", new_version, note)
            ).to_string();
        }
    }
    
    fs::write(&dotmeta_path, new_content).expect("Failed to write .dotmeta");
    
    println!("✅ Bumped {}: v{} → v{}", package, old_version, new_version);
    if let Some(note) = change_note {
        println!("   Note: {}", note);
    }
    println!("   Updated: {}", today);
}

fn extract_version(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("version = ") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if start < end {
                        return Some(line[start + 1..end].to_string());
                    }
                }
            }
        }
    }
    None
}

fn chrono_lite_today() -> String {
    // Get current date without chrono dependency
    use std::process::Command;
    let output = Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .expect("Failed to get date");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
