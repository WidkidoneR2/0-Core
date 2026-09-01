// INT-140: deploy-output triage -- classify nixos-rebuild/deploy output into
// three severities so the human instantly knows: ignore, apply a known fix, or
// stop and look. This module ONLY reads output; it never alters the deploy or
// its abort behavior. Unrecognized errors are treated as SERIOUS by default
// (whitelist, not catch-all) -- a wrong "all clear" is worse than a false alarm.

use std::io::Read;

/// A recognized cold: the matched line plus the exact fix to show.
pub struct ColdHit {
    pub line: String,
    pub fix: String,
}

/// Returns true if the line is a recognized benign SNIFFLE.
fn is_sniffle(line: &str) -> bool {
    let l = line.to_lowercase();
    (l.contains("git tree") && l.contains("is dirty"))
        || l.contains("builddepsonly will ignore")
        || l.contains("rebuilt since boot")
        || l.contains("generation drift")
        || l.contains("advisory")
        || l.contains("declining health")
        || l.contains("investigate")
        || l.contains("already running")
        || l.contains("reboot to apply")
        || l.contains("reboot to clear")
}

/// If the line is a recognized COLD (real but known fix), return the fix message.
fn cold_fix(line: &str) -> Option<String> {
    let l = line.to_lowercase();
    // Stale Cargo.lock after adding a crate.
    if l.contains("cargo.lock")
        && (l.contains("out of date")
            || l.contains("--locked")
            || l.contains("needs to be updated")
            || l.contains("is not up to date"))
    {
        return Some(
            "New crate not in the lockfile. Fix: run `cargo check --workspace` (no --locked), then retry deploy."
                .to_string(),
        );
    }
    // Untracked flake file (new .rs/.nix the flake can't see).
    if (l.contains("does not exist")
        || l.contains("getting status of")
        || l.contains("is not tracked")
        || l.contains("access to absolute path"))
        && (l.contains(".rs") || l.contains(".nix") || l.contains("path"))
    {
        return Some(
            "A new file isn't tracked by git, so the flake can't see it. Fix: `git add <file>`, then retry deploy."
                .to_string(),
        );
    }
    // home-manager refuses to overwrite a file it did not place.
    if l.contains("would be clobbered") {
        return Some(
            "home-manager will not overwrite a hand-written file. Fix: rm the file, or set force = true on that option."
                .to_string(),
        );
    }
    // Nix syntax error.
    if l.contains("syntax error") || (l.contains("unexpected") && l.contains(".nix")) {
        return Some("Nix syntax error -- check the named file/line above.".to_string());
    }
    // Nix undefined variable.
    if l.contains("undefined variable") {
        return Some(
            "Nix undefined variable -- likely a typo or a missing import at the named location."
                .to_string(),
        );
    }
    None
}

/// Returns true if the line looks like a SERIOUS (heart-attack) error.
fn is_serious(line: &str) -> bool {
    let l = line.to_lowercase();
    l.contains("infinite recursion")
        || l.contains("stack overflow")
        || (l.contains("builder for") && l.contains("failed"))
        || l.contains("error: build of")
        || l.contains("switch failed")
        || l.contains("failed to switch")
        // Activation-phase failures. These do NOT speak the build phase's
        // vocabulary: a home-manager failure printed "Failed to restart",
        // "the following units failed" and "returned non-zero exit status 4"
        // and the triage reported "benign, nothing needs your attention",
        // because the default-serious rule only fires on a line containing
        // "error:" and none of those lines contain the word.
        || l.contains("failed to restart")
        || l.contains("units failed")
        || l.contains("returned non-zero exit status")
}

