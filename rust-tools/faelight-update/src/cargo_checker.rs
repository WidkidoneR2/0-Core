use std::process::Command;

/// Check for cargo-installed tool updates
pub fn check_cargo_updates() -> Vec<crate::UpdateItem> {
    println!("   Checking cargo-installed tools...");
    
    let check = Command::new("cargo")
        .args(["install-update", "--list"])
        .output();
    
    match check {
        Ok(output) => {
            if !output.status.success() {
                // Check if cargo-update is not installed
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("no such subcommand") || stderr.contains("not found") {
                    println!("      ⚠️  cargo-update not installed");
                    println!("      💡 Run: cargo install cargo-update");
                    return Vec::new();
                }
                
                // Other error
                eprintln!("      ⚠️  Failed to check cargo updates: {}", stderr);
                return Vec::new();
            }
            
            parse_cargo_update_list(&output.stdout)
        }
        Err(e) => {
            eprintln!("      ⚠️  Failed to run cargo: {}", e);
            Vec::new()
        }
    }
}

/// Parse cargo-update output
/// 
/// Expected format:
/// ```
/// package v1.0.0 -> v1.1.0
/// another-tool v2.3.4 -> v2.4.0
/// ```
fn parse_cargo_update_list(output: &[u8]) -> Vec<crate::UpdateItem> {
    use once_cell::sync::Lazy;
    use regex::Regex;
    
    // Regex to match: "package-name v1.2.3 -> v1.2.4"
    static CARGO_UPDATE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(\S+)\s+v?(\S+)\s+->\s+v?(\S+)").unwrap()
    });
    
    let text = String::from_utf8_lossy(output);
    let mut items = Vec::new();
    
    for line in text.lines() {
        if let Some(caps) = CARGO_UPDATE_REGEX.captures(line) {
            items.push(crate::UpdateItem {
                name: caps.get(1).unwrap().as_str().to_string(),
                current: caps.get(2).unwrap().as_str().trim_start_matches('v').to_string(),
                new: caps.get(3).unwrap().as_str().trim_start_matches('v').to_string(),
                repository: None,
            });
        }
    }
    
    items
}

/// Check if 0-Core workspace needs rebuilding
pub fn check_workspace_updates() -> Vec<crate::UpdateItem> {
    println!("   Checking 0-Core workspace...");
    
    let output = Command::new("git")
        .args(["-C", "/home/christian/0-core", "status", "--porcelain"])
        .output();
    
    match output {
        Ok(out) => {
            if !out.status.success() {
                eprintln!("      ⚠️  Failed to check git status");
                return Vec::new();
            }
            
            let text = String::from_utf8_lossy(&out.stdout);
            
            // Check if any Rust source files were modified
            let has_rust_changes = text.lines().any(|line| {
                line.contains("rust-tools/") && 
                (line.contains(".rs") || line.contains("Cargo.toml"))
            });
            
            if has_rust_changes {
                vec![crate::UpdateItem {
                    name: "0-core-workspace".to_string(),
                    current: "modified".to_string(),
                    new: "rebuild needed".to_string(),
                    repository: None,
                }]
            } else {
                Vec::new()
            }
        }
        Err(e) => {
            eprintln!("      ⚠️  Failed to run git: {}", e);
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_cargo_update_list() {
        let input = b"cargo-update v10.0.0 -> v10.1.0\nbat v0.24.0 -> v0.25.0\n";
        let items = parse_cargo_update_list(input);
        
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "cargo-update");
        assert_eq!(items[0].current, "10.0.0");
        assert_eq!(items[0].new, "10.1.0");
    }
}
