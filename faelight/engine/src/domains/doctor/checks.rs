// doctor/checks.rs — all 23 health check functions
#![allow(dead_code)]
use super::{CheckResult, Status, Tier};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

pub fn check_services() -> CheckResult {
    // Per-session daemons the doctor expects up. faelight-bar joined this list
    // when INT-053 shipped it as a real systemd user service.
    let services = [
        ("faelight-notify", "Notifications"),
        ("faelight-bar", "Bar"),
        ("faelight-wsd", "Workspaces"),
    ];
    let total = services.len();

    // INT-146: distinguish "couldn't query the bus" from "service inactive". The user session
    // bus is unreachable in bus-less contexts (deploy activation, early boot, headless). Probing
    // is-active there Errs, and treating Err as "down" was a false red (was: .unwrap_or(true)).
    // Probe reachability ONCE; if the bus is unavailable, report honestly rather than crying wolf.
    let bus_reachable = Command::new("systemctl")
        .args(["--user", "show-environment"])
        .output() // .output() captures stdout; .status() would leak the
        .map(|o| o.status.success()) // env dump to the terminal. Same reachability check.
        .unwrap_or(false);

    if !bus_reachable {
        // INT-148: was Status::Pass with a note (146 workaround, before Unknown existed).
        // Now Unknown -- "couldn't determine service state" is honest, and it's excluded from
        // the health denominator rather than counting as a free Pass in bus-less contexts.
        return CheckResult {
            tier: Tier::System,
            id: "services".into(),
            name: "System Services".into(),
            status: Status::Unknown,
            message: format!(
                "{} services (session bus unavailable -- could not check in this context)",
                total
            ),
            fix: None,
        };
    }

    // Bus is reachable -> is-active answers are trustworthy. A non-success now genuinely means
    // the service is inactive, not that we failed to ask. (unwrap_or(true) is safe here because
    // the bus is up; a residual Err would be a real problem worth flagging as down.)
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
    let running = total - down.len();
    if running == total {
        CheckResult {
            tier: Tier::System,
            id: "services".into(),
            name: "System Services".into(),
            status: Status::Pass,
            message: format!("{}/{} services running", running, total),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::System,
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
            tier: Tier::System,
            id: "broken_symlinks".into(),
            name: "Broken Symlinks".into(),
            status: Status::Pass,
            message: "No broken symlinks found".into(),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::System,
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
        "nvim",
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
            tier: Tier::Critical,
            id: "binaries".into(),
            name: "Binary Dependencies".into(),
            status: Status::Pass,
            message: format!("All {} binaries found", bins.len()),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::Critical,
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

pub fn check_rust_docs(core_root: &str) -> CheckResult {
    // INT-151: catch rustdoc warnings on every `d` (prevents INT-150-class silent accumulation).
    // cargo doc is ~0.12s warm, ~2s cold (measured). Bounded at 3s via the thread+recv_timeout
    // pattern (mirrors check_network): a stuck/locked cargo -> Unknown (INT-148), never a hang.
    // Warnings are cosmetic -> Warn, never Fail.
    let manifest = format!("{}/faelight/engine/Cargo.toml", core_root);
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let out = Command::new("cargo")
            .args([
                "doc",
                "-p",
                "core",
                "--no-deps",
                "--manifest-path",
                &manifest,
            ])
            .output();
        let _ = tx.send(out);
    });
    match rx.recv_timeout(std::time::Duration::from_secs(3)) {
        Ok(Ok(output)) => {
            // Rustdoc prints a summary line "generated N warning(s)" -- parse THAT number
            // (authoritative), never count "warning:" lines (the summary line is itself one,
            // which would double-count -- confirmed by INT-151 calibration). No summary line
            // present -> 0 warnings (clean docs emit none).
            let stderr = String::from_utf8_lossy(&output.stderr);
            let warns: u32 = stderr
                .lines()
                .find(|l| l.contains("generated") && l.contains("warning"))
                .and_then(|l| l.split_whitespace().find_map(|w| w.parse::<u32>().ok()))
                .unwrap_or(0);
            if warns == 0 {
                CheckResult {
                    tier: Tier::User,
                    id: "rust_docs".into(),
                    name: "Rust Docs".into(),
                    status: Status::Pass,
                    message: "cargo doc clean, 0 warnings".into(),
                    fix: None,
                }
            } else {
                CheckResult {
                    tier: Tier::User,
                    id: "rust_docs".into(),
                    name: "Rust Docs".into(),
                    status: Status::Warn,
                    message: format!("{} rustdoc warning(s)", warns),
                    fix: Some("dev doc core  -- to see them".into()),
                }
            }
        }
        // Timed out, or cargo could not be spawned: cannot determine -> Unknown (INT-148).
        _ => CheckResult {
            tier: Tier::User,
            id: "rust_docs".into(),
            name: "Rust Docs".into(),
            status: Status::Unknown,
            message: "docs not checked (cargo busy or unavailable)".into(),
            fix: None,
        },
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
            tier: Tier::User,
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
            tier: Tier::User,
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
            tier: Tier::User,
            id: "themes".into(),
            name: "Theme Packages".into(),
            status: Status::Pass,
            message: format!("{}/1 theme packages present", count),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::User,
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
            tier: Tier::User,
            id: "scripts".into(),
            name: "Scripts".into(),
            status: Status::Pass,
            message: "Tools deployed as Nix binaries (NixOS)".into(),
            fix: None,
        };
    }
    if issues.is_empty() {
        CheckResult {
            tier: Tier::User,
            id: "scripts".into(),
            name: "Scripts".into(),
            status: Status::Pass,
            message: "All scripts present and executable".into(),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::User,
            id: "scripts".into(),
            name: "Scripts".into(),
            status: Status::Warn,
            message: format!("{} script issues", issues.len()),
            fix: Some("chmod +x ~/0-core/scripts/*".into()),
        }
    }
}

