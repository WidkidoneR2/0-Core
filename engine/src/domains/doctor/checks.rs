// doctor/checks.rs — all 23 health check functions
#![allow(dead_code)]
use super::{CheckResult, Status};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

pub fn check_stow(core_root: &str, home: &str) -> CheckResult {
    let stow_dir = PathBuf::from(core_root).join("config");
    let mut stowed = 0;
    let mut total = 0;

    if let Ok(entries) = fs::read_dir(&stow_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let has_files = WalkDir::new(&path)
                .into_iter()
                .filter_map(|e| e.ok())
                .any(|e| e.file_type().is_file());
            if !has_files {
                continue;
            }
            total += 1;
            // Check if any symlink in home points to this package
            let found = WalkDir::new(home)
                .max_depth(5)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .any(|e| {
                    let p = e.path();
                    if p.is_symlink() {
                        if let Ok(target) = fs::read_link(p) {
                            return target
                                .to_string_lossy()
                                .contains(&format!("0-core/config/{}", name));
                        }
                    }
                    false
                });
            if found {
                stowed += 1;
            }
        }
    }

    if std::path::Path::new("/etc/NIXOS").exists() {
        return CheckResult {
            id: "stow".into(),
            name: "Stow Symlinks".into(),
            status: Status::Pass,
            message: "Managed by home-manager (NixOS)".into(),
            fix: None,
        };
    }
    if stowed == total {
        CheckResult {
            id: "stow".into(),
            name: "Stow Symlinks".into(),
            status: Status::Pass,
            message: format!("All {}/{} packages properly stowed", stowed, total),
            fix: None,
        }
    } else {
        CheckResult {
            id: "stow".into(),
            name: "Stow Symlinks".into(),
            status: Status::Fail,
            message: format!("Only {}/{} packages stowed", stowed, total),
            fix: Some("Run: rebuild to apply config changes".into()),
        }
    }
}

