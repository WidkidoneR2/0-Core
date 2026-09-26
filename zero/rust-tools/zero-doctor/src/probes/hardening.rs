//! Probes about how hard this machine is to get into, and whether it is safe to update.

use crate::measurement::Measurement;
use std::process::Command;

/// Units that enforce a firewall.
///
/// ⚠️ "firewall" IS ONE DISTRIBUTION'S UNIT NAME, NOT THE QUESTION. NixOS called it
/// firewall.service; Arch machines run ufw, firewalld or plain nftables. Looking for one name
/// REPORTED NO FIREWALL on a machine whose ufw was active and denying traffic -- a false alarm
/// in a check whose own comment says fail safe, not fail loud. A false alarm is its own failure:
/// it teaches the reader to discount the line.
const FIREWALLS: [&str; 5] = ["firewall", "ufw", "firewalld", "nftables", "iptables"];

fn unit_active(unit: &str) -> bool {
    Command::new("systemctl")
        .args(["is-active", unit])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false)
}

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

fn ssh_open_doors(cfg: &str) -> Vec<&'static str> {
    let mut open = vec![];
    let yes = |k: &str| sshd_setting(cfg, k).unwrap_or_else(|| "yes".into()) == "yes";
    if yes("PasswordAuthentication") {
        open.push("password");
    }
    if yes("KbdInteractiveAuthentication") {
        open.push("keyboard-interactive");
    }
    if yes("PermitRootLogin") {
        open.push("root login");
    }
    open
}

/// ⭐ THE MAIN FILE IS NOT THE CONFIGURATION. `sshd_config` line 2 is `Include
/// sshd_config.d/*.conf`, and on Arch THAT DIRECTORY IS WHERE SETTINGS ARE SUPPOSED TO LIVE --
/// the distribution ships its defaults there and expects yours beside them, because editing the
/// packaged file leaves a .pacnew to merge on every upgrade. Reading only the main file reported
/// password doors OPEN on a machine that had explicitly closed them one directory over.
///
/// ⚠️ ORDER MATTERS AND IT IS NOT ALPHABETICAL-AFTER: OpenSSH takes the FIRST value for most
/// keywords and the Include sits at the TOP of the main file, so the includes are read BEFORE
/// the rest of it. Concatenating the other way round would give a CONFIDENT WRONG ANSWER, which
/// is worse than the blind one this replaces.
fn sshd_config() -> Option<String> {
    let main_path = std::path::Path::new("/etc/ssh/sshd_config");
    if !main_path.exists() {
        return None;
    }
    let mut parts: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/etc/ssh/sshd_config.d") {
        let mut confs: Vec<std::path::PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "conf"))
            .collect();
        confs.sort();
        for c in confs {
            if let Ok(t) = std::fs::read_to_string(&c) {
                parts.push(t);
            }
        }
    }
    if let Ok(main) = std::fs::read_to_string(main_path) {
        parts.push(main);
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

/// Firewall enforcing, SSH doors shut, fail2ban noted.
pub fn security_hardening() -> Measurement {
    let firewall_active = FIREWALLS.iter().any(|u| unit_active(u));
    let mut facts: Vec<String> = Vec::new();
    let mut ssh_warn = false;

    facts.push(if firewall_active {
        "firewall on".to_string()
    } else {
        "firewall OFF".to_string()
    });

    match sshd_config() {
        // No sshd_config at all means the daemon is off: nothing to harden, and the safest
        // state there is.
        None => facts.push("sshd off".to_string()),
        Some(cfg) => {
            let open = ssh_open_doors(&cfg);
            if open.is_empty() {
                facts.push("sshd: key-only".to_string());
            } else {
                facts.push(format!("sshd: {} ON", open.join(" + ")));
                ssh_warn = true;
            }
        }
    }

    // ⚠️ fail2ban IS A FACT, NEVER A TICK. Counting jails needs root and this runs unprivileged,
    // so it reports only what it can prove: the process is up. It does NOT claim protection --
    // on 2026-07-17 it was up with ZERO jails while this line said "fail2ban OK".
    if unit_active("fail2ban") {
        facts.push("fail2ban running".to_string());
    }

    // A REAL CONJUNCTION. Not "details > 0".
    let message = facts.join("  ");
    if !firewall_active {
        Measurement::fail(message)
    } else if ssh_warn {
        Measurement::warn(message)
    } else {
        Measurement::pass(message)
    }
}

/// ⭐ THE RECOVERY NAMES A FIREWALL THAT EXISTS ON THIS MACHINE.
///
/// ⚠️ The old text said `networking.firewall.enable = true in configuration.nix` -- NixOS syntax
/// on an Arch box. A red line whose recovery step cannot be taken is worse than no line: the
/// finding is real and the fix is fiction. This asks which firewall is actually installed.
pub fn firewall_recovery() -> String {
    for unit in ["ufw", "firewalld", "nftables"] {
        if which::which(unit).is_ok() {
            return format!("Start it: sudo systemctl enable --now {}", unit);
        }
    }
    "Install a firewall (ufw, firewalld or nftables), then enable it".to_string()
}

/// Whether it is safe to update: kernel current, working tree clean.
///
/// ⚠️ EVERY SIGNAL PRODUCES A BLOCKER, AN UNREADABLE ENTRY, OR A PASS -- none may go quiet. The
/// old body read two NixOS paths that both Err off NixOS, `.ok()` turned them into None, the
/// if-let never matched, and NO BLOCKER WAS PUSHED: it reported "safe to update" having measured
/// only the git half. A MISSING MEASUREMENT RENDERED AS A PASS, which is quieter and therefore
/// worse than a false amber.
pub fn update_readiness() -> Measurement {
    let mut blockers: Vec<String> = Vec::new();
    let mut unreadable: Vec<String> = Vec::new();

    match (
        super::system::running_kernel(),
        super::system::installed_kernels(),
    ) {
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

    let root = zero_core::paths::core_root_string();
    match Command::new("git")
        .args(["-C", &root, "diff", "--quiet", "HEAD"])
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
        Measurement::warn(format!("hold off -- {}", blockers.join("; ")))
    } else if !unreadable.is_empty() {
        Measurement::unknown(format!(
            "could not judge -- unread: {}",
            unreadable.join("; ")
        ))
    } else {
        Measurement::pass("safe to update -- kernel current, tree clean")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Status;

    #[test]
    fn hardening_asks_every_firewall_not_one_name() {
        let m = security_hardening();
        assert!(!m.message.is_empty());
        // ⭐ If ANY firewall unit is active, this must not report the firewall off.
        if FIREWALLS.iter().any(|u| unit_active(u)) {
            assert!(
                !m.message.contains("firewall OFF"),
                "a firewall is active: {}",
                m.message
            );
            assert_ne!(m.status, Status::Fail);
        }
    }

    #[test]
    fn the_firewall_recovery_names_something_that_exists_here() {
        // ⚠️ THE NIXOS STRING THIS REPLACES. If a firewall is installed the advice must name it,
        // not a configuration syntax from a system that was wiped.
        let r = firewall_recovery();
        assert!(!r.contains("configuration.nix"));
        assert!(!r.contains("networking.firewall"));
        if which::which("ufw").is_ok() {
            assert!(
                r.contains("ufw"),
                "ufw is installed but the advice says: {}",
                r
            );
        }
    }

    #[test]
    fn update_readiness_never_passes_on_an_unread_signal() {
        let m = update_readiness();
        assert!(!m.message.is_empty());
        if m.status == Status::Pass {
            assert!(
                !m.message.contains("could not"),
                "a pass may not carry an unread signal: {}",
                m.message
            );
        }
    }
}
