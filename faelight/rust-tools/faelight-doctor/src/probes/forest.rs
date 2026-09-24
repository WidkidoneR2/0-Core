//! Probes about Project 0's own machinery: the ledger, Friday.

use crate::measurement::Measurement;
use std::process::Command;

/// The intent ledger validates.
///
/// ⭐ THIS PROBE ASKS THE TOOL THAT OWNS THE ANSWER. Validation lives in the engine's intent
/// domain, and `core intent validate` is its front door. Reimplementing it here would create a
/// SECOND VALIDATOR that can disagree with the first -- which is the defect this check was
/// already fixed for once: it used to be decoration, a hardcoded Pass with a substring match
/// over whole files, and INT-135 made it call the ONE validator.
///
/// ⚠️ THE COST, STATED: if `core` is not on PATH this returns unknown rather than a result. That
/// is correct -- nobody could look -- and it is the same answer `which` gives everywhere else in
/// this crate.
pub fn intent_ledger() -> Measurement {
    if which::which("core").is_err() {
        return Measurement::unknown("core is not on PATH -- cannot validate the ledger");
    }
    let out = match Command::new("core").args(["intent", "validate"]).output() {
        Ok(o) => o,
        Err(e) => {
            return Measurement::unknown(format!("could not run core intent validate -- {}", e))
        }
    };
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        return Measurement::unknown("core intent validate said nothing");
    }
    if out.status.success() {
        // ⚠️ NOT `.last()`. The final line of `core intent validate` is a box-drawing rule,
        // and this probe reported it as its measurement -- DECORATION RENDERED AS A FINDING,
        // which is the exact defect INT-222 exists to remove, reintroduced by the port.
        // Take the last line THAT CARRIES WORDS.
        let finding = text
            .lines()
            .map(str::trim)
            .filter(|l| l.chars().any(|c| c.is_alphanumeric()))
            .last()
            .unwrap_or("validated");
        // The validator prefixes its own tick. The render states the status itself, so
        // carrying it here would put a tick inside a tick.
        let finding = finding.trim_start_matches(['\u{2705}', ' ']);
        Measurement::pass(finding.to_string())
    } else {
        // ⚠️ AND A COUNT IS NOT A FINDING. The old message was "N issue(s) -- first: X", the
        // same half-applied rule schema_validation had. Carry what the validator actually said.
        Measurement::warn(text.lines().take(4).collect::<Vec<_>>().join("; "))
    }
}

/// Friday's learning vital signs.
///
/// Passes while learning; warns only on a genuine stall -- too few patterns, or no new fact in a
/// week. Confidence is shown for the trend but does NOT trigger a warn: LOW CONFIDENCE IS HONEST
/// UNCERTAINTY, NOT ILL HEALTH.
///
/// ⭐ ONE ARM CORRECTED IN THE PORT: "Could not open state.db" was a `Warn`. A database it cannot
/// open is not a stalled Friday, it is a check that did not run.
pub fn friday() -> Measurement {
    let db_path = faelight_core::paths::state_db();
    let db = match rusqlite::Connection::open(&db_path) {
        Ok(d) => d,
        Err(e) => {
            return Measurement::unknown(format!("could not open {} -- {}", db_path.display(), e))
        }
    };
    let one = |sql: &str| -> Option<i64> { db.query_row(sql, [], |r| r.get(0)).ok() };
    let patterns = match one("SELECT COUNT(*) FROM friday_patterns") {
        Some(n) => n,
        None => return Measurement::unknown("friday_patterns is not readable"),
    };
    let facts = match one("SELECT COUNT(*) FROM friday_knowledge") {
        Some(n) => n,
        None => return Measurement::unknown("friday_knowledge is not readable"),
    };
    let avg_conf: f64 = db
        .query_row(
            "SELECT COALESCE(AVG(confidence), 0.0) FROM friday_patterns",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0.0);
    let last_learned =
        one("SELECT COALESCE(MAX(updated_at), 0) FROM friday_knowledge").unwrap_or(0);

    let now = chrono::Utc::now().timestamp();
    let stale = last_learned > 0 && (now - last_learned) > 604_800;

    if patterns < 10 {
        Measurement::warn(format!(
            "warming up -- {} patterns, {} facts",
            patterns, facts
        ))
    } else if stale {
        Measurement::warn(format!(
            "learning stalled -- no new facts in {} days ({} patterns, {} facts)",
            (now - last_learned) / 86_400,
            patterns,
            facts
        ))
    } else {
        Measurement::pass(format!(
            "{} patterns · {} facts · {:.2} avg confidence",
            patterns, facts, avg_conf
        ))
    }
}

