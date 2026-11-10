//! Probes about how hard this machine is to get into, and whether it is safe to update.

use crate::measurement::Measurement;
use std::process::Command;

/// INT-277 G7: THE BASH HANDOFF SURVIVES OMARCHY UPDATES, OR THE DOCTOR SAYS IT DID NOT.
///
/// `omarchy-reinstall-configs` overwrites `~/.bashrc` with no backup, and two installers append
/// below whatever is there. Neither tells anyone, and the first sign would be a terminal that
/// quietly opens bash. Two shapes count as the handoff. The older one is the 27-line block in
/// `.bashrc`, identical to the canonical file. The packaged one (INT-295) is one marked line
/// that sources `/usr/share/novashell-omarchy/bashrc-handoff.sh`, and that file has to be the
/// same bytes. Either way `~/.local/bin/nsh` has to be able to run, and nothing but blank lines
/// and comments may sit below the handoff. A line appended below it runs only when nsh is off
/// or has crashed, so an nsh session never sees the PATH or activation lines an installer added
/// there.
///
/// ⚠️ NEVER `fail`. The handoff is a choice, and a machine that has not adopted it is not broken.
/// It warns, and the recovery text says how to restore it. An intact inline block with no
/// package installed still passes: that is the laptop until G6.
const HANDOFF_IN_CRATE: &str = "handoff/bashrc-handoff.sh";

/// The file the package installs. Tests pass another path; this is the default.
const PACKAGED_HANDOFF: &str = "/usr/share/novashell-omarchy/bashrc-handoff.sh";

/// The one line `novashell-omarchy install` appends. The token is the marker.
const MARKED_SOURCE_LINE: &str = "[ -r /usr/share/novashell-omarchy/bashrc-handoff.sh ] && source /usr/share/novashell-omarchy/bashrc-handoff.sh # novashell-omarchy";

/// `override_path` is `ZERO_DOCTOR_HANDOFF` when that variable is set. Empty is
/// ignored, so the default path is not replaced by a blank value.
fn packaged_handoff_path(override_path: Option<&str>) -> &str {
    match override_path {
        Some(path) if !path.is_empty() => path,
        _ => PACKAGED_HANDOFF,
    }
}

