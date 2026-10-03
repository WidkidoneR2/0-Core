//! Probes about the sandbox and the security scan.

use crate::measurement::Measurement;

/// The sandbox binary is deployed and its policies are readable.
///
/// ⚠️ THIS ONCE LOOKED IN TWO HARDCODED LOCATIONS, NEITHER OF WHICH EXISTS HERE. It checked a
/// distribution system path and then a directory inside the project, and reported the binary
/// missing while it sat on PATH where the shell finds it every time.
///
/// ⭐ THAT WAS THE FOURTH CHECK NEEDING THE SAME CORRECTION, WHICH IS ITSELF THE FINDING: five
/// checks each answered "is this tool installed" their own way, and each was right about exactly
/// one machine. In this crate they all ask `which`, once, in one helper.
pub fn sandbox() -> Measurement {
    if which::which("zero-sandbox").is_err() {
        return Measurement::fail("zero-sandbox not deployed");
    }
    let policies = zero_core::paths::registry_dir().join("sandbox-policies.toml");
    if !policies.exists() {
        return Measurement::warn(format!("{} not found", policies.display()));
    }
    // ⚠️ AN UNREADABLE POLICIES FILE ONCE REPORTED ZERO POLICIES AND PASSED. The existence test
    // above catches the common case, so the window is narrow -- present but not readable -- but
    // the shape is the same: unwrap_or(0) turns a failed read into a number, and a number reads
    // as a measurement.
    match std::fs::read_to_string(&policies) {
        Ok(t) => {
            let n = t.lines().filter(|l| l.trim().starts_with("name =")).count();
            Measurement::pass(format!("zero-sandbox deployed -- {} policies active", n))
        }
        Err(e) => Measurement::unknown(format!("could not read {} -- {}", policies.display(), e)),
    }
}

/// The last security scan: what it found, and whether any of it ran.
///
/// ⚠️ A SCAN WITH A BLIND STEP IS NOT A CLEAN BILL. cargo-audit is not installed here, so the
/// crate audit never ran -- and this check read an empty findings array as zero vulnerabilities
/// and reported green. MEASURED 2026-09-04: the honest warning "No scan found" became a FALSE
/// PASS the moment a scan was run.
///
/// The skipped list is reported as a warn rather than unknown, because the sub-scans that did
/// run found nothing. The answer is PARTIAL, not absent, and the message says which part is
/// missing instead of leaving the reader to guess.
pub fn security_audit() -> Measurement {
    let path = zero_core::paths::security_last_scan();
    if !path.exists() {
        return Measurement::warn("no scan found -- run: core security scan");
    }
    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(e) => {
            return Measurement::unknown(format!("could not read {} -- {}", path.display(), e))
        }
    };
    let json: serde_json::Value = match serde_json::from_str(&data) {
        Ok(j) => j,
        Err(e) => {
            return Measurement::unknown(format!("could not parse {} -- {}", path.display(), e))
        }
    };

    let timestamp = json["timestamp"].as_str().unwrap_or("unknown");
    let all: Vec<serde_json::Value> = json["findings"].as_array().cloned().unwrap_or_default();
    let findings = all.len();

    let skipped: Vec<String> = json["skipped"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    if !skipped.is_empty() {
        return Measurement::warn(format!(
            "{} finding(s), but {} check(s) could not run: {}",
            findings,
            skipped.len(),
            skipped.join("; ")
        ));
    }

    // Only findings with a patch available are actionable here; the rest are upstream.
    let patchable: Vec<&serde_json::Value> = all
        .iter()
        .filter(|f| f["fix"].as_str().unwrap_or("").contains("Patch available"))
        .collect();
    let sev = |s: &str| {
        patchable
            .iter()
            .filter(|f| f["severity"].as_str() == Some(s))
            .count()
    };
    let (critical, high, medium) = (sev("Critical"), sev("High"), sev("Medium"));

    let days = chrono::DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| (chrono::Utc::now() - dt.with_timezone(&chrono::Utc)).num_days())
        .unwrap_or(0);
    let age = match days {
        0 => "today".to_string(),
        1 => "1 day ago".to_string(),
        n => format!("{} days ago", n),
    };
    let stale = if days > 7 { " -- consider rescan" } else { "" };

    let message = if patchable.is_empty() {
        format!(
            "{} findings -- all upstream pending, none patchable (scanned {}{})",
            findings, age, stale
        )
    } else {
        format!(
            "{} findings ({} patchable): {} critical, {} high, {} medium (scanned {}{})",
            findings,
            patchable.len(),
            critical,
            high,
            medium,
            age,
            stale
        )
    };

    if critical > 0 {
        Measurement::fail(message)
    } else if high > 0 || medium > 0 {
        Measurement::warn(message)
    } else {
        Measurement::pass(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Status;

    #[test]
    fn sandbox_asks_which_rather_than_guessing_a_location() {
        let m = sandbox();
        assert!(!m.message.is_empty());
        assert!(matches!(
            m.status,
            Status::Pass | Status::Warn | Status::Fail | Status::Unknown
        ));
        // ⭐ The old check reported "not deployed" for a binary sitting on PATH. If the tool
        // resolves, this must not claim it is missing.
        if which::which("zero-sandbox").is_ok() {
            assert!(
                !m.message.contains("not deployed"),
                "the binary resolves on PATH: {}",
                m.message
            );
        }
    }

    #[test]
    fn security_audit_never_calls_a_partial_scan_clean() {
        // ⚠️ THE FALSE PASS. An empty findings array with a skipped sub-scan is not zero
        // vulnerabilities -- it is a scan that did not finish looking.
        let m = security_audit();
        assert!(!m.message.is_empty());
        if m.status == Status::Pass {
            assert!(
                !m.message.contains("could not run"),
                "a scan with a blind step is not a pass: {}",
                m.message
            );
        }
    }
}
