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
    // INT-230: was `intents_root().is_dir()` against a PRIVATE helper returning
    // a bare PathBuf. That helper was a second definition of the same name as
    // the public accessor below, and the compiler resolved three call sites to
    // the wrong one. One owner: presence IS the accessor answering Some.
    intents_root().is_some()
}

/// The 0-Core rust-tools source tree, when 0-Core is present.
///
/// `None` on a machine without a forest -- which is every packaged install.
pub fn tools_root() -> Option<PathBuf> {
    let dir = faelight_core::paths::rust_tools_dir();
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

/// The Cargo.toml for one tool, or for novashell when `tool` is empty.
///
/// The existence check lives HERE on purpose. `dev_cmd` built this path in its
/// `test` arm and again in its `watch` arm -- byte-identical, forty lines apart
/// -- and the two had drifted: `test` checked the file existed and returned an
/// error, `watch` did not and announced cargo watch on a manifest that was not
/// there. Two owners of one path, agreeing on where it was and disagreeing on
/// whether it was real. One owner now, and an arm cannot skip a check that is
/// not the arm's to skip.
pub fn tool_manifest(tool: &str) -> Option<PathBuf> {
    let root = tools_root()?;
    let manifest = if tool.is_empty() {
        root.join("novashell/Cargo.toml")
    } else {
        root.join(tool).join("Cargo.toml")
    };
    if manifest.is_file() {
        Some(manifest)
    } else {
        None
    }
}

/// The forest version string, when 0-Core is present.
///
/// ⚠️ THIS IS THE FOREST'S VERSION, NOT THE SHELL'S. `nsh --version` answers
/// from CARGO_PKG_VERSION with no file involved, and must keep doing so -- a
/// shell that read its own version off disk would report nothing when packaged.
///
/// Four callers read faelight/meta/VERSION independently and had already
/// drifted on what absence means: three fell back to "unknown", one to the
/// EMPTY STRING, which printed as though it were a version. Returning Option
/// moves that choice to the display, where it belongs.
pub fn forest_version() -> Option<String> {
    let v = std::fs::read_to_string(faelight_core::paths::version_file()).ok()?;
    let v = v.trim().to_string();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// The current release name from the changelog, when 0-Core is present.
///
/// Returns the text AFTER the em-dash in the `## [` heading -- the release
/// name, not the whole heading. The caller previously did `unwrap_or_default()`
/// on the file, so an absent changelog produced no release name and no signal
/// that anything was missing. The display fallback stays with the caller, which
/// is whose choice it is.
pub fn release_name() -> Option<String> {
    let changelog = std::fs::read_to_string(faelight_core::paths::changelog_file()).ok()?;
    changelog
        .lines()
        .find(|l| l.starts_with("## ["))
        .and_then(|l| l.split(RELEASE_SEPARATOR).nth(1))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// The em-dash separating a changelog version from its release name.
///
/// U+2014, and it is LOAD-BEARING: the split depends on this exact character.
/// It is the one that has broken patch anchors in this repo before, so it lives
/// HERE, once, rather than being re-typed at a call site.
const RELEASE_SEPARATOR: char = '—';

/// The last health percentage the doctor recorded, when it has ever run.
///
/// ⚠️ THE THREE CALLERS OF THIS WERE ALREADY CORRECT. `paths::read_health()`
/// returns Option and all three match it honestly; the doubled-95 and
/// doubled-100 fallbacks that once made a machine assert peak health from a
/// missing file are gone. This wraps for the 0-Core presence check only, and
/// deliberately does not change what any caller displays.
pub fn health() -> Option<u8> {
    faelight_core::paths::read_health()
}

/// The intent `cistart` last focused, when 0-Core is present.
///
/// ⚠️ `focus.toml` IS THE SOURCE OF TRUTH and the engine says so
/// (`friday/attention.rs:98`: "written by cistart, source of truth"). It holds
/// the id AND the title, which is what a focus line wants.
///
/// ⚠️ THE OTHER OWNER IS BROKEN AND IS NOT USED HERE: `db::set_focus_intent`
/// writes `shell_state.focus_intent`, a key no reader reads, while
/// `db::get_focus_intent` reads this file. A setter and a getter sharing a name
/// and not a storage. Filed as INT-242, deliberately not fixed inside this
/// boundary.
///
/// 📍 The path was hand-built from `env::var("HOME")` at `db.rs:495` and never
/// asked `paths.rs` -- so the census, which only finds `paths::` calls, could
/// not see this coupling at all. INT-240's subject, inside the mechanism.
pub fn focus() -> Option<(String, String)> {
    let path = faelight_core::paths::state_home().join("0-core/intent/focus.toml");
    let content = std::fs::read_to_string(path).ok()?;
    let mut id = String::new();
    let mut title = String::new();
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("id = ") {
            id = rest.trim().trim_matches('"').to_string();
        } else if let Some(rest) = line.strip_prefix("title = ") {
            title = rest.trim().trim_matches('"').to_string();
        }
    }
    if id.is_empty() {
        None
    } else {
        Some((id, title))
    }
}

/// The 0-Core intents tree, when 0-Core is present.
///
/// ⚠️ CALLERS MUST REFUSE ON `None`, NOT FALL BACK. Every consumer of this path
/// hands it to a tool -- `fd`, `rg`, `read_dir` -- and
/// `unwrap_or_default()` on a PathBuf yields the EMPTY path, which is the
/// current directory to every filesystem call. Claude shipped exactly that bug
/// at `commands/mod.rs:4294` two passes ago: it overwrote a good `core_root`
/// default with an empty path and ran `fd` against it.
pub fn intents_root() -> Option<PathBuf> {
    let dir = faelight_core::paths::intents_dir();
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
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
    // INT-230: was present() then intents_root() -- two existence checks for one
    // question. The accessor answers both.
    let root = intents_root()?;
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
