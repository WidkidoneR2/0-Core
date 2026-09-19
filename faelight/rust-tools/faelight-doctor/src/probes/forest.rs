//! Probes about the forest's own machinery: services, the ledger, Friday.

use crate::measurement::Measurement;
use std::process::Command;

/// The per-session services systemd was asked to want.
///
/// ⭐ ASK SYSTEMD WHICH SERVICES SHOULD BE RUNNING, DO NOT NAME THEM HERE. The old list was
/// hardcoded to three -- notify, bar, wsd -- while the target wanted five. faelight-insightd had
/// been invisible for weeks, and faelight-idle became invisible the moment it was wired.
///
/// A PANEL THAT SAYS "3/3 RUNNING" ABOUT A SET OF FIVE IS NOT REPORTING HEALTH, IT IS REPORTING
/// ITS OWN MEMORY. Wiring a service now makes it appear here by itself.
pub fn services_running() -> Measurement {
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
        .map(String::from)
        .collect();
    let total = services.len();
    if total == 0 {
        // ⚠️ NO WANTS MEANS THE QUERY FAILED OR THE TARGET IS ABSENT -- not zero services
        // healthy. 0/0 would render as a clean pass, the free pass this intent exists to remove.
        return Measurement::unknown("could not read faelight-session.target wants");
    }

    // ⚠️ DISTINGUISH "COULD NOT QUERY THE BUS" FROM "SERVICE INACTIVE" (INT-146). The user bus
    // is unreachable in bus-less contexts -- early boot, headless, activation -- where is-active
    // Errs, and treating Err as "down" was a false red. Probe reachability ONCE.
    let bus_reachable = Command::new("systemctl")
        .args(["--user", "show-environment"])
        // .output() captures stdout; .status() would leak the env dump to the terminal.
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !bus_reachable {
        return Measurement::unknown(format!(
            "{} services (session bus unavailable -- could not check in this context)",
            total
        ));
    }

    // Bus is up, so is-active answers are trustworthy: a non-success now genuinely means the
    // service is inactive rather than that we failed to ask.
    let down: Vec<String> = services
        .iter()
        .filter(|name| {
            // by unit name, which survives binary swaps
            Command::new("systemctl")
                .args(["--user", "is-active", "--quiet", name.as_str()])
                .status()
                .map(|s| !s.success())
                .unwrap_or(true)
        })
        .cloned()
        .collect();
    let running = total - down.len();
    if down.is_empty() {
        Measurement::pass(format!("{}/{} services running", running, total))
    } else {
        Measurement::warn(format!(
            "{}/{} running -- down: {}",
            running,
            total,
            down.join(", ")
        ))
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Status;

    #[test]
    fn services_never_reports_zero_of_zero_as_healthy() {
        let m = services_running();
        assert!(!m.message.is_empty());
        if m.status == Status::Pass {
            assert!(
                !m.message.starts_with("0/0"),
                "0/0 is not a healthy set, it is an empty question: {}",
                m.message
            );
        }
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