/// Classify a full deploy log. Returns (sniffle_count, colds, serious_lines).
/// A line that is an unrecognized `error:` becomes serious (default-serious rule).
pub fn classify(log: &str) -> (usize, Vec<ColdHit>, Vec<String>) {
    let mut sniffles = 0usize;
    let mut colds: Vec<ColdHit> = Vec::new();
    let mut serious: Vec<String> = Vec::new();

    for raw in log.lines() {
        let line = raw.trim_end();
        if line.is_empty() {
            continue;
        }
        // Order matters: cold (known fix) first, then sniffle, then serious,
        // then the default-serious catch for any unrecognized error.
        if let Some(fix) = cold_fix(line) {
            colds.push(ColdHit {
                line: line.to_string(),
                fix,
            });
            continue;
        }
        if is_sniffle(line) {
            sniffles += 1;
            continue;
        }
        if is_serious(line) {
            serious.push(line.to_string());
            continue;
        }
        // Default-serious rule: an unrecognized error: line is treated as serious.
        let l = line.to_lowercase();
        if l.contains("error:") || l.starts_with("error") {
            serious.push(line.to_string());
        }
    }
    (sniffles, colds, serious)
}

/// Render a triage summary to stdout. Read-only; prints alongside the raw log
/// (which the deploy script still shows in full). Returns 0 if nothing serious,
/// 1 if serious findings. NOTE: the caller must NOT let this override the
/// rebuild's own exit status -- triage never changes deploy success/failure.
pub fn render(
    sniffles: usize,
    colds: &[ColdHit],
    serious: &[String],
    rebuild_rc: Option<i32>,
) -> i32 {
    println!("\n  🩺 Deploy triage");
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if !serious.is_empty() {
        println!("  🔴 SERIOUS ({}) -- stop and look:", serious.len());
        for s in serious {
            println!("     {}", s);
        }
    }
    if !colds.is_empty() {
        println!("  🟡 KNOWN ISSUE ({}) -- here's the fix:", colds.len());
        for c in colds {
            println!("     • {}", c.line);
            println!("       -> {}", c.fix);
        }
    }
    if sniffles > 0 {
        println!(
            "  🟢 benign: {} informational warning(s) (dirty-tree, build noise, health advisory) -- safe to ignore.",
            sniffles
        );
    }
    // The rebuild exit code answers "did it succeed"; classification answers
    // "why". Kept apart: a non-zero code never promotes a cold or sniffle to
    // serious, and a zero code never suppresses serious evidence. But a FAILED
    // deploy with no recognised evidence is itself a finding -- the pattern list
    // will always trail whatever failure reality invents next.
    let unexplained =
        matches!(rebuild_rc, Some(rc) if rc != 0) && serious.is_empty() && colds.is_empty();
    if unexplained {
        println!(
            "  \u{1F534} UNEXPLAINED -- the deploy FAILED (exit {}) and triage found no cause.",
            rebuild_rc.unwrap_or(0)
        );
        println!("     Read the log directly: /tmp/faelight-deploy.log");
    } else if serious.is_empty() && colds.is_empty() {
        println!("  ✅ nothing needs your attention beyond the benign notes above.");
    }
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if serious.is_empty() {
        0
    } else {
        1
    }
}

