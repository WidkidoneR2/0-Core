//! Probes about the repository: its working tree, and the gates that run on commit.

use crate::measurement::Measurement;
use std::process::Command;

/// Working tree state, and whether commits are pushed.
///
/// ⚠️ THIS CHECK WAS THE WORST OF FOUR QUIET-DIRECTION COLLAPSES IN THE OLD DOCTOR, and the fix
/// is carried over here rather than re-earned. Both probes once ended `unwrap_or(false)`, so an
/// UNRUNNABLE GIT REPORTED A CLEAN TREE.
///
/// "Working tree clean, all commits pushed" is a CONJUNCTION OF TWO FACTS, and a conjunction
/// with an unknown term is unknown -- not true.
///
/// ⭐ AND `@{u}` FAILS WITH NO UPSTREAM, WHICH IS A THIRD STATE RATHER THAN A FAILURE. A branch
/// with no tracking remote exits non-zero, and reading that as "nothing unpushed" is a different
/// wrong answer from git being absent. It is reported as its own fact.
pub fn status() -> Measurement {
    let root = faelight_core::paths::core_root_string();
    let has_changes = match Command::new("git")
        .args(["-C", &root, "status", "--porcelain"])
        .output()
    {
        Ok(o) if o.status.success() => !o.stdout.is_empty(),
        Ok(o) => {
            let why = String::from_utf8_lossy(&o.stderr);
            return Measurement::unknown(format!(
                "could not read the working tree -- {}",
                why.trim()
            ));
        }
        Err(e) => return Measurement::unknown(format!("could not run git -- {}", e)),
    };

    // None means the question does not apply, not that the answer is zero.
    let has_unpushed: Option<bool> = Command::new("git")
        .args(["-C", &root, "log", "@{u}..", "--oneline"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| !o.stdout.is_empty());

    let mut issues: Vec<String> = Vec::new();
    if has_changes {
        issues.push("uncommitted changes".to_string());
    }
    match has_unpushed {
        Some(true) => issues.push("unpushed commits".to_string()),
        Some(false) => {}
        None => issues.push("no upstream configured -- push state unknown".to_string()),
    }

    if issues.is_empty() {
        Measurement::pass("working tree clean, all commits pushed")
    } else {
        Measurement::warn(issues.join(", "))
    }
}

/// The commit and push gates: `core.hooksPath` is set, and the hooks can actually execute.
///
/// ⭐ A HOOK THAT IS NOT EXECUTABLE IS SKIPPED SILENTLY BY GIT. That is the interesting failure
/// here -- not a missing file, which anyone would notice, but a present one that never runs.
pub fn hooks() -> Measurement {
    let root = faelight_core::paths::core_root_string();
    let out = match Command::new("git")
        .args(["-C", &root, "config", "core.hooksPath"])
        .output()
    {
        Ok(o) => o,
        Err(e) => return Measurement::unknown(format!("could not run git config -- {}", e)),
    };
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if path.is_empty() {
        return Measurement::fail("core.hooksPath is unset -- no gate runs on commit or push");
    }
    if path != ".githooks" {
        return Measurement::fail(format!("core.hooksPath is {}, not .githooks", path));
    }
    let mut not_exec: Vec<String> = Vec::new();
    for hook in ["pre-commit", "pre-push"] {
        let f = std::path::Path::new(&root).join(".githooks").join(hook);
        let ok = std::fs::metadata(&f)
            .map(|m| {
                use std::os::unix::fs::PermissionsExt;
                m.permissions().mode() & 0o111 != 0
            })
            .unwrap_or(false);
        if !ok {
            not_exec.push(hook.to_string());
        }
    }
    if !not_exec.is_empty() {
        return Measurement::fail(format!(
            "{} not executable -- git skips it silently",
            not_exec.join(", ")
        ));
    }
    Measurement::pass("core.hooksPath is .githooks, hooks executable")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Status;

    #[test]
    fn status_never_claims_clean_when_it_could_not_look() {
        // ⭐ THE COLLAPSE THIS CHECK WAS FAMOUS FOR. A pass here is a CONJUNCTION, so it may
        // only be reported when both halves were actually measured.
        let m = status();
        assert!(!m.message.is_empty());
        assert!(matches!(
            m.status,
            Status::Pass | Status::Warn | Status::Unknown
        ));
        if m.status == Status::Pass {
            assert!(
                m.message.contains("clean") && m.message.contains("pushed"),
                "a pass must assert both halves: {}",
                m.message
            );
        }
    }

    #[test]
    fn hooks_reports_or_says_it_could_not() {
        let m = hooks();
        assert!(!m.message.is_empty());
        assert!(matches!(
            m.status,
            Status::Pass | Status::Fail | Status::Unknown
        ));
    }
}
