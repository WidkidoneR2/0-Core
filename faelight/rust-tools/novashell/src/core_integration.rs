//! INT-230 G2 -- THE ADAPTER BOUNDARY.
//!
//! ONE place where novashell asks about 0-Core. Everything else reads the answer.
//!
//! WHY THIS EXISTS. The G1 census (CORE-COUPLING.md) measured 82 `paths::` calls
//! across 14 functions. 31 are core shell state and already resolve without
//! 0-Core -- XDG plus the `FAELIGHT_STATE_DB` override. The other 51 read
//! 0-Core's directory layout, and `paths.rs` never checks whether any of them
//! exists. So today `intents_dir()` returns a confident path on a machine with
//! no forest and every caller proceeds as though it were there.
//!
//! WHAT THAT COST, measured 2026-09-04 under `HOME=/tmp/g3home`:
//! `intl` printed a formatted ledger reading `Total: 0` and exited 0. An
//! unavailable capability became a successful-looking empty result, which is
//! exactly the invariant INT-227 forbids.
//!
//! ⚠️ THIS MODULE DOES NOT COPY `paths`. It CALLS it. INT-230 rejected vendoring
//! because two authorities over one layout drift, and this ledger has removed
//! that shape repeatedly. `paths` stays the single authority over WHERE; this
//! module owns WHETHER, and answers questions instead of handing out paths.
//!
//! ⚠️ NO `cfg`. INT-230 G6 forbids a compile-time feature flag without a recorded
//! reason to compile two products. There is none. Absence is a runtime fact.
//!
//! THE RETURN-TYPE RULE, and it is the whole point:
//! every 0-Core capability returns `Option`, never an empty collection. A caller
//! must say what absence means for its own display rather than receiving a zero
//! it cannot distinguish from a measurement.

use std::collections::HashSet;
use std::path::PathBuf;

/// Is 0-Core present on this machine?
///
/// THE ONE EXISTENCE CHECK. `paths.rs` has three `exists()` calls in the whole
/// file and none of them asks this -- two resolve the XDG-versus-legacy runtime
/// directory, one checks a font. Every other 0-Core path is returned
/// unconditionally. This is where that stops.
pub fn present() -> bool {
    intents_root().is_dir()
}

fn intents_root() -> PathBuf {
    faelight_core::paths::intents_dir()
}

/// The status a lifecycle folder carries in frontmatter.
///
/// ⚠️ MATCHES `core` DELIBERATELY. `engine/src/domains/intent/mod.rs` filters on
/// `i.status == "in-progress"` (lines 240, 262, 323) -- the FRONTMATTER FIELD,
/// not the folder name and not a content search. `cistart` maintains both
/// together (1026-1035: rewrite the status, then move the file), so they agree.
///
/// THE DEFECT THIS REPLACES: `session.rs:177` and `health_tui.rs:240` each
/// scanned `future/` and tested `content.contains("in-progress")` -- the WHOLE
/// FILE BODY, so an intent whose prose merely mentions the word counted as
/// active. Measured 2026-09-04: the content search returned 5 while
/// `in-progress/` held 4. Not a dead count, a WRONG one, drifting with prose.
const STATUS_IN_PROGRESS: &str = "in-progress";
const STATUS_PLANNED: &str = "planned";

/// One intent, as much of it as the shell needs.
#[derive(Debug, Clone)]
pub struct Intent {
    pub id: String,
    pub status: String,
    pub depends_on: Vec<String>,
}

impl Intent {
    pub fn is_active(&self) -> bool {
        self.status == STATUS_IN_PROGRESS
    }

    pub fn is_planned(&self) -> bool {
        self.status == STATUS_PLANNED
    }
}

/// The ledger, read once.
///
/// ⚠️ FOUR CALLERS USED TO PARSE THIS DIRECTORY THEMSELVES -- `prompt.rs:448`,
/// `session.rs:177`, `digest.rs:30`, `health_tui.rs:240` -- each with its own
/// `read_dir` and its own idea of what counted. That is four owners of one
/// question, which is the shape INT-193 and INT-195 both existed to end. One
/// scan, one parse, four consumers reading the answer.
#[derive(Debug, Clone)]
pub struct Ledger {
    intents: Vec<Intent>,
    complete_ids: HashSet<String>,
}

