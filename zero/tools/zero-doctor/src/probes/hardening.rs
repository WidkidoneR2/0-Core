//! Probes about how hard this machine is to get into, and whether it is safe to update.

use crate::measurement::Measurement;
use std::process::Command;

/// INT-277 G7: THE BASH HANDOFF SURVIVES OMARCHY UPDATES, OR THE DOCTOR SAYS IT DID NOT.
///
/// `omarchy-reinstall-configs` overwrites `~/.bashrc` with no backup, and two installers append
/// below whatever is there. Neither tells anyone, and the first sign would be a terminal that
/// quietly opens bash. This measures three things: the block is in `.bashrc` and identical to the
/// canonical file, `~/.local/bin/nsh` can run, and nothing but blank lines and comments sits
/// below the block. A line appended below it runs only when nsh is off or has crashed, so an
/// nsh session never sees the PATH or activation lines an installer added there.
///
/// ⚠️ NEVER `fail`. The handoff is a choice, and a machine that has not adopted it is not broken.
/// It warns, and the recovery text says how to restore it.
const HANDOFF_IN_CRATE: &str = "handoff/bashrc-handoff.sh";

/// The verdict from three texts and one fact, so every case is testable without a home directory.
fn assess_handoff(rc: &str, block: &str, target_runs: bool) -> Measurement {
    let want: Vec<&str> = block.lines().collect();
    let have: Vec<&str> = rc.lines().collect();
    let first = match want.first() {
        Some(first) => *first,
        None => return Measurement::unknown("the canonical handoff block is empty"),
    };
    let start = match have.iter().position(|line| *line == first) {
        Some(start) => start,
        None => return Measurement::warn("the handoff is not in .bashrc; terminals open bash"),
    };
    let end = start + want.len();
    if have.len() < end || have[start..end] != want[..] {
        return Measurement::warn(
            "the handoff block in .bashrc differs from the canonical block (edited, or only partly restored)",
        );
    }
    if !target_runs {
        return Measurement::warn(
            "~/.local/bin/nsh is missing or not executable; terminals stay in bash",
        );
    }
    let below: Vec<&str> = have[end..]
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    if !below.is_empty() {
        return Measurement::warn(format!(
            "{} line(s) below the handoff run only when nsh is off or has crashed; nsh sessions never see them; first: {}",
            below.len(),
            below[0]
        ));
    }
    Measurement::pass("handoff block intact in .bashrc; nsh is executable; nothing below it")
}

/// `ZERO_DOCTOR_BASHRC` names the rc file to judge instead of `~/.bashrc`, so a copy can be
/// checked without touching the real one.
pub fn shell_handoff() -> Measurement {
    let home = std::env::var("HOME").unwrap_or_default();
    let rc_path = match std::env::var("ZERO_DOCTOR_BASHRC") {
        Ok(path) if !path.is_empty() => path,
        _ => format!("{}/.bashrc", home),
    };
    let crate_dir = match zero_core::paths::crate_rel_dir("novashell") {
        Some(dir) => dir,
        None => return Measurement::unknown("could not locate the novashell crate"),
    };
    let block_path = format!(
        "{}/{}/{}",
        zero_core::paths::core_root_string(),
        crate_dir,
        HANDOFF_IN_CRATE
    );
    let rc = match std::fs::read_to_string(&rc_path) {
        Ok(text) => text,
        Err(e) => return Measurement::unknown(format!("could not read {} ({})", rc_path, e)),
    };
    let block = match std::fs::read_to_string(&block_path) {
        Ok(text) => text,
        Err(e) => {
            return Measurement::unknown(format!(
                "could not read the canonical handoff {} ({})",
                block_path, e
            ))
        }
    };
    let target = format!("{}/.local/bin/nsh", home);
    let runs = std::fs::metadata(&target)
        .map(|m| {
            use std::os::unix::fs::PermissionsExt;
            m.is_file() && m.permissions().mode() & 0o111 != 0
        })
        .unwrap_or(false);
    assess_handoff(&rc, &block, runs)
}