/// Entry point for `faelight-shell --triage-deploy [logfile]`.
/// Reads from the given file, or stdin if no path. READ-ONLY.
pub fn run_triage(logfile: Option<&str>, rebuild_rc: Option<i32>) -> i32 {
    let content = match logfile {
        Some(path) => match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("  ✗ triage: could not read {}: {}", path, e);
                return 2;
            }
        },
        None => {
            let mut buf = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
                eprintln!("  ✗ triage: could not read stdin: {}", e);
                return 2;
            }
            buf
        }
    };
    let (sniffles, colds, serious) = classify(&content);
    render(sniffles, &colds, &serious, rebuild_rc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_tree_is_sniffle() {
        assert!(is_sniffle(
            "warning: Git tree '/home/christian/0-core' is dirty"
        ));
    }

    /// INT-227: THE PATH IS INCIDENTAL DATA, and this proves it rather than assuming it.
    /// `is_sniffle` matches the shape -- "git tree" plus "is dirty" on a lowercased line -- so a
    /// checkout belonging to anyone, anywhere, classifies the same way. The first case preserves
    /// today's behaviour; the other two are the portability property, and the third has a SPACE in
    /// the path because that is what a lazier matcher would break on.
    #[test]
    fn dirty_tree_is_a_sniffle_for_any_checkout() {
        for line in [
            "warning: Git tree '/home/christian/0-core' is dirty",
            "warning: Git tree '/home/other/project' is dirty",
            "warning: Git tree '/tmp/some checkout' is dirty",
        ] {
            assert!(
                is_sniffle(line),
                "should classify regardless of path: {line}"
            );
        }
    }

    /// ⚠️ AND THE OTHER HALF: an unrelated git warning must NOT be swallowed. A matcher that fired
    /// on "git tree" alone would classify a real problem as benign, which is worse than missing a
    /// benign one.
    #[test]
    fn an_unrelated_git_warning_is_not_a_sniffle() {
        assert!(!is_sniffle(
            "error: Git tree '/home/other/project' has conflicts"
        ));
        assert!(!is_sniffle("warning: object file is dirty"));
    }

    #[test]
    fn builddeps_is_sniffle() {
        assert!(is_sniffle(
            "evaluation warning: buildDepsOnly will ignore `src` when `dummySrc` is specified"
        ));
    }

    #[test]
    fn cargo_lock_is_cold() {
        assert!(cold_fix("error: The Cargo.lock is out of date").is_some());
    }

    #[test]
    fn untracked_file_is_cold() {
        assert!(
            cold_fix("error: getting status of '/nix/store/x-source/foo.rs': No such file")
                .is_some()
        );
    }

    #[test]
    fn infinite_recursion_is_serious() {
        assert!(is_serious("error: infinite recursion encountered"));
    }

    #[test]
    fn nonzero_exit_with_no_evidence_is_unexplained() {
        // The contract: a failed deploy whose log says nothing recognisable is
        // itself a finding. render() prints it; classify() stays evidence-only.
        let (s, c, sv) = classify("some output nobody has a pattern for yet");
        assert!(sv.is_empty() && c.is_empty(), "precondition: no evidence");
        assert_eq!(
            render(s, &c, &sv, Some(4)),
            0,
            "unexplained must not fake a serious exit"
        );
    }

    #[test]
    fn nonzero_exit_does_not_promote_a_cold() {
        let (_, colds, serious) =
            classify("error: getting status of '/nix/store/x/foo.rs': No such file");
        assert!(!colds.is_empty(), "should be a cold");
        assert!(
            serious.is_empty(),
            "a non-zero exit must not promote a cold to serious"
        );
    }

    #[test]
    fn zero_exit_keeps_serious_evidence() {
        let (_, _, serious) = classify("error: infinite recursion encountered");
        assert!(
            !serious.is_empty(),
            "a zero exit must not suppress serious evidence"
        );
    }

    #[test]
    fn activation_failure_is_serious() {
        // The 2026-08-21 reproduction: a deploy exited 4 and triage said
        // "benign, nothing needs your attention" because no line said "error:".
        let log = "Failed to restart home-manager-christian.service
                   warning: the following units failed: home-manager-christian.service
                   Command 'systemd-run ...' returned non-zero exit status 4.";
        let (_, _, serious) = classify(log);
        assert!(
            !serious.is_empty(),
            "activation failure must not read as benign"
        );
    }

    #[test]
    fn clobbered_file_is_cold() {
        assert!(
            cold_fix("Existing file '/home/x/.config/q/shell.qml' would be clobbered").is_some()
        );
    }

    #[test]
    fn unknown_error_defaults_serious() {
        let (_, colds, serious) = classify("error: something totally unrecognized happened");
        assert!(colds.is_empty());
        assert_eq!(serious.len(), 1);
    }

    #[test]
    fn clean_deploy_only_sniffles() {
        let log = "warning: Git tree is dirty\nevaluation warning: buildDepsOnly will ignore src\n✅ Deploy complete";
        let (sniffles, colds, serious) = classify(log);
        assert_eq!(sniffles, 2);
        assert!(colds.is_empty());
        assert!(serious.is_empty());
    }
}
