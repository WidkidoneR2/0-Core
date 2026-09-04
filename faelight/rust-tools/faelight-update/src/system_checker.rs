use faelight_core::check::{Checked, Skipped};
use std::process::Command;

/// System packages pending, WITHOUT touching anything.
///
/// checkupdates syncs to a temporary database and lists what pacman would upgrade. Exit 2
/// means no updates, 0 means here they are, anything else is a failure worth reporting.
///
/// INT-192: an absent checkupdates is UNKNOWN, not zero pending. That distinction is the
/// whole point -- reporting no system updates on a machine that could not look is how a
/// stale system reads as a current one.
pub fn check_system_updates() -> Checked<Vec<String>> {
    let out = Command::new("checkupdates")
        .output()
        .map_err(|e| Skipped::new("checkupdates", e))?;
    match out.status.code() {
        // 2 is checkupdates saying there is nothing pending. A real answer, not a failure.
        Some(2) => Ok(Vec::new()),
        Some(0) => {
            let text = String::from_utf8_lossy(&out.stdout);
            Ok(text
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect())
        }
        other => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let why = stderr
                .lines()
                .next()
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| format!("exit {:?}", other));
            Err(Skipped::new("checkupdates", why))
        }
    }
}

// NO update_system(), AND THAT IS THE DECISION. INT-129: the distribution owns system
// packages, and reimplementing that is how you end up fighting it. omarchy-update is also
// unscriptable by design -- it opens with a box saying you cannot stop the update once you
// start, and waits for a keypress. There is no check-only mode and no non-interactive flag.
//
// So this module REPORTS and stops. The operator runs omarchy-update themselves.
