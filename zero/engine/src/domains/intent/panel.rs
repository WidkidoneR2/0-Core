//! INT-330. Open the ledger panel. A missing quickshell is a safe stop:
//! nothing is started, and the install line is the way forward.

use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::errors::{CoreError, CoreResult};

const INSTALL: &str = "sudo pacman -S quickshell";

/// Start `quickshell -p <repo>/zero/panels/ledger`, detached.
pub fn panel() -> CoreResult<()> {
    launch(std::env::var_os("PATH").as_deref())
}

/// `path` is the PATH value to search. None reads the process PATH.
pub fn launch(path: Option<&OsStr>) -> CoreResult<()> {
    let panel_dir = zero_core::paths::panels_dir().join("ledger");
    match zero_core::paths::find_on_path_in("quickshell", path) {
        None => {
            zero_core::refuse::print_stop(&missing_stop());
            Err(CoreError::Reported("quickshell is not on PATH".to_string()))
        }
        Some(bin) => match spawn_detached(&bin, &panel_dir) {
            Ok(()) => Ok(()),
            Err(err) => {
                let reason = format!("quickshell did not start: {err}");
                zero_core::refuse::print_stop(&zero_core::refuse::Stop {
                    safe: true,
                    reason: &reason,
                    changed: &[],
                    finish: INSTALL,
                });
                Err(CoreError::Reported(reason))
            }
        },
    }
}

fn missing_stop() -> zero_core::refuse::Stop<'static> {
    zero_core::refuse::Stop {
        safe: true,
        reason: "quickshell is not on PATH",
        changed: &[],
        finish: INSTALL,
    }
}

fn spawn_detached(bin: &Path, panel_dir: &Path) -> std::io::Result<()> {
    use std::os::unix::process::CommandExt;
    // A new process group, and we do not wait: the window outlives this command.
    let _child = Command::new(bin)
        .arg("-p")
        .arg(panel_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::launch;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn scratch(tag: &str) -> Scratch {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("ledger-panel-{tag}-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }

    /// INT-330 G6. No quickshell on PATH: a green safe stop, and nothing starts.
    #[test]
    fn quickshell_missing_is_a_safe_stop_and_starts_nothing() {
        let dir = scratch("missing");
        fs::write(
            dir.0.join("not-quickshell"),
            "#!/bin/sh\necho ran > \"$0.out\"\n",
        )
        .unwrap();
        let mut perms = fs::metadata(dir.0.join("not-quickshell"))
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(dir.0.join("not-quickshell"), perms).unwrap();
        let err = launch(Some(dir.0.as_os_str())).expect_err("missing quickshell must stop");
        assert!(
            matches!(err, crate::errors::CoreError::Reported(_)),
            "the top level must not print a second line"
        );
        assert!(
            !dir.0.join("not-quickshell.out").exists(),
            "a binary started"
        );
        assert!(!dir.0.join("argv.txt").exists(), "quickshell started");
        let plain = zero_core::refuse::format_stop(&super::missing_stop(), false);
        assert!(
            plain.contains("Stopped safely") && plain.contains("nothing changed"),
            "{plain}"
        );
        assert!(plain.contains("sudo pacman -S quickshell"), "{plain}");
        let green = zero_core::refuse::format_stop(&super::missing_stop(), true);
        assert!(
            green.contains("\u{1b}[32m") && green.contains("Stopped safely"),
            "the safe stop is green: {green:?}"
        );
    }

    /// INT-330 G6. quickshell on PATH is started detached with -p and the panels path.
    #[test]
    fn quickshell_present_starts_detached_with_the_panels_path() {
        let dir = scratch("present");
        let _guard = crate::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let bin = dir.0.join("quickshell");
        fs::write(
            &bin,
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$(dirname \"$0\")/argv.txt\"\n",
        )
        .unwrap();
        let mut perms = fs::metadata(&bin).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&bin, perms).unwrap();
        launch(Some(dir.0.as_os_str())).expect("quickshell is on PATH");
        let argv_path = dir.0.join("argv.txt");
        let mut text = String::new();
        for _ in 0..50 {
            if let Ok(got) = fs::read_to_string(&argv_path) {
                if !got.is_empty() {
                    text = got;
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.first().copied(), Some("-p"), "argv: {text:?}");
        let panel = zero_core::paths::panels_dir().join("ledger");
        assert_eq!(
            lines.get(1).copied(),
            Some(panel.to_str().unwrap()),
            "argv: {text:?}"
        );
    }
}