pub fn check_intents(_core_root: &str) -> CheckResult {
    // INT-135 Gate 7: was decoration -- hardcoded Status::Pass, a phantom "active/" folder,
    // no "in-progress", and a substring match for "status: complete" over whole files.
    // Now calls the ONE validator. Doctor and `core intent validate` cannot disagree.
    let (count, issues) = crate::domains::intent::validate_issues();

    if issues.is_empty() {
        CheckResult {
            tier: Tier::User,
            id: "intents".into(),
            name: "Intent Ledger".into(),
            status: Status::Pass,
            message: format!("{} intents, all valid", count),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::User,
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
            tier: Tier::System,
            id: "config".into(),
            name: "Faelight Config".into(),
            status: Status::Pass,
            message: "All config files valid".into(),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::System,
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
            tier: Tier::User,
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
                tier: Tier::User,
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
            tier: Tier::User,
            id: "keybinds".into(),
            name: "Compositor Keybinds".into(),
            status: Status::Pass,
            message: format!("{}: {} keybindings, no conflicts", wm_name, count),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::User,
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

/// Read ONE sshd_config directive, honestly.
///
/// INT-164: the old check used `cfg.contains("PasswordAuthentication no")`, which is a substring
/// match on the whole file. It would be satisfied by a COMMENTED line -- `#PasswordAuthentication no`
/// -- i.e. by a setting explicitly turned OFF. Latent rather than live here (NixOS generates the
/// full effective config with no commented defaults, measured: 20 lines, gen 392), but a check that
/// can be fooled by a `#` is not a check.
/// Directive match, not substring. Comments skipped. Case-insensitive, as sshd itself is.
fn sshd_setting(cfg: &str, key: &str) -> Option<String> {
    cfg.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .find_map(|l| {
            let mut it = l.split_whitespace();
            let k = it.next()?;
            if k.eq_ignore_ascii_case(key) {
                it.next().map(|v| v.to_ascii_lowercase())
            } else {
                None
            }
        })
}

/// Which authentication doors an sshd_config leaves OPEN. Separated from the check so it can be
/// TESTED against a real config instead of read and believed -- the `||` bug below looked fine to
/// every reader it ever had.
///
/// TWO PASSWORD DOORS, NOT ONE. `sshd -T` proved it 2026-07-17: PasswordAuthentication=no ALONE does
/// not close password login while KbdInteractiveAuthentication=yes and UsePAM=yes -- PAM still
/// offers a prompt. Setting only the first is the classic half-fix, and the old check called the
/// result "hardened".
///
/// OpenSSH defaults BOTH password doors to `yes`. An ABSENT directive is therefore an OPEN door.
/// unwrap_or("yes") is the honest reading, not a pessimistic one: a config-file check cannot prove
/// a negative, so it fails safe rather than fails flattering.
fn ssh_open_doors(cfg: &str) -> Vec<&'static str> {
    let mut open = vec![];
    if sshd_setting(cfg, "PasswordAuthentication").unwrap_or_else(|| "yes".into()) == "yes" {
        open.push("password");
    }
    if sshd_setting(cfg, "KbdInteractiveAuthentication").unwrap_or_else(|| "yes".into()) == "yes" {
        open.push("keyboard-interactive");
    }
    if sshd_setting(cfg, "PermitRootLogin").unwrap_or_else(|| "yes".into()) == "yes" {
        open.push("root login");
    }
    open
}

/// INT-164: report FACTS, not a verdict. A fact cannot lie; a verdict can.
///
/// WHAT THIS CHECK USED TO SAY, and why it was wrong -- measured 2026-07-17:
///   1. `PermitRootLogin no || PasswordAuthentication no` -- ONE `||` where an `&&` belongs. It read
///      the RIGHT two settings and accepted EITHER. PermitRootLogin was `no`, so it printed
///      "SSH hardened OK" while `PasswordAuthentication yes` sat on the line above it. That is the
///      bug this intent was filed for, and it would have lied about ANY sshd, including one enabled
///      later for a real reason.
///   2. fail2ban: `systemctl is-active` -- but IS-ACTIVE IS NOT IS-PROTECTING. After sshd was
///      removed, `fail2ban-client status` reported `Number of jail: 0` and this check went right on
///      printing "fail2ban OK". A daemon watching an empty room.
///   3. `if details > 0 { Pass }` -- ANY ONE of three made the WHOLE check green. A firewall alone
///      earned "Security Hardening: Pass" no matter what else was open.
///
/// WHAT IT SAYS NOW: what it can actually see, named. "sshd off" is a fact. "sshd: password auth ON"
/// is a fact. "Firewall" is a fact. The STATUS is a real conjunction, not a count.
/// It does NOT claim key-only, because a config-file read cannot prove the negative -- an absent
/// directive means OpenSSH's DEFAULT, and for both password doors that default is `yes`. Absent is
/// therefore read as OPEN. Fail safe, not fail flattering.
pub fn check_security_hardening() -> CheckResult {
    let firewall_active = Command::new("systemctl")
        .args(["is-active", "firewall"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false);

    let sshd_path = PathBuf::from("/etc/ssh/sshd_config");
    let sshd_cfg = if sshd_path.exists() {
        fs::read_to_string(&sshd_path).ok()
    } else {
        None
    };

    let mut facts: Vec<String> = vec![];
    let mut ssh_warn: Option<String> = None;

    facts.push(if firewall_active {
        "Firewall \u{2705}".to_string()
    } else {
        "Firewall \u{274C}".to_string()
    });

    match &sshd_cfg {
        // No sshd_config at all -> the daemon is off. Nothing to harden. This is the strongest
        // state there is, and INT-164 is why this machine is in it.
        None => facts.push("sshd off".to_string()),
        Some(cfg) => {
            let open = ssh_open_doors(cfg);
            if open.is_empty() {
                facts.push("sshd: key-only".to_string());
            } else {
                let msg = format!("sshd: {} ON", open.join(" + "));
                facts.push(msg.clone());
                ssh_warn = Some(msg);
            }
        }
    }

    // fail2ban is a FACT, never a tick. Counting jails needs root, and this check runs as the user
    // -- so it reports only what it can prove: the process is up. It does NOT claim protection,
    // because on 2026-07-17 it was up with ZERO jails and this line said "fail2ban OK".
    let f2b_active = Command::new("systemctl")
        .args(["is-active", "fail2ban"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false);
    if f2b_active {
        facts.push("fail2ban running".to_string());
    }

    // A REAL CONJUNCTION. Not `details > 0`.
    let status = if !firewall_active {
        Status::Fail
    } else if ssh_warn.is_some() {
        Status::Warn
    } else {
        Status::Pass
    };

    CheckResult {
        tier: Tier::System,
        id: "security".into(),
        name: "Security Hardening".into(),
        status,
        message: format!("Security: {}", facts.join("  ")),
        fix: if !firewall_active {
            Some("Enable firewall: networking.firewall.enable = true in configuration.nix".into())
        } else if let Some(w) = &ssh_warn {
            Some(format!(
                "{} -- set PasswordAuthentication = false AND KbdInteractiveAuthentication = false \
                 (both: PAM offers a password prompt via keyboard-interactive), or disable sshd \
                 entirely with services.openssh.enable = false",
                w
            ))
        } else {
            None
        },
    }
}

pub fn check_security_audit(home: &str) -> CheckResult {
    let scan_path = PathBuf::from(home).join(".local/state/0-core/security/last-scan.json");
    if !scan_path.exists() {
        return CheckResult {
            tier: Tier::System,
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
                tier: Tier::System,
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
            tier: Tier::System,
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
        tier: Tier::System,
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
                tier: Tier::User,
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
            tier: Tier::User,
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
            tier: Tier::User,
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
            tier: Tier::System,
            id: "rust_toolchain".into(),
            name: "Rust Toolchain".into(),
            status: Status::Pass,
            message: "Rust toolchain available".into(),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::System,
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
            tier: Tier::Critical,
            id: "disk_space".into(),
            name: "Disk Space".into(),
            status: Status::Pass,
            message: "Sufficient disk space".into(),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::Critical,
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
            tier: Tier::System,
            id: "tool_installation".into(),
            name: "Tool Installation".into(),
            status: Status::Pass,
            message: format!("All {} key tools installed", tools.len()),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::System,
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
        tier: Tier::System,
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
            tier: Tier::System,
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
            tier: Tier::System,
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
        tier: Tier::System,
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
                tier: Tier::Critical,
                id: "boot_errors".into(),
                name: "Boot Errors".into(),
                status: Status::Pass,
                message: msg,
                fix: None,
            }
        }
        Some(n) => CheckResult {
            tier: Tier::Critical,
            id: "boot_errors".into(),
            name: "Boot Errors".into(),
            status: Status::Warn,
            message: format!("{} critical kernel error(s) since last boot", n),
            fix: Some("journalctl -b -k -p crit".into()),
        },
        None => CheckResult {
            tier: Tier::Critical,
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
                tier: Tier::System,
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
                tier: Tier::System,
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
            tier: Tier::System,
            id: "boot_time".into(),
            name: "Boot Time".into(),
            status: Status::Warn,
            message: format!("Userspace startup {} (over 15s target)", userspace),
            fix: Some("systemd-analyze blame".into()),
        }
    } else {
        CheckResult {
            tier: Tier::System,
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
                tier: Tier::Info,
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
        tier: Tier::Info,
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
                tier: Tier::Info,
                id: "compositor".into(),
                name: "Compositor".into(),
                status: Status::Pass,
                message: format!("{} running", label),
                fix: None,
            };
        }
    }
    CheckResult {
        tier: Tier::Info,
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
                tier: Tier::System,
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
            tier: Tier::System,
            id: "nix_store".into(),
            name: "Nix Store".into(),
            status: Status::Warn,
            message: format!("{} -- consider nix-collect-garbage", message),
            fix: Some("Run nix-collect-garbage -d to reclaim space".into()),
        }
    } else {
        CheckResult {
            tier: Tier::System,
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
                tier: Tier::User,
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
            tier: Tier::User,
            id: "friday".into(),
            name: "Friday".into(),
            status: Status::Warn,
            message: format!("Warming up -- {} patterns, {} facts", patterns, facts),
            fix: Some("Friday learns from daily use; patterns build over time".into()),
        }
    } else if stale {
        let days = (now - last_learned) / 86_400;
        CheckResult {
            tier: Tier::User,
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
            tier: Tier::User,
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
            tier: Tier::System,
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
            tier: Tier::System,
            id: "network".into(),
            name: "Network".into(),
            status: Status::Pass,
            message: "Online -- DNS resolving".into(),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::System,
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
            tier: Tier::System,
            id: "generation_drift".into(),
            name: "Generation Drift".into(),
            status: Status::Pass,
            message: "Booted generation is current".into(),
            fix: None,
        },
        (Some(_), Some(_)) => CheckResult {
            tier: Tier::System,
            id: "generation_drift".into(),
            name: "Generation Drift".into(),
            status: Status::Warn,
            message: "Rebuilt since boot -- reboot to apply (kernel/initrd changes need it)".into(),
            fix: Some("Reboot to activate the current generation".into()),
        },
        _ => CheckResult {
            tier: Tier::System,
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
            tier: Tier::User,
            id: "generation_count".into(),
            name: "Generation Count".into(),
            status: Status::Warn,
            message: format!("{} generations ({} older than 14d, prunable)", total, old),
            fix: Some("sudo nix-collect-garbage --delete-older-than 14d".into()),
        }
    } else {
        CheckResult {
            tier: Tier::User,
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
            tier: Tier::User,
            id: "flake_lock_age".into(),
            name: "Flake Lock Age".into(),
            status: Status::Warn,
            message: format!("flake.lock is {} days old -- deps may be stale", days),
            fix: Some("nix flake update".into()),
        },
        Some(days) => CheckResult {
            tier: Tier::User,
            id: "flake_lock_age".into(),
            name: "Flake Lock Age".into(),
            status: Status::Pass,
            message: format!("flake.lock updated {} days ago", days),
            fix: None,
        },
        None => CheckResult {
            tier: Tier::User,
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
            tier: Tier::User,
            id: "update_readiness".into(),
            name: "Update Readiness".into(),
            status: Status::Pass,
            message: "Safe to update -- booted current, tree clean".into(),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::User,
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
                tier: Tier::User,
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
                tier: Tier::User,
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
            tier: Tier::User,
            id: "nix_hygiene".into(),
            name: "Nix Hygiene".into(),
            status: Status::Pass,
            message: "Nix code clean -- no dead code or anti-patterns (tuned deadnix + statix)"
                .into(),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::User,
            id: "nix_hygiene".into(),
            name: "Nix Hygiene".into(),
            status: Status::Warn,
            message: format!(
                "{} Nix hygiene finding(s): {} dead-code, {} anti-pattern",
                total, dead_count, statix_count
            ),
            fix: Some(
                "Run `deadnix --no-lambda-pattern-names nix/` and `statix check nix/` to see them"
                    .into(),
            ),
        }
    }
}

#[cfg(test)]
mod int164_security_tests {
    use super::*;

    /// The EXACT config that was live on generation 392, verbatim from
    /// /nix/var/nix/profiles/system-392-link/etc/ssh/sshd_config on 2026-07-17.
    /// The old check called this "SSH hardened OK".
    const GEN_392: &str = "\
AuthorizedPrincipalsFile none
GatewayPorts no
KbdInteractiveAuthentication yes
LogLevel VERBOSE
PasswordAuthentication yes
PermitRootLogin no
PrintMotd no
StrictModes yes
UseDns no
UsePAM yes
X11Forwarding no
AddressFamily any
Port 22
";

    #[test]
    fn gen_392_config_is_not_hardened() {
        // THE REGRESSION. The old check was:
        //     c.contains("PermitRootLogin no") || c.contains("PasswordAuthentication no")
        // PermitRootLogin IS "no" here, so the old check returned TRUE and printed
        // "SSH hardened OK" -- while password auth AND keyboard-interactive were both wide open.
        // This test FAILS on the old logic and passes on the new. That is the whole point.
        let open = ssh_open_doors(GEN_392);
        assert!(
            open.contains(&"password"),
            "PasswordAuthentication yes must register as an open door"
        );
        assert!(
            open.contains(&"keyboard-interactive"),
            "KbdInteractiveAuthentication yes is the SECOND password door -- the half-fix this \
             intent exists to prevent"
        );
        assert!(
            !open.contains(&"root login"),
            "PermitRootLogin no was the ONE thing set correctly, and the old check let it vouch \
             for everything else"
        );
    }

    /// THE OLD LOGIC, preserved verbatim so the bug can be DEMONSTRATED rather than described.
    /// This is exactly what checks.rs:533 did before INT-164:
    ///     c.contains("PermitRootLogin no") || c.contains("PasswordAuthentication no")
    fn old_check_said_hardened(cfg: &str) -> bool {
        cfg.contains("PermitRootLogin no") || cfg.contains("PasswordAuthentication no")
    }

    #[test]
    fn the_old_check_called_gen_392_hardened_and_it_was_wrong() {
        // WATCH THE GATE FAIL FIRST. The tests above prove the NEW function is right; on their own
        // they do not prove the OLD one was wrong, because the old one is gone. This one does.
        //
        // The old check returned TRUE for the config that was LIVE on generation 392 -- and the
        // dashboard printed "SSH hardened OK" on the strength of it, for months.
        assert!(
            old_check_said_hardened(GEN_392),
            "the old check really did pass this config -- that is the bug"
        );
        // And the machine it vouched for had BOTH password doors wide open.
        let open = ssh_open_doors(GEN_392);
        assert_eq!(open, vec!["password", "keyboard-interactive"]);
        // ONE `||` between an honest dashboard and a lying one. PermitRootLogin was the single
        // thing set correctly, and the `||` let it vouch for everything else.
    }

    #[test]
    fn the_old_check_would_pass_a_config_with_nothing_set() {
        // Worse than gen 392: the old check would ALSO have passed a config where the setting is
        // explicitly COMMENTED OUT, because contains() does not care about a leading `#`.
        // Latent on NixOS (which generates the full effective config, no comments) -- but this is
        // what "it looked rigorous" bought.
        let commented = "#PasswordAuthentication no\nPermitRootLogin yes\n";
        assert!(
            old_check_said_hardened(commented),
            "the old check passed a DISABLED line"
        );
        assert!(
            ssh_open_doors(commented).contains(&"root login"),
            "meanwhile root login was open"
        );
    }

    #[test]
    fn a_genuinely_key_only_config_is_clean() {
        let cfg =
            "PasswordAuthentication no\nKbdInteractiveAuthentication no\nPermitRootLogin no\n";
        assert!(ssh_open_doors(cfg).is_empty());
    }

    #[test]
    fn half_fix_is_caught() {
        // The trap: close the door you know about, leave the one you do not.
        let cfg =
            "PasswordAuthentication no\nKbdInteractiveAuthentication yes\nPermitRootLogin no\n";
        assert_eq!(ssh_open_doors(cfg), vec!["keyboard-interactive"]);
    }

    #[test]
    fn absent_directive_is_an_open_door() {
        // OpenSSH defaults both to yes. Silence is not safety.
        assert_eq!(
            ssh_open_doors("Port 22\n"),
            vec!["password", "keyboard-interactive", "root login"]
        );
    }

    #[test]
    fn commented_setting_does_not_count_as_set() {
        // The latent bug in the old check: contains() matched a DISABLED line.
        let cfg =
            "#PasswordAuthentication no\nKbdInteractiveAuthentication no\nPermitRootLogin no\n";
        assert_eq!(ssh_open_doors(cfg), vec!["password"]);
    }

    #[test]
    fn directive_match_is_not_substring_match() {
        assert_eq!(
            sshd_setting("PermitRootLogin no\n", "PermitRootLogin"),
            Some("no".into())
        );
        assert_eq!(sshd_setting("PermitRootLogin no\n", "RootLogin"), None);
        // sshd itself is case-insensitive on directive names.
        assert_eq!(
            sshd_setting("permitrootlogin NO\n", "PermitRootLogin"),
            Some("no".into())
        );
    }
}
