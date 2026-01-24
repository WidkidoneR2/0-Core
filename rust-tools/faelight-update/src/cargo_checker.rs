use std::process::Command;

pub fn check_cargo_updates() -> Vec<crate::UpdateItem> {
    println!("   Checking cargo-installed tools...");
    
    let check = Command::new("cargo")
        .args(["install-update", "--list"])
        .output();
    
    match check {
        Ok(output) => {
            if !output.status.success() {
                println!("      ⚠️  cargo-update not installed");
                println!("      Run: cargo install cargo-update");
                return Vec::new();
            }
            parse_cargo_update_list(&output.stdout)
        }
        Err(_) => Vec::new(),
    }
}

fn parse_cargo_update_list(output: &[u8]) -> Vec<crate::UpdateItem> {
    let text = String::from_utf8_lossy(output);
    let mut items = Vec::new();
    
    for line in text.lines() {
        if line.contains(" -> ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                items.push(crate::UpdateItem {
                    name: parts[0].to_string(),
                    current: parts[1].trim_start_matches('v').to_string(),
                    new: parts[3].trim_start_matches('v').to_string(),
                });
            }
        }
    }
    items
}

pub fn check_workspace_updates() -> Vec<crate::UpdateItem> {
    println!("   Checking 0-Core workspace...");
    
    let output = Command::new("git")
        .args(["-C", "/home/christian/0-core", "status", "--porcelain"])
        .output();
    
    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            let has_changes = text.lines().any(|line| line.contains("rust-tools/"));
            
            if has_changes {
                vec![crate::UpdateItem {
                    name: "0-core-workspace".to_string(),
                    current: "modified".to_string(),
                    new: "rebuild needed".to_string(),
                }]
            } else {
                Vec::new()
            }
        }
        Err(_) => Vec::new(),
    }
}