pub fn check_services() -> CheckResult {
    let services = [
        ("faelight-bar", "Status bar"),
        ("faelight-notify", "Notifications"),
    ];
    let running = services
        .iter()
        .filter(|(name, _)| {
            // Match full path to avoid false positives
            let full = format!("/run/current-system/sw/bin/{}", name);
            Command::new("pgrep")
                .arg("-f")
                .arg(&full)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
        .count();
    if running == 2 {
        CheckResult {
            id: "services".into(),
            name: "System Services".into(),
            status: Status::Pass,
            message: "All 2/2 services running".into(),
            fix: None,
        }
    } else {
        CheckResult {
            id: "services".into(),
            name: "System Services".into(),
            status: Status::Warn,
            message: format!("Only {}/2 services running", running),
            fix: Some("Configure faelight-bar/faelight-notify as systemd user services".into()),
        }
    }
}

pub fn check_broken_symlinks(_core_root: &str, home: &str) -> CheckResult {
    let mut broken = 0;
    let config = PathBuf::from(home).join(".config");
    for entry in WalkDir::new(&config)
        .max_depth(6)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.is_symlink()
            && !p.exists()
            && !p.to_string_lossy().contains("BraveSoftware")
            && !p.to_string_lossy().contains("Notesnook")
            && !p.to_string_lossy().contains("Singleton")
        {
            broken += 1;
        }
    }

    if broken == 0 {
        CheckResult {
            id: "broken_symlinks".into(),
            name: "Broken Symlinks".into(),
            status: Status::Pass,
            message: "No broken symlinks found".into(),
            fix: None,
        }
    } else {
        CheckResult {
            id: "broken_symlinks".into(),
            name: "Broken Symlinks".into(),
            status: Status::Fail,
            message: format!("{} broken symlinks found", broken),
            fix: Some("Review and remove broken links".into()),
        }
    }
}

pub fn check_broot(_home: &str) -> CheckResult {
    let broot_exists = std::process::Command::new("which")
        .arg("broot")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if broot_exists {
        CheckResult {
            id: "yazi_plugins".into(),
            name: "Broot".into(),
            status: Status::Pass,
            message: "broot installed".into(),
            fix: None,
        }
    } else {
        CheckResult {
            id: "yazi_plugins".into(),
            name: "Broot".into(),
            status: Status::Warn,
            message: "broot not found".into(),
            fix: Some("Add broot to NixOS packages".into()),
        }
    }
}

pub fn check_binaries() -> CheckResult {
    let bins = [
        "mango",
        "alacritty",
        "faelight-palette",
        "broot",
        "hx",
        "git",
        "bat",
        "eza",
        "fd",
        "rg",
        "zoxide",
        "brightnessctl",
        "wpctl",
        "wl-copy",
        "nix-tree",
        "nvd",
    ];
    let missing: Vec<_> = bins
        .iter()
        .filter(|b| {
            !Command::new("which")
                .arg(b)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
        .map(|b| b.to_string())
        .collect();
    if missing.is_empty() {
        CheckResult {
            id: "binaries".into(),
            name: "Binary Dependencies".into(),
            status: Status::Pass,
            message: format!("All {} binaries found", bins.len()),
            fix: None,
        }
    } else {
        CheckResult {
            id: "binaries".into(),
            name: "Binary Dependencies".into(),
            status: Status::Fail,
            message: format!("{} binaries missing", missing.len()),
            fix: Some("Install with: sudo pacman -S <package>".into()),
        }
    }
}

pub fn check_git(core_root: &str) -> CheckResult {
    let has_changes = Command::new("git")
        .args(["-C", core_root, "status", "--porcelain"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);
    let has_unpushed = Command::new("git")
        .args(["-C", core_root, "log", "@{u}..", "--oneline"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);
    if !has_changes && !has_unpushed {
        CheckResult {
            id: "git".into(),
            name: "Git Repository".into(),
            status: Status::Pass,
            message: "Working tree clean, all commits pushed".into(),
            fix: None,
        }
    } else {
        let mut issues = vec![];
        if has_changes {
            issues.push("Uncommitted changes");
        }
        if has_unpushed {
            issues.push("Unpushed commits");
        }
        CheckResult {
            id: "git".into(),
            name: "Git Repository".into(),
            status: Status::Warn,
            message: issues.join(", "),
            fix: Some("Commit and push changes".into()),
        }
    }
}

pub fn check_themes(core_root: &str) -> CheckResult {
    let stow = PathBuf::from(core_root).join("config");
    let count = fs::read_dir(&stow)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| {
                    let n = e.file_name().to_string_lossy().to_string();
                    n.starts_with("faelight") || n.starts_with("theme-")
                })
                .count()
        })
        .unwrap_or(0);
    if count >= 1 {
        CheckResult {
            id: "themes".into(),
            name: "Theme Packages".into(),
            status: Status::Pass,
            message: format!("{}/1 theme packages present", count),
            fix: None,
        }
    } else {
        CheckResult {
            id: "themes".into(),
            name: "Theme Packages".into(),
            status: Status::Warn,
            message: "0/1 theme packages found".into(),
            fix: None,
        }
    }
}

pub fn check_scripts(core_root: &str) -> CheckResult {
    let scripts_dir = PathBuf::from(core_root).join("scripts");
    let required = ["faelight", "profile", "intent"];
    let issues: Vec<_> = required
        .iter()
        .filter_map(|s| {
            let path = scripts_dir.join(s);
            if !path.exists() {
                Some(format!("{} missing", s))
            } else if path
                .metadata()
                .map(|m| m.permissions().mode() & 0o111 == 0)
                .unwrap_or(false)
            {
                Some(format!("{} not executable", s))
            } else {
                None
            }
        })
        .collect();
    if std::path::Path::new("/etc/NIXOS").exists() {
        return CheckResult {
            id: "scripts".into(),
            name: "Scripts".into(),
            status: Status::Pass,
            message: "Tools deployed as Nix binaries (NixOS)".into(),
            fix: None,
        };
    }
    if issues.is_empty() {
        CheckResult {
            id: "scripts".into(),
            name: "Scripts".into(),
            status: Status::Pass,
            message: "All scripts present and executable".into(),
            fix: None,
        }
    } else {
        CheckResult {
            id: "scripts".into(),
            name: "Scripts".into(),
            status: Status::Warn,
            message: format!("{} script issues", issues.len()),
            fix: Some("chmod +x ~/0-core/scripts/*".into()),
        }
    }
}

pub fn check_dotmeta() -> CheckResult {
    CheckResult {
        id: "dotmeta".into(),
        name: "Package Metadata".into(),
        status: Status::Pass,
        message: ".dotmeta files intentionally removed (stow conflict resolution)".into(),
        fix: None,
    }
}

pub fn check_intents(core_root: &str) -> CheckResult {
    let intent_dir = PathBuf::from(core_root).join("intents");
    let mut total = 0;
    let mut complete = 0;
    let mut planned = 0;
    for category in [
        "complete",
        "decisions",
        "experiments",
        "philosophy",
        "future",
        "cancelled",
        "deferred",
        "incidents",
        "active",
    ] {
        let cat_dir = intent_dir.join(category);
        if let Ok(entries) = fs::read_dir(&cat_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                    total += 1;
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.contains("status: complete") {
                            complete += 1;
                        } else if content.contains("status: planned") {
                            planned += 1;
                        }
                    }
                }
            }
        }
    }
    CheckResult {
        id: "intents".into(),
        name: "Intent Ledger".into(),
        status: Status::Pass,
        message: format!(
            "{} intents ({} complete, {} planned)",
            total, complete, planned
        ),
        fix: None,
    }
}

