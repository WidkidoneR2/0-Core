use std::process::Command;

pub fn check_flatpak_updates() -> Vec<String> {
    println!("   Checking flatpak...");
    Vec::new()
}

pub fn update_flatpak() -> anyhow::Result<()> {
    println!("   Running: flatpak update -y");

    let status = Command::new("flatpak").arg("update").arg("-y").status();

    match status {
        Ok(s) if s.success() => {
            println!("   ✅  Flatpak packages updated");
            Ok(())
        }
        Ok(_) => {
            println!("   ⚠️  No flatpak updates available");
            Ok(())
        }
        Err(e) => {
            println!("   ⚠️  Flatpak not available: {}", e);
            Ok(())
        }
    }
}