fn lines_below<'a>(lines: &'a [&'a str], end: usize) -> Vec<&'a str> {
    lines[end..]
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

fn warn_below(below: &[&str]) -> Measurement {
    Measurement::warn(format!(
        "{} line(s) below the handoff run only when nsh is off or has crashed; nsh sessions never see them; first: {}",
        below.len(),
        below[0]
    ))
}

fn warn_nsh_missing() -> Measurement {
    Measurement::warn("~/.local/bin/nsh is missing or not executable; terminals stay in bash")
}

fn assess_inline(have: &[&str], start: usize, want: &[&str], target_runs: bool) -> Measurement {
    let end = start + want.len();
    if have.len() < end || have[start..end] != want[..] {
        return Measurement::warn(
            "the handoff block in .bashrc differs from the canonical block (edited, or only partly restored)",
        );
    }
    if !target_runs {
        return warn_nsh_missing();
    }
    let below = lines_below(have, end);
    if !below.is_empty() {
        return warn_below(&below);
    }
    Measurement::pass("handoff block intact in .bashrc; nsh is executable; nothing below it")
}

fn assess_marked(
    have: &[&str],
    marked_at: usize,
    block: &str,
    packaged_path: &str,
    packaged: Option<&str>,
    target_runs: bool,
) -> Measurement {
    let Some(text) = packaged else {
        return Measurement::warn(format!(
            "the novashell-omarchy marked line is in .bashrc but {packaged_path} is missing; terminals open bash"
        ));
    };
    if text != block {
        return Measurement::warn(format!(
            "the novashell-omarchy marked line is in .bashrc but {packaged_path} differs from the canonical block; terminals open bash"
        ));
    }
    if !target_runs {
        return warn_nsh_missing();
    }
    let below = lines_below(have, marked_at + 1);
    if !below.is_empty() {
        return warn_below(&below);
    }
    Measurement::pass(
        "marked line present; packaged handoff matches the canonical block; nsh is executable; nothing below it",
    )
}

/// The verdict from the rc text, the canonical block, and the packaged file.
///
/// `packaged` is the text of the file the marked line sources. `None` means that
/// file is missing. `packaged_path` is where it was read from: the default under
/// `/usr/share`, or the path a test passed in. Every case is decidable without a
/// home directory, and none of them returns `fail`.
fn assess_handoff(
    rc: &str,
    block: &str,
    packaged_path: &str,
    packaged: Option<&str>,
    target_runs: bool,
) -> Measurement {
    let want: Vec<&str> = block.lines().collect();
    let have: Vec<&str> = rc.lines().collect();
    let first = match want.first() {
        Some(first) => *first,
        None => return Measurement::unknown("the canonical handoff block is empty"),
    };
    let marked: Vec<usize> = have
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == MARKED_SOURCE_LINE)
        .map(|(index, _)| index)
        .collect();
    let inline = have.iter().position(|line| *line == first);
    match (inline, marked.len()) {
        (Some(_), n) if n > 0 => Measurement::warn(
            "both the inline handoff block and the novashell-omarchy marked line are in .bashrc; that is a double handoff",
        ),
        (Some(start), _) => assess_inline(&have, start, &want, target_runs),
        (None, 0) => Measurement::warn(
            "the handoff is not in .bashrc; the novashell-omarchy marked line is gone; terminals open bash; run novashell-omarchy install to put it back",
        ),
        (None, _) => assess_marked(
            &have,
            marked[0],
            block,
            packaged_path,
            packaged,
            target_runs,
        ),
    }
}

fn nsh_executable(home: &str) -> bool {
    let target = format!("{home}/.local/bin/nsh");
    std::fs::metadata(&target)
        .map(|m| {
            use std::os::unix::fs::PermissionsExt;
            m.is_file() && m.permissions().mode() & 0o111 != 0
        })
        .unwrap_or(false)
}

fn canonical_block_path() -> Result<String, Measurement> {
    let crate_dir = match zero_core::paths::crate_rel_dir("novashell") {
        Some(dir) => dir,
        None => return Err(Measurement::unknown("could not locate the novashell crate")),
    };
    Ok(format!(
        "{}/{}/{}",
        zero_core::paths::core_root_string(),
        crate_dir,
        HANDOFF_IN_CRATE
    ))
}

/// Judge one rc and one packaged file. `shell_handoff` resolves the real paths,
/// including `ZERO_DOCTOR_HANDOFF`, and then calls this. Tests pass a stand-in
/// handoff path, and the canonical block by its path in this crate, so nothing
/// is written under `/usr` and the lookup does not require `~/0-core`.
fn shell_handoff_at(
    rc_path: &std::path::Path,
    handoff_path: &std::path::Path,
    home: &std::path::Path,
    block_path: &std::path::Path,
) -> Measurement {
    let rc = match std::fs::read_to_string(rc_path) {
        Ok(text) => text,
        Err(e) => {
            return Measurement::unknown(format!("could not read {} ({})", rc_path.display(), e))
        }
    };
    let block = match std::fs::read_to_string(&block_path) {
        Ok(text) => text,
        Err(e) => {
            return Measurement::unknown(format!(
                "could not read the canonical handoff {} ({})",
                block_path.display(),
                e
            ))
        }
    };
    let packaged = std::fs::read_to_string(handoff_path).ok();
    let runs = nsh_executable(&home.to_string_lossy());
    assess_handoff(
        &rc,
        &block,
        &handoff_path.to_string_lossy(),
        packaged.as_deref(),
        runs,
    )
}

/// `ZERO_DOCTOR_BASHRC` names the rc file to judge instead of `~/.bashrc`.
/// `ZERO_DOCTOR_HANDOFF` names the packaged handoff file instead of
/// `/usr/share/novashell-omarchy/bashrc-handoff.sh`. Either one lets a copy be
/// checked without touching the real home or `/usr`.
pub fn shell_handoff() -> Measurement {
    let home = std::env::var("HOME").unwrap_or_default();
    let rc_path = match std::env::var("ZERO_DOCTOR_BASHRC") {
        Ok(path) if !path.is_empty() => path,
        _ => format!("{home}/.bashrc"),
    };
    let override_path = std::env::var("ZERO_DOCTOR_HANDOFF").ok();
    let handoff_path = packaged_handoff_path(override_path.as_deref()).to_string();
    let block_path = match canonical_block_path() {
        Ok(path) => path,
        Err(measurement) => return measurement,
    };
    shell_handoff_at(
        std::path::Path::new(&rc_path),
        std::path::Path::new(&handoff_path),
        std::path::Path::new(&home),
        std::path::Path::new(&block_path),
    )
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

    /// Inline-block cases. No packaged file: that is the laptop before G6.
    fn judge_rc(rc: &str, block: &str, runs: bool) -> Measurement {
        assess_handoff(rc, block, PACKAGED_HANDOFF, None, runs)
    }

    #[test]
    fn shell_handoff_intact_block_passes() {
        let b = handoff_canonical();
        let m = judge_rc(&handoff_rc("# omarchy\nsource x", &b, ""), &b, true);
        assert_eq!(m.status, Status::Pass, "{}", m.message);
    }

    #[test]
    fn shell_handoff_missing_block_warns() {
        let b = handoff_canonical();
        let m = judge_rc("# omarchy\nsource x\n", &b, true);
        assert_eq!(m.status, Status::Warn);
        assert!(m.message.contains("not in .bashrc"), "{}", m.message);
    }

    #[test]
    fn shell_handoff_edited_block_warns() {
        let b = handoff_canonical();
        let edited = b.replace("exit \"$nsh_status\"", "true");
        assert_ne!(edited, b, "the edit must change the block");
        let m = judge_rc(&handoff_rc("# omarchy", &edited, ""), &b, true);
        assert_eq!(m.status, Status::Warn);
        assert!(m.message.contains("differs"), "{}", m.message);
    }

    #[test]
    fn shell_handoff_a_line_appended_below_warns_and_names_it() {
        let b = handoff_canonical();
        let rc = handoff_rc("# omarchy", &b, "export PATH=\"$HOME/.cargo/bin:$PATH\"\n");
        let m = judge_rc(&rc, &b, true);
        assert_eq!(m.status, Status::Warn);
        assert!(m.message.contains(".cargo/bin"), "{}", m.message);
    }

    #[test]
    fn shell_handoff_blank_lines_and_comments_below_are_fine() {
        let b = handoff_canonical();
        let rc = handoff_rc("# omarchy", &b, "\n# a note\n   \n");
        let m = judge_rc(&rc, &b, true);
        assert_eq!(m.status, Status::Pass, "{}", m.message);
    }

    #[test]
    fn shell_handoff_a_target_that_cannot_run_warns() {
        let b = handoff_canonical();
        let m = judge_rc(&handoff_rc("# omarchy", &b, ""), &b, false);
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
    // G5: assess_handoff warns when the marked line is gone, and that warning
    // names the marked source line, the token novashell-omarchy. The reinstall
    // test below is that case, and it is no longer ignored.

    /// Omarchy v4.5.0 default/bashrc, commit ac2690cd.
    /// Source: https://github.com/omacom/omarchy/blob/ac2690cd/default/bashrc
    const OMBASHRC_FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../shell/novashell/handoff/omarchy-v4.5.0-default.bashrc"
    );
    const OMBASHRC_SHA: &str = "d78bd5405919c9c0d9aa5a3db47696286aa9c9d0a1c759785719054605da9a4b";
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
        let verdict = judge_rc(&after, &handoff_canonical(), true);
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
            .env_remove("NSH_ACTIVE");
        detach_from_controlling_terminal(&mut cmd);
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

        let mut piped_cmd = std::process::Command::new(&setup);
        piped_cmd
            .arg("status")
            .env("HOME", &home)
            .env_remove("NO_COLOR");
        detach_from_controlling_terminal(&mut piped_cmd);
        let piped = piped_cmd.output().unwrap();
        assert_ok(&piped, "piped status");
        let piped_text = output_text(&piped.stdout) + &output_text(&piped.stderr);
        assert!(
            !piped_text.contains('\u{1b}'),
            "a pipe should not be colored:\n{piped_text}"
        );

        let log = root.0.join("color.log");
        let mut colored_cmd = std::process::Command::new("script");
        colored_cmd.args([
            "-q",
            "-e",
            "-c",
            &format!(
                "env -u NO_COLOR HOME={} {} status",
                home.display(),
                setup.display()
            ),
            log.to_str().unwrap(),
        ]);
        detach_from_controlling_terminal(&mut colored_cmd);
        let colored = colored_cmd
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

        let mut plain_cmd = std::process::Command::new("script");
        plain_cmd.args([
            "-q",
            "-e",
            "-c",
            &format!(
                "NO_COLOR=1 HOME={} {} status",
                home.display(),
                setup.display()
            ),
            log.to_str().unwrap(),
        ]);
        detach_from_controlling_terminal(&mut plain_cmd);
        let plain = plain_cmd.output().unwrap();
        assert!(plain.status.success(), "{}", output_text(&plain.stderr));
        let shown = output_text(&plain.stdout);
        assert!(
            !shown.contains('\u{1b}'),
            "NO_COLOR on a tty still colored:\n{shown:?}"
        );
        assert!(shown.contains("line: absent"), "{shown:?}");
    }

    /// stdin is `/dev/null`, and the child calls setsid before exec.
    ///
    /// `process_group(0)` only makes a new process group in the same session.
    /// Measured on a pty: that child stays stopped (state T) and an interactive
    /// bash still opens `/dev/tty` and calls tcsetpgrp, which stops the cargo
    /// test job with SIGTTIN or SIGTTOU. setsid drops the controlling terminal,
    /// so the child cannot take it. The `setsid` symbol is libc, which libstd
    /// already links; this crate does not take a libc dependency for it.
    fn detach_from_controlling_terminal(cmd: &mut std::process::Command) {
        use std::os::unix::process::CommandExt;
        cmd.stdin(std::process::Stdio::null());
        // SAFETY: setsid is async-signal-safe. pre_exec runs in the forked
        // child before any other thread exists there. That child is not a
        // process-group leader, so setsid is allowed to start a session.
        unsafe {
            cmd.pre_exec(|| {
                extern "C" {
                    fn setsid() -> i32;
                }
                if setsid() < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
        }
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
        detach_from_controlling_terminal(&mut cmd);
        cmd.output().unwrap_or_else(|e| panic!("bash: {e}"))
    }

    /// Session id from /proc/pid/stat, the field after pgrp.
    fn proc_session(pid: u32) -> String {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"))
            .unwrap_or_else(|e| panic!("stat {pid}: {e}"));
        let rest = stat
            .rsplit_once(')')
            .map(|(_, tail)| tail.trim_start())
            .unwrap_or("");
        rest.split_whitespace().nth(3).unwrap_or("").to_string()
    }

    fn tagged_line(text: &str, tag: &str) -> String {
        text.lines()
            .find_map(|line| line.trim().strip_prefix(tag).map(|rest| rest.to_string()))
            .unwrap_or_default()
    }

    /// INT-295. `bash -i` with the test's controlling terminal calls tcsetpgrp
    /// and the cargo test job stops (SIGTTIN / SIGTTOU). bash_rc is the spawn
    /// the handoff tests use. It has to start a new session, with stdin
    /// `/dev/null`, so the child has no controlling terminal.
    #[test]
    fn bash_rc_cannot_take_the_controlling_terminal() {
        let root = stand_in("detach");
        let home = home_dir(&root);
        std::fs::create_dir(&home).unwrap();
        refuse_real_home(&home);
        let rc = home.join("rc");
        std::fs::write(
            &rc,
            "stat=$(cat /proc/self/stat)\n\
             rest=${stat##*)}\n\
             set -- $rest\n\
             printf 'sid:%s\\n' \"$4\"\n\
             printf 'tpgid:%s\\n' \"$6\"\n\
             printf 'fd:%s\\n' \"$(readlink /proc/self/fd/0)\"\n",
        )
        .unwrap();
        let out = bash_rc(&home, &rc, &[]);
        assert!(
            out.status.success(),
            "bash did not finish (stopped?): {}\n{}",
            out.status,
            output_text(&out.stderr)
        );
        let text = output_text(&out.stdout);
        let sid = tagged_line(&text, "sid:");
        let parent = proc_session(std::process::id());
        assert_ne!(
            sid, parent,
            "bash stayed in the test's session {parent}\n{text}"
        );
        assert!(
            text.contains("fd:/dev/null"),
            "stdin was not /dev/null:\n{text}"
        );
        assert_eq!(
            tagged_line(&text, "tpgid:"),
            "-1",
            "bash still has a controlling terminal:\n{text}"
        );
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

    /// INT-295 G4. The G0 census of what Omarchy's bash rc does not carry into
    /// nsh (v4.5.0 default/bash at ac2690cd, ruling 5). The package README has
    /// to name every entry. A missing one is a failure, not silence.
    const CENSUS_DOES_NOT_CARRY: &[&str] = &[
        "`c`",
        "`d`",
        "`h`",
        "`ls`",
        "hide nsh builtins",
        "d is the doctor",
        "bash functions",
        "`try`",
        "mise functions",
        "starship",
        "mise activation",
        "fzf key bindings",
        "bash completions",
    ];

    #[test]
    fn g4_readme_names_every_census_entry_that_does_not_carry() {
        let path = package_dir().join("README.md");
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let missing: Vec<&str> = CENSUS_DOES_NOT_CARRY
            .iter()
            .copied()
            .filter(|entry| !text.contains(entry))
            .collect();
        assert!(
            missing.is_empty(),
            "README {} is missing census entries. A missing entry is a failure, not silence.\nmissing:\n{}",
            path.display(),
            missing.join("\n")
        );
    }

    // INT-295 G5. The packaged form: one marked line, and the file it sources
    // is the canonical block. Stand-in homes only. bash and the setup command
    // go through bash_rc and run_setup, which call setsid with stdin /dev/null.

    fn assert_not_fail(m: &Measurement, what: &str) {
        assert_ne!(
            m.status,
            Status::Fail,
            "{what}: shell_handoff must warn, never fail: {}",
            m.message
        );
    }

    fn rc_with_marked_line() -> String {
        let mut rc = vendored_bashrc();
        if !rc.ends_with('\n') {
            rc.push('\n');
        }
        rc.push_str(MARKED_SOURCE_LINE);
        rc.push('\n');
        rc
    }

    fn canonical_block_file() -> std::path::PathBuf {
        std::path::PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../shell/novashell/handoff/bashrc-handoff.sh"
        ))
    }

    #[test]
    fn g5_marked_line_with_matching_packaged_file_passes() {
        let block = handoff_canonical();
        assert!(
            sha256_hex(block.as_bytes()).starts_with("178960349fa03a05"),
            "the canonical block is not sha256 178960349fa03a05"
        );
        let rc = rc_with_marked_line();
        assert!(
            !rc.contains("if [[ $- == *i* ]]"),
            "harness: the rc must be the marked line, not the inline block"
        );
        let stand_in_path = "/tmp/int295-g5-packaged/bashrc-handoff.sh";
        let m = assess_handoff(&rc, &block, stand_in_path, Some(&block), true);
        assert_not_fail(&m, "matching packaged file");
        assert_eq!(m.status, Status::Pass, "{}", m.message);
        assert!(
            m.message.contains("marked line present"),
            "pass did not name the marked line: {}",
            m.message
        );
    }

    #[test]
    fn g5_marked_line_without_the_packaged_file_warns() {
        let block = handoff_canonical();
        let m = assess_handoff(&rc_with_marked_line(), &block, PACKAGED_HANDOFF, None, true);
        assert_not_fail(&m, "missing packaged file");
        assert_eq!(m.status, Status::Warn, "{}", m.message);
        assert!(m.message.contains("novashell-omarchy"), "{}", m.message);
        assert!(m.message.contains("missing"), "{}", m.message);
        assert!(m.message.contains(PACKAGED_HANDOFF), "{}", m.message);
    }

    #[test]
    fn g5_marked_line_with_a_different_packaged_file_warns() {
        let block = handoff_canonical();
        let mut different = block.clone();
        different.push_str("# not the canonical block\n");
        assert_ne!(different.as_bytes(), block.as_bytes());
        let path = "/tmp/int295-g5-differs/bashrc-handoff.sh";
        let m = assess_handoff(&rc_with_marked_line(), &block, path, Some(&different), true);
        assert_not_fail(&m, "different packaged file");
        assert_eq!(m.status, Status::Warn, "{}", m.message);
        assert!(m.message.contains("differs"), "{}", m.message);
        assert!(m.message.contains(path), "{}", m.message);
        assert!(m.message.contains("novashell-omarchy"), "{}", m.message);
    }

    #[test]
    fn g5_intact_inline_block_with_no_package_still_passes() {
        let block = handoff_canonical();
        let rc = handoff_rc("# omarchy\nsource x", &block, "");
        assert!(
            !rc.contains(MARKED_SOURCE_LINE),
            "harness: this case is the inline block alone"
        );
        let m = assess_handoff(&rc, &block, PACKAGED_HANDOFF, None, true);
        assert_not_fail(&m, "inline block");
        assert_eq!(m.status, Status::Pass, "{}", m.message);
        assert!(
            m.message.contains("handoff block intact"),
            "the inline block no longer reports OK: {}",
            m.message
        );
    }

    #[test]
    fn g5_inline_block_and_marked_line_warns_double_handoff() {
        let block = handoff_canonical();
        let mut rc = handoff_rc("# omarchy", &block, "");
        rc.push_str(MARKED_SOURCE_LINE);
        rc.push('\n');
        let m = assess_handoff(&rc, &block, PACKAGED_HANDOFF, Some(&block), true);
        assert_not_fail(&m, "double handoff");
        assert_eq!(m.status, Status::Warn, "{}", m.message);
        assert!(m.message.contains("double handoff"), "{}", m.message);
        assert!(m.message.contains("novashell-omarchy"), "{}", m.message);
    }

    #[test]
    fn g5_missing_line_names_novashell_omarchy_install() {
        let block = handoff_canonical();
        let m = assess_handoff(
            "# omarchy\nsource x\n",
            &block,
            PACKAGED_HANDOFF,
            None,
            true,
        );
        assert_not_fail(&m, "missing line");
        assert_eq!(m.status, Status::Warn, "{}", m.message);
        assert!(m.message.contains("not in .bashrc"), "{}", m.message);
        assert!(m.message.contains("novashell-omarchy"), "{}", m.message);
        assert!(
            m.message.contains("novashell-omarchy install"),
            "the warning does not say to run novashell-omarchy install: {}",
            m.message
        );
    }

    #[test]
    fn g5_a_line_below_the_marked_line_warns() {
        let block = handoff_canonical();
        let mut rc = rc_with_marked_line();
        rc.push_str("export PATH=\"$HOME/.cargo/bin:$PATH\"\n");
        let m = assess_handoff(&rc, &block, PACKAGED_HANDOFF, Some(&block), true);
        assert_not_fail(&m, "line below");
        assert_eq!(m.status, Status::Warn, "{}", m.message);
        assert!(m.message.contains(".cargo/bin"), "{}", m.message);
    }

    #[test]
    fn g5_packaged_path_is_overridable() {
        assert_eq!(packaged_handoff_path(None), PACKAGED_HANDOFF);
        assert_eq!(packaged_handoff_path(Some("")), PACKAGED_HANDOFF);
        let stand = "/tmp/int295-g5-override/bashrc-handoff.sh";
        assert_eq!(packaged_handoff_path(Some(stand)), stand);

        let root = stand_in("g5-override");
        let home = home_dir(&root);
        std::fs::create_dir(&home).unwrap();
        refuse_real_home(&home);
        let handoff = root.0.join("share/bashrc-handoff.sh");
        std::fs::create_dir_all(handoff.parent().unwrap()).unwrap();
        let block = handoff_canonical();
        std::fs::write(&handoff, &block).unwrap();
        std::fs::write(home.join(".bashrc"), rc_with_marked_line()).unwrap();
        write_stand_in_nsh(&home, "#!/bin/sh\nexit 0\n");
        let m = shell_handoff_at(
            &home.join(".bashrc"),
            &handoff,
            &home,
            &canonical_block_file(),
        );
        assert_not_fail(&m, "override path");
        assert_eq!(m.status, Status::Pass, "{}", m.message);
        assert_ne!(
            handoff,
            std::path::PathBuf::from(PACKAGED_HANDOFF),
            "the stand-in path must not be the hardcoded /usr path"
        );
    }

    fn rc_stderr_lines(stderr: &str) -> Vec<String> {
        stderr_from_the_rc(stderr)
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect()
    }

    fn marked_line_sourcing(path: &std::path::Path) -> String {
        format!(
            "[ -r {p} ] && source {p} # novashell-omarchy",
            p = path.display()
        )
    }

    /// The packaged block, under the stand-in root, and an rc that sources it
    /// by that path. /usr/share is not writable here. The bytes are the
    /// canonical file.
    struct PackagedBash {
        _root: StandIn,
        home: std::path::PathBuf,
        rc: std::path::PathBuf,
    }

    fn packaged_bash(label: &str) -> PackagedBash {
        let root = stand_in(label);
        let home = root.0.join("home");
        std::fs::create_dir(&home).unwrap();
        refuse_real_home(&home);
        let share = root.0.join("usr/share/novashell-omarchy");
        std::fs::create_dir_all(&share).unwrap();
        let handoff = share.join("bashrc-handoff.sh");
        let block = handoff_canonical();
        std::fs::write(&handoff, &block).unwrap();
        assert_eq!(
            std::fs::read(&handoff).unwrap(),
            block.as_bytes(),
            "the sourced file is not the canonical handoff"
        );
        let rc = home.join("rc");
        std::fs::write(
            &rc,
            format!("{}\necho STILL_IN_BASH\n", marked_line_sourcing(&handoff)),
        )
        .unwrap();
        PackagedBash {
            _root: root,
            home,
            rc,
        }
    }

    #[test]
    fn g5_crash_statuses_stay_in_bash_with_one_stderr_line() {
        let pack = packaged_bash("g5-crash");
        let cases = [
            (101, "panic"),
            (132, "SIGILL"),
            (134, "SIGABRT"),
            (135, "SIGBUS"),
            (139, "SIGSEGV"),
        ];
        for (status, why) in cases {
            write_stand_in_nsh(&pack.home, &format!("#!/bin/sh\nexit {status}\n"));
            let out = bash_rc(&pack.home, &pack.rc, &[]);
            assert_eq!(
                out.status.code(),
                Some(0),
                "status {status} did not stay in bash: {}",
                output_text(&out.stderr)
            );
            let stdout = output_text(&out.stdout);
            assert!(
                stdout.contains("STILL_IN_BASH"),
                "status {status} left bash:\n{stdout}"
            );
            assert!(
                stdout.contains("AFTER"),
                "status {status} did not reach the next command:\n{stdout}"
            );
            let lines = rc_stderr_lines(&output_text(&out.stderr));
            assert_eq!(
                lines.len(),
                1,
                "status {status} should be exactly one stderr line: {lines:?}"
            );
            let want = format!("nsh crashed ({why}, status {status})");
            assert!(lines[0].contains(&want), "status {status}: {lines:?}");
        }
    }

    #[test]
    fn g5_any_other_status_exits_bash_with_that_status() {
        let pack = packaged_bash("g5-other");
        let others = [0, 1, 2, 100, 130, 131, 133, 136, 137, 138, 140, 200, 255];
        for status in others {
            write_stand_in_nsh(&pack.home, &format!("#!/bin/sh\nexit {status}\n"));
            let out = bash_rc(&pack.home, &pack.rc, &[]);
            assert_eq!(
                out.status.code(),
                Some(status),
                "status {status} did not exit bash with that status\nstdout:\n{}\nstderr:\n{}",
                output_text(&out.stdout),
                output_text(&out.stderr)
            );
            let stdout = output_text(&out.stdout);
            assert!(
                !stdout.contains("STILL_IN_BASH"),
                "status {status} stayed in bash:\n{stdout}"
            );
            assert!(
                !stdout.contains("AFTER"),
                "status {status} ran the command after rc:\n{stdout}"
            );
        }
    }

    #[test]
    fn g5_off_file_lands_in_bash() {
        let pack = packaged_bash("g5-off");
        let ran = pack.home.join("ran");
        write_stand_in_nsh(&pack.home, "#!/bin/sh\necho RAN > \"$HOME/ran\"\nexit 0\n");
        let off = pack.home.join(".config/nsh/off");
        std::fs::create_dir_all(off.parent().unwrap()).unwrap();
        std::fs::write(&off, "").unwrap();
        let out = bash_rc(&pack.home, &pack.rc, &[]);
        assert_eq!(out.status.code(), Some(0), "{:?}", output_text(&out.stderr));
        assert!(!ran.exists(), "the off file still ran nsh");
        assert!(
            output_text(&out.stdout).contains("STILL_IN_BASH"),
            "the off file left bash:\n{}",
            output_text(&out.stdout)
        );
        let lines = rc_stderr_lines(&output_text(&out.stderr));
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(
            lines[0].contains("nsh handoff is off"),
            "off stderr: {lines:?}"
        );
    }

    #[test]
    fn g5_nsh_active_lands_in_bash() {
        let pack = packaged_bash("g5-active");
        let ran = pack.home.join("ran");
        write_stand_in_nsh(&pack.home, "#!/bin/sh\necho RAN > \"$HOME/ran\"\nexit 0\n");
        let out = bash_rc(&pack.home, &pack.rc, &[("NSH_ACTIVE", "1")]);
        assert_eq!(out.status.code(), Some(0), "{:?}", output_text(&out.stderr));
        assert!(!ran.exists(), "NSH_ACTIVE=1 still ran nsh");
        assert!(
            output_text(&out.stdout).contains("STILL_IN_BASH"),
            "NSH_ACTIVE=1 left bash:\n{}",
            output_text(&out.stdout)
        );
        let lines = rc_stderr_lines(&output_text(&out.stderr));
        assert!(
            lines
                .iter()
                .all(|line| !line.contains("nsh crashed") && !line.contains("handoff is off")),
            "NSH_ACTIVE=1 wrote a handoff line: {lines:?}"
        );
    }

    #[test]
    fn g5_reinstall_removes_the_line_and_install_puts_it_back() {
        let root = stand_in("g5-reinstall");
        let skel = root.0.join("skel");
        let home = root.0.join("home");
        std::fs::create_dir(&skel).unwrap();
        std::fs::create_dir(&home).unwrap();
        refuse_real_home(&home);
        let omarchy = root.0.join("omarchy");
        std::fs::create_dir_all(omarchy.join("default/bash")).unwrap();
        std::fs::write(omarchy.join("default/bash/rc"), "# stand-in omarchy rc\n").unwrap();

        let base = vendored_bashrc();
        std::fs::write(skel.join(".bashrc"), &base).unwrap();
        std::fs::write(home.join(".bashrc"), rc_with_marked_line()).unwrap();
        let share = root.0.join("usr/share/novashell-omarchy");
        std::fs::create_dir_all(&share).unwrap();
        let handoff = share.join("bashrc-handoff.sh");
        let block = handoff_canonical();
        std::fs::write(&handoff, &block).unwrap();
        write_stand_in_nsh(&home, "#!/bin/sh\nexit 0\n");
        let bashrc = home.join(".bashrc");
        let block_file = canonical_block_file();

        let planted = shell_handoff_at(&bashrc, &handoff, &home, &block_file);
        assert_not_fail(&planted, "before reinstall");
        assert_eq!(planted.status, Status::Pass, "{}", planted.message);

        let skel_dot = format!("{}/.", skel.display());
        let home_slash = format!("{}/", home.display());
        let cp = std::process::Command::new("cp")
            .args(["-af", &skel_dot, &home_slash])
            .output()
            .unwrap_or_else(|e| panic!("cp -af: {e}"));
        assert!(
            cp.status.success(),
            "cp -af failed: {}",
            output_text(&cp.stderr)
        );
        let wiped = std::fs::read_to_string(&bashrc).unwrap();
        assert!(
            !wiped.contains(MARKED_SOURCE_LINE),
            "cp -af left the marked line in place"
        );

        let omarchy_path = omarchy.to_str().expect("omarchy path is utf-8");
        let bash = bash_rc(&home, &bashrc, &[("OMARCHY_PATH", omarchy_path)]);
        assert_eq!(
            bash.status.code(),
            Some(0),
            "reinstall did not leave a working bash: {}",
            output_text(&bash.stderr)
        );
        assert!(
            output_text(&bash.stdout).contains("AFTER"),
            "bash did not run a command after reinstall:\n{}",
            output_text(&bash.stdout)
        );

        let noticed = shell_handoff_at(&bashrc, &handoff, &home, &block_file);
        assert_not_fail(&noticed, "after reinstall");
        assert_eq!(noticed.status, Status::Warn, "{}", noticed.message);
        assert!(
            noticed.message.contains("novashell-omarchy install"),
            "the doctor did not say to run novashell-omarchy install: {}",
            noticed.message
        );

        assert_ok(&run_setup(&home, &["install"]), "install");
        let restored = std::fs::read(&bashrc).unwrap();
        assert_eq!(
            marked_count(&restored),
            1,
            "install did not put the line back"
        );
        let ok = shell_handoff_at(&bashrc, &handoff, &home, &block_file);
        assert_not_fail(&ok, "after install");
        assert_eq!(ok.status, Status::Pass, "{}", ok.message);
        assert!(
            ok.message.contains("marked line present"),
            "the doctor did not go back to OK: {}",
            ok.message
        );
    }
}
