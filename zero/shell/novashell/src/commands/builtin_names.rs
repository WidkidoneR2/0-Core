//! Every word nsh runs itself: the top-level arms of commands::execute's `match cmd.as_str()`,
//! the words the REPL catches before execute, and its job-control words (is_repl_state_command).
//! ONE LIST, read by the highlighter; a test
//! compares it with the dispatcher's source both ways, so an arm added without a name here,
//! or a name left after its arm is gone, is a red test (INT-267, COMMAND COLOUR TELLS THE
//! TRUTH). Generated from the dispatcher on 2026-10-03; kept by that test since.

/// One group per arm: its first name is the builtin, the rest are the arm's other spellings.
pub const BUILTINS: &[&[&str]] = &[
    &["on"],
    &["help", "h"],
    &["?"],
    &["exit", "quit", "q"],
    &["last_error", "last-error"],
    &["errors"],
    &["last_command", "lc"],
    &["failures"],
    &["observe"],
    &["memory"],
    &["zero-stats", "zstats"],
    &["describe"],
    &["explain"],
    &["open"],
    &["from"],
    &["to"],
    &["where"],
    &["command"],
    &["health"],
    &["events"],
    &["decisions"],
    &["deploys", "deployments"],
    &["friday-patterns"],
    &["bump-versions"],
    &["ade"],
    &["friday"],
    &["intents"],
    &["project", "projects"],
    &["experiment", "experiments"],
    &["tools"],
    &["version"],
    &["schema"],
    &["commits"],
    &["story"],
    &["advise"],
    &["audit"],
    &["nsh", "fsh"],
    &["plan"],
    &["why"],
    &["dry-run"],
    &["clean", "fix"],
    &["dev"],
    &["sandbox", "devbox"],
    &["checkpoint", "cpc"],
    &["let"],
    &["run"],
    &["python", "py"],
    &["js", "node"],
    &["undo"],
    &["pv"],
    &["snapshot"],
    &["rewind", "time-travel"],
    &["debug"],
    &["usage", "usage-report"],
    &["theme"],
    &["since"],
    &["timeline"],
    &["snap-diff"],
    &["dashboard", "dash"],
    &["chart"],
    &["select"],
    &["git"],
    &["search", "s"],
    &["pick"],
    &["where_old_disabled"],
    &["tools-table", "tt"],
    &["events-table", "et"],
    &["audit-table", "at"],
    &["decisions-table", "dt"],
    &["count"],
    &["history-table", "ht", "history"],
    &["history-search", "hs", "hsearch"],
    &["zsh", "bash"],
    &["trace"],
    &["hstats"],
    &["histogram"],
    &["hpattern"],
    &["checkpoints-table", "ct"],
    &["domains"],
    &["logs"],
    &["ps", "processes"],
    &["signals", "sig"],
    &["ports"],
    &["services", "svc"],
    &["files", "ls"],
    &["fd"],
    &["grep"],
    &["tree"],
    &["fstat", "stat"],
    &["peek", "preview"],
    &["exec"],
    &["realpath", "rp"],
    &["time"],
    &["reload"],
    &["source"],
    &["net", "network"],
    &["power", "pwr"],
    &["packages", "pkgs"],
    &["pkg-search", "pkgsearch"],
    &["git-commits", "gc", "git.commits"],
    &["git-files", "gf"],
    &["git-churn", "gchurn", "git.files"],
    &["git-branches", "gbr", "git.branches"],
    &["watch"],
    &["alias"],
    &["unalias"],
    &["plugins"],
    &["z", "zi"],
    &["which"],
    &["echo"],
    &["query"],
    &["goto"],
    &["session"],
    &["history-replay"],
    &["env-save"],
    &["env-load"],
    &["env-rollback"],
    &["env-diff"],
    &["env-export"],
    &["env-import"],
    &["audit-log"],
    &["cmdguard"],
    &["make"],
    &["launch"],
    &["rename"],
    &["replace"],
    &["rspatch"],
    &["patch-multi"],
    &["fdiff"],
    &["show"],
    &["db"],
    &["copy"],
    &["move"],
    &["list"],
    &["read"],
    &["write"],
    &["terminate"],
    &["delete", "del"],
    &["gt"],
    &["find"],
    &["fsearch"],
    &["patch"],
    &["type"],
    &["cat"],
    &["env"],
    &["pwd"],
    &["last"],
    &["save"],
    &["how"],
    &["recall"],
    &["cd"],
    &["d"],
    &["edit"],
    &["clear", "c", "cls"],
    &["cheat"],
    &["it"],
    &["jobs"],
    &["fg"],
    &["bg"],
    &["kill"],
];