pub fn check_profiles(core_root: &str, home: &str) -> CheckResult {
    let profile_script = PathBuf::from(core_root).join("scripts/profile");
    let state_dir = PathBuf::from(home).join(".local/state/0-core");
    let current = fs::read_to_string(state_dir.join("current-profile"))
        .unwrap_or_else(|_| "default".into())
        .trim()
        .to_string();
    let profile_present = if std::path::Path::new("/etc/NIXOS").exists() {
        Command::new("which").arg("profile").output().map(|o| o.status.success()).unwrap_or(false)
    } else {
        profile_script.exists()
    };
    if !profile_present {
        CheckResult {
            id: "profiles".into(),
            name: "Profile System".into(),
            status: Status::Warn,
            message: "Profile script missing".into(),
            fix: Some("Check scripts/profile".into()),
        }
    } else {
        CheckResult {
            id: "profiles".into(),
            name: "Profile System".into(),
            status: Status::Pass,
            message: format!("Profile system OK (current: {})", current),
            fix: None,
        }
    }
}

pub fn check_faelight_config(home: &str) -> CheckResult {
    let config_dir = PathBuf::from(home).join(".config/faelight");
    let files = ["config.toml", "profiles.toml", "themes.toml"];
    let mut issues = 0;
    for file in files {
        let path = config_dir.join(file);
        if !path.exists() {
            issues += 1;
        } else if let Ok(content) = fs::read_to_string(&path) {
            if toml::from_str::<toml::Value>(&content).is_err() {
                issues += 1;
            }
        }
    }
    if issues == 0 {
        CheckResult {
            id: "config".into(),
            name: "Faelight Config".into(),
            status: Status::Pass,
            message: "All config files valid".into(),
            fix: None,
        }
    } else {
        CheckResult {
            id: "config".into(),
            name: "Faelight Config".into(),
            status: Status::Warn,
            message: format!("{} config issues", issues),
            fix: Some("Run: faelight config validate".into()),
        }
    }
}

pub fn check_keybinds(_core_root: &str, home: &str) -> CheckResult {
    // Compositor-aware: read the active compositor's keybind config.
    // mango (daily driver): ~/.config/mango/config.conf -- bind=MODS,key,action,...
    // niri (retired fallback): ~/.config/niri/config.kdl
    let mango_config = PathBuf::from(home).join(".config/mango/config.conf");
    let niri_config = PathBuf::from(home).join(".config/niri/config.kdl");
    let (wm_name, wm_config, is_mango) = if mango_config.exists() {
        ("mango", mango_config, true)
    } else if niri_config.exists() {
        ("niri", niri_config, false)
    } else {
        return CheckResult {
            id: "keybinds".into(),
            name: "Compositor Keybinds".into(),
            status: Status::Warn,
            message: "No compositor keybind config found".into(),
            fix: Some("Deploy a compositor config (mango/niri)".into()),
        };
    };
    let config_content = match std::fs::read_to_string(&wm_config) {
        Ok(c) => c,
        Err(_) => return CheckResult {
            id: "keybinds".into(),
            name: "Compositor Keybinds".into(),
            status: Status::Warn,
            message: "Could not read keybind config".into(),
            fix: None,
        },
    };
    let keybinds: Vec<String> = if is_mango {
        config_content
            .lines()
            .map(|l| l.trim())
            .filter_map(|l| l.strip_prefix("bind="))
            .filter_map(|rest| {
                let mut parts = rest.splitn(3, ',');
                let mods = parts.next()?;
                let key = parts.next()?;
                Some(format!("{},{}", mods, key))
            })
            .collect()
    } else {
        config_content
            .lines()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("Mod+") || t.starts_with("Ctrl+") || t.starts_with("Shift+")
                    || t.starts_with("Alt+") || t.starts_with("Super+")
            })
            .map(|l| l.trim().split('{').next().unwrap_or("").trim().to_string())
            .collect()
    };
    let count = keybinds.len();
    let mut seen = std::collections::HashSet::new();
    let mut conflicts = 0;
    for bind in &keybinds {
        if !seen.insert(bind.as_str()) {
            conflicts += 1;
        }
    }
    if conflicts == 0 {
        CheckResult {
            id: "keybinds".into(),
            name: "Compositor Keybinds".into(),
            status: Status::Pass,
            message: format!("{}: {} keybindings, no conflicts", wm_name, count),
            fix: None,
        }
    } else {
        CheckResult {
            id: "keybinds".into(),
            name: "Compositor Keybinds".into(),
            status: Status::Warn,
            message: format!("{}: {} keybind conflicts detected -- review {} config", wm_name, conflicts, wm_name),
            fix: None,
        }
    }
}