/// `cargo doc` emits no warnings.
///
/// Bounded at 3 seconds by a worker thread, so a stuck or locked cargo yields `unknown` rather
/// than hanging the doctor. Warnings here are cosmetic, so they warn and never fail.
///
/// ⭐ PARSE RUSTDOC'S OWN SUMMARY LINE, DO NOT COUNT "warning:" LINES. The summary is itself a
/// line containing the word, so counting would double-count -- confirmed by INT-151 calibration.
pub fn rust_docs() -> Measurement {
    let manifest = faelight_core::paths::core_root_string() + "/faelight/engine/Cargo.toml";
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
            let stderr = String::from_utf8_lossy(&output.stderr);
            // No summary line means clean docs: rustdoc emits none when there is nothing to say.
            let warns: u32 = stderr
                .lines()
                .find(|l| l.contains("generated") && l.contains("warning"))
                .and_then(|l| l.split_whitespace().find_map(|w| w.parse::<u32>().ok()))
                .unwrap_or(0);
            if warns == 0 {
                Measurement::pass("cargo doc clean, 0 warnings")
            } else {
                Measurement::warn(format!("{} rustdoc warning(s)", warns))
            }
        }
        // Timed out, or cargo could not be spawned. Neither is a statement about the docs.
        _ => Measurement::unknown("docs not checked (cargo busy or unavailable)"),
    }
}

/// Structural orphans in the tree, as `faelight-deadwood` counts them.
///
/// ⚠️ A NON-NUMERIC FIELD MEANS THE CHECK COULD NOT RUN. deadwood emits `?` where it could not
/// look, and `unwrap_or(0)` read that as zero findings -- the exact collapse INT-192 ended.
/// PROVEN 2026-09-04: moving config.nsh aside gave `?|?|0|0` and the doctor called it healthy.
pub fn deadwood_scan() -> Measurement {
    let out = match Command::new("faelight-deadwood").arg("--summary").output() {
        Ok(o) if o.status.success() => o,
        _ => {
            return Measurement::unknown(
                "faelight-deadwood did not run -- hygiene is unmeasured, not clean",
            )
        }
    };
    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let parts: Vec<&str> = line.split('|').collect();
    let field = |n: usize| -> Option<usize> { parts.get(n).and_then(|s| s.parse::<usize>().ok()) };
    let (total, registry) = match (field(0), field(3)) {
        (Some(t), Some(r)) => (t, r),
        _ => {
            return Measurement::unknown(format!(
                "deadwood could not complete every check: {}",
                line
            ))
        }
    };
    if registry > 0 {
        Measurement::warn(format!(
            "{} orphans flagged ({} structural: {} registry)",
            total, registry, registry
        ))
    } else {
        Measurement::pass(format!(
            "{} low-priority items (stale .baks); no structural orphans",
            total
        ))
    }
}

/// Packages pacman considers orphaned.
pub fn orphan_packages() -> Measurement {
    let out = match Command::new("pacman").arg("-Qdtq").output() {
        Ok(o) => o,
        Err(e) => return Measurement::unknown(format!("could not run pacman -- {}", e)),
    };
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let names: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if names.is_empty() {
        Measurement::pass("no orphaned packages")
    } else {
        Measurement::warn(format!(
            "{} orphaned package(s): {}",
            names.len(),
            names.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Status;

    #[test]
    fn deadwood_never_reads_a_question_mark_as_zero() {
        // ⭐ THE COLLAPSE INT-192 ENDED. A `?` field is "could not look", not "found none".
        let m = deadwood_scan();
        assert!(!m.message.is_empty());
        if m.status == Status::Pass {
            assert!(
                !m.message.contains('?'),
                "a question mark is not a count: {}",
                m.message
            );
        }
    }

    #[test]
    fn orphan_packages_reports_or_says_it_could_not() {
        let m = orphan_packages();
        assert!(!m.message.is_empty());
        assert!(matches!(
            m.status,
            Status::Pass | Status::Warn | Status::Unknown
        ));
    }

    #[test]
    fn intent_ledger_says_unknown_when_it_cannot_ask() {
        let m = intent_ledger();
        assert!(!m.message.is_empty());
        if which::which("core").is_err() {
            assert_eq!(m.status, Status::Unknown);
        }
    }

    #[test]
    fn friday_never_warns_about_an_unopenable_database() {
        // ⭐ THE CORRECTED ARM. A database it cannot open is not a stalled Friday.
        let m = friday();
        assert!(!m.message.is_empty());
        if m.status == Status::Warn {
            assert!(
                m.message.contains("patterns"),
                "a warn must be about learning, not about opening the db: {}",
                m.message
            );
        }
    }
}