/// Whether `word` is a word nsh runs itself.
pub fn is_builtin(word: &str) -> bool {
    BUILTINS.iter().any(|g| g.contains(&word))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// The top-level arms of execute's `match cmd.as_str()`, read the way the cheatsheet read
    /// them: a line at eight spaces that starts with a string and holds `=>`, up to `_ =>`.
    fn dispatched() -> BTreeSet<String> {
        let lines: Vec<&str> = include_str!("mod.rs").lines().collect();
        let start = lines
            .iter()
            .position(|l| l.contains("match cmd.as_str()"))
            .expect("commands/mod.rs has no match cmd.as_str()");
        let mut out = BTreeSet::new();
        for l in &lines[start + 1..] {
            if l.starts_with("        _ =>") {
                break;
            }
            if !l.starts_with("        \"") || !l.contains("=>") {
                continue;
            }
            let head = l.trim_start().split("=>").next().unwrap_or("");
            for seg in head.split('|') {
                let seg = seg.trim();
                if seg.starts_with('"') {
                    if let Some(name) = seg.split('"').nth(1) {
                        out.insert(name.to_string());
                    }
                }
            }
        }
        out
    }

    /// The words the REPL catches by exact text before execute ever sees them.
    fn repl_caught() -> BTreeSet<String> {
        include_str!("../main.rs")
            .lines()
            .filter_map(|l| {
                let word = l.split("line.trim() == \"").nth(1)?.split('"').next()?;
                let plain = !word.is_empty() && word.chars().all(|c| c.is_ascii_lowercase());
                plain.then(|| word.to_string())
            })
            .collect()
    }

    /// The job-control words the REPL routes before execute: the arms of is_repl_state_command.
    fn repl_state() -> BTreeSet<String> {
        let lines: Vec<&str> = include_str!("../main.rs").lines().collect();
        let start = lines
            .iter()
            .position(|l| l.starts_with("pub(crate) fn is_repl_state_command("))
            .expect("main.rs has no is_repl_state_command");
        let m = (start..lines.len())
            .find(|&i| lines[i].contains("match first.as_str() {"))
            .expect("is_repl_state_command has no match first.as_str()");
        let mut out = BTreeSet::new();
        for l in &lines[m + 1..] {
            if l.starts_with("        _ =>") {
                break;
            }
            if l.starts_with("        \"") && l.contains("=>") {
                if let Some(name) = l.trim_start().split('"').nth(1) {
                    out.insert(name.to_string());
                }
            }
        }
        out
    }

    /// INT-267 gate COMMAND COLOUR TELLS THE TRUTH: the list is exactly what nsh runs itself --
    /// every dispatcher arm, REPL catch and job-control word is listed, and every listed name is one of them.
    #[test]
    fn the_list_is_the_dispatcher_and_the_repl() {
        let mut want: BTreeSet<String> = dispatched().union(&repl_caught()).cloned().collect();
        want.extend(repl_state());
        let listed: BTreeSet<String> = BUILTINS
            .iter()
            .flat_map(|g| g.iter())
            .map(|n| n.to_string())
            .collect();
        let unlisted: Vec<&String> = want.difference(&listed).collect();
        let stale: Vec<&String> = listed.difference(&want).collect();
        assert!(
            unlisted.is_empty() && stale.is_empty(),
            "BUILTINS and the dispatcher disagree.\n  run but not listed: {unlisted:?}\n  listed but not run: {stale:?}"
        );
    }
}
