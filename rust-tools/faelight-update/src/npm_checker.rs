use std::process::Command;

pub fn check_npm_updates() -> Vec<String> {
    // Check if npm is installed
    if Command::new("npm").arg("--version").output().is_err() {
        return vec![];
    }
    
    // List globally installed packages
    match Command::new("npm")
        .args(["outdated", "-g", "--json"])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().is_empty() || stdout == "{}" {
                vec![]
            } else {
                vec!["Global NPM packages".to_string()]
            }
        }
        Err(_) => vec![],
    }
}

pub fn update_npm() -> std::io::Result<()> {
    // Check if npm is installed first
    if Command::new("npm").arg("--version").output().is_err() {
        println!("   ⚠️  npm not installed, skipping");
        return Ok(());
    }
    
    println!("   Running: npm update -g");
    let status = Command::new("npm")
        .args(["update", "-g"])
        .status();
    
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(_) => {
            println!("   ⚠️  npm update had warnings (non-critical)");
            Ok(())
        }
        Err(e) => {
            println!("   ⚠️  npm update failed: {}", e);
            Ok(())
        }
    }
}
