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
    use std::os::unix::fs::PermissionsExt;

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

    // INT-295 G2 and G3. The stand-in HOME is a temp directory. The real HOME
    // is never read or written.
    //
    //   bash zero/shell/novashell/handoff/test-g2-red.sh
    //
    // runs the ignored reinstall case. G3 turned the setup-command case into a
    // normal test: the command is
    // zero/shell/novashell/handoff/novashell-omarchy/novashell-omarchy
    // (G2 looked for a file named novashell-omarchy directly in handoff/; the
    // package is a directory, so the lookup is one level deeper).
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
    /// The `[ -r ... ] &&` guard means a removed package does not make bash
    /// print "No such file" on every terminal.
    const MARKED_SOURCE_LINE: &str =
        "[ -r /usr/share/novashell-omarchy/bashrc-handoff.sh ] && source /usr/share/novashell-omarchy/bashrc-handoff.sh # novashell-omarchy";

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

    /// The setup command inside the package directory. Relative to this crate,
    /// the same way the handoff block is found. G2 looked for a file named
    /// novashell-omarchy directly in handoff/. The package lives in a directory
    /// of that name, so this is one level deeper. PATH is still checked first.
    fn setup_command_place() -> std::path::PathBuf {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shell/novashell/handoff");
        std::fs::canonicalize(&dir)
            .unwrap_or(dir)
            .join("novashell-omarchy/novashell-omarchy")
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

    // INT-295 G3. These run in cargo test -p zero-doctor. They use a temp HOME
    // and refuse the real one. They do not need root, Arch, or makepkg.

    fn require_setup() -> std::path::PathBuf {
        let path = setup_command_place();
        assert!(
            path.is_file(),
            "no novashell-omarchy setup command at {}",
            path.display()
        );
        path
    }

    fn package_dir() -> std::path::PathBuf {
        require_setup()
            .parent()
            .expect("the setup command has a parent directory")
            .to_path_buf()
    }

    fn state_file(home: &std::path::Path) -> std::path::PathBuf {
        home.join(".local/state/novashell-omarchy/bashrc.sha256")
    }

    fn with_marked_line(original: &[u8]) -> Vec<u8> {
        let mut out = original.to_vec();
        if !out.is_empty() && !out.ends_with(b"\n") {
            out.push(b'\n');
        }
        out.extend_from_slice(MARKED_SOURCE_LINE.as_bytes());
        out.push(b'\n');
        out
    }

    fn marked_count(bytes: &[u8]) -> usize {
        String::from_utf8_lossy(bytes)
            .lines()
            .filter(|line| *line == MARKED_SOURCE_LINE)
            .count()
    }

    fn run_setup(home: &std::path::Path, args: &[&str]) -> std::process::Output {
        refuse_real_home(home);
        require_setup();
        let mut cmd = std::process::Command::new(setup_command_place());
        cmd.args(args)
            .env("HOME", home)
            .env("NO_COLOR", "1")
            .env_remove("NSH_ACTIVE")
            .stdin(std::process::Stdio::null());
        cmd.output()
            .unwrap_or_else(|e| panic!("run novashell-omarchy: {e}"))
    }

    fn output_text(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).into_owned()
    }

    fn assert_ok(out: &std::process::Output, what: &str) {
        assert!(
            out.status.success(),
            "{what} exited {}\nstdout:\n{}\nstderr:\n{}",
            out.status,
            output_text(&out.stdout),
            output_text(&out.stderr)
        );
    }

    fn home_with_bashrc(label: &str, bytes: &[u8]) -> StandIn {
        let root = stand_in(label);
        let home = root.0.join("home");
        std::fs::create_dir(&home).unwrap_or_else(|e| panic!("home: {e}"));
        refuse_real_home(&home);
        std::fs::write(home.join(".bashrc"), bytes).unwrap_or_else(|e| panic!("bashrc: {e}"));
        root
    }

    fn home_dir(root: &StandIn) -> std::path::PathBuf {
        root.0.join("home")
    }

    /// Relative paths of regular files, sorted, with their bytes.
    fn files_under(root: &std::path::Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
        fn walk(dir: &std::path::Path, out: &mut Vec<(std::path::PathBuf, Vec<u8>)>) {
            let mut entries: Vec<_> = std::fs::read_dir(dir)
                .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
                .map(|e| e.unwrap().path())
                .collect();
            entries.sort();
            for path in entries {
                if path.is_dir() {
                    walk(&path, out);
                } else {
                    let bytes = std::fs::read(&path)
                        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
                    out.push((path, bytes));
                }
            }
        }
        let mut out = Vec::new();
        walk(root, &mut out);
        out
    }

    #[test]
    fn g3_install_twice_is_the_same_bytes_and_saves_the_sha256() {
        let original = vendored_bashrc().into_bytes();
        let root = home_with_bashrc("g3-twice", &original);
        let home = home_dir(&root);
        let first = run_setup(&home, &["install"]);
        assert_ok(&first, "install");
        let mid = std::fs::read(home.join(".bashrc")).unwrap();
        assert_eq!(
            mid,
            with_marked_line(&original),
            "install added more than the marked line"
        );
        assert_eq!(
            marked_count(&mid),
            1,
            "install must append exactly one marked line"
        );
        let saved = std::fs::read(state_file(&home)).unwrap_or_else(|e| panic!("sha256 file: {e}"));
        let want = format!("{}\n", sha256_hex(&original));
        assert_eq!(
            saved,
            want.as_bytes(),
            "the saved sha256 is not the pre-install bytes"
        );
        let state_after_first = saved;
        let second = run_setup(&home, &["install"]);
        assert_ok(&second, "second install");
        let after = std::fs::read(home.join(".bashrc")).unwrap();
        assert_eq!(after, mid, "running install twice changed ~/.bashrc");
        assert_eq!(
            std::fs::read(state_file(&home)).unwrap(),
            state_after_first,
            "the second install rewrote the saved sha256"
        );
        let said = output_text(&second.stdout) + &output_text(&second.stderr);
        assert!(
            said.contains("nothing was written"),
            "second install did not say it wrote nothing:\n{said}"
        );
    }

    #[test]
    fn g3_revert_restores_the_preinstall_sha256() {
        let cases: [(&str, Vec<u8>); 3] = [
            ("nl", vendored_bashrc().into_bytes()),
            ("nonewline", b"export A=1\nexport B=2".to_vec()),
            ("empty", Vec::new()),
        ];
        for (label, original) in cases {
            let root = home_with_bashrc(label, &original);
            let home = home_dir(&root);
            let before = sha256_hex(&original);
            assert_ok(&run_setup(&home, &["install"]), "install");
            let back = run_setup(&home, &["revert"]);
            assert_ok(&back, "revert");
            let now = std::fs::read(home.join(".bashrc")).unwrap();
            assert_eq!(now, original, "{label}: revert did not restore the bytes");
            assert_eq!(sha256_hex(&now), before, "{label}: sha256 drifted");
            assert_eq!(
                marked_count(&now),
                0,
                "{label}: the marked line is still there"
            );
        }
    }

    #[test]
    fn g3_refuses_an_inline_block_and_writes_nothing() {
        let mut rc = vendored_bashrc();
        if !rc.ends_with('\n') {
            rc.push('\n');
        }
        rc.push_str(&handoff_canonical());
        let root = home_with_bashrc("g3-inline", rc.as_bytes());
        let home = home_dir(&root);
        let before = files_under(&home);
        let out = run_setup(&home, &["install"]);
        assert!(
            !out.status.success(),
            "install of an inline block exited 0\n{}",
            output_text(&out.stdout)
        );
        assert_eq!(
            files_under(&home),
            before,
            "an inline-block refuse changed a file under HOME"
        );
        assert!(
            !state_file(&home).exists(),
            "a refused install wrote the sha256"
        );
        let said = output_text(&out.stderr) + &output_text(&out.stdout);
        assert!(
            said.contains("nothing was written"),
            "refuse did not say nothing was written:\n{said}"
        );
        assert!(
            said.contains("Delete that block"),
            "refuse did not give the delete step:\n{said}"
        );
        assert!(
            said.contains("novashell-omarchy install"),
            "refuse did not give the install step:\n{said}"
        );
        assert!(
            said.contains("if [[ $- == *i* ]]"),
            "refuse did not name the inline block:\n{said}"
        );
    }

    #[test]
    fn g3_install_does_not_touch_terminal_files_or_call_chsh() {
        let script = std::fs::read_to_string(require_setup()).unwrap();
        assert!(!script.contains("chsh"), "the setup command contains chsh");
        let pkgbuild = package_dir().join("PKGBUILD");
        let pkg = std::fs::read_to_string(&pkgbuild)
            .unwrap_or_else(|e| panic!("PKGBUILD {}: {e}", pkgbuild.display()));
        assert!(!pkg.contains("chsh"), "the PKGBUILD contains chsh");
        assert!(
            !pkg.contains("post_install"),
            "the PKGBUILD has a post_install"
        );
        assert!(
            !pkg.lines()
                .any(|line| line.trim_start().starts_with("install=")),
            "the PKGBUILD names a pacman install script"
        );

        let original = vendored_bashrc().into_bytes();
        let root = home_with_bashrc("g3-files", &original);
        let home = home_dir(&root);
        let plants = [
            ".config/ghostty/config",
            ".config/foot/foot.ini",
            ".config/alacritty/alacritty.toml",
            ".config/kitty/kitty.conf",
            ".config/hypr/bindings.conf",
        ];
        for rel in plants {
            let path = home.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, format!("plant {rel}\n")).unwrap();
        }
        let planted: Vec<_> = plants
            .iter()
            .map(|rel| std::fs::read(home.join(rel)).unwrap())
            .collect();
        let before = files_under(&home);
        assert_ok(&run_setup(&home, &["install"]), "install");
        for (rel, bytes) in plants.iter().zip(planted.iter()) {
            assert_eq!(
                std::fs::read(home.join(rel)).unwrap(),
                *bytes,
                "{rel} changed"
            );
        }
        let mut after = files_under(&home);
        let state = state_file(&home);
        after.retain(|(path, _)| path != &state);
        let bashrc = home.join(".bashrc");
        let expected: Vec<_> = before
            .into_iter()
            .map(|(path, bytes)| {
                if path == bashrc {
                    (path, with_marked_line(&original))
                } else {
                    (path, bytes)
                }
            })
            .collect();
        assert_eq!(
            after, expected,
            "install wrote something other than the marked line and the sha256 file"
        );
        assert!(state.is_file(), "the sha256 file was not written");
    }

    #[test]
    fn g3_revert_mismatch_removes_the_line_and_says_so() {
        let original = vendored_bashrc().into_bytes();
        let root = home_with_bashrc("g3-mismatch", &original);
        let home = home_dir(&root);
        assert_ok(&run_setup(&home, &["install"]), "install");
        let mut edited = std::fs::read(home.join(".bashrc")).unwrap();
        edited.extend_from_slice(b"# a note added later\n");
        std::fs::write(home.join(".bashrc"), &edited).unwrap();
        let out = run_setup(&home, &["revert"]);
        assert!(
            !out.status.success(),
            "a sha256 mismatch exited 0\n{}",
            output_text(&out.stdout)
        );
        let now = std::fs::read(home.join(".bashrc")).unwrap();
        assert_eq!(marked_count(&now), 0, "the marked line is still there");
        let mut expect = original.clone();
        expect.extend_from_slice(b"# a note added later\n");
        assert_eq!(now, expect, "revert changed more than the marked line");
        let said = output_text(&out.stderr) + &output_text(&out.stdout);
        assert!(
            said.contains("does not match"),
            "mismatch was not said loudly:\n{said}"
        );
        assert!(
            said.contains("nothing else was changed"),
            "mismatch did not say what was left alone:\n{said}"
        );
    }

    #[test]
    fn g3_status_reports_the_line_the_handoff_the_off_file_and_nsh() {
        let original = vendored_bashrc().into_bytes();
        let root = home_with_bashrc("g3-status", &original);
        let home = home_dir(&root);
        let before = run_setup(&home, &["status"]);
        assert_ok(&before, "status");
        let text = output_text(&before.stdout);
        assert!(text.contains("line: absent"), "{text}");
        assert!(text.contains("off: absent"), "{text}");
        assert!(text.contains("nsh: missing"), "{text}");
        assert!(
            text.contains("/usr/share/novashell-omarchy/bashrc-handoff.sh"),
            "{text}"
        );
        assert!(
            text.contains(&format!("{}/.config/nsh/off", home.display())),
            "{text}"
        );
        assert!(
            text.contains(&format!("{}/.local/bin/nsh", home.display())),
            "{text}"
        );
        assert!(
            !text.contains('\u{1b}'),
            "status on a pipe used color:\n{text}"
        );
        let handoff = std::path::Path::new("/usr/share/novashell-omarchy/bashrc-handoff.sh");
        if handoff.is_file() {
            assert!(text.contains("handoff: present"), "{text}");
        } else {
            assert!(text.contains("handoff: absent"), "{text}");
        }

        assert_ok(&run_setup(&home, &["install"]), "install");
        let nsh = home.join(".local/bin/nsh");
        std::fs::create_dir_all(nsh.parent().unwrap()).unwrap();
        std::fs::write(&nsh, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&nsh, std::fs::Permissions::from_mode(0o755)).unwrap();
        let off = home.join(".config/nsh/off");
        std::fs::create_dir_all(off.parent().unwrap()).unwrap();
        std::fs::write(&off, "").unwrap();
        let after = run_setup(&home, &["status"]);
        assert_ok(&after, "status after install");
        let text = output_text(&after.stdout);
        assert!(text.contains("line: present"), "{text}");
        assert!(text.contains("off: present"), "{text}");
        assert!(text.contains("nsh: executable"), "{text}");
        assert!(!text.contains('\u{1b}'), "piped status used color:\n{text}");
    }

    #[test]
    fn g3_status_color_follows_the_tty_and_no_color() {
        let original = vendored_bashrc().into_bytes();
        let root = home_with_bashrc("g3-color", &original);
        let home = home_dir(&root);
        let setup = require_setup();

        let piped = std::process::Command::new(&setup)
            .arg("status")
            .env("HOME", &home)
            .env_remove("NO_COLOR")
            .output()
            .unwrap();
        assert_ok(&piped, "piped status");
        let piped_text = output_text(&piped.stdout) + &output_text(&piped.stderr);
        assert!(
            !piped_text.contains('\u{1b}'),
            "a pipe should not be colored:\n{piped_text}"
        );

        let log = root.0.join("color.log");
        let colored = std::process::Command::new("script")
            .args([
                "-q",
                "-e",
                "-c",
                &format!(
                    "env -u NO_COLOR HOME={} {} status",
                    home.display(),
                    setup.display()
                ),
                log.to_str().unwrap(),
            ])
            .output()
            .unwrap_or_else(|e| panic!("script: {e}"));
        assert!(
            colored.status.success(),
            "script exited {}\n{}",
            colored.status,
            output_text(&colored.stderr)
        );
        let shown = output_text(&colored.stdout);
        assert!(
            shown.contains('\u{1b}'),
            "a tty without NO_COLOR produced no color:\n{shown:?}"
        );
        assert!(shown.contains("line:"), "{shown:?}");

        let plain = std::process::Command::new("script")
            .args([
                "-q",
                "-e",
                "-c",
                &format!(
                    "NO_COLOR=1 HOME={} {} status",
                    home.display(),
                    setup.display()
                ),
                log.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(plain.status.success(), "{}", output_text(&plain.stderr));
        let shown = output_text(&plain.stdout);
        assert!(
            !shown.contains('\u{1b}'),
            "NO_COLOR on a tty still colored:\n{shown:?}"
        );
        assert!(shown.contains("line: absent"), "{shown:?}");
    }

    fn bash_rc(
        home: &std::path::Path,
        rc: &std::path::Path,
        extra: &[(&str, &str)],
    ) -> std::process::Output {
        refuse_real_home(home);
        let mut cmd = std::process::Command::new("bash");
        cmd.args([
            "--noprofile",
            "--rcfile",
            rc.to_str().unwrap(),
            "-i",
            "-c",
            "echo AFTER",
        ]);
        cmd.env("HOME", home)
            .env("PS1", "$ ")
            .env_remove("NSH_ACTIVE")
            .env_remove("BASH_ENV");
        for (key, value) in extra {
            cmd.env(key, value);
        }
        cmd.output().unwrap_or_else(|e| panic!("bash: {e}"))
    }

    fn write_stand_in_nsh(home: &std::path::Path, body: &str) {
        let nsh = home.join(".local/bin/nsh");
        std::fs::create_dir_all(nsh.parent().unwrap()).unwrap();
        std::fs::write(&nsh, body).unwrap();
        std::fs::set_permissions(&nsh, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn g3_sourced_handoff_exits_on_0_and_stays_on_139() {
        let root = stand_in("g3-handoff");
        let home = home_dir(&root);
        std::fs::create_dir(&home).unwrap();
        refuse_real_home(&home);
        let share = root.0.join("usr/share/novashell-omarchy");
        std::fs::create_dir_all(&share).unwrap();
        let block = handoff_canonical();
        std::fs::write(share.join("bashrc-handoff.sh"), &block).unwrap();
        let rc = home.join("rc");
        std::fs::write(
            &rc,
            format!(
                "source {}\necho STILL_IN_BASH\n",
                share.join("bashrc-handoff.sh").display()
            ),
        )
        .unwrap();

        write_stand_in_nsh(
            &home,
            "#!/bin/sh\ncat /proc/$PPID/comm > \"$HOME/parent.comm\"\necho ran >> \"$HOME/nsh.log\"\nexit 0\n",
        );
        let clean = bash_rc(&home, &rc, &[]);
        assert_eq!(clean.status.code(), Some(0), "status 0: {clean:?}");
        let stdout = output_text(&clean.stdout);
        assert!(
            !stdout.contains("STILL_IN_BASH"),
            "status 0 stayed in bash:\n{stdout}"
        );
        assert!(
            !stdout.contains("AFTER"),
            "status 0 ran the command after rc:\n{stdout}"
        );
        let parent = std::fs::read_to_string(home.join("parent.comm")).unwrap();
        assert_eq!(parent.trim(), "bash", "nsh's parent was {parent:?}");
        assert!(
            std::fs::read_to_string(home.join("nsh.log"))
                .unwrap()
                .contains("ran"),
            "the stand-in nsh did not run"
        );

        write_stand_in_nsh(&home, "#!/bin/sh\nexit 139\n");
        let crashed = bash_rc(&home, &rc, &[]);
        assert_eq!(
            crashed.status.code(),
            Some(0),
            "status 139 should stay in bash"
        );
        let stdout = output_text(&crashed.stdout);
        let stderr = output_text(&crashed.stderr);
        assert!(
            stdout.contains("STILL_IN_BASH"),
            "status 139 left bash:\n{stdout}"
        );
        assert!(
            stdout.contains("AFTER"),
            "status 139 did not reach the next command:\n{stdout}"
        );
        assert!(
            stderr.contains("nsh crashed (SIGSEGV, status 139); staying in bash."),
            "status 139 stderr:\n{stderr}"
        );

        let off = home.join(".config/nsh/off");
        std::fs::create_dir_all(off.parent().unwrap()).unwrap();
        std::fs::write(&off, "").unwrap();
        let ran = home.join("ran");
        write_stand_in_nsh(&home, "#!/bin/sh\necho RAN > \"$HOME/ran\"\nexit 0\n");
        let held = bash_rc(&home, &rc, &[]);
        assert_eq!(held.status.code(), Some(0));
        assert!(!ran.exists(), "the off file still ran nsh");
        assert!(
            output_text(&held.stderr).contains("nsh handoff is off"),
            "off stderr:\n{}",
            output_text(&held.stderr)
        );
        assert!(output_text(&held.stdout).contains("STILL_IN_BASH"));
        std::fs::remove_file(&off).unwrap();

        let _ = std::fs::remove_file(&ran);
        let door = bash_rc(&home, &rc, &[("NSH_ACTIVE", "1")]);
        assert_eq!(door.status.code(), Some(0));
        assert!(!ran.exists(), "NSH_ACTIVE=1 still ran nsh");
        let stderr = output_text(&door.stderr);
        assert!(!stderr.contains("nsh crashed"), "{stderr}");
        assert!(!stderr.contains("nsh handoff is off"), "{stderr}");
        assert!(output_text(&door.stdout).contains("STILL_IN_BASH"));
    }

    /// Lines bash writes because this session has no terminal. The source line
    /// is judged on what is left.
    fn stderr_from_the_rc(stderr: &str) -> String {
        stderr
            .lines()
            .filter(|line| {
                !line.contains("cannot set terminal process group")
                    && !line.contains("no job control in this shell")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn g3_a_missing_handoff_file_is_silent_and_bash_continues() {
        assert_eq!(
            MARKED_SOURCE_LINE,
            "[ -r /usr/share/novashell-omarchy/bashrc-handoff.sh ] && source /usr/share/novashell-omarchy/bashrc-handoff.sh # novashell-omarchy"
        );
        let root = stand_in("g3-absent");
        let home = home_dir(&root);
        std::fs::create_dir(&home).unwrap();
        refuse_real_home(&home);
        let missing = root.0.join("usr/share/novashell-omarchy/bashrc-handoff.sh");
        assert!(
            !missing.exists(),
            "the stand-in handoff file must be absent"
        );
        let real = std::path::Path::new("/usr/share/novashell-omarchy/bashrc-handoff.sh");
        let line = if !real.exists() {
            MARKED_SOURCE_LINE.to_string()
        } else {
            format!(
                "[ -r {p} ] && source {p} # novashell-omarchy",
                p = missing.display()
            )
        };
        let rc = home.join("rc");
        std::fs::write(&rc, format!("{line}\necho STILL_IN_BASH\n")).unwrap();
        let guarded = bash_rc(&home, &rc, &[]);
        assert_eq!(guarded.status.code(), Some(0), "{guarded:?}");
        let stdout = output_text(&guarded.stdout);
        assert!(
            stdout.contains("STILL_IN_BASH"),
            "bash did not continue:\n{stdout}"
        );
        assert!(
            stdout.contains("AFTER"),
            "bash did not reach the next command:\n{stdout}"
        );
        let noise = stderr_from_the_rc(&output_text(&guarded.stderr));
        assert!(
            noise.is_empty(),
            "a missing handoff file wrote to stderr:\n{noise}"
        );

        let bare = home.join("bare");
        std::fs::write(
            &bare,
            format!("source {}\necho STILL_IN_BASH\n", missing.display()),
        )
        .unwrap();
        let noisy = bash_rc(&home, &bare, &[]);
        let noise = stderr_from_the_rc(&output_text(&noisy.stderr));
        assert!(
            noise.contains("No such file"),
            "an unguarded source of a missing file was silent:\n{noise}"
        );
    }

    #[test]
    fn g3_pkgbuild_packages_the_handoff_and_the_command() {
        let dir = package_dir();
        let pkgbuild = dir.join("PKGBUILD");
        let pkg = std::fs::read_to_string(&pkgbuild)
            .unwrap_or_else(|e| panic!("PKGBUILD {}: {e}", pkgbuild.display()));
        let version =
            std::fs::read_to_string(dir.join("VERSION")).unwrap_or_else(|e| panic!("VERSION: {e}"));
        let version = version.trim();
        assert_eq!(version, "277.1", "handoff version");
        assert!(
            pkg.contains(&format!("pkgver={version}")),
            "pkgver is not the handoff version {version}"
        );
        assert!(pkg.contains("pkgname=novashell-omarchy"), "{pkg}");
        assert!(pkg.contains("arch=('any')"), "{pkg}");
        assert!(pkg.contains("license=('MIT')"), "{pkg}");
        assert!(
            pkg.contains("depends=('bash' 'coreutils')"),
            "depends are not only bash and coreutils:\n{pkg}"
        );
        assert!(pkg.contains("/usr/share/novashell-omarchy/bashrc-handoff.sh"));
        assert!(pkg.contains("/usr/bin/novashell-omarchy"));
        assert!(
            pkg.contains("source=('bashrc-handoff.sh' 'novashell-omarchy')"),
            "source= does not name both files:\n{pkg}"
        );
        assert!(
            pkg.contains("\"$srcdir/bashrc-handoff.sh\""),
            "package() does not install the checksummed handoff source"
        );
        assert!(
            !pkg.contains("$startdir/../"),
            "package() reads the handoff from outside the checksummed sources"
        );
        assert!(!pkg.contains("post_install"));
        assert!(!pkg.contains("chsh"));
        assert!(!pkg
            .lines()
            .any(|line| line.trim_start().starts_with("install=")));
        for entry in std::fs::read_dir(&dir).unwrap() {
            let name = entry.unwrap().file_name();
            let name = name.to_string_lossy();
            assert!(
                !name.ends_with(".install"),
                "pacman install script {name} is in the package directory"
            );
        }
        let script = std::fs::read(dir.join("novashell-omarchy")).unwrap();
        let script_text = String::from_utf8_lossy(&script);
        assert!(
            script_text.contains("VERSION=277.1"),
            "the setup command does not carry handoff version 277.1"
        );
        let link = std::fs::read_link(dir.join("bashrc-handoff.sh"))
            .unwrap_or_else(|e| panic!("bashrc-handoff.sh symlink: {e}"));
        assert_eq!(
            link,
            std::path::PathBuf::from("../bashrc-handoff.sh"),
            "the packaged handoff is not a link to the canonical block"
        );
        let handoff = std::fs::read(dir.join("bashrc-handoff.sh")).unwrap();
        assert_eq!(
            handoff,
            handoff_canonical().into_bytes(),
            "the file the PKGBUILD installs is not the canonical 277 block"
        );
        let sums: Vec<&str> = pkg
            .split(|c: char| !c.is_ascii_hexdigit())
            .filter(|s| s.len() == 64)
            .collect();
        assert_eq!(
            sums,
            vec![sha256_hex(&handoff), sha256_hex(&script)],
            "sha256sums are not the handoff file then the setup command"
        );

        let root = stand_in("g3-help");
        let home = home_dir(&root);
        std::fs::create_dir(&home).unwrap();
        let help = run_setup(&home, &["--help"]);
        assert_ok(&help, "--help");
        let text = output_text(&help.stdout);
        assert!(text.contains("install"), "{text}");
        assert!(text.contains("revert"), "{text}");
        assert!(text.contains("status"), "{text}");
        assert!(files_under(&home).is_empty(), "--help wrote under HOME");
        let unknown = run_setup(&home, &["nope"]);
        assert!(!unknown.status.success(), "an unknown command exited 0");
        assert!(
            files_under(&home).is_empty(),
            "an unknown command wrote under HOME"
        );
    }
}
