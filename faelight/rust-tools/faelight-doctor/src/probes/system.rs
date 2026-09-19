//! Probes about the machine itself: its kernel, its disks, its package cache.

use crate::measurement::Measurement;
use std::process::Command;

/// Size and count of the package cache.
///
/// ⭐ A LABEL, BY DECLARATION. Its definition declares `severities = ["pass"]`, so this probe
/// MAY NOT return warn or fail and the engine would refuse it if it tried.
///
/// That is deliberate, and the original check said why: any warning threshold here would be a
/// number somebody typed, and a typed threshold goes stale exactly the way the check count did.
/// It reports the size; a human decides whether that is too much.
pub fn package_cache() -> Measurement {
    let dir = "/var/cache/pacman/pkg";
    let rd = match std::fs::read_dir(dir) {
        Ok(v) => v,
        Err(e) => return Measurement::unknown(format!("could not read {} -- {}", dir, e)),
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
    let gb = bytes as f64 / 1_073_741_824.0;
    Measurement::pass(format!("{} cached packages, {:.1} GB", files, gb))
}

/// The running kernel against what is installed on disk.
///
/// The measurement is two reads and a membership test. Everything else the original 60-line
/// check contained was CheckResult boilerplate, which is now the engine's job.
pub fn reboot_needed() -> Measurement {
    let running = match running_kernel() {
        Ok(v) => v,
        Err(why) => return Measurement::unknown(format!("could not check -- {}", why)),
    };
    let installed = match installed_kernels() {
        Ok(v) => v,
        Err(why) => return Measurement::unknown(format!("could not check -- {}", why)),
    };
    if installed.iter().any(|k| k == &running) {
        Measurement::pass(format!("running kernel {} is the installed one", running))
    } else {
        Measurement::warn(format!(
            "kernel changed since boot -- running {}, installed {}",
            running,
            installed.join(", ")
        ))
    }
}

/// Free space on the root filesystem, against its own size.
///
/// ⚠️ THE THRESHOLD IS DERIVED, NOT TYPED. It is a PERCENTAGE of the filesystem, so a bigger
/// disk does not silently make the check meaningless the way a fixed "warn under 10 GB" would.
pub fn disk_space() -> Measurement {
    let out = match Command::new("df")
        .args(["-B1", "--output=size,avail", "/"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        Ok(o) => return Measurement::unknown(format!("df exited {}", o.status)),
        Err(e) => return Measurement::unknown(format!("could not run df -- {}", e)),
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let nums: Vec<u64> = text
        .lines()
        .nth(1)
        .unwrap_or("")
        .split_whitespace()
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();
    if nums.len() != 2 || nums[0] == 0 {
        return Measurement::unknown("could not read size and available from df".to_string());
    }
    let (size, avail) = (nums[0], nums[1]);
    let free_pct = (avail as f64 / size as f64) * 100.0;
    let gb = avail as f64 / 1_073_741_824.0;
    if free_pct < 10.0 {
        Measurement::warn(format!(
            "{:.1} GB free -- {:.0}% of the filesystem, under the 10% floor",
            gb, free_pct
        ))
    } else {
        Measurement::pass(format!(
            "{:.1} GB free -- {:.0}% of the filesystem",
            gb, free_pct
        ))
    }
}

/// Every binary the system expects is present.
///
/// ⚠️ THE LIST IS HAND-MAINTAINED AND IT DRIFTS, exactly the way the check counts did. It once
/// named a compositor, a terminal and two package-manager tools from the PREVIOUS system --
/// four critical-tier failures for facilities this machine deliberately does not have, which
/// kept a notification on screen that could not be acted on.
///
/// ⏭ The durable fix is deriving this from something declared rather than typed. Until then a
/// wrong entry here reports a CRITICAL failure, so treat it as code, not as a note.
pub fn binaries() -> Measurement {
    const BINS: [&str; 12] = [
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
    let missing: Vec<&str> = BINS
        .iter()
        .copied()
        .filter(|b| which::which(b).is_err())
        .collect();
    if missing.is_empty() {
        Measurement::pass(format!("all {} binaries found", BINS.len()))
    } else {
        Measurement::fail(format!("{} missing: {}", missing.len(), missing.join(", ")))
    }
}

/// Critical kernel events since boot.
///
/// Benign error-priority noise (USB-C, EC, HID init) is the baseline on this hardware, so this
/// judges on CRITICAL-or-worse only and reports the error-level count for transparency.
///
/// ⭐ AND IT CORRECTS A DEFECT IN THE PORT. The original returned `Status::Warn` when it could
/// not read the journal at all -- "could not check" rendered as a judgement, the same confusion
/// `check_vm_state` has. A journal it cannot read is UNKNOWN: the check did not run.
pub fn boot_errors() -> Measurement {
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
        Some(0) => match errs {
            Some(0) | None => Measurement::pass("no kernel errors since last boot"),
            Some(n) => Measurement::pass(format!(
                "no critical kernel errors since boot ({} low-priority notices)",
                n
            )),
        },
        Some(n) => Measurement::warn(format!("{} critical kernel error(s) since last boot", n)),
        None => Measurement::unknown("could not read the kernel journal"),
    }
}

fn running_kernel() -> Result<String, String> {
    match Command::new("uname").arg("-r").output() {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                Err("uname -r returned nothing".into())
            } else {
                Ok(s)
            }
        }
        Err(e) => Err(format!("cannot run uname -- {}", e)),
    }
}

fn installed_kernels() -> Result<Vec<String>, String> {
    let rd = std::fs::read_dir("/usr/lib/modules")
        .map_err(|e| format!("cannot read /usr/lib/modules: {}", e))?;
    let out: Vec<String> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    if out.is_empty() {
        Err("no kernels found in /usr/lib/modules".into())
    } else {
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Status;

    /// ⚠️ THESE RUN AGAINST THE REAL MACHINE, so they assert the SHAPE of the answer rather
    /// than its value -- a test that demanded "3 GB free" would fail on a different disk and
    /// teach nobody anything.
    ///
    /// What they do assert is the rule this whole design exists for: A PROBE NEVER RETURNS A
    /// STATUS IT WAS NOT ABLE TO JUSTIFY. If it could not measure, it says unknown.
    #[test]
    fn binaries_reports_or_says_it_could_not() {
        let m = binaries();
        assert!(!m.message.is_empty());
        assert!(matches!(m.status, Status::Pass | Status::Fail));
    }

    #[test]
    fn boot_errors_never_warns_about_its_own_blindness() {
        // ⭐ THE CORRECTED DEFECT. The original said Warn when the journal was unreadable.
        // Whatever this returns, a warn must be about the KERNEL, not about the check.
        let m = boot_errors();
        assert!(!m.message.is_empty());
        assert!(matches!(
            m.status,
            Status::Pass | Status::Warn | Status::Unknown
        ));
        if m.status == Status::Warn {
            assert!(
                m.message.contains("kernel error"),
                "a warn here must be about the kernel, not about reading the journal: {}",
                m.message
            );
        }
    }

    #[test]
    fn package_cache_reports_or_says_it_could_not() {
        let m = package_cache();
        assert!(!m.message.is_empty());
        assert!(matches!(m.status, Status::Pass | Status::Unknown));
    }

    #[test]
    fn reboot_needed_reports_or_says_it_could_not() {
        let m = reboot_needed();
        assert!(!m.message.is_empty());
        assert!(matches!(
            m.status,
            Status::Pass | Status::Warn | Status::Unknown
        ));
    }

    #[test]
    fn disk_space_reports_or_says_it_could_not() {
        let m = disk_space();
        assert!(!m.message.is_empty());
        assert!(matches!(
            m.status,
            Status::Pass | Status::Warn | Status::Unknown
        ));
    }
}
