//! INT-230 G2 -- THE ADAPTER BOUNDARY.
//!
//! ONE place where novashell asks about 0-Core. Everything else reads the answer.
//!
//! WHY THIS EXISTS. The G1 census (CORE-COUPLING.md) measured 82 `paths::` calls
//! across 14 functions. 31 are core shell state and already resolve without
//! 0-Core -- XDG plus the `ZERO_STATE_DB` override. The other 51 read
//! 0-Core's directory layout, and `paths.rs` never checks whether any of them
//! exists. So today `intents_dir()` returns a confident path on a machine with
//! no 0-Core and every caller proceeds as though it were there.
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

/// INT-300 G13 -- the bug line for the welcome banner, from the one place that derives it.
///
/// `core bug welcome` prints `bugs: N open (H high)` and exits 0, or says `bugs: unknown` and
/// exits 3 when it cannot tell. This returns the line only for a clean exit whose output is one
/// bugs line. Core missing, a failing run, any other output: None, and the banner says unknown.
/// Nothing is parsed and no number is made up.
pub fn bugs_line() -> Option<String> {
    let out = std::process::Command::new("core")
        .args(["bug", "welcome"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let line = text.trim_end_matches('\n');
    if line.starts_with("bugs: ") && !line.contains('\n') {
        Some(line.to_string())
    } else {
        None
    }
}

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

/// The 0-Core source tree (zero/), when 0-Core is present: the home of every Rust crate --
/// engine, tools and shell (INT-267 step 2b; @rust names it). `None` on a machine without
/// 0-Core -- which is every packaged install.
pub fn source_root() -> Option<PathBuf> {
    let dir = zero_core::paths::source_dir();
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

/// What `rm -rf` refuses to touch: every crate parent, every single crate and the intent
/// ledger, from the owner (INT-267 step 2b). Only directories that exist are entries -- an
/// absent one guards nothing, or, as an empty string, everything (INT-230).
pub fn rm_guarded_dirs() -> Vec<String> {
    let core = zero_core::paths::core_dir();
    zero_core::paths::CRATE_PARENTS
        .iter()
        .chain(zero_core::paths::SINGLE_CRATES.iter())
        .map(|p| core.join(p))
        .chain(std::iter::once(zero_core::paths::intents_dir()))
        .filter(|d| d.is_dir())
        .map(|d| d.to_string_lossy().to_string())
        .collect()
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
    let name = if tool.is_empty() { "novashell" } else { tool };
    let manifest = zero_core::paths::crate_dir(name)?.join("Cargo.toml");
    if manifest.is_file() {
        Some(manifest)
    } else {
        None
    }
}

/// The Project 0 version string, when 0-Core is present.
///
/// ⚠️ THIS IS PROJECT 0'S VERSION, NOT THE SHELL'S. `nsh --version` answers
/// from CARGO_PKG_VERSION with no file involved, and must keep doing so -- a
/// shell that read its own version off disk would report nothing when packaged.
///
/// Four callers read zero/meta/VERSION independently and had already
/// drifted on what absence means: three fell back to "unknown", one to the
/// EMPTY STRING, which printed as though it were a version. Returning Option
/// moves that choice to the display, where it belongs.
pub fn project_version() -> Option<String> {
    let v = std::fs::read_to_string(zero_core::paths::version_file()).ok()?;
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
    let changelog = std::fs::read_to_string(zero_core::paths::changelog_file()).ok()?;
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
    health_if_current(
        zero_core::paths::read_health(),
        file_written_secs(&zero_core::paths::health_status_file()),
        boot_time_secs(),
    )
}

#[cfg(test)]
mod health_stale_tests {
    use super::*;

    #[test]
    fn g7_a_number_older_than_this_boot_is_not_current() {
        assert_eq!(health_if_current(Some(89), Some(100), Some(200)), None);
    }

    #[test]
    fn g7_a_number_written_at_or_after_boot_is_kept() {
        assert_eq!(health_if_current(Some(89), Some(300), Some(200)), Some(89));
        assert_eq!(health_if_current(Some(89), Some(200), Some(200)), Some(89));
    }

    #[test]
    fn g7_an_age_that_cannot_be_proven_keeps_the_number() {
        assert_eq!(health_if_current(Some(89), None, Some(200)), Some(89));
        assert_eq!(health_if_current(Some(89), Some(100), None), Some(89));
    }

    #[test]
    fn g7_btime_is_read_from_proc_stat() {
        let stat = "cpu  1 2 3\nbtime 1700000000\nprocesses 5\n";
        assert_eq!(parse_btime(stat), Some(1_700_000_000));
        assert_eq!(parse_btime("cpu  1 2 3\nprocesses 5\n"), None);
        assert_eq!(parse_btime("btime soon\n"), None);
    }

    #[test]
    fn g7_the_file_time_comes_from_the_file() {
        let dir = std::env::temp_dir().join(format!("nsh-g7-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("health-status");
        std::fs::write(&f, "89").unwrap();
        let t = file_written_secs(&f).expect("a written file has a time");
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(
            t + 5 >= now && t <= now + 5,
            "file time {t} is not near now {now}"
        );
        assert_eq!(file_written_secs(&dir.join("absent")), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn g7_this_machine_reports_a_boot_time() {
        assert!(boot_time_secs().is_some());
    }

    #[test]
    fn g7_no_number_stays_no_number() {
        assert_eq!(health_if_current(None, Some(300), Some(200)), None);
        assert_eq!(health_if_current(None, Some(100), Some(200)), None);
    }
}

/// INT-299 G7 (R5). A number written before this boot is not current. Pure, so a test can inject
/// the file time and the boot time. When either time is unknown the age cannot be proven, so the
/// number is kept: it is hidden only on evidence that it is old.
pub(crate) fn health_if_current(
    value: Option<u8>,
    written: Option<u64>,
    boot: Option<u64>,
) -> Option<u8> {
    match (written, boot) {
        (Some(w), Some(b)) if w < b => None,
        _ => value,
    }
}

/// When the health file was last written, in seconds since the epoch.
fn file_written_secs(path: &std::path::Path) -> Option<u64> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    let since = modified.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(since.as_secs())
}

/// The btime line of /proc/stat: when this boot started, in seconds since the epoch.
pub(crate) fn parse_btime(stat: &str) -> Option<u64> {
    stat.lines()
        .find_map(|l| l.strip_prefix("btime "))
        .and_then(|v| v.trim().parse().ok())
}

fn boot_time_secs() -> Option<u64> {
    parse_btime(&std::fs::read_to_string("/proc/stat").ok()?)
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
    let path = zero_core::paths::state_home().join("0-core/intent/focus.toml");
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
    let dir = zero_core::paths::intents_dir();
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

/// The 0-Core registry directory, when 0-Core is present.
pub fn registry_root() -> Option<PathBuf> {
    let dir = zero_core::paths::registry_dir();
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
/// difference between `intl` saying 0-Core is absent and `intl` printing
/// `Total: 0`.
pub fn ledger() -> Option<Ledger> {
    // INT-230: was present() then intents_root() -- two existence checks for one
    // question. The accessor answers both.
    let _root = intents_root()?;
    let mut intents = Vec::new();
    for file in zero_core::ledger::live() {
        if file.status == "complete" || file.status == "cancelled" || file.id.is_empty() {
            continue;
        }
        intents.push(parse(&file.id, &file.content));
    }
    let complete_ids = zero_core::ledger::completed_ids();
    Some(Ledger {
        intents,
        complete_ids,
    })
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

    /// Intents whose frontmatter status is `planned`. Records are not loaded.
    pub fn planned_count(&self) -> usize {
        self.intents.iter().filter(|i| i.is_planned()).count()
    }

    /// Completed intent ids: live status complete, plus every id git recorded
    /// under `zero/intents/complete/`. Same set the doctor prints.
    pub fn done_count(&self) -> usize {
        self.complete_ids.len()
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

    /// Welcome planned/done. A body that quotes a status does not count, a file
    /// on an old shelf does not count, and a missing shelf is None rather than
    /// a zero.
    #[test]
    fn welcome_counts_follow_the_ledger_and_absence_is_unknown() {
        // HOME is process-global. The count runs in a child so a parallel test
        // that reads the real checkout does not see this shelf.
        if std::env::var_os("NSH_WELCOME_LEDGER_CHILD").is_none() {
            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "core_integration::tests::welcome_counts_follow_the_ledger_and_absence_is_unknown",
                    "--exact",
                    "--test-threads=1",
                ])
                .env("NSH_WELCOME_LEDGER_CHILD", "1")
                .status()
                .unwrap();
            assert!(status.success());
            return;
        }
        let dir = std::env::temp_dir().join(format!(
            "nsh-welcome-ledger-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let prev = std::env::var("HOME").ok();
        // SAFETY: restored before the test returns.
        unsafe { std::env::set_var("HOME", &dir) };
        assert!(super::ledger().is_none(), "no shelf is not an empty count");
        assert!(!super::present());

        let open = dir.join("0-core/zero/intents/open");
        std::fs::create_dir_all(&open).unwrap();
        std::fs::write(
            open.join("010-planned.md"),
            "---\nid: 10\nstatus: planned\ntitle: \"Stay\"\n---\n\nstatus: in-progress\nstatus: complete\n",
        )
        .unwrap();
        std::fs::write(
            open.join("020-active.md"),
            "---\nid: 20\nstatus: in-progress\ntitle: \"Working\"\n---\n\n",
        )
        .unwrap();
        std::fs::write(
            open.join("030-done.md"),
            "---\nid: 30\nstatus: complete\ntitle: \"Done\"\n---\n\n",
        )
        .unwrap();
        let future = dir.join("0-core/zero/intents/future");
        std::fs::create_dir_all(&future).unwrap();
        std::fs::write(
            future.join("040-old.md"),
            "---\nid: 40\nstatus: planned\ntitle: \"Old shelf\"\n---\n",
        )
        .unwrap();
        let decisions = dir.join("0-core/zero/intents/decisions");
        std::fs::create_dir_all(&decisions).unwrap();
        std::fs::write(
            decisions.join("137-x.md"),
            "---\nid: 137\nstatus: planned\n---\n",
        )
        .unwrap();

        let ledger = super::ledger().expect("the open shelf is present");
        assert_eq!(ledger.planned_count(), 1);
        assert_eq!(ledger.active_count(), 1);
        assert_eq!(ledger.done_count(), 1);

        match prev {
            Some(h) => unsafe { std::env::set_var("HOME", h) },
            None => unsafe { std::env::remove_var("HOME") },
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
