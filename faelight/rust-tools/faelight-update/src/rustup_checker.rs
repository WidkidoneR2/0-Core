use std::process::Command;

pub fn check_rustup_updates() -> Vec<String> {
    // Check if rustup is installed
    if Command::new("rustup").arg("--version").output().is_err() {
        return vec![];
    }

    // Run rustup check
    match Command::new("rustup").arg("check").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Update available") || stdout.contains("outdated") {
                vec!["Rust toolchain".to_string()]
            } else {
                vec![]
            }
        }
        Err(_) => vec![],
    }
}

pub fn update_rustup() -> std::io::Result<()> {
    println!("   Running: rustup update");
    Command::new("rustup").arg("update").status()?;
    Ok(())
}
