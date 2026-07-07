// faelight-fm v3.1 -- Nix store awareness

use std::process::Command;

pub fn describe_store_path(store_path: &str) -> String {
    let mut info = format!("❄️  Nix Store Path\n{}\n\n", store_path);

    // Extract package name from path
    if let Some(pkg) = store_path
        .strip_prefix("/nix/store/")
        .and_then(|s| s.splitn(2, '-').nth(1))
    {
        info.push_str(&format!("Package: {}\n", pkg));
    }

    // Who depends on this path
    let refs = Command::new("nix-store")
        .args(["--query", "--referrers", store_path])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    if !refs.is_empty() {
        let count = refs.lines().count();
        info.push_str(&format!("\nReferrers: {} paths depend on this\n", count));
        for line in refs.lines().take(3) {
            if let Some(pkg) = line
                .strip_prefix("/nix/store/")
                .and_then(|s| s.splitn(2, '-').nth(1))
            {
                info.push_str(&format!("  ← {}\n", pkg));
            }
        }
        if count > 3 {
            info.push_str(&format!("  ... and {} more\n", count - 3));
        }
    }

    // What built this path
    let deriver = Command::new("nix-store")
        .args(["--query", "--deriver", store_path])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if !deriver.is_empty() && deriver != "unknown-deriver" {
        if let Some(drv) = deriver
            .strip_prefix("/nix/store/")
            .and_then(|s| s.splitn(2, '-').nth(1))
        {
            info.push_str(&format!("\nBuilt by: {}\n", drv));
        }
    }

    info
}

#[allow(dead_code)]
pub fn query_file_owner(path: &str) -> Option<String> {
    // Use nix-store to find which derivation owns a file
    let out = Command::new("nix-store")
        .args(["--query", "--roots", path])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    if s.trim().is_empty() {
        return None;
    }
    Some(s.lines().take(3).collect::<Vec<_>>().join("\n"))
}

#[allow(dead_code)]
pub fn current_generation() -> String {
    let out = Command::new("nixos-version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if out.is_empty() {
        return String::new();
    }
    format!("NixOS {}", out)
}
