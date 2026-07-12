// doctor/checks.rs — all 23 health check functions
#![allow(dead_code)]
use super::{CheckResult, Status};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

pub fn check_stow(_core_root: &str, _home: &str) -> CheckResult {
    // Dotfiles are owned by home-manager on NixOS (stow subsystem decommissioned, INT-107).
    CheckResult {
        id: "stow".into(),
        name: "Dotfile Symlinks".into(),
        status: Status::Pass,
        message: "Managed by home-manager (NixOS)".into(),
        fix: None,
    }
}

pub fn check_services() -> CheckResult {
    // Per-session daemons the doctor expects up. faelight-bar joined this list
    // when INT-053 shipped it as a real systemd user service.
    let services = [
        ("faelight-notify", "Notifications"),
        ("faelight-bar", "Bar"),
        ("faelight-wsd", "Workspaces"),
    ];
    let down: Vec<&str> = services
        .iter()
        .filter(|(name, _)| {
            // is-active by unit name survives binary swaps (INT-053 changed
            // faelight-bar's ExecStart to the faelight-bar-gtk binary).
            Command::new("systemctl")
                .args(["--user", "is-active", "--quiet", *name])
                .status()
                .map(|s| !s.success())
                .unwrap_or(true)
        })
        .map(|(name, _)| *name)
        .collect();
    let total = services.len();
    let running = total - down.len();
    if running == total {
        CheckResult {
            id: "services".into(),
            name: "System Services".into(),
            status: Status::Pass,
            message: format!("{}/{} services running", running, total),
            fix: None,
        }
    } else {
        CheckResult {
            id: "services".into(),
            name: "System Services".into(),
            status: Status::Warn,
            message: format!("{}/{} running -- down: {}", running, total, down.join(", ")),
            fix: Some(format!("Start: {}", down.join(", "))),
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

pub fn check_binaries() -> CheckResult {
    let bins = [
        "mango",
        "alacritty",
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
            fix: Some(
                "Add the package to configuration.nix, or: nix profile install nixpkgs#<package>"
                    .into(),
            ),
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
    // INT-061: config/ moved to nix/home/dotfiles/
    let stow = PathBuf::from(core_root).join("nix/home/dotfiles");
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

pub fn check_intents(_core_root: &str) -> CheckResult {
    // INT-135 Gate 7: was decoration -- hardcoded Status::Pass, a phantom "active/" folder,
    // no "in-progress", and a substring match for "status: complete" over whole files.
    // Now calls the ONE validator. Doctor and `core intent validate` cannot disagree.
    let (count, issues) = crate::domains::intent::validate_issues();

    if issues.is_empty() {
        CheckResult {
            id: "intents".into(),
            name: "Intent Ledger".into(),
            status: Status::Pass,
            message: format!("{} intents, all valid", count),
            fix: None,
        }
    } else {
        CheckResult {
            id: "intents".into(),
            name: "Intent Ledger".into(),
            status: Status::Warn,
            message: format!("{} issue(s) -- first: {}", issues.len(), issues[0]),
            fix: Some("core intent validate".into()),
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
    let mango_config = PathBuf::from(home).join(".config/mango/config.conf");
    let (wm_name, wm_config, is_mango) = if mango_config.exists() {
        ("mango", mango_config, true)
    } else {
        return CheckResult {
            id: "keybinds".into(),
            name: "Compositor Keybinds".into(),
            status: Status::Warn,
            message: "No compositor keybind config found".into(),
            fix: Some("Deploy a compositor config (mango)".into()),
        };
    };
    let config_content = match std::fs::read_to_string(&wm_config) {
        Ok(c) => c,
        Err(_) => {
            return CheckResult {
                id: "keybinds".into(),
                name: "Compositor Keybinds".into(),
                status: Status::Warn,
                message: "Could not read keybind config".into(),
                fix: None,
            }
        }
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
                t.starts_with("Mod+")
                    || t.starts_with("Ctrl+")
                    || t.starts_with("Shift+")
                    || t.starts_with("Alt+")
                    || t.starts_with("Super+")
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
            message: format!(
                "{}: {} keybind conflicts detected -- review {} config",
                wm_name, conflicts, wm_name
            ),
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
            Some("Run: update  (or: sudo nixos-rebuild switch)".into())
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
            message: format!(
                "All {} tools have aliases ({} total)",
                EXPECTED_TOOLS.len(),
                aliases.len()
            ),
            fix: None,
        }
    } else {
        CheckResult {
            id: "alias_coverage".into(),
            name: "Alias Coverage".into(),
            status: Status::Warn,
            message: format!(
                "{} tools missing aliases: {}",
                missing.len(),
                missing.join(", ")
            ),
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
    let registry_path = faelight_core::paths::tools_registry();
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
    let registry_path = faelight_core::paths::tools_registry();

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
                Command::new("which")
                    .arg(n)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
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
    let binary_exists = std::path::Path::new("/run/current-system/sw/bin/faelight-sandbox")
        .exists()
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
    let policies_path = faelight_core::paths::registry_dir().join("sandbox-policies.toml");

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
                Some(n) => format!(
                    "No critical kernel errors since boot ({} low-priority notices)",
                    n
                ),
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

pub fn check_vm_state() -> CheckResult {
    // A running VM is NOT a fault -- VM-first development (compositor work in
    // INT-005/006/024/038/052/056) means VMs are often up by design. We report
    // the count so a forgotten VM gets noticed, and let the human judge intent.
    let count = match Command::new("pgrep")
        .arg("-f")
        .arg("-c")
        .arg("qemu-system")
        .output()
    {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .trim()
            .parse::<u32>()
            .unwrap_or(0),
        Err(_) => {
            return CheckResult {
                id: "vm_state".into(),
                name: "VM State".into(),
                status: Status::Warn,
                message: "Could not check for running VMs".into(),
                fix: Some("Verify pgrep is available".into()),
            };
        }
    };
    let message = if count == 0 {
        "No VMs running".to_string()
    } else {
        format!("{} QEMU VM(s) running", count)
    };
    CheckResult {
        id: "vm_state".into(),
        name: "VM State".into(),
        status: Status::Pass,
        message,
        fix: None,
    }
}

pub fn check_compositor() -> CheckResult {
    // Identify the running compositor (mango/pinnacle) by process.
    // "none" is not a fault -- d can run from a TTY or headless session --
    // so we report it as info rather than crying wolf, same as VM State.
    let candidates = [("mango", "MangoWM"), ("pinnacle", "Pinnacle")];
    for (proc_name, label) in candidates {
        let found = Command::new("pgrep")
            .arg("-x")
            .arg(proc_name)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if found {
            return CheckResult {
                id: "compositor".into(),
                name: "Compositor".into(),
                status: Status::Pass,
                message: format!("{} running", label),
                fix: None,
            };
        }
    }
    CheckResult {
        id: "compositor".into(),
        name: "Compositor".into(),
        status: Status::Pass,
        message: "No compositor detected (TTY or headless)".into(),
        fix: None,
    }
}

pub fn check_nix_store() -> CheckResult {
    // Store size via the Nix path DB: SUM(narSize) over ValidPaths --
    // milliseconds, no filesystem walk. Read-only; the DB is root-owned 0644.
    // narSize is the logical NAR size (a hair above true on-disk due to dedup),
    // so it errs high -- the safe direction for a "getting large" signal.
    let bytes: i64 = match rusqlite::Connection::open_with_flags(
        "/nix/var/nix/db/db.sqlite",
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .and_then(|db| {
        db.query_row("SELECT SUM(narSize) FROM ValidPaths", [], |r| {
            r.get::<_, i64>(0)
        })
    }) {
        Ok(b) => b,
        Err(_) => {
            return CheckResult {
                id: "nix_store".into(),
                name: "Nix Store".into(),
                status: Status::Warn,
                message: "Could not read Nix store DB".into(),
                fix: Some("Verify /nix/var/nix/db/db.sqlite is readable".into()),
            };
        }
    };

    let gib = bytes as f64 / 1_073_741_824.0;

    let mut disk_total: u64 = 0;
    if let Ok(p) = std::ffi::CString::new("/nix") {
        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
        if unsafe { libc::statvfs(p.as_ptr(), &mut stat) } == 0 {
            disk_total = stat.f_blocks as u64 * stat.f_frsize as u64;
        }
    }

    let message = if disk_total > 0 {
        let pct = bytes as f64 / disk_total as f64 * 100.0;
        let tib = disk_total as f64 / 1_099_511_627_776.0;
        format!("{:.1} GiB ({:.1}% of {:.1} TiB)", gib, pct, tib)
    } else {
        format!("{:.1} GiB", gib)
    };

    if gib > 250.0 {
        CheckResult {
            id: "nix_store".into(),
            name: "Nix Store".into(),
            status: Status::Warn,
            message: format!("{} -- consider nix-collect-garbage", message),
            fix: Some("Run nix-collect-garbage -d to reclaim space".into()),
        }
    } else {
        CheckResult {
            id: "nix_store".into(),
            name: "Nix Store".into(),
            status: Status::Pass,
            message,
            fix: None,
        }
    }
}

pub fn check_friday(_core_root: &str) -> CheckResult {
    // Friday learning vital signs, read from the same state.db the footer uses
    // (friday_patterns / friday_knowledge). PASS while learning; WARN only on a
    // genuine stall -- too few patterns, or no new fact in a week. Confidence is
    // shown for the trend but does NOT trigger a warn: low confidence is honest
    // uncertainty, not ill health.
    let db_path = faelight_core::paths::state_db();
    let db = match rusqlite::Connection::open(&db_path) {
        Ok(d) => d,
        Err(_) => {
            return CheckResult {
                id: "friday".into(),
                name: "Friday".into(),
                status: Status::Warn,
                message: "Could not open state.db".into(),
                fix: Some("Verify 0-core/runtime/state.db exists".into()),
            };
        }
    };
    let patterns: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0))
        .unwrap_or(0);
    let facts: i64 = db
        .query_row("SELECT COUNT(*) FROM friday_knowledge", [], |r| r.get(0))
        .unwrap_or(0);
    let avg_conf: f64 = db
        .query_row(
            "SELECT COALESCE(AVG(confidence), 0.0) FROM friday_patterns",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0.0);
    let last_learned: i64 = db
        .query_row(
            "SELECT COALESCE(MAX(updated_at), 0) FROM friday_knowledge",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let now = chrono::Utc::now().timestamp();
    let stale = last_learned > 0 && (now - last_learned) > 604_800;

    if patterns < 10 {
        CheckResult {
            id: "friday".into(),
            name: "Friday".into(),
            status: Status::Warn,
            message: format!("Warming up -- {} patterns, {} facts", patterns, facts),
            fix: Some("Friday learns from daily use; patterns build over time".into()),
        }
    } else if stale {
        let days = (now - last_learned) / 86_400;
        CheckResult {
            id: "friday".into(),
            name: "Friday".into(),
            status: Status::Warn,
            message: format!(
                "Learning stalled -- no new facts in {} days ({} patterns, {} facts)",
                days, patterns, facts
            ),
            fix: Some("Check that Friday is recording from sessions".into()),
        }
    } else {
        CheckResult {
            id: "friday".into(),
            name: "Friday".into(),
            status: Status::Pass,
            message: format!(
                "{} patterns · {} facts · {:.2} avg confidence",
                patterns, facts, avg_conf
            ),
            fix: None,
        }
    }
}

pub fn check_network() -> CheckResult {
    // Bounded so a down network cannot blow the 2s budget: TCP connect to
    // 1.1.1.1:443 (an IP, no DNS) with a 1s cap, then a DNS resolve of
    // github.com in a worker thread capped at 1s. Offline / DNS-down are WARNs:
    // a connected workstation losing the network is worth knowing.
    let addr: std::net::SocketAddr = "1.1.1.1:443".parse().unwrap();
    let online =
        std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(1)).is_ok();
    if !online {
        return CheckResult {
            id: "network".into(),
            name: "Network".into(),
            status: Status::Warn,
            message: "Offline -- 1.1.1.1 unreachable".into(),
            fix: Some("Check network connection".into()),
        };
    }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::net::ToSocketAddrs;
        let ok = "github.com:443"
            .to_socket_addrs()
            .map(|mut a| a.next().is_some())
            .unwrap_or(false);
        let _ = tx.send(ok);
    });
    let dns_ok = rx
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap_or(false);
    if dns_ok {
        CheckResult {
            id: "network".into(),
            name: "Network".into(),
            status: Status::Pass,
            message: "Online -- DNS resolving".into(),
            fix: None,
        }
    } else {
        CheckResult {
            id: "network".into(),
            name: "Network".into(),
            status: Status::Warn,
            message: "Online but DNS not resolving".into(),
            fix: Some("Check /etc/resolv.conf / DNS settings".into()),
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
                        if now
                            .duration_since(mtime)
                            .map(|a| a > PRUNE_AGE)
                            .unwrap_or(false)
                        {
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

pub fn check_nix_hygiene(core_root: &str) -> CheckResult {
    // INT-133: tuned deadnix + statix over nix/. Both tools exit 0 even with
    // findings, so we parse output, not exit codes. Tuning: deadnix runs with
    // --no-lambda-pattern-names (idiomatic `{ config, pkgs, lib, ... }:` headers
    // are not dead code); statix reads statix.toml at repo root (repeated_keys
    // disabled -- flat-dotted keys are idiomatic). Green when clean, warns on
    // genuine findings only (dead code, empty patterns, real anti-patterns).
    use std::process::Command;

    let nix_dir = format!("{}/nix", core_root);

    // deadnix: prints findings to stdout; empty stdout == clean.
    let deadnix_out = Command::new("deadnix")
        .args(["--no-lambda-pattern-names", &nix_dir])
        .output();
    let dead_count = match &deadnix_out {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout);
            // deadnix prints one block per finding; count lines that name a .nix file.
            s.lines().filter(|l| l.contains(".nix")).count()
        }
        Err(_) => {
            return CheckResult {
                id: "nix_hygiene".into(),
                name: "Nix Hygiene".into(),
                status: Status::Warn,
                message: "deadnix not found -- install it to lint Nix code".into(),
                fix: Some("Add deadnix to your packages".into()),
            };
        }
    };

    // statix: exits 0 but prints "Warning:" per finding; count those.
    let statix_out = Command::new("statix")
        .args(["check", &nix_dir])
        .current_dir(core_root) // so statix.toml at repo root is picked up
        .output();
    let statix_count = match &statix_out {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout);
            s.matches("Warning:").count()
        }
        Err(_) => {
            return CheckResult {
                id: "nix_hygiene".into(),
                name: "Nix Hygiene".into(),
                status: Status::Warn,
                message: "statix not found -- install it to lint Nix code".into(),
                fix: Some("Add statix to your packages".into()),
            };
        }
    };

    let total = dead_count + statix_count;
    if total == 0 {
        CheckResult {
            id: "nix_hygiene".into(),
            name: "Nix Hygiene".into(),
            status: Status::Pass,
            message: "Nix code clean -- no dead code or anti-patterns (tuned deadnix + statix)".into(),
            fix: None,
        }
    } else {
        CheckResult {
            id: "nix_hygiene".into(),
            name: "Nix Hygiene".into(),
            status: Status::Warn,
            message: format!(
                "{} Nix hygiene finding(s): {} dead-code, {} anti-pattern",
                total, dead_count, statix_count
            ),
            fix: Some("Run `deadnix --no-lambda-pattern-names nix/` and `statix check nix/` to see them".into()),
        }
    }
}