/// Units that enforce a firewall.
///
/// ⚠️ "firewall" IS ONE DISTRIBUTION'S UNIT NAME, NOT THE QUESTION. One OS called it
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
/// ⚠️ The old text gave another OS's firewall syntax
/// on an Arch box. A red line whose recovery step cannot be taken is worse than no line: the
/// finding is real and the fix is fiction. This asks which firewall is actually installed.
pub fn firewall_recovery() -> String {
    for unit in ["ufw", "firewalld", "nftables"] {
        if zero_core::paths::on_path(unit) {
            return format!("Start it: sudo systemctl enable --now {}", unit);
        }
    }
    "Install a firewall (ufw, firewalld or nftables), then enable it".to_string()
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
        // ⚠️ THE OLD STRING THIS REPLACES. If a firewall is installed the advice must name it,
        // not a configuration syntax from a system that was wiped.
        let r = firewall_recovery();
        assert!(!r.contains("networking.firewall"));
        if zero_core::paths::on_path("ufw") {
            assert!(
                r.contains("ufw"),
                "ufw is installed but the advice says: {}",
                r
            );
        }
    }

    fn handoff_canonical() -> String {
        std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../shell/novashell/handoff/bashrc-handoff.sh"
        ))
        .expect("the canonical handoff block")
    }

    fn handoff_rc(above: &str, block: &str, below: &str) -> String {
        format!("{above}\n{block}{below}")
    }

    #[test]
    fn shell_handoff_intact_block_passes() {
        let b = handoff_canonical();
        let m = assess_handoff(&handoff_rc("# omarchy\nsource x", &b, ""), &b, true);
        assert_eq!(m.status, Status::Pass, "{}", m.message);
    }

    #[test]
    fn shell_handoff_missing_block_warns() {
        let b = handoff_canonical();
        let m = assess_handoff("# omarchy\nsource x\n", &b, true);
        assert_eq!(m.status, Status::Warn);
        assert!(m.message.contains("not in .bashrc"), "{}", m.message);
    }

    #[test]
    fn shell_handoff_edited_block_warns() {
        let b = handoff_canonical();
        let edited = b.replace("exit \"$nsh_status\"", "true");
        assert_ne!(edited, b, "the edit must change the block");
        let m = assess_handoff(&handoff_rc("# omarchy", &edited, ""), &b, true);
        assert_eq!(m.status, Status::Warn);
        assert!(m.message.contains("differs"), "{}", m.message);
    }

    #[test]
    fn shell_handoff_a_line_appended_below_warns_and_names_it() {
        let b = handoff_canonical();
        let rc = handoff_rc("# omarchy", &b, "export PATH=\"$HOME/.cargo/bin:$PATH\"\n");
        let m = assess_handoff(&rc, &b, true);
        assert_eq!(m.status, Status::Warn);
        assert!(m.message.contains(".cargo/bin"), "{}", m.message);
    }

    #[test]
    fn shell_handoff_blank_lines_and_comments_below_are_fine() {
        let b = handoff_canonical();
        let rc = handoff_rc("# omarchy", &b, "\n# a note\n   \n");
        let m = assess_handoff(&rc, &b, true);
        assert_eq!(m.status, Status::Pass, "{}", m.message);
    }

    #[test]
    fn shell_handoff_a_target_that_cannot_run_warns() {
        let b = handoff_canonical();
        let m = assess_handoff(&handoff_rc("# omarchy", &b, ""), &b, false);
        assert_eq!(m.status, Status::Warn);
        assert!(m.message.contains("not executable"), "{}", m.message);
    }

    // INT-295 G2. Ignored so cargo test -p novashell, nsh-test, d and
    // core intent validate do not run them. Both fail until later gates.
    //
    //   bash zero/shell/novashell/handoff/test-g2-red.sh
    //
    // The stand-in HOME is a temp directory. The real HOME is never read or written.
    // G3 turns g2_there_is_no_novashell_omarchy_setup_command green by adding
    // novashell-omarchy on PATH, or at zero/shell/novashell/handoff/novashell-omarchy.
    // G5 turns g2_reinstall_deletes_the_marked_line_and_nothing_notices green when
    // assess_handoff (the judgment shell_handoff uses) warns and that warning
    // names the marked source line, the token novashell-omarchy.

    /// Omarchy v4.5.0 default/bashrc, commit ac2690cd.
    /// Source: https://github.com/omacom/omarchy/blob/ac2690cd/default/bashrc
    const OMBASHRC_FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../shell/novashell/handoff/omarchy-v4.5.0-default.bashrc"
    );
    const OMBASHRC_SHA: &str = "d78bd5405919c9c0d9aa5a3db47696286aa9c9d0a1c759785719054605da9a4b";
    /// One marked source line. The token is what a notice has to name.
    const MARKED_SOURCE_LINE: &str =
        "source /usr/share/novashell-omarchy/bashrc-handoff.sh # novashell-omarchy";

    struct StandIn(std::path::PathBuf);

    impl Drop for StandIn {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn refuse_real_home(path: &std::path::Path) {
        if let Some(home) = std::env::var_os("HOME") {
            let home = std::path::PathBuf::from(home);
            assert!(
                !path.starts_with(&home),
                "refusing to write under the real HOME {}",
                home.display()
            );
        }
    }

    fn stand_in(label: &str) -> StandIn {
        let path = std::env::temp_dir().join(format!(
            "int295-g2-{}-{}-{}",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        refuse_real_home(&path);
        std::fs::create_dir_all(&path).unwrap_or_else(|e| panic!("stand-in dir: {}", e));
        StandIn(path)
    }

    fn vendored_bashrc() -> String {
        let raw = std::fs::read_to_string(OMBASHRC_FIXTURE)
            .unwrap_or_else(|e| panic!("vendored bashrc {}: {}", OMBASHRC_FIXTURE, e));
        let marker = "# --- omarchy default/bashrc ---\n";
        let payload = raw
            .split_once(marker)
            .expect("fixture is missing the marker line")
            .1;
        assert_eq!(
            payload.lines().count(),
            14,
            "Omarchy v4.5.0 default/bashrc is 14 lines"
        );
        assert_eq!(
            sha256_hex(payload.as_bytes()),
            OMBASHRC_SHA,
            "vendored bashrc bytes drifted from ac2690cd"
        );
        payload.to_string()
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        use std::io::Write;
        let mut child = std::process::Command::new("sha256sum")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("sha256sum: {}", e));
        child
            .stdin
            .take()
            .unwrap()
            .write_all(bytes)
            .unwrap_or_else(|e| panic!("sha256sum write: {}", e));
        let out = child
            .wait_with_output()
            .unwrap_or_else(|e| panic!("sha256sum wait: {}", e));
        assert!(
            out.status.success(),
            "sha256sum failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    }

    /// Beside the handoff block. Relative to this crate, the same way that block is
    /// found. G3 drops the setup command here, or on PATH.
    fn setup_command_place() -> std::path::PathBuf {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shell/novashell/handoff");
        std::fs::canonicalize(&dir)
            .unwrap_or(dir)
            .join("novashell-omarchy")
    }

    /// PATH, then the handoff directory.
    fn setup_command() -> Option<std::path::PathBuf> {
        if let Some(path) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path) {
                let candidate = dir.join("novashell-omarchy");
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        let candidate = setup_command_place();
        if candidate.is_file() {
            Some(candidate)
        } else {
            None
        }
    }

    fn names_marked(text: &str) -> bool {
        text.contains("novashell-omarchy")
    }

    #[test]
    #[ignore = "INT-295 G2 red; G3 turns green"]
    fn g2_there_is_no_novashell_omarchy_setup_command() {
        let found = setup_command();
        assert!(
            found.is_some(),
            "no novashell-omarchy setup command\nlooked on PATH and at {}\nnovashell-omarchy setup: No such file or directory",
            setup_command_place().display()
        );
    }

    #[test]
    #[ignore = "INT-295 G2 red; G5 turns green"]
    fn g2_reinstall_deletes_the_marked_line_and_nothing_notices() {
        let root = stand_in("reinstall");
        let skel = root.0.join("skel");
        let home = root.0.join("home");
        std::fs::create_dir(&skel).unwrap_or_else(|e| panic!("skel: {}", e));
        std::fs::create_dir(&home).unwrap_or_else(|e| panic!("home: {}", e));
        refuse_real_home(&home);

        let base = vendored_bashrc();
        let mut planted = base.clone();
        if !planted.ends_with('\n') {
            planted.push('\n');
        }
        planted.push_str(MARKED_SOURCE_LINE);
        planted.push('\n');
        assert!(
            planted.contains(MARKED_SOURCE_LINE),
            "harness: the marked line was not planted"
        );
        std::fs::write(skel.join(".bashrc"), &base).unwrap_or_else(|e| panic!("skel rc: {}", e));
        std::fs::write(home.join(".bashrc"), &planted).unwrap_or_else(|e| panic!("home rc: {}", e));

        // omarchy-reinstall-configs at ac2690cd: cp -af /etc/skel/. ~/
        let skel_dot = format!("{}/.", skel.display());
        let home_slash = format!("{}/", home.display());
        let cp = std::process::Command::new("cp")
            .args(["-af", &skel_dot, &home_slash])
            .output()
            .unwrap_or_else(|e| panic!("cp -af: {}", e));
        assert!(
            cp.status.success(),
            "harness: cp -af failed: {}",
            String::from_utf8_lossy(&cp.stderr)
        );

        let after = std::fs::read_to_string(home.join(".bashrc"))
            .unwrap_or_else(|e| panic!("read home rc: {}", e));
        assert!(
            !after.contains(MARKED_SOURCE_LINE),
            "harness: cp -af left the marked line in place"
        );

        let stderr = String::from_utf8_lossy(&cp.stderr).into_owned();
        let mut extra: Vec<String> = Vec::new();
        let skel_names: Vec<String> = std::fs::read_dir(&skel)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        for entry in std::fs::read_dir(&home).unwrap() {
            let name = entry.unwrap().file_name().to_string_lossy().into_owned();
            if !skel_names.iter().any(|n| n == &name) {
                extra.push(name);
            }
        }
        extra.sort();

        let mut traced = names_marked(&stderr);
        for name in &extra {
            traced = traced || names_marked(name);
            let body = std::fs::read_to_string(home.join(name)).unwrap_or_default();
            traced = traced || names_marked(&body);
        }
        let verdict = assess_handoff(&after, &handoff_canonical(), true);
        let warned = verdict.status == Status::Warn && names_marked(&verdict.message);
        assert!(
            traced || warned,
            "cp -af deleted the planted marked source line and nothing noticed\n\
             marked line absent: true\n\
             cp stderr: {:?}\n\
             trace files: {:?}\n\
             shell_handoff: {:?} {}\n\
             expected a warning that names the marked source line (novashell-omarchy)",
            stderr,
            extra,
            verdict.status,
            verdict.message
        );
    }
}
