#![allow(dead_code)]
pub mod aliases;
pub mod bins;
pub mod entropy;
use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Status {
    Pass,
    Warn,
    Fail,
    Blocked,
}

#[derive(Debug)]
struct CheckResult {
    id: String,
    name: String,
    status: Status,
    message: String,
    fix: Option<String>,
}

fn check_stow(core_root: &str, home: &str) -> CheckResult {
    let stow_dir = PathBuf::from(core_root).join("03-interfaces/stow");
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
                                .contains(&format!("0-core/03-interfaces/stow/{}", name));
                        }
                    }
                    false
                });
            if found {
                stowed += 1;
            }
        }
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
            fix: Some("Run: cd ~/0-core && stow --dir=stow -R <package>".into()),
        }
    }
}

fn check_services() -> CheckResult {
    let services = [
        ("faelight-bar", "Status bar"),
        ("faelight-notify", "Notifications"),
    ];
    let running = services
        .iter()
        .filter(|(name, _)| {
            Command::new("pgrep")
                .arg("-x")
                .arg(name)
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
            fix: Some("Run: faelight-bar & faelight-notify &".into()),
        }
    }
}

fn check_broken_symlinks(_core_root: &str, home: &str) -> CheckResult {
    let mut broken = 0;
    let config = PathBuf::from(home).join(".config");
    for entry in WalkDir::new(&config)
        .max_depth(6)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.is_symlink() && !p.exists() && !p.to_string_lossy().contains("BraveSoftware") && !p.to_string_lossy().contains("Notesnook") && !p.to_string_lossy().contains("Singleton") {
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

fn check_yazi_plugins(home: &str) -> CheckResult {
    let plugin_dir = PathBuf::from(home).join(".config/yazi/plugins");
    let plugins = [
        "full-border.yazi",
        "git.yazi",
        "jump-to-char.yazi",
        "smart-enter.yazi",
    ];
    let count = plugins
        .iter()
        .filter(|p| plugin_dir.join(p).is_dir())
        .count();
    if count == 4 {
        CheckResult {
            id: "yazi_plugins".into(),
            name: "Yazi Plugins".into(),
            status: Status::Pass,
            message: "All 4 plugins installed".into(),
            fix: None,
        }
    } else {
        CheckResult {
            id: "yazi_plugins".into(),
            name: "Yazi Plugins".into(),
            status: Status::Warn,
            message: format!("Only {}/4 plugins installed", count),
            fix: Some("Install missing plugins via ya pack".into()),
        }
    }
}

fn check_binaries() -> CheckResult {
    let bins = [
        "sway",
        "foot",
        "faelight-palette",
        "yazi",
        "nvim",
        "git",
        "stow",
        "starship",
        "bat",
        "eza",
        "fd",
        "rg",
        "zoxide",
        "brightnessctl",
        "wpctl",
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

fn check_git(core_root: &str) -> CheckResult {
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

fn check_themes(core_root: &str) -> CheckResult {
    let stow = PathBuf::from(core_root).join("03-interfaces/stow");
    let count = fs::read_dir(&stow)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| {
                    let n = e.file_name().to_string_lossy().to_string();
                    n.starts_with("config-faelight") || n.starts_with("theme-")
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

fn check_scripts(core_root: &str) -> CheckResult {
    let scripts_dir = PathBuf::from(core_root).join("scripts");
    let required = ["dot-doctor", "dotctl", "faelight", "profile", "intent"];
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

fn check_dotmeta() -> CheckResult {
    CheckResult {
        id: "dotmeta".into(),
        name: "Package Metadata".into(),
        status: Status::Pass,
        message: ".dotmeta files intentionally removed (stow conflict resolution)".into(),
        fix: None,
    }
}

fn check_intents(core_root: &str) -> CheckResult {
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

fn check_profiles(core_root: &str, home: &str) -> CheckResult {
    let profile_script = PathBuf::from(core_root).join("scripts/profile");
    let state_dir = PathBuf::from(home).join(".local/state/0-core");
    let current = fs::read_to_string(state_dir.join("current-profile"))
        .unwrap_or_else(|_| "default".into())
        .trim()
        .to_string();
    if !profile_script.exists() {
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

fn check_faelight_config(home: &str) -> CheckResult {
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

fn check_keybinds(core_root: &str, home: &str) -> CheckResult {
    // Niri is primary compositor — check Niri config first, fall back to Sway
    let niri_config = PathBuf::from(home).join(".config/niri/config.kdl");
    let sway_config = PathBuf::from(home).join(".config/sway/config");

    let (wm_name, wm_config) = if niri_config.exists() {
        ("Niri", niri_config)
    } else if sway_config.exists() {
        ("Sway", sway_config)
    } else {
        return CheckResult {
            id: "keybinds".into(),
            name: "WM Keybinds".into(),
            status: Status::Warn,
            message: "No compositor config found (niri or sway)".into(),
            fix: Some("Ensure wm-niri or wm-sway is stowed".into()),
        };
    };

    let keyscan = PathBuf::from(core_root).join("scripts/keyscan");
    let output = Command::new(&keyscan)
        .arg(wm_config.to_string_lossy().to_string())
        .output();
    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            if stdout.contains("No conflicts detected") {
                let count = stdout
                    .lines()
                    .find(|l| l.contains("unique keybindings"))
                    .and_then(|l| l.split_whitespace().next())
                    .unwrap_or("0");
                CheckResult {
                    id: "keybinds".into(),
                    name: format!("{} Keybinds", wm_name),
                    status: Status::Pass,
                    message: format!("{} unique keybindings, no conflicts", count),
                    fix: None,
                }
            } else {
                CheckResult {
                    id: "keybinds".into(),
                    name: format!("{} Keybinds", wm_name),
                    status: Status::Fail,
                    message: "Keybind conflicts detected".into(),
                    fix: Some(format!("Run: keyscan ~/.config/{}/config", wm_name.to_lowercase())),
                }
            }
        }
        _ => CheckResult {
            id: "keybinds".into(),
            name: format!("{} Keybinds", wm_name),
            status: Status::Warn,
            message: "keyscan not available".into(),
            fix: Some("Ensure keyscan is in ~/0-core/scripts/".into()),
        },
    }
}

fn check_security_hardening() -> CheckResult {
    let mut details = 0;
    let ufw_active = fs::read_to_string("/etc/ufw/ufw.conf")
        .map(|c| c.contains("ENABLED=yes"))
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
    if ufw_active { active.push("UFW ✅"); }
    if f2b_active { active.push("fail2ban ✅"); }
    if ssh_ok { active.push("SSH hardened ✅"); }
    if !ufw_active { active.push("UFW ❌"); }

    CheckResult {
        id: "security".into(),
        name: "Security Hardening".into(),
        status: if details > 0 {
            Status::Pass
        } else {
            Status::Fail
        },
        message: format!("Security: {}", active.join("  ")),
        fix: if !ufw_active { Some("Enable UFW: sudo ufw enable".into()) } else { None },
    }
}

fn check_security_audit(home: &str) -> CheckResult {
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

    let message = if patchable_count == 0 {
        format!(
            "{} findings — all upstream pending, none patchable (scan: {})",
            findings, timestamp
        )
    } else {
        format!(
            "{} findings ({} patchable): {} critical, {} high, {} medium (scan: {})",
            findings, patchable_count, critical, high, medium, timestamp
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

fn check_alias_coverage() -> CheckResult {
    // alias-audit --doctor is complex enough to keep delegating
    let scripts = format!(
        "{}/scripts/alias-audit",
        std::env::var("HOME").unwrap_or_default() + "/0-core"
    );
    let output = Command::new(&scripts).arg("--doctor").output();
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("✅") && stdout.contains("Alias Coverage") {
            CheckResult {
                id: "alias_coverage".into(),
                name: "Alias Coverage".into(),
                status: Status::Pass,
                message: stdout
                    .lines()
                    .next()
                    .unwrap_or("All tools have aliases")
                    .trim()
                    .to_string(),
                fix: None,
            }
        } else {
            CheckResult {
                id: "alias_coverage".into(),
                name: "Alias Coverage".into(),
                status: Status::Warn,
                message: "alias-audit returned unexpected output".into(),
                fix: Some("Run: alias-audit".into()),
            }
        }
    } else {
        CheckResult {
            id: "alias_coverage".into(),
            name: "Alias Coverage".into(),
            status: Status::Fail,
            message: "alias-audit not found".into(),
            fix: Some("Rebuild: cargo build --release -p alias-audit".into()),
        }
    }
}

fn check_rust_toolchain() -> CheckResult {
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

fn check_disk_space() -> CheckResult {
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

fn check_tool_installation() -> CheckResult {
    let tools = [
        "faelight-git",
        "intent",
        "dot-doctor",
        "faelight-hooks",
        "faelight-update",
        "alias-audit",
        "core-diff",
    ];
    let missing: Vec<_> = tools
        .iter()
        .filter(|t| {
            !Command::new("which")
                .arg(t)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
        .map(|t| t.to_string())
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
                "{}/{} key tools installed",
                tools.len() - missing.len(),
                tools.len()
            ),
            fix: Some("Install missing tools with cargo install".into()),
        }
    }
}

fn check_path_resilience(core_root: &str) -> CheckResult {
    let rust_tools_dir = PathBuf::from(core_root).join("rust-tools");
    let scripts_dir = PathBuf::from(core_root).join("scripts");
    let skip = [
        "faelight-core",
        "faelight-daemon",
        "faelight-browser",
        "bin-doctor",
        "faelight-menu",
        "verify-bootstrap",
        "archaeology-0-core",
        "bump-system-version.archived",
        "faelight-dashboard.archived",
    ];
    let rust_tools: Vec<String> = fs::read_dir(&rust_tools_dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|n| !skip.contains(&n.as_str()))
                .collect()
        })
        .unwrap_or_default();
    let total = rust_tools.len();
    let deployed = rust_tools
        .iter()
        .filter(|n| scripts_dir.join(n).exists())
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

fn check_archaeology(core_root: &str) -> CheckResult {
    let script = std::path::PathBuf::from(core_root).join("scripts/archaeology-0-core");

    if !script.exists() {
        return CheckResult {
            id: "archaeology".into(),
            name: "Archaeology".into(),
            status: Status::Warn,
            message: "archaeology-0-core not deployed — run: cargo build -p archaeology-0-core"
                .into(),
            fix: Some("cargo build --release -p archaeology-0-core".into()),
        };
    }

    // Run stats to verify it can read git history
    let output = std::process::Command::new(&script).arg("stats").output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Extract commit count
            let commits = stdout
                .lines()
                .find(|l| l.contains("Commits:"))
                .and_then(|l| l.split_whitespace().last())
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            if commits > 0 {
                CheckResult {
                    id: "archaeology".into(),
                    name: "Archaeology".into(),
                    status: Status::Pass,
                    message: format!("History accessible: {} commits indexed", commits),
                    fix: None,
                }
            } else {
                CheckResult {
                    id: "archaeology".into(),
                    name: "Archaeology".into(),
                    status: Status::Warn,
                    message: "archaeology-0-core running but no commits found".into(),
                    fix: Some("Check git repository integrity".into()),
                }
            }
        }
        _ => CheckResult {
            id: "archaeology".into(),
            name: "Archaeology".into(),
            status: Status::Warn,
            message: "archaeology-0-core failed to run".into(),
            fix: Some("Check binary: archaeology-0-core stats".into()),
        },
    }
}

fn check_core_protect(core_root: &str) -> CheckResult {
    let installed = Command::new("which")
        .arg("core-protect")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !installed {
        return CheckResult {
            id: "core_protect".into(),
            name: "Core Protection".into(),
            status: Status::Fail,
            message: "core-protect not installed".into(),
            fix: Some("Build: cargo build --release -p core-protect".into()),
        };
    }
    let output = Command::new("lsattr").arg("-d").arg(core_root).output();
    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let flags: String = stdout.chars().take(20).collect();
            if flags.contains('i') {
                CheckResult {
                    id: "core_protect".into(),
                    name: "Core Protection".into(),
                    status: Status::Pass,
                    message: "🔒 Core is LOCKED (immutable)".into(),
                    fix: None,
                }
            } else {
                CheckResult {
                    id: "core_protect".into(),
                    name: "Core Protection".into(),
                    status: Status::Warn,
                    message: "🔓 Core is UNLOCKED — remember to lock before shutdown".into(),
                    fix: Some("Run: core-protect lock".into()),
                }
            }
        }
        Err(_) => CheckResult {
            id: "core_protect".into(),
            name: "Core Protection".into(),
            status: Status::Warn,
            message: "Could not determine protection status".into(),
            fix: Some("Check: core-protect status".into()),
        },
    }
}

fn print_result(r: &CheckResult) {
    match r.status {
        Status::Pass => println!("✅ {}: {}", r.name, r.message),
        Status::Warn => println!("⚠️  {}: {}", r.name, r.message),
        Status::Fail => println!("❌ {}: {}", r.name, r.message),
        Status::Blocked => println!("⏭  {}: blocked", r.name),
    }
}

pub fn run(ctx: &AppContext, _preflight: bool) -> CoreResult<()> {
    ctx.capabilities.require(
        "doctor",
        &[
            Capability::OrchestratorAccess,
            Capability::FilesystemReadHome,
        ],
    )?;
    let home = std::env::var("HOME").unwrap_or_default();
    let core_root = ctx.core_root.clone();

    let version = fs::read_to_string(PathBuf::from(&core_root).join("00-meta/VERSION"))
        .unwrap_or_else(|_| "unknown".into())
        .trim()
        .to_string();

    println!(
        "{}",
        "🏥 0-Core Health Check - Faelight Forest {}"
            .replace("{}", &version)
            .bold()
    );
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let checks: Vec<CheckResult> = vec![
        check_stow(&core_root, &home),
        check_services(),
        check_broken_symlinks(&core_root, &home),
        check_yazi_plugins(&home),
        check_binaries(),
        check_git(&core_root),
        check_themes(&core_root),
        check_scripts(&core_root),
        check_dotmeta(),
        check_intents(&core_root),
        check_profiles(&core_root, &home),
        check_faelight_config(&home),
        check_keybinds(&core_root, &home),
        check_security_hardening(),
        check_security_audit(&home),
        check_alias_coverage(),
        check_rust_toolchain(),
        check_disk_space(),
        check_tool_installation(),
        check_path_resilience(&core_root),
        check_archaeology(&core_root),
        check_core_protect(&core_root),
    ];

    for r in &checks {
        print_result(r);
    }

    let total = checks.len() as u32;
    let passed = checks.iter().filter(|r| r.status == Status::Pass).count() as u32;
    let warnings = checks.iter().filter(|r| r.status == Status::Warn).count() as u32;
    let failed = checks.iter().filter(|r| r.status == Status::Fail).count() as u32;
    let health = if total > 0 { (passed * 100) / total } else { 0 };

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    if health >= 95 {
        println!(
            "{}",
            format!("✅ System healthy ({}%)", health)
                .bright_green()
                .bold()
        );
    } else if health >= 80 {
        println!(
            "{}",
            format!("⚠️  System mostly healthy ({}%)", health)
                .yellow()
                .bold()
        );
    } else {
        println!(
            "{}",
            format!("❌ System unhealthy ({}%)", health)
                .bright_red()
                .bold()
        );
    }
    println!("Statistics:");
    println!("   Passed:   {}", passed);
    println!("   Warnings: {}", warnings);
    println!("   Failed:   {}", failed);
    println!("   Total:    {}", total);
    println!("   Health:   {}%", health);

    // Core v5 Phase 2 — inline forecast after doctor run
    {
        let db = &ctx.runtime.db;
        let mut stmt = db.prepare(
            "SELECT payload, timestamp FROM events WHERE domain='doctor' ORDER BY id DESC LIMIT 10"
        );
        if let Ok(mut stmt) = stmt {
            let points_result = stmt.query_map([], |r| {
                let payload: Option<String> = r.get(0)?;
                let ts: i64 = r.get(1)?;
                let h = payload.as_deref()
                    .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
                    .and_then(|v| v["detail"]["health"].as_i64())
                    .unwrap_or(health as i64);
                Ok((h, ts))
            });
            let points: Vec<(i64, i64)> = match points_result {
                Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
                Err(_) => vec![],
            };
            let points = points;

            if points.len() >= 3 {
                // Compute trend slope
                let n = points.len() as f64;
                let sum_h: f64 = points.iter().map(|(h,_)| *h as f64).sum();
                let avg_h = sum_h / n;
                let recent_avg: f64 = points.iter().take(3).map(|(h,_)| *h as f64).sum::<f64>() / 3.0;
                let older_avg: f64 = points.iter().skip(3).map(|(h,_)| *h as f64).sum::<f64>() / (n - 3.0).max(1.0);
                let trend = recent_avg - older_avg;

                let forecast_24h = (health as f64 + trend * 0.5).round() as i64;
                let forecast_7d = (health as f64 + trend * 2.0).round() as i64;
                let forecast_24h = forecast_24h.max(0).min(100);
                let forecast_7d = forecast_7d.max(0).min(100);

                let trend_icon = if trend > 1.0 { "📈" } else if trend < -1.0 { "📉" } else { "➡️ " };
                let trend_str = if trend > 0.5 { format!("+{:.1}", trend) }
                    else if trend < -0.5 { format!("{:.1}", trend) }
                    else { "stable".to_string() };

                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("{}  Forecast  24h: {}%  7d: {}%  trend: {}",
                    trend_icon,
                    forecast_24h,
                    forecast_7d,
                    trend_str,
                );
            }
        }
    }
    // Write health score to cache for bar/prompt/palette
    let cache_dir =
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache/faelight");
    let _ = std::fs::create_dir_all(&cache_dir);
    let _ = std::fs::write(cache_dir.join("health-status"), format!("{}", health));

    // Event Ledger
    let writer = crate::runtime::EventWriter::new(&ctx.runtime.db);
    writer.write(
        "doctor",
        "run",
        "core doctor run",
        if failed == 0 { "ok" } else { "warn" },
        Some(&format!(
            r#"{{"health":{},"passed":{},"warnings":{},"failed":{}}}"#,
            health, passed, warnings, failed
        )),
    );

    Ok(())
}

pub fn aliases(_ctx: &AppContext, subcmd: Option<&str>) -> CoreResult<()> {
    aliases::run_full_audit(subcmd)
}

pub fn entropy(_ctx: &AppContext, baseline: bool, trends: bool, json: bool) -> CoreResult<()> {
    entropy::run(baseline, trends, json)
}

pub fn bins(_ctx: &AppContext, subcmd: Option<&str>) -> CoreResult<()> {
    bins::run(subcmd, false)
}

pub fn simulate(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require(
        "doctor",
        &[
            Capability::OrchestratorAccess,
            Capability::FilesystemReadHome,
        ],
    )?;

    let home = std::env::var("HOME").unwrap_or_default();
    let core_root = ctx.core_root.clone();

    // Read current cached health
    let cached: u32 =
        fs::read_to_string(PathBuf::from(&home).join(".cache/faelight/health-status"))
            .unwrap_or_default()
            .trim()
            .parse()
            .unwrap_or(0);

    // Run all checks silently
    let checks: Vec<CheckResult> = vec![
        check_stow(&core_root, &home),
        check_services(),
        check_broken_symlinks(&core_root, &home),
        check_yazi_plugins(&home),
        check_binaries(),
        check_git(&core_root),
        check_themes(&core_root),
        check_scripts(&core_root),
        check_dotmeta(),
        check_intents(&core_root),
        check_profiles(&core_root, &home),
        check_faelight_config(&home),
        check_keybinds(&core_root, &home),
        check_security_hardening(),
        check_security_audit(&home),
        check_alias_coverage(),
        check_rust_toolchain(),
        check_disk_space(),
        check_tool_installation(),
        check_path_resilience(&core_root),
        check_archaeology(&core_root),
        check_core_protect(&core_root),
    ];

    let total = checks.len() as u32;
    let passed = checks.iter().filter(|r| r.status == Status::Pass).count() as u32;
    let warnings = checks.iter().filter(|r| r.status == Status::Warn).count() as u32;
    let failed = checks.iter().filter(|r| r.status == Status::Fail).count() as u32;
    let predicted = if total > 0 { (passed * 100) / total } else { 0 };

    println!("{}", "🔮 core simulate doctor".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    // Show only changed or failing checks
    let mut changes = 0;
    for r in &checks {
        match r.status {
            Status::Fail => {
                println!("  {} {}", "✗".bright_red(), r.name.bright_white());
                println!("    {} {}", "→".dimmed(), r.message.bright_red());
                if let Some(ref fix) = r.fix {
                    println!("    {} {}", "fix:".yellow(), fix.dimmed());
                }
                changes += 1;
            }
            Status::Warn => {
                println!("  {} {}", "⚠".yellow(), r.name.bright_white());
                println!("    {} {}", "→".dimmed(), r.message.yellow());
                changes += 1;
            }
            _ => {}
        }
    }

    if changes == 0 {
        println!("  {}", "All checks passing — no issues found.".green());
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());

    // Health diff
    let delta = predicted as i32 - cached as i32;
    let delta_str = if delta > 0 {
        format!("+{}%", delta).green().to_string()
    } else if delta < 0 {
        format!("{}%", delta).bright_red().to_string()
    } else {
        "no change".dimmed().to_string()
    };

    let predicted_colored = if predicted >= 95 {
        format!("{}%", predicted).green().to_string()
    } else if predicted >= 80 {
        format!("{}%", predicted).yellow().to_string()
    } else {
        format!("{}%", predicted).bright_red().to_string()
    };

    println!();
    println!("  current health   {}%", cached.to_string().dimmed());
    println!("  predicted health {}  ({})", predicted_colored, delta_str);
    println!(
        "  checks           {}  passed  {}  warnings  {}  failed",
        passed.to_string().green(),
        warnings.to_string().yellow(),
        if failed > 0 {
            failed.to_string().bright_red()
        } else {
            failed.to_string().dimmed()
        }
    );
    println!();
    println!("  {} No changes made to system.", "ℹ".cyan());

    Ok(())
}

// ── Phase 6: Health Forecasting ───────────────────────────────────────────────

pub fn trend(ctx: &AppContext) -> CoreResult<()> {
    let conn = &ctx.runtime.db;

    let mut stmt = conn
        .prepare(
            "SELECT payload, timestamp FROM events WHERE domain='doctor' ORDER BY timestamp ASC",
        )
        .map_err(crate::errors::CoreError::Database)?;

    struct HealthPoint {
        health: i64,
        ts: i64,
    }

    let points: Vec<HealthPoint> = stmt
        .query_map([], |row| {
            let payload: String = row.get(0)?;
            let ts: i64 = row.get(1)?;
            Ok((payload, ts))
        })
        .map_err(crate::errors::CoreError::Database)?
        .filter_map(|r| r.ok())
        .filter_map(|(p, ts)| {
            let v: serde_json::Value = serde_json::from_str(&p).ok()?;
            let health = v["detail"]["health"].as_i64()?;
            Some(HealthPoint { health, ts })
        })
        .collect();

    if points.is_empty() {
        println!("  {} No health history found", "○".dimmed());
        return Ok(());
    }

    println!("{}", "🔬 Health Trend Analysis".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    // Summary stats
    let avg = points.iter().map(|p| p.health).sum::<i64>() / points.len() as i64;
    let min = points.iter().map(|p| p.health).min().unwrap_or(0);
    let max = points.iter().map(|p| p.health).max().unwrap_or(0);
    let current = points.last().map(|p| p.health).unwrap_or(0);
    let first = points.first().map(|p| p.health).unwrap_or(0);
    let drift = current - first;

    println!(
        "  {} {} readings over {} sessions",
        "📊".cyan(),
        points.len().to_string().bright_white(),
        points.len().to_string().dimmed(),
    );
    println!(
        "  {} Current:  {}%",
        "▶".dimmed(),
        current.to_string().bright_white()
    );
    println!(
        "  {} Average:  {}%",
        "▶".dimmed(),
        avg.to_string().bright_white()
    );
    println!("  {} Range:    {}% – {}%", "▶".dimmed(), min, max);

    let drift_str = if drift > 0 {
        format!("+{}% since first run", drift).green().to_string()
    } else if drift < 0 {
        format!("{}% since first run", drift).yellow().to_string()
    } else {
        "stable since first run".dimmed().to_string()
    };
    println!("  {} Drift:    {}", "▶".dimmed(), drift_str);

    // Sparkline — last 20 readings
    println!();
    println!("  {} Recent history (oldest → newest):", "📈".cyan());
    print!("    ");
    let recent: Vec<_> = points.iter().rev().take(20).rev().collect();
    for p in &recent {
        let ch = match p.health {
            h if h >= 95 => "█".bright_green(),
            h if h >= 85 => "▇".green(),
            h if h >= 75 => "▅".yellow(),
            _ => "▂".red(),
        };
        print!("{}", ch);
    }
    println!();
    println!(
        "    {}  {}  {}  {}",
        "█ 95%+".bright_green(),
        "▇ 85%+".green(),
        "▅ 75%+".yellow(),
        "▂ <75%".red(),
    );

    // Pattern detection
    println!();
    println!("  {} Pattern:", "🔍".cyan());
    let dips: usize = points
        .windows(2)
        .filter(|w| w[1].health < w[0].health)
        .count();
    let recoveries: usize = points
        .windows(2)
        .filter(|w| w[1].health > w[0].health)
        .count();

    if dips == 0 {
        println!("    ✅ No health dips detected — stable system");
    } else {
        println!(
            "    ⚠️  {} dip(s) detected, {} recovery(s)",
            dips, recoveries
        );
    }

    let at_95 = points.iter().filter(|p| p.health >= 95).count();
    let pct_healthy = (at_95 * 100) / points.len();
    println!("    📊 {}% of readings at 95%+ health", pct_healthy);

    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

pub fn forecast(ctx: &AppContext) -> CoreResult<()> {
    let conn = &ctx.runtime.db;

    let mut stmt = conn.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='doctor' ORDER BY timestamp DESC LIMIT 10"
    ).map_err(crate::errors::CoreError::Database)?;

    struct HealthPoint {
        health: i64,
        warnings: i64,
    }

    let points: Vec<HealthPoint> = stmt
        .query_map([], |row| {
            let payload: String = row.get(0)?;
            Ok(payload)
        })
        .map_err(crate::errors::CoreError::Database)?
        .filter_map(|r| r.ok())
        .filter_map(|p| {
            let v: serde_json::Value = serde_json::from_str(&p).ok()?;
            let health = v["detail"]["health"].as_i64()?;
            let warnings = v["detail"]["warnings"].as_i64().unwrap_or(0);
            Some(HealthPoint { health, warnings })
        })
        .collect();

    if points.is_empty() {
        println!("  {} No health history to forecast from", "○".dimmed());
        return Ok(());
    }

    println!("{}", "🔮 Health Forecast".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    let current = points.first().map(|p| p.health).unwrap_or(0);
    let avg_warnings = points.iter().map(|p| p.warnings).sum::<i64>() / points.len() as i64;

    // Simple linear trend from last 5 readings
    let recent: Vec<i64> = points.iter().take(5).map(|p| p.health).rev().collect();
    let trend_delta: i64 = if recent.len() >= 2 {
        let first = recent[0];
        let last = *recent.last().unwrap();
        (last - first) / (recent.len() as i64 - 1)
    } else {
        0
    };

    let predicted = (current + trend_delta).clamp(0, 100);

    println!(
        "  {} Current health:   {}%",
        "▶".dimmed(),
        current.to_string().bright_white()
    );
    println!(
        "  {} Recent trend:     {} per run",
        "▶".dimmed(),
        if trend_delta >= 0 {
            format!("+{}", trend_delta).green().to_string()
        } else {
            format!("{}", trend_delta).yellow().to_string()
        }
    );
    println!(
        "  {} Avg warnings:     {} per run",
        "▶".dimmed(),
        avg_warnings
    );
    println!();

    // Prediction
    let pred_str = match predicted {
        p if p >= 95 => format!("{}% ✅ Excellent", p).bright_green().to_string(),
        p if p >= 85 => format!("{}% ✓ Good", p).green().to_string(),
        p if p >= 75 => format!("{}% ⚠️  Watch", p).yellow().to_string(),
        p => format!("{}% ❌ Needs attention", p).red().to_string(),
    };
    println!("  {} Next run forecast: {}", "🔮".cyan(), pred_str);

    // Risk factors
    println!();
    println!("  {} Risk factors:", "⚠️".yellow());
    let mut risks = 0;

    if avg_warnings >= 2 {
        println!("    • Persistent warnings — {} avg per run", avg_warnings);
        risks += 1;
    }
    if trend_delta < 0 {
        println!(
            "    • Declining trend — health dropping {} per run",
            trend_delta
        );
        risks += 1;
    }
    if current < 95 {
        println!("    • Below optimal — currently {}% (target 95%+)", current);
        risks += 1;
    }

    if risks == 0 {
        println!("    ✅ No risk factors detected");
    }

    // Recommendations
    println!();
    println!("  {} Recommendations:", "💡".cyan());
    if current < 95 {
        println!("    • Run 'csd' to simulate what's causing the warning");
        println!("    • Run 'cwh' to see health trajectory");
    } else {
        println!("    • System on track — maintain current practices");
    }

    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}