/// Ask for the ledger.
///
/// `None` means 0-Core is not on this machine. It does NOT mean an empty
/// ledger, and the type makes that impossible to confuse -- which is the
/// difference between `intl` saying the forest is absent and `intl` printing
/// `Total: 0`.
pub fn ledger() -> Option<Ledger> {
    if !present() {
        return None;
    }
    let root = intents_root();
    let mut intents = Vec::new();
    // future/ and in-progress/ are both live work. deferred/ is paused, not
    // abandoned, and is read so a caller can distinguish the two.
    for folder in ["future", "in-progress", "deferred"] {
        collect(&root.join(folder), &mut intents);
    }
    let mut complete_ids = HashSet::new();
    if let Ok(entries) = std::fs::read_dir(root.join("complete")) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(id) = name.split('-').next() {
                if !id.is_empty() {
                    complete_ids.insert(id.to_string());
                }
            }
        }
    }
    Some(Ledger {
        intents,
        complete_ids,
    })
}

fn collect(dir: &PathBuf, out: &mut Vec<Intent>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        // A missing lifecycle folder is not a missing forest. `present()` has
        // already answered that question; this is one folder that may simply
        // hold nothing yet.
        Err(_) => return,
    };
    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.extension().map(|x| x != "md").unwrap_or(true) {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();
        let id = name.split('-').next().unwrap_or("").to_string();
        if id.is_empty() {
            continue;
        }
        out.push(parse(&id, &content));
    }
}

/// Read the frontmatter fields the shell needs.
///
/// ⚠️ FRONTMATTER ONLY. Scanning starts at the top and stops at the closing
/// delimiter, so a `status:` written in prose further down cannot be read as
/// the intent's status. That is precisely the bug the content search had.
fn parse(id: &str, content: &str) -> Intent {
    let mut status = String::new();
    let mut depends_on = Vec::new();
    let mut in_frontmatter = false;
    for (n, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if in_frontmatter {
                break;
            }
            if n == 0 {
                in_frontmatter = true;
                continue;
            }
        }
        // Some intents carry bare `key: value` headers with no `---` fence.
        // Reading the first lines is still bounded, so a prose mention deep in
        // the body cannot reach this.
        if n > 20 {
            break;
        }
        if let Some(v) = trimmed.strip_prefix("status:") {
            status = v.trim().trim_matches(|c| c == '[' || c == ']').to_string();
        } else if let Some(v) = trimmed.strip_prefix("depends_on:") {
            depends_on = v
                .trim()
                .trim_matches(|c| c == '[' || c == ']')
                .split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    Intent {
        id: id.to_string(),
        status,
        depends_on,
    }
}

impl Ledger {
    /// Intents whose frontmatter status is `in-progress`.
    ///
    /// Agrees with `core intent` by construction: same field, same predicate.
    pub fn active(&self) -> Vec<&Intent> {
        self.intents.iter().filter(|i| i.is_active()).collect()
    }

    pub fn active_count(&self) -> usize {
        self.intents.iter().filter(|i| i.is_active()).count()
    }

    /// (blocked, ready) among planned intents.
    ///
    /// A dependency is satisfied when its id is in `complete/`. ⚠️ This is the
    /// NARROW rule; `core` additionally treats a cancelled dependency as
    /// clearing-but-questionable (the G4 contract in INT-213). The shell shows a
    /// count, not a verdict, so it uses the narrow rule and does not restate a
    /// contract it does not own.
    pub fn blocked_ready(&self) -> (usize, usize) {
        let mut blocked = 0;
        let mut ready = 0;
        for intent in self.intents.iter().filter(|i| i.is_planned()) {
            let unmet = intent
                .depends_on
                .iter()
                .any(|d| !self.complete_ids.contains(d.trim_start_matches("INT-")));
            if unmet {
                blocked += 1;
            } else {
                ready += 1;
            }
        }
        (blocked, ready)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_status_is_read_and_prose_is_not() {
        // THE REGRESSION THIS MODULE EXISTS TO PREVENT. The old content search
        // tested `content.contains("in-progress")` against the whole body, so
        // this intent counted as active. It is planned.
        let content = "---\nid: 999\nstatus: planned\n---\n\nThis intent describes what happens when an intent is in-progress.\n";
        let intent = parse("999", content);
        assert_eq!(intent.status, "planned");
        assert!(!intent.is_active());
    }

    #[test]
    fn an_in_progress_intent_is_active() {
        let content = "---\nid: 230\nstatus: in-progress\n---\n\nbody\n";
        let intent = parse("230", content);
        assert!(intent.is_active());
    }

    #[test]
    fn depends_on_is_parsed_as_a_list() {
        let content = "---\nid: 212\nstatus: planned\ndepends_on: [211]\n---\n";
        let intent = parse("212", content);
        assert_eq!(intent.depends_on, vec!["211".to_string()]);
    }

    #[test]
    fn an_empty_depends_on_is_not_a_dependency() {
        let content = "---\nid: 230\nstatus: planned\ndepends_on: []\n---\n";
        let intent = parse("230", content);
        assert!(intent.depends_on.is_empty());
    }
}