pub fn check_security_hardening() -> CheckResult {
    let mut details = 0;
    // NixOS: check native nftables firewall instead of UFW
    let ufw_active = Command::new("systemctl")
        .args(["is-active", "firewall"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false);
    if ufw_active {
        details += 1;
    }
    let f2b_active = Command::new("systemctl")
        .args(["is-active", "fail2ban"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false);
    if f2b_active {
        details += 1;
    }
    let sshd = PathBuf::from("/etc/ssh/sshd_config");
    let ssh_ok = sshd.exists()
        && fs::read_to_string(&sshd)
            .map(|c| c.contains("PermitRootLogin no") || c.contains("PasswordAuthentication no"))
            .unwrap_or(false);
    if ssh_ok {
        details += 1;
    }
    let mut active = vec![];
    if ufw_active {
        active.push("Firewall ✅");
    }
    if f2b_active {
        active.push("fail2ban ✅");
    }
    if ssh_ok {
        active.push("SSH hardened ✅");
    }
    if !ufw_active {
        active.push("Firewall ❌");
    }

    CheckResult {
        id: "security".into(),
        name: "Security Hardening".into(),
        status: if details > 0 {
            Status::Pass
        } else {
            Status::Fail
        },
        message: format!("Security: {}", active.join("  ")),
        fix: if !ufw_active {
            Some("Enable firewall: networking.firewall.enable = true in configuration.nix".into())
        } else {
            None
        },
    }
}

pub fn check_security_audit(home: &str) -> CheckResult {
    let scan_path = PathBuf::from(home).join(".local/state/0-core/security/last-scan.json");
    if !scan_path.exists() {
        return CheckResult {
            id: "security_audit".into(),
            name: "Security Audit".into(),
            status: Status::Warn,
            message: "No scan found — run: core security scan".into(),
            fix: Some("Run: core security scan".into()),
        };
    }
    let data = match fs::read_to_string(&scan_path) {
        Ok(d) => d,
        Err(_) => {
            return CheckResult {
                id: "security_audit".into(),
                name: "Security Audit".into(),
                status: Status::Warn,
                message: "Could not read scan results".into(),
                fix: Some("Run: core security scan".into()),
            }
        }
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) else {
        return CheckResult {
            id: "security_audit".into(),
            name: "Security Audit".into(),
            status: Status::Warn,
            message: "Could not parse scan results".into(),
            fix: Some("Run: core security scan".into()),
        };
    };
    let timestamp = json["timestamp"].as_str().unwrap_or("unknown");
    let all_findings = json["findings"].as_array().cloned().unwrap_or_default();
    let findings = all_findings.len();

    // Only count findings that have patches available
    let patchable: Vec<_> = all_findings
        .iter()
        .filter(|f| f["fix"].as_str().unwrap_or("").contains("Patch available"))
        .collect();

    let critical = patchable
        .iter()
        .filter(|f| f["severity"].as_str() == Some("Critical"))
        .count();
    let high = patchable
        .iter()
        .filter(|f| f["severity"].as_str() == Some("High"))
        .count();
    let medium = patchable
        .iter()
        .filter(|f| f["severity"].as_str() == Some("Medium"))
        .count();
    let patchable_count = patchable.len();

    let status = if critical > 0 {
        Status::Fail
    } else if high > 0 || medium > 0 {
        Status::Warn
    } else {
        Status::Pass
    };

    // Calculate days since scan
    let days_since = chrono::DateTime::parse_from_rfc3339(timestamp)
        .or_else(|_| chrono::DateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S"))
        .map(|dt| {
            let now = chrono::Utc::now();
            let scan_time = dt.with_timezone(&chrono::Utc);
            (now - scan_time).num_days()
        })
        .unwrap_or(0);
    let age_note = if days_since == 0 {
        "today".to_string()
    } else if days_since == 1 {
        "1 day ago".to_string()
    } else {
        format!("{} days ago", days_since)
    };
    let stale_note = if days_since > 7 {
        " — consider rescan"
    } else {
        ""
    };
    let message = if patchable_count == 0 {
        format!(
            "{} findings — all upstream pending, none patchable (scanned {}{})",
            findings, age_note, stale_note
        )
    } else {
        format!(
            "{} findings ({} patchable): {} critical, {} high, {} medium (scanned {}{})",
            findings, patchable_count, critical, high, medium, age_note, stale_note
        )
    };

    CheckResult {
        id: "security_audit".into(),
        name: "Security Audit".into(),
        status,
        message,
        fix: if patchable_count > 0 {
            Some("Run: sudo pacman -Syu".into())
        } else {
            None
        },
    }
}

pub fn check_alias_coverage() -> CheckResult {
    use crate::domains::doctor::aliases::{parse_aliases, EXPECTED_TOOLS};
    use faelight_core::paths;

    let aliases_path = paths::aliases_file();
    let aliases = match parse_aliases(&aliases_path) {
        Ok(a) => a,
        Err(_) => {
            return CheckResult {
                id: "alias_coverage".into(),
                name: "Alias Coverage".into(),
                status: Status::Fail,
                message: "Could not read aliases file".into(),
                fix: Some("Check aliases file exists".into()),
            };
        }
    };

    let mut missing: Vec<&str> = Vec::new();
    for tool in EXPECTED_TOOLS {
        if *tool == "faelight-daemon" || *tool == "faelight-core" {
            continue;
        }
        if !aliases.values().any(|v| v.contains(tool)) {
            missing.push(tool);
        }
    }

    if missing.is_empty() {
        CheckResult {
            id: "alias_coverage".into(),
            name: "Alias Coverage".into(),
            status: Status::Pass,
            message: format!("All {} tools have aliases ({} total)", EXPECTED_TOOLS.len(), aliases.len()),
            fix: None,
        }
    } else {
        CheckResult {
            id: "alias_coverage".into(),
            name: "Alias Coverage".into(),
            status: Status::Warn,
            message: format!("{} tools missing aliases: {}", missing.len(), missing.join(", ")),
            fix: Some("Run: core audit aliases".into()),
        }
    }
}
pub fn check_rust_toolchain() -> CheckResult {
    let cargo_ok = Command::new("cargo")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let rustc_ok = Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if cargo_ok && rustc_ok {
        CheckResult {
            id: "rust_toolchain".into(),
            name: "Rust Toolchain".into(),
            status: Status::Pass,
            message: "Rust toolchain available".into(),
            fix: None,
        }
    } else {
        CheckResult {
            id: "rust_toolchain".into(),
            name: "Rust Toolchain".into(),
            status: Status::Fail,
            message: "Rust toolchain missing".into(),
            fix: Some(
                "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                    .into(),
            ),
        }
    }
}

pub fn check_disk_space() -> CheckResult {
    let warnings: Vec<_> = ["/", "/home"]
        .iter()
        .filter_map(|mount| {
            Command::new("df")
                .args(["-h", mount])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|s| s.lines().nth(1).map(|l| l.to_string()))
                .and_then(|line| line.split_whitespace().nth(4).map(|u| u.to_string()))
                .and_then(|u| u.strip_suffix('%').and_then(|p| p.parse::<u32>().ok()))
                .filter(|&p| p > 90)
                .map(|p| format!("{} at {}%", mount, p))
        })
        .collect();
    if warnings.is_empty() {
        CheckResult {
            id: "disk_space".into(),
            name: "Disk Space".into(),
            status: Status::Pass,
            message: "Sufficient disk space".into(),
            fix: None,
        }
    } else {
        CheckResult {
            id: "disk_space".into(),
            name: "Disk Space".into(),
            status: Status::Warn,
            message: "Low disk space detected".into(),
            fix: Some("Clean up old files or expand partition".into()),
        }
    }
}

pub fn check_tool_installation() -> CheckResult {
    // Registry-aware: read deployable, non-retired, high-usage tools
    let core_root = std::env::var("HOME").unwrap_or_default() + "/0-core";
    let registry_path = PathBuf::from(&core_root).join("registry/tools.toml");
    let tools: Vec<String> = fs::read_to_string(&registry_path)
        .map(|content| {
            let mut tools = vec![];
            let mut name = String::new();
            let mut deployable = false;
            let mut retired = false;
            let mut expected_usage = String::new();
            for line in content.lines() {
                let line = line.trim();
                if line == "[[tool]]" {
                    if !name.is_empty()
                        && deployable
                        && !retired
                        && (expected_usage == "high" || expected_usage == "medium")
                    {
                        tools.push(name.clone());
                    }
                    name.clear();
                    deployable = false;
                    retired = false;
                    expected_usage.clear();
                } else if let Some(v) = line.strip_prefix("name = \"") {
                    name = v.trim_end_matches('"').to_string();
                } else if let Some(v) = line.strip_prefix("expected_usage = \"") {
                    expected_usage = v.trim_end_matches('"').to_string();
                } else if line == "deployable = true" {
                    deployable = true;
                } else if line == "retired = true" {
                    retired = true;
                }
            }
            if !name.is_empty()
                && deployable
                && !retired
                && (expected_usage == "high" || expected_usage == "medium")
            {
                tools.push(name);
            }
            tools
        })
        .unwrap_or_default();
    let missing: Vec<_> = tools
        .iter()
        .filter(|t| {
            !Command::new("which")
                .arg(t.as_str())
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    if missing.is_empty() {
        CheckResult {
            id: "tool_installation".into(),
            name: "Tool Installation".into(),
            status: Status::Pass,
            message: format!("All {} key tools installed", tools.len()),
            fix: None,
        }
    } else {
        CheckResult {
            id: "tool_installation".into(),
            name: "Tool Installation".into(),
            status: Status::Warn,
            message: format!(
                "{}/{} key tools installed — missing: {}",
                tools.len() - missing.len(),
                tools.len(),
                missing.join(", ")
            ),
            fix: Some("Run: deploy <tool>".into()),
        }
    }
}

pub fn check_path_resilience(core_root: &str) -> CheckResult {
    let scripts_dir = PathBuf::from(core_root).join("scripts");
    let registry_path = PathBuf::from(core_root).join("registry/tools.toml");

    // Read deployable, non-retired tools from registry (INT-183)
    let rust_tools: Vec<String> = fs::read_to_string(&registry_path)
        .map(|content| {
            let mut tools = vec![];
            let mut name = String::new();
            let mut deployable = false;
            let mut retired = false;
            let mut tool_type = String::new();
            for line in content.lines() {
                let line = line.trim();
                if line == "[[tool]]" {
                    if !name.is_empty() && deployable && !retired && tool_type == "rust" {
                        tools.push(name.clone());
                    }
                    name.clear();
                    deployable = false;
                    retired = false;
                    tool_type.clear();
                } else if let Some(v) = line.strip_prefix("name = \"") {
                    name = v.trim_end_matches('"').to_string();
                } else if let Some(v) = line.strip_prefix("type = \"") {
                    tool_type = v.trim_end_matches('"').to_string();
                } else if line == "deployable = true" {
                    deployable = true;
                } else if line == "retired = true" {
                    retired = true;
                }
            }
            // Last tool
            if !name.is_empty() && deployable && !retired && tool_type == "rust" {
                tools.push(name);
            }
            tools
        })
        .unwrap_or_default();

    let total = rust_tools.len();
    let deployed = rust_tools
        .iter()
        .filter(|n| {
            if std::path::Path::new("/etc/NIXOS").exists() {
                Command::new("which").arg(n).output().map(|o| o.status.success()).unwrap_or(false)
            } else {
                scripts_dir.join(n).exists()
            }
        })
        .count();
    let pct = if total > 0 {
        (deployed * 100) / total
    } else {
        0
    };
    CheckResult {
        id: "path_resilience".into(),
        name: "Path Resilience".into(),
        status: if pct >= 90 {
            Status::Pass
        } else {
            Status::Warn
        },
        message: format!("{}/{} tools deployed ({}%)", deployed, total, pct),
        fix: if pct < 90 {
            Some("Build and deploy missing tools".into())
        } else {
            None
        },
    }
}

pub fn check_sandbox(core_root: &str) -> CheckResult {
    // Check sandbox binary exists and policies are valid
    // NixOS: check /run/current-system/sw/bin first, fall back to scripts/
    let binary_exists = std::path::Path::new("/run/current-system/sw/bin/faelight-sandbox").exists()
        || std::path::PathBuf::from(core_root)
            .join("scripts/faelight-sandbox")
            .exists();

    if !binary_exists {
        return CheckResult {
            id: "sandbox".into(),
            name: "Sandbox".into(),
            status: Status::Fail,
            message: "faelight-sandbox not deployed".into(),
            fix: Some("cargo build --release -p faelight-sandbox && cp target/release/faelight-sandbox scripts/".into()),
        };
    }

    // Check policies file exists
    let policies_path =
        std::path::PathBuf::from(core_root).join("registry/sandbox-policies.toml");

    if !policies_path.exists() {
        return CheckResult {
            id: "sandbox".into(),
            name: "Sandbox".into(),
            status: Status::Warn,
            message: "sandbox-policies.toml not found".into(),
            fix: Some("Create registry/sandbox-policies.toml".into()),
        };
    }

    // Count policies
    let policy_count = std::fs::read_to_string(&policies_path)
        .map(|t| t.lines().filter(|l| l.trim().starts_with("name =")).count())
        .unwrap_or(0);

    CheckResult {
        id: "sandbox".into(),
        name: "Sandbox".into(),
        status: Status::Pass,
        message: format!(
            "faelight-sandbox deployed — {} policies active",
            policy_count
        ),
        fix: None,
    }
}


pub fn check_boot_errors() -> CheckResult {
    // Honest boot health: benign error-priority noise (USB-C/EC/HID init) is the
    // baseline on this hardware, so WARN only on critical-or-worse kernel events.
    // The error-level count is shown for transparency, not as an alarm.
    let count = |args: &[&str]| -> Option<usize> {
        Command::new("journalctl")
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .count()
            })
    };
    let crit = count(&["-b", "-k", "-p", "crit", "--no-pager", "-q"]);
    let errs = count(&["-b", "-k", "-p", "err", "--no-pager", "-q"]);
    match crit {
        Some(0) => {
            let msg = match errs {
                Some(0) | None => "No kernel errors since last boot".to_string(),
                Some(n) => format!("No critical kernel errors since boot ({} low-priority notices)", n),
            };
            CheckResult {
                id: "boot_errors".into(),
                name: "Boot Errors".into(),
                status: Status::Pass,
                message: msg,
                fix: None,
            }
        }
        Some(n) => CheckResult {
            id: "boot_errors".into(),
            name: "Boot Errors".into(),
            status: Status::Warn,
            message: format!("{} critical kernel error(s) since last boot", n),
            fix: Some("journalctl -b -k -p crit".into()),
        },
        None => CheckResult {
            id: "boot_errors".into(),
            name: "Boot Errors".into(),
            status: Status::Warn,
            message: "Could not read the kernel journal".into(),
            fix: None,
        },
    }
}

pub fn check_boot_time() -> CheckResult {
    // Measure userspace startup (time to the login/graphical target) -- the part
    // we control. The full systemd-analyze total folds in firmware POST and the
    // wall-clock spent waiting at the LUKS password prompt, which is not a boot-
    // performance signal. The bar we want is time-to-greetd, i.e. userspace.
    let output = Command::new("systemd-analyze").arg("time").output();
    let out = match output {
        Ok(o) if o.status.success() => o,
        _ => {
            return CheckResult {
                id: "boot_time".into(),
                name: "Boot Time".into(),
                status: Status::Warn,
                message: "Could not read boot time (systemd-analyze)".into(),
                fix: None,
            }
        }
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let userspace = if text.contains("(userspace)") {
        text.split("(userspace)")
            .next()
            .and_then(|pre| pre.split_whitespace().last())
            .map(|s| s.trim().to_string())
    } else {
        None
    };
    let userspace = match userspace {
        Some(u) if !u.is_empty() => u,
        _ => {
            return CheckResult {
                id: "boot_time".into(),
                name: "Boot Time".into(),
                status: Status::Warn,
                message: "Could not parse userspace boot time".into(),
                fix: None,
            }
        }
    };
    let slow = userspace.contains("min") || userspace.contains('h');
    let secs = userspace.trim_end_matches('s').parse::<f64>().ok();
    let over = slow || secs.map(|s| s > 15.0).unwrap_or(false);
    if over {
        CheckResult {
            id: "boot_time".into(),
            name: "Boot Time".into(),
            status: Status::Warn,
            message: format!("Userspace startup {} (over 15s target)", userspace),
            fix: Some("systemd-analyze blame".into()),
        }
    } else {
        CheckResult {
            id: "boot_time".into(),
            name: "Boot Time".into(),
            status: Status::Pass,
            message: format!("Login ready in {} (userspace)", userspace),
            fix: None,
        }
    }
}

pub fn check_generation_drift() -> CheckResult {
    let current = std::fs::read_link("/run/current-system").ok();
    let booted = std::fs::read_link("/run/booted-system").ok();
    match (current, booted) {
        (Some(c), Some(b)) if c == b => CheckResult {
            id: "generation_drift".into(),
            name: "Generation Drift".into(),
            status: Status::Pass,
            message: "Booted generation is current".into(),
            fix: None,
        },
        (Some(_), Some(_)) => CheckResult {
            id: "generation_drift".into(),
            name: "Generation Drift".into(),
            status: Status::Warn,
            message: "Rebuilt since boot -- reboot to apply (kernel/initrd changes need it)".into(),
            fix: Some("Reboot to activate the current generation".into()),
        },
        _ => CheckResult {
            id: "generation_drift".into(),
            name: "Generation Drift".into(),
            status: Status::Warn,
            message: "Could not read current/booted system links".into(),
            fix: None,
        },
    }
}


pub fn check_generation_count() -> CheckResult {
    // Warn only when a GC could actually prune something: generations whose link
    // mtime is older than 14d (approximates --delete-older-than 14d). Total shown
    // for transparency; the warn is the actionable part.
    use std::time::{Duration, SystemTime};
    const PRUNE_AGE: Duration = Duration::from_secs(14 * 24 * 3600);
    let now = SystemTime::now();
    let mut total = 0usize;
    let mut old = 0usize;
    if let Ok(entries) = std::fs::read_dir("/nix/var/nix/profiles") {
        for e in entries.filter_map(|e| e.ok()) {
            let n = e.file_name();
            let n = n.to_string_lossy();
            if n.starts_with("system-") && n.ends_with("-link") {
                total += 1;
                if let Ok(meta) = std::fs::symlink_metadata(e.path()) {
                    if let Ok(mtime) = meta.modified() {
                        if now.duration_since(mtime).map(|a| a > PRUNE_AGE).unwrap_or(false) {
                            old += 1;
                        }
                    }
                }
            }
        }
    }
    if old > 0 {
        CheckResult {
            id: "generation_count".into(),
            name: "Generation Count".into(),
            status: Status::Warn,
            message: format!("{} generations ({} older than 14d, prunable)", total, old),
            fix: Some("sudo nix-collect-garbage --delete-older-than 14d".into()),
        }
    } else {
        CheckResult {
            id: "generation_count".into(),
            name: "Generation Count".into(),
            status: Status::Pass,
            message: format!("{} generations, none older than 14d", total),
            fix: None,
        }
    }
}


pub fn check_flake_lock_age(core_root: &str) -> CheckResult {
    use std::time::SystemTime;
    let path = std::path::Path::new(core_root).join("flake.lock");
    let age_days = std::fs::metadata(&path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|mtime| SystemTime::now().duration_since(mtime).ok())
        .map(|d| d.as_secs() / 86400);
    match age_days {
        Some(days) if days > 30 => CheckResult {
            id: "flake_lock_age".into(),
            name: "Flake Lock Age".into(),
            status: Status::Warn,
            message: format!("flake.lock is {} days old -- deps may be stale", days),
            fix: Some("nix flake update".into()),
        },
        Some(days) => CheckResult {
            id: "flake_lock_age".into(),
            name: "Flake Lock Age".into(),
            status: Status::Pass,
            message: format!("flake.lock updated {} days ago", days),
            fix: None,
        },
        None => CheckResult {
            id: "flake_lock_age".into(),
            name: "Flake Lock Age".into(),
            status: Status::Warn,
            message: "Could not read flake.lock mtime".into(),
            fix: None,
        },
    }
}


pub fn check_update_readiness(core_root: &str) -> CheckResult {
    // Synthesis: is the system in a safe STATE to run an update?
    // booted == current (no pending reboot) AND no uncommitted tracked changes.
    // Folds the drift + git signals into one go/no-go pre-update verdict.
    let mut blockers: Vec<&str> = Vec::new();
    let current = std::fs::read_link("/run/current-system").ok();
    let booted = std::fs::read_link("/run/booted-system").ok();
    if let (Some(c), Some(b)) = (current, booted) {
        if c != b {
            blockers.push("reboot to clear generation drift");
        }
    }
    let dirty = Command::new("git")
        .arg("-C")
        .arg(core_root)
        .args(["diff", "--quiet", "HEAD"])
        .status()
        .map(|s| !s.success())
        .unwrap_or(false);
    if dirty {
        blockers.push("commit or stash tracked changes");
    }
    if blockers.is_empty() {
        CheckResult {
            id: "update_readiness".into(),
            name: "Update Readiness".into(),
            status: Status::Pass,
            message: "Safe to update -- booted current, tree clean".into(),
            fix: None,
        }
    } else {
        CheckResult {
            id: "update_readiness".into(),
            name: "Update Readiness".into(),
            status: Status::Warn,
            message: format!("Hold off -- {}", blockers.join("; ")),
            fix: Some("Resolve the above, then: nix flake update && rebuild".into()),
        }
    }
}
