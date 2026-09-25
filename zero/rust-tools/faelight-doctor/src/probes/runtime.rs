//! Probes about what is running now: the toolchain, the network, VMs, and how long boot took.

use crate::measurement::Measurement;
use std::process::Command;

/// The Rust toolchain is present and usable.
pub fn rust_toolchain() -> Measurement {
    let ok = |bin: &str| {
        Command::new(bin)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };
    match (ok("cargo"), ok("rustc")) {
        (true, true) => Measurement::pass("rust toolchain available"),
        (false, false) => Measurement::fail("neither cargo nor rustc is usable"),
        (true, false) => Measurement::fail("cargo works but rustc does not"),
        (false, true) => Measurement::fail("rustc works but cargo does not"),
    }
}

/// Online, and DNS resolving.
///
/// ⭐ THE CONTROL CASE FOR THIS WHOLE PORT. Every warn here is about THE NETWORK -- offline, or
/// online-but-DNS-down. Neither is the check confessing it could not look, which is why neither
/// becomes `unknown`. A connected workstation losing the network is worth knowing.
///
/// Bounded so a down network cannot blow the time budget: a TCP connect to an IP (no DNS) with a
/// 1s cap, then a DNS resolve in a worker thread also capped at 1s.
pub fn network() -> Measurement {
    let addr: std::net::SocketAddr = match "1.1.1.1:443".parse() {
        Ok(a) => a,
        Err(e) => {
            return Measurement::unknown(format!("could not parse the probe address -- {}", e))
        }
    };
    let online =
        std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(1)).is_ok();
    if !online {
        return Measurement::warn("offline -- 1.1.1.1 unreachable");
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
    match rx.recv_timeout(std::time::Duration::from_secs(1)) {
        Ok(true) => Measurement::pass("online -- DNS resolving"),
        Ok(false) => Measurement::warn("online but DNS not resolving"),
        Err(_) => Measurement::warn("online but the DNS probe did not answer within 1s"),
    }
}

/// Running QEMU VMs.
///
/// ⚠️ A RUNNING VM IS NOT A FAULT. VM-first development means VMs are often up by design. This
/// reports the count so a forgotten one gets noticed, and lets the human judge intent.
///
/// ⭐ AND THIS IS THE CHECK INT-222 IS NAMED AFTER. The original returned `Tier::Info` with
/// `Status::Warn` when pgrep could not run -- a warning in a tier the verdict filters out, so
/// the system could say "could not check" and still report Green. Its declaration is now
/// `severities = ["pass"]`, and an unrunnable pgrep is `unknown`, which is always permitted.
pub fn vm_state() -> Measurement {
    match Command::new("pgrep")
        .args(["-f", "-c", "qemu-system"])
        .output()
    {
        // pgrep exits 1 with no matches. That is zero VMs, not a failure to look.
        Ok(o) => {
            let n = String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse::<u32>()
                .unwrap_or(0);
            if n == 0 {
                Measurement::pass("no VMs running")
            } else {
                Measurement::pass(format!("{} QEMU VM(s) running", n))
            }
        }
        Err(e) => Measurement::unknown(format!("could not run pgrep -- {}", e)),
    }
}

/// How long userspace took to become ready.
///
/// Measures USERSPACE only -- the part we control. The full `systemd-analyze` total folds in
/// firmware POST and the wall-clock spent at the LUKS prompt, neither of which is a boot
/// performance signal.
///
/// ⭐ TWO ARMS CORRECTED IN THE PORT. "could not read boot time" and "could not parse userspace
/// boot time" both returned `Warn` -- the check reporting its own blindness as a judgement about
/// the machine. Both are `unknown`.
pub fn boot_time() -> Measurement {
    let out = match Command::new("systemd-analyze").arg("time").output() {
        Ok(o) if o.status.success() => o,
        Ok(o) => return Measurement::unknown(format!("systemd-analyze exited {}", o.status)),
        Err(e) => return Measurement::unknown(format!("could not run systemd-analyze -- {}", e)),
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let userspace = text
        .split("(userspace)")
        .next()
        .filter(|_| text.contains("(userspace)"))
        .and_then(|pre| pre.split_whitespace().last())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let userspace = match userspace {
        Some(u) => u,
        None => {
            return Measurement::unknown("could not find a userspace figure in systemd-analyze")
        }
    };
    // ⚠️ THE THRESHOLD IS TYPED, and this intent says a typed threshold goes stale. 15s is kept
    // from the original so the port changes nothing; deriving it is its own question.
    let slow = userspace.contains("min") || userspace.contains('h');
    let secs = userspace.trim_end_matches('s').parse::<f64>().ok();
    if slow || secs.map(|s| s > 15.0).unwrap_or(false) {
        Measurement::warn(format!(
            "userspace startup {} (over the 15s target)",
            userspace
        ))
    } else {
        Measurement::pass(format!("login ready in {} (userspace)", userspace))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Status;

    #[test]
    fn rust_toolchain_reports() {
        let m = rust_toolchain();
        assert!(matches!(m.status, Status::Pass | Status::Fail));
    }

    #[test]
    fn network_warns_only_about_the_network() {
        // ⭐ THE CONTROL. A warn here must name a network condition, never the probe's own
        // inability to look -- that is what `unknown` is for.
        let m = network();
        assert!(!m.message.is_empty());
        if m.status == Status::Warn {
            assert!(
                m.message.contains("offline") || m.message.contains("DNS"),
                "a warn must be about the network: {}",
                m.message
            );
        }
    }

    #[test]
    fn vm_state_never_warns() {
        // Its declaration is pass-only. A warn would be refused by the engine, so the probe
        // must never produce one -- including when pgrep is missing.
        let m = vm_state();
        assert!(matches!(m.status, Status::Pass | Status::Unknown));
    }

    #[test]
    fn boot_time_says_unknown_rather_than_warning_about_itself() {
        let m = boot_time();
        assert!(!m.message.is_empty());
        if m.status == Status::Warn {
            assert!(
                m.message.contains("startup"),
                "a warn must be about the boot, not about reading it: {}",
                m.message
            );
        }
    }
}
