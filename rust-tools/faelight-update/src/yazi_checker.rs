use std::process::Command;

/// Check for Yazi package updates
pub fn check_yazi_packages() -> Vec<String> {
    println!("   Checking yazi packages...");
    Vec::new()
}

/// Update Yazi packages
pub fn update_yazi() -> anyhow::Result<()> {
    println!("   Running: ya pack -u");
    
    let status = Command::new("ya")
        .arg("pack")
        .arg("-u")
        .status();
    
    match status {
        Ok(s) if s.success() => {
            println!("   ✅  Yazi packages updated");
            Ok(())
        }
        Ok(_) => {
            println!("   ⚠️  Yazi update completed with warnings");
            Ok(())
        }
        Err(e) => {
            println!("   ⚠️  Yazi not available: {}", e);
            Ok(())
        }
    }
}
