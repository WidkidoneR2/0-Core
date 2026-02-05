use std::process::Command;

pub fn check_pip_updates() -> Vec<String> {
    let mut outdated = vec![];
    
    // Check pip
    if let Ok(output) = Command::new("pip")
        .args(&["list", "--outdated", "--format=freeze"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            outdated.push("pip packages".to_string());
        }
    }
    
    // Check pipx
    if let Ok(output) = Command::new("pipx").arg("list").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("can be upgraded") {
            outdated.push("pipx packages".to_string());
        }
    }
    
    outdated
}

pub fn update_pip() -> std::io::Result<()> {
    println!("   Running: pip install --upgrade pip");
    Command::new("pip")
        .args(&["install", "--upgrade", "pip"])
        .status()?;
    
    if Command::new("pipx").arg("--version").output().is_ok() {
        println!("   Running: pipx upgrade-all");
        Command::new("pipx")
            .arg("upgrade-all")
            .status()?;
    }
    
    Ok(())
}
