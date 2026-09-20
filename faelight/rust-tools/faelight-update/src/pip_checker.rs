use std::path::Path;
use std::process::Command;

/// Whether this interpreter refuses pip installs, per PEP 668.
///
/// A marker file beside the standard library, which both Arch and NixOS ship and which pip
/// itself honours. Reading it asks the interpreter rather than guessing from the distribution.
fn externally_managed() -> bool {
    let out = match Command::new("python3")
        .args([
            "-c",
            "import sysconfig; print(sysconfig.get_path('stdlib'))",
        ])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return false,
    };
    let stdlib = String::from_utf8_lossy(&out.stdout).trim().to_string();
    !stdlib.is_empty() && Path::new(&stdlib).join("EXTERNALLY-MANAGED").exists()
}

pub fn check_pip_updates() -> Vec<String> {
    // ⚠️ THE GUARD WAS RIGHT AND ITS TEST WAS WRONG. It read /etc/NIXOS, so it stopped
    // firing when NixOS went -- but PEP 668 is not a NixOS idea. ARCH SHIPS
    // EXTERNALLY-MANAGED TOO (measured 2026-09-20: /usr/lib/python3.14/EXTERNALLY-MANAGED
    // is present), and pip refuses to touch a managed environment on either system.
    // Ask the question the interpreter answers, not the one a dead distribution did.
    if externally_managed() {
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
    if externally_managed() {
        println!("   ⏭️  Skipping pip -- this Python is externally managed (PEP 668)");
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
            println!("   💡 On NixOS, use nixpkgs for Python packages");
            return Ok(());
        }
    }

    // Check pipx
    if Command::new("pipx").arg("--version").output().is_ok() {
        println!("   Running: pipx upgrade-all");
        let _ = Command::new("pipx").arg("upgrade-all").status();
    }

    Ok(())
}
