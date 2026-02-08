use std::process::Command;
use std::path::Path;

pub fn check_pip_updates() -> Vec<String> {
    // Skip on Arch (PEP 668 - externally managed)
    if Path::new("/etc/arch-release").exists() {
        return vec![];
    }
    
    let mut outdated = vec![];
    
    // Check pip
    if let Ok(output) = Command::new("pip")
        .args(["list", "--outdated", "--format=freeze"])
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
    // Skip on Arch (PEP 668)
    if Path::new("/etc/arch-release").exists() {
        println!("   ⏭️  Skipping pip on Arch (use pacman for Python packages)");
        return Ok(());
    }
    
    // Check if pip is installed first
    if Command::new("pip").arg("--version").output().is_err() {
        println!("   ⚠️  pip not installed, skipping");
        return Ok(());
    }
    
    println!("   Running: pip install --upgrade pip");
    match Command::new("pip")
        .args(["install", "--upgrade", "pip"])
        .status()
    {
        Ok(s) if s.success() => {
            // Success - continue to pipx
        }
        Ok(_) => {
            println!("   ⚠️  pip update had warnings (non-critical)");
        }
        Err(e) => {
            println!("   ⚠️  pip update failed: {}", e);
            println!("   💡 On Arch, use pacman for Python packages");
            return Ok(());
        }
    }
    
    // Check pipx
    if Command::new("pipx").arg("--version").output().is_ok() {
        println!("   Running: pipx upgrade-all");
        let _ = Command::new("pipx")
            .arg("upgrade-all")
            .status();
    }
    
    Ok(())
}
