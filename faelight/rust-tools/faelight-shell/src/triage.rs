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
pub fn render(sniffles: usize, colds: &[ColdHit], serious: &[String]) -> i32 {
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
    if serious.is_empty() && colds.is_empty() {
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
pub fn run_triage(logfile: Option<&str>) -> i32 {
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
    render(sniffles, &colds, &serious)
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
        assert!(cold_fix("error: getting status of '/nix/store/x-source/foo.rs': No such file")
            .is_some());
    }

    #[test]
    fn infinite_recursion_is_serious() {
        assert!(is_serious("error: infinite recursion encountered"));
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
