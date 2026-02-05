use std::process::Command;

pub fn check_npm_updates() -> Vec<String> {
    // Check if npm is installed
    if Command::new("npm").arg("--version").output().is_err() {
        return vec![];
    }
    
    // List globally installed packages
    match Command::new("npm")
        .args(&["outdated", "-g", "--json"])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().is_empty() || stdout == "{}" {
                vec![]
            } else {
                // Parse JSON and extract package names
                // For now, simple check if there's content
                vec!["Global NPM packages".to_string()]
            }
        }
        Err(_) => vec![],
    }
}

pub fn update_npm() -> std::io::Result<()> {
    println!("   Running: npm update -g");
    Command::new("npm")
        .args(&["update", "-g"])
        .status()?;
    Ok(())
}
