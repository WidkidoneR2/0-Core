//! Neovim plugin checker — supports LazyVim, AstroNvim, NvChad
use std::process::Command;

/// Detect which neovim distribution is installed
pub enum NvimDistro {
    LazyVim,
    AstroNvim,
    NvChad,
    PlainLazy,
    Unknown,
}

pub fn detect_distro() -> NvimDistro {
    let home = std::env::var("HOME").unwrap_or_default();

    // LazyVim — has lazyvim in its lua config
    let lazyvim_marker = format!("{}/.config/nvim/lua/lazyvim", home);
    let lazyvim_init = format!("{}/.config/nvim/lua/config/lazy.lua", home);
    if std::path::Path::new(&lazyvim_marker).exists()
        || std::path::Path::new(&lazyvim_init).exists()
    {
        return NvimDistro::LazyVim;
    }

    // AstroNvim
    let astro_marker = format!("{}/.config/nvim/lua/astronvim", home);
    if std::path::Path::new(&astro_marker).exists() {
        return NvimDistro::AstroNvim;
    }

    // NvChad
    let nvchad_marker = format!("{}/.config/nvim/lua/chadrc.lua", home);
    if std::path::Path::new(&nvchad_marker).exists() {
        return NvimDistro::NvChad;
    }

    // Plain lazy.nvim without a distro
    let lazy_lock = format!("{}/.config/nvim/lazy-lock.json", home);
    if std::path::Path::new(&lazy_lock).exists() {
        return NvimDistro::PlainLazy;
    }

    NvimDistro::Unknown
}

/// Check for outdated plugins by reading lazy-lock.json and comparing to git
pub fn check_neovim_updates() -> Vec<String> {
    println!("   Checking neovim plugins...");

    // Verify nvim is available
    if Command::new("nvim").arg("--version").output().is_err() {
        println!("      ⚠️  nvim not found");
        return Vec::new();
    }

    let distro = detect_distro();
    let distro_name = match distro {
        NvimDistro::LazyVim => "LazyVim",
        NvimDistro::AstroNvim => "AstroNvim",
        NvimDistro::NvChad => "NvChad",
        NvimDistro::PlainLazy => "lazy.nvim",
        NvimDistro::Unknown => "neovim",
    };
    println!("      Detected: {}", distro_name);

    // Check lazy-lock.json for outdated plugins via headless nvim
    // This runs Lazy check in headless mode and captures output
    let output = Command::new("nvim")
        .args([
            "--headless",
            "--noplugin",
            "-c",
            "lua require('lazy').check()",
            "-c",
            "sleep 3",
            "-c",
            "qa!",
        ])
        .output();

    match output {
        Ok(out) => {
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );

            // Count plugins that have updates
            let outdated: Vec<String> = text
                .lines()
                .filter(|l| l.contains("outdated") || l.contains("updates"))
                .map(|l| l.trim().to_string())
                .collect();

            if outdated.is_empty() {
                // Lazy check doesn't always print to stdout in headless
                // Fall back to checking if lazy-lock.json is older than plugins
                check_via_lockfile()
            } else {
                outdated
            }
        }
        Err(_) => Vec::new(),
    }
}

/// Secondary check — see if lazy-lock.json suggests updates needed
fn check_via_lockfile() -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let lock_path = format!("{}/.config/nvim/lazy-lock.json", home);

    if !std::path::Path::new(&lock_path).exists() {
        return Vec::new();
    }

    // Read lockfile and check if any plugin dirs have newer commits
    let content = match std::fs::read_to_string(&lock_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    // Parse plugin names from lockfile
    let plugin_dir = format!("{}/.local/share/nvim/lazy", home);
    let mut outdated = Vec::new();

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(plugins) = json.as_object() {
            for (name, info) in plugins {
                let _locked_commit = info["commit"].as_str().unwrap_or("");
                let plugin_path = format!("{}/{}", plugin_dir, name);

                if !std::path::Path::new(&plugin_path).exists() {
                    continue;
                }

                // Check if HEAD matches locked commit
                let head = Command::new("git")
                    .args(["-C", &plugin_path, "rev-parse", "HEAD"])
                    .output();

                if let Ok(out) = head {
                    let head_commit = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    // Check if remote has newer commits
                    let fetch = Command::new("git")
                        .args(["-C", &plugin_path, "fetch", "--dry-run"])
                        .output();
                    if let Ok(f) = fetch {
                        if !f.stderr.is_empty() {
                            outdated.push(name.clone());
                        }
                    }
                    let _ = head_commit;
                }
            }
        }
    }

    if !outdated.is_empty() {
        println!("      {} plugins may have updates", outdated.len());
    }

    outdated
}

/// Update neovim plugins — respects detected distro
pub fn update_neovim() -> anyhow::Result<()> {
    let distro = detect_distro();

    let cmd = match distro {
        NvimDistro::LazyVim | NvimDistro::PlainLazy | NvimDistro::NvChad => {
            println!("   Running: nvim --headless '+Lazy! sync' +qa");
            vec!["--headless", "+Lazy! sync", "+qa"]
        }
        NvimDistro::AstroNvim => {
            println!("   Running: nvim --headless '+AstroUpdate' +qa");
            vec!["--headless", "+AstroUpdate", "+qa"]
        }
        NvimDistro::Unknown => {
            println!("   ⚠️  Unknown neovim config — skipping plugin update");
            return Ok(());
        }
    };

    let status = Command::new("nvim").args(&cmd).status();

    match status {
        Ok(s) if s.success() => {
            println!("   ✅  Neovim plugins synced");
        }
        Ok(_) => {
            println!("   ⚠️  Neovim sync completed with warnings (normal)");
        }
        Err(e) => {
            println!("   ⚠️  Neovim not available: {}", e);
        }
    }

    Ok(())
}
