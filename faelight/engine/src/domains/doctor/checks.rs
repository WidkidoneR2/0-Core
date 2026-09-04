// doctor/checks.rs -- the health check functions. The count is deliberately NOT
// written here: it said 23 while this file held 30, and the docs said 22 and 14.
#![allow(dead_code)]
use super::{CheckResult, Status, Tier};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

pub fn check_services() -> CheckResult {
    // Per-session daemons the doctor expects up. faelight-bar joined this list
    // when INT-053 shipped it as a real systemd user service.
    // INT-222: ASK systemd which services should be running, do not name them here.
    // This list was hardcoded to three -- notify, bar, wsd -- while the target wanted
    // five: faelight-insightd had been invisible to the panel for weeks, and
    // faelight-idle became invisible the moment it was wired. A panel that says
    // "3/3 running" about a set of five is not reporting health, it is reporting
    // its own memory. Wiring a service now makes it appear here by itself.
    let wants = Command::new("systemctl")
        .args(["--user", "show", "-p", "Wants", "faelight-session.target"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let services: Vec<String> = wants
        .trim()
        .strip_prefix("Wants=")
        .unwrap_or("")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    let total = services.len();

    if total == 0 {
        // INT-222: no wants means the query failed or the target is absent -- NOT zero
        // services healthy. 0/0 would render as a clean Pass, which is the free pass this
        // whole intent exists to remove.
        return CheckResult {
            tier: Tier::System,
            id: "services".into(),
            name: "System Services".into(),
            status: Status::Unknown,
            message: "could not read faelight-session.target wants".into(),
            fix: None,
        };
    }

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
    let down: Vec<String> = services
        .iter()
        .filter(|name| {
            // is-active by unit name survives binary swaps (INT-053 changed
            // faelight-bar's ExecStart to the faelight-bar-gtk binary).
            Command::new("systemctl")
                .args(["--user", "is-active", "--quiet", name.as_str()])
                .status()
                .map(|s| !s.success())
                .unwrap_or(true)
        })
        .cloned()
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
        // ⚠️ A DANGLING LINK INTO A RUNTIME DIRECTORY MEANS THE APP IS NOT RUNNING, not that the
        // configuration is broken. Chromium and PulseAudio both drop lock and socket links under
        // ~/.config pointing into /tmp, and those targets vanish the moment the process exits.
        //
        // ⭐ THE EXCLUSION IS A PROPERTY, NOT A NAME. Three application names were listed here and
        // the list would have grown with every program installed -- the same hand-maintained drift
        // as the binary list one function above. Asking where the TARGET lives answers it for every
        // application at once: /tmp and /run are runtime paths by definition.
        // ⚠️ TWO CONDITIONS, BECAUSE NEITHER CATCHES ALL OF THEM. A first attempt replaced the
        // name list with a target test alone and the count went from one to four: Chromium writes
        // SingletonLock and SingletonCookie with RELATIVE targets, so asking where the target lives
        // says nothing about them, while the socket beside them points at /tmp and the name says
        // nothing about it. The property is "an artifact a running program manages", and neither
        // test expresses that on its own.
        let target = std::fs::read_link(p).unwrap_or_default();
        let runtime_target = target.starts_with("/tmp") || target.starts_with("/run");
        let name = p.to_string_lossy();
        let runtime_name = name.contains("Singleton")
            || name.contains("BraveSoftware")
            || name.contains("Notesnook");
        if p.is_symlink() && !p.exists() && !runtime_target && !runtime_name {
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
    // ⚠️ THIS LIST IS ONE MACHINE'S EXPECTATIONS AND IT IS HAND-MAINTAINED, so it drifts the way
    // the check counts did. It named a compositor, a terminal and two package-manager tools from
    // the previous system -- four Critical-tier failures for facilities this machine deliberately
    // does not have, which is what kept a notification on screen that could not be acted on.
    //
    // ⭐ WHAT CHANGED AND WHY: mango -> hyprland (the compositor actually running) ·
    // alacritty -> foot (the terminal actually running) · nix-tree and nvd DELETED outright,
    // because they manage a package manager that is gone and nothing here can want them.
    //
    // ⏭ The durable fix is deriving this from something declared rather than typed; until then a
    // wrong entry here reports a Critical failure, so treat it as code, not as a note.
    let bins = [
        "hyprland",
        "foot",
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

/// ⭐ WILL A HOOK ACTUALLY RUN? Nothing measured this, and there are three ways for the answer to
/// be no.
///
/// The hooks are TRACKED, in .githooks/, but git does not use them until someone runs
/// `git config core.hooksPath .githooks` -- and that setting is LOCAL TO THE CLONE. It does not
/// travel with a fetch, a fresh clone, or a machine migration. Git refuses to let a repository arm
/// its own hooks, and that refusal is a security property rather than an oversight.
///
/// So after the Omarchy wipe the gate was one manual command away from never having run, and no
/// check anywhere would have said so. INT-113 and INT-119 both died exactly this way: a gate
/// that had never existed reported nothing, for days, and looked identical to a gate that passed.
///
/// THREE FAILURE MODES, ONE QUESTION:
///   unset            -- the gate is disarmed and git is running its own empty .git/hooks
///   set elsewhere    -- a deliberate override, named rather than guessed at
///   not executable   -- git SKIPS a hook without the bit, silently, which a fresh clone can cause
///
/// Unknown when git itself cannot be asked, per INT-148: could-not-determine is not health.
pub fn check_hooks(core_root: &str) -> CheckResult {
    let id = "hooks";
    let name = "Git Hooks";
    let out = Command::new("git")
        .args(["-C", core_root, "config", "core.hooksPath"])
        .output();
    let Ok(out) = out else {
        return CheckResult {
            tier: Tier::User,
            id: id.into(),
            name: name.into(),
            status: Status::Unknown,
            message: "could not run git config".into(),
            fix: None,
        };
    };
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if path.is_empty() {
        return CheckResult {
            tier: Tier::User,
            id: id.into(),
            name: name.into(),
            status: Status::Fail,
            message: "core.hooksPath is unset -- no gate runs on commit or push".into(),
            fix: Some("git config core.hooksPath .githooks".into()),
        };
    }
    if path != ".githooks" {
        return CheckResult {
            tier: Tier::User,
            id: id.into(),
            name: name.into(),
            status: Status::Fail,
            message: format!("core.hooksPath is {path}, not .githooks"),
            fix: Some("git config core.hooksPath .githooks".into()),
        };
    }
    let mut not_exec: Vec<String> = Vec::new();
    for hook in ["pre-commit", "pre-push"] {
        let f = std::path::Path::new(core_root).join(".githooks").join(hook);
        let ok = fs::metadata(&f)
            .map(|m| {
                use std::os::unix::fs::PermissionsExt;
                m.permissions().mode() & 0o111 != 0
            })
            .unwrap_or(false);
        if !ok {
            not_exec.push(hook.to_string());
        }
    }
    if !not_exec.is_empty() {
        return CheckResult {
            tier: Tier::User,
            id: id.into(),
            name: name.into(),
            status: Status::Fail,
            message: format!(
                "{} not executable -- git skips it silently",
                not_exec.join(", ")
            ),
            fix: Some(format!(
                "chmod +x .githooks/{}",
                not_exec.join(" .githooks/")
            )),
        };
    }
    CheckResult {
        tier: Tier::User,
        id: id.into(),
        name: name.into(),
        status: Status::Pass,
        message: "core.hooksPath is .githooks, hooks executable".into(),
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
            name: "Zero Config".into(),
            status: Status::Pass,
            message: "All config files valid".into(),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::System,
            id: "config".into(),
            name: "Zero Config".into(),
            status: Status::Warn,
            message: format!("{} config issues", issues),
            fix: Some("Run: faelight config validate".into()),
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
    // ⚠️ "firewall" IS ONE DISTRIBUTION'S UNIT NAME, NOT THE QUESTION. NixOS calls it
    // firewall.service; Arch machines run ufw, firewalld or a plain nftables/iptables unit. Asking
    // for one name reported NO FIREWALL on a machine whose ufw was active and denying all incoming
    // traffic -- a false alarm in a check whose own comment says fail safe, not fail flattering.
    // A false alarm is its own failure: it teaches the reader to discount the line.
    //
    // ⭐ THE QUESTION IS WHETHER SOMETHING IS ENFORCING, so ask about every unit that could be.
    let firewall_active = ["firewall", "ufw", "firewalld", "nftables", "iptables"]
        .iter()
        .any(|unit| {
            Command::new("systemctl")
                .args(["is-active", unit])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
                .unwrap_or(false)
        });

    // ⚠️ THE MAIN FILE IS NOT THE CONFIGURATION. Its line 2 is `Include sshd_config.d/*.conf`,
    // and on Arch that directory is WHERE SETTINGS ARE SUPPOSED TO LIVE -- the distribution ships
    // its own defaults there and expects yours beside them, because editing the packaged file
    // leaves a .pacnew to merge on every upgrade. Reading only the main file reported both
    // password doors OPEN on a machine that had explicitly closed them one directory over.
    //
    // ⭐ ORDER MATTERS AND IT IS NOT ALPHABETICAL-AFTER: OpenSSH takes the FIRST value it sees for
    // most keywords, and the Include sits at the TOP of the main file -- so the included files are
    // read BEFORE the rest of it. Concatenating the other way round would produce a confident
    // wrong answer, which is worse than the blind one this replaces.
    let sshd_path = PathBuf::from("/etc/ssh/sshd_config");
    let sshd_cfg = if sshd_path.exists() {
        let mut parts: Vec<String> = Vec::new();
        if let Ok(entries) = fs::read_dir("/etc/ssh/sshd_config.d") {
            let mut confs: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "conf"))
                .collect();
            confs.sort();
            for c in confs {
                if let Ok(t) = fs::read_to_string(&c) {
                    parts.push(t);
                }
            }
        }
        if let Ok(main) = fs::read_to_string(&sshd_path) {
            parts.push(main);
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(
                "
",
            ))
        }
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

    // INT-192: A SCAN WITH A BLIND STEP IS NOT A CLEAN BILL. cargo-audit is not installed
    // here, so the crate audit never ran -- and this check read an empty findings array as
    // zero vulnerabilities and reported a green line. Measured 2026-09-04: the honest
    // warning No scan found became a false pass the moment a scan was run.
    //
    // Warn rather than Unknown, because three of the four sub-scans DID run and found
    // nothing. The answer is partial, not absent, and the message says which part is
    // missing rather than leaving the reader to guess.
    let skipped: Vec<String> = json["skipped"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    if !skipped.is_empty() {
        return CheckResult {
            tier: Tier::System,
            id: "security_audit".into(),
            name: "Security Audit".into(),
            status: Status::Warn,
            message: format!(
                "{} finding(s), but {} check(s) could not run: {}",
                findings,
                skipped.len(),
                skipped.join("; ")
            ),
            fix: Some("Install the missing tool, then: core security scan".into()),
        };
    }

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
    use crate::domains::doctor::aliases::{expected_tools, parse_aliases};
    use faelight_core::paths;

    let aliases_path = paths::shell_config();
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
    let expected = expected_tools();
    for tool in &expected {
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
                expected.len(),
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
    // ⚠️ AN UNREADABLE REGISTRY IS NOT AN EMPTY ONE. The chain below ends unwrap_or_default(), so a
    // missing tools.toml produced an empty list, an empty list has nothing missing, and this
    // printed a green `All 0 key tools installed`. FOUND IN THE VM 2026-08-23 on a guest with
    // THIRTY-TWO tools actually deployed -- so it was not merely silent, it described the wrong
    // machine confidently. Same defect as the intent ledger one function over, and the same shape
    // INT-227 removed from the shell.
    if !registry_path.exists() {
        return CheckResult {
            tier: Tier::System,
            id: "tool_installation".into(),
            name: "Tool Installation".into(),
            status: Status::Warn,
            message: format!(
                "tool registry not found at {} -- cannot say what should be installed",
                registry_path.display()
            ),
            fix: None,
        };
    }
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

// The parameter stays for the shared check signature; this check no longer needs it, because
// asking `which` does not require knowing where the project lives.
pub fn check_path_resilience(_core_root: &str) -> CheckResult {
    let registry_path = faelight_core::paths::tools_registry();
    // ⚠️ A PERCENTAGE OF NOTHING IS NOT ZERO PERCENT. The chain below ends unwrap_or_default(), so
    // an unreadable registry gave total = 0, and the divide-by-zero guard chose 0 as the answer --
    // printing `0/0 tools deployed (0%)`, which READS AS A MEASUREMENT. Found in the VM alongside
    // the same swallow in check_tool_installation. The guard was right that zero cannot be
    // divided; it was wrong that the answer is zero.
    if !registry_path.exists() {
        return CheckResult {
            tier: Tier::System,
            id: "path_resilience".into(),
            name: "Path Resilience".into(),
            status: Status::Warn,
            message: format!(
                "tool registry not found at {} -- cannot say what should be deployed",
                registry_path.display()
            ),
            fix: None,
        };
    }

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
            // ⚠️ THE QUESTION IS "CAN I RUN THIS", AND `which` ANSWERS IT ON EVERY SYSTEM.
            // This used to branch on /etc/NIXOS and, off NixOS, look inside 0-core/scripts/ --
            // one deployment shape hardcoded as if it were the only one. On Omarchy with all
            // 29 tools on PATH it reported 0/29 deployed while the installation check, which
            // uses `which`, reported 24/25. Two checks over the same tools disagreeing, and
            // the one that assumed a location was the wrong one.
            Command::new("which")
                .arg(n)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
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

// The parameter stays for the shared check signature; asking `which` does not require knowing
// where the project lives. Second check today to shed it for the same reason.
pub fn check_sandbox(_core_root: &str) -> CheckResult {
    // ⚠️ TWO HARDCODED LOCATIONS, NEITHER OF WHICH EXISTS HERE. It looked in one distribution's
    // system path and then in a directory inside the project, and reported the binary undeployed
    // while it sat on PATH where the shell finds it every time. That is the FOURTH check today
    // needing the same correction, which is itself the finding: five checks each answer "is this
    // tool installed" their own way, and each was right about exactly one machine.
    //
    // ⭐ `which` asks the question the shell answers, so it is true wherever the tool actually is.
    let binary_exists = Command::new("which")
        .arg("faelight-sandbox")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

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

// -- Arch system state (INT-222, Omarchy) ------------------------------------
// ONE owner for the kernel question. Both checks below call these two helpers;
// a second place deciding what the running kernel is would be the two-owners
// disease arriving on day one.
fn running_kernel() -> Result<String, String> {
    match Command::new("uname").arg("-r").output() {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                Err("uname -r produced no output".to_string())
            } else {
                Ok(s)
            }
        }
        Err(e) => Err(format!("uname -r failed: {}", e)),
    }
}

fn installed_kernels() -> Result<Vec<String>, String> {
    let rd = std::fs::read_dir("/usr/lib/modules")
        .map_err(|e| format!("cannot read /usr/lib/modules: {}", e))?;
    let mut out: Vec<String> = rd
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    out.sort();
    if out.is_empty() {
        Err("no kernel module trees in /usr/lib/modules".to_string())
    } else {
        Ok(out)
    }
}

// Replaces check_generation_drift. SAME QUESTION -- is what is running still
// what is installed -- with a mechanism that exists on this machine. Both reads
// are unprivileged, which matters because d runs at session start.
pub fn check_reboot_needed() -> CheckResult {
    let id = "reboot_needed";
    let name = "Reboot Needed";
    let running = match running_kernel() {
        Ok(v) => v,
        Err(why) => {
            return CheckResult {
                tier: Tier::System,
                id: id.into(),
                name: name.into(),
                status: Status::Unknown,
                message: format!("could not check -- {}", why),
                fix: None,
            }
        }
    };
    let installed = match installed_kernels() {
        Ok(v) => v,
        Err(why) => {
            return CheckResult {
                tier: Tier::System,
                id: id.into(),
                name: name.into(),
                status: Status::Unknown,
                message: format!("could not check -- {}", why),
                fix: None,
            }
        }
    };
    if installed.iter().any(|k| k == &running) {
        CheckResult {
            tier: Tier::System,
            id: id.into(),
            name: name.into(),
            status: Status::Pass,
            message: format!("running kernel {} is the installed one", running),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::System,
            id: id.into(),
            name: name.into(),
            status: Status::Warn,
            message: format!(
                "kernel changed since boot -- running {}, installed {}",
                running,
                installed.join(", ")
            ),
            fix: Some("Reboot to load the installed kernel".into()),
        }
    }
}

// INT-222: the old body read /run/current-system and /run/booted-system. Off
// NixOS both read_link calls returned Err, .ok() turned them into None, the
// if-let never matched, and NO BLOCKER WAS PUSHED -- so it reported "safe to
// update" having measured only the git half. A MISSING MEASUREMENT RENDERED AS
// A PASS, which is quieter and therefore worse than a false amber.
// Rule enforced below: every signal produces a blocker, an unreadable entry, or
// a pass. None of them may go quiet.
pub fn check_update_readiness(core_root: &str) -> CheckResult {
    let id = "update_readiness";
    let name = "Update Readiness";
    let mut blockers: Vec<String> = Vec::new();
    let mut unreadable: Vec<String> = Vec::new();

    match (running_kernel(), installed_kernels()) {
        (Ok(r), Ok(i)) => {
            if !i.iter().any(|k| k == &r) {
                blockers.push(format!(
                    "reboot -- running {}, installed {}",
                    r,
                    i.join(", ")
                ));
            }
        }
        (Err(why), _) | (_, Err(why)) => unreadable.push(format!("kernel state ({})", why)),
    }

    match Command::new("git")
        .arg("-C")
        .arg(core_root)
        .args(["diff", "--quiet", "HEAD"])
        .status()
    {
        Ok(s) => {
            if !s.success() {
                blockers.push("commit or stash tracked changes".to_string());
            }
        }
        Err(e) => unreadable.push(format!("git worktree ({})", e)),
    }

    if !blockers.is_empty() {
        CheckResult {
            tier: Tier::User,
            id: id.into(),
            name: name.into(),
            status: Status::Warn,
            message: format!("hold off -- {}", blockers.join("; ")),
            fix: Some("Resolve the above, then: omarchy-update".into()),
        }
    } else if !unreadable.is_empty() {
        CheckResult {
            tier: Tier::User,
            id: id.into(),
            name: name.into(),
            status: Status::Unknown,
            message: format!("could not judge -- unread: {}", unreadable.join("; ")),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::User,
            id: id.into(),
            name: name.into(),
            status: Status::Pass,
            message: "safe to update -- kernel current, tree clean".into(),
            fix: None,
        }
    }
}

// Tier::Info -- a REPORTER. It measures truly and never renders a judgement, so
// it stays out of the health denominator. Deliberate: any warning threshold here
// would be a number somebody typed, and a typed threshold goes stale exactly the
// way the check count did.
// NOTE: du on this directory EXITS 1 because two root-only download-* temp dirs
// live in it, while still printing a correct total. Walking entries and asking
// each for metadata skips those without ever consulting an exit code.
pub fn check_package_cache() -> CheckResult {
    let id = "package_cache";
    let name = "Package Cache";
    let dir = "/var/cache/pacman/pkg";
    let rd = match std::fs::read_dir(dir) {
        Ok(v) => v,
        Err(e) => {
            return CheckResult {
                tier: Tier::Info,
                id: id.into(),
                name: name.into(),
                status: Status::Unknown,
                message: format!("could not read {} -- {}", dir, e),
                fix: None,
            }
        }
    };
    let mut bytes: u64 = 0;
    let mut files: u64 = 0;
    for e in rd.filter_map(|e| e.ok()) {
        if let Ok(m) = e.metadata() {
            if m.is_file() {
                bytes += m.len();
                files += 1;
            }
        }
    }
    let gb = bytes as f64 / 1073741824.0;
    CheckResult {
        tier: Tier::Info,
        id: id.into(),
        name: name.into(),
        status: Status::Pass,
        message: format!("{} cached packages, {:.1} GB", files, gb),
        fix: Some("Reclaim with: paccache -r".into()),
    }
}

// WARNING WORTH KEEPING: pacman -Qdtq EXITS 1 WHEN THERE ARE NO ORPHANS -- the
// healthiest possible answer returns a failure status. Reading that status would
// report a broken check on a clean machine, which is INT-192 in the wild. Read
// stdout and count lines instead.
pub fn check_orphan_packages() -> CheckResult {
    let id = "orphan_packages";
    let name = "Orphan Packages";
    let out = match Command::new("pacman").args(["-Qdtq"]).output() {
        Ok(o) => o,
        Err(e) => {
            return CheckResult {
                tier: Tier::User,
                id: id.into(),
                name: name.into(),
                status: Status::Unknown,
                message: format!("could not run pacman -- {}", e),
                fix: None,
            }
        }
    };
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let names: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    if names.is_empty() {
        CheckResult {
            tier: Tier::User,
            id: id.into(),
            name: name.into(),
            status: Status::Pass,
            message: "no orphaned packages".into(),
            fix: None,
        }
    } else {
        let preview: Vec<&str> = names.iter().take(5).cloned().collect();
        CheckResult {
            tier: Tier::User,
            id: id.into(),
            name: name.into(),
            status: Status::Warn,
            message: format!(
                "{} orphaned package(s): {}",
                names.len(),
                preview.join(", ")
            ),
            fix: Some("Review, then: pacman -Qdtq | pacman -Rns -".into()),
        }
    }
}
