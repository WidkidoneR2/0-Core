//! INT-212 G4: the ONE place the intent directories are named.
//!
//! decisions/137: the lifecycle directories share ONE id namespace, because a file MOVES
//! between them; each record directory owns its own sequence. Before this module the names
//! were written out nine times in intent/mod.rs and no two lists agreed.

use std::fs;
use std::path::Path;

// -- the sets --
/// The one shelf. Status is the frontmatter line, not a directory. One id namespace.
pub const LIFECYCLE: &[&str] = &["open"];

/// Where work in flight lives: what complete and cancel search.
pub const ACTIVE: &[&str] = &["open"];

/// Work not yet started: what start searches. The status line still has to be planned.
pub const PENDING: &[&str] = &["open"];

/// Record directories. Each owns its own id sequence.
pub const RECORDS: &[&str] = &["decisions", "experiments", "incidents", "philosophy"];

/// Read by the ledger but owned by another module, with its own id scheme (F-NNNN).
#[allow(dead_code)] // classification only: the tree test reads it
pub const OTHER: &[&str] = &["findings"];

/// Shelves an intent file may have lived in. The live shelf is `open`; the others are
/// gone from disk and still visible to git. Record shelves are not here.
const GIT_LIFECYCLE: &[&str] = &[
    "future",
    "planned",
    "in-progress",
    "complete",
    "cancelled",
    "open",
];

/// The ONE numberer. One plus the highest three-digit id in the directories that share the
/// category's namespace: all of LIFECYCLE for a lifecycle directory, only its own otherwise.
/// A lifecycle number also counts `closed.tsv` and, when git can answer about this shelf,
/// every id git has stored under a lifecycle folder.
pub fn next_number(base: &Path, category: &str) -> u32 {
    let scan: Vec<&str> = if LIFECYCLE.contains(&category) {
        LIFECYCLE.to_vec()
    } else {
        vec![category]
    };
    let mut highest = scan
        .iter()
        .flat_map(|d| fs::read_dir(base.join(d)).ok().into_iter().flatten())
        .flatten()
        .filter_map(|e| three_digit_prefix(&e.file_name().to_string_lossy()))
        .max()
        .unwrap_or(0);
    if LIFECYCLE.contains(&category) {
        match zero_core::ledger::read_closed(base) {
            Ok(index) => {
                for warning in &index.warnings {
                    eprintln!("closed.tsv: {warning}");
                }
                for row in &index.rows {
                    if let Some(n) = three_digit_prefix(&row.id) {
                        highest = highest.max(n);
                    }
                }
            }
            Err(err) => eprintln!("closed.tsv: {err}"),
        }
        if let Some(n) = highest_git_lifecycle_id(base) {
            highest = highest.max(n);
        }
    }
    highest + 1
}

fn three_digit_prefix(name: &str) -> Option<u32> {
    let prefix = name.split('-').next()?;
    if prefix.len() == 3 && prefix.bytes().all(|b| b.is_ascii_digit()) {
        prefix.parse().ok()
    } else {
        None
    }
}

/// The highest lifecycle id git has stored, when `base` is that repo's intent shelf.
/// None when git cannot answer, or when `base` is not the shelf of the repo it sits in,
/// so a scratch directory does not inherit some other repository's numbers.
fn highest_git_lifecycle_id(base: &Path) -> Option<u32> {
    let top = git_stdout(base, &["rev-parse", "--show-toplevel"])?;
    let top = Path::new(top.trim());
    let shelf = top.join("zero/intents");
    if base.canonicalize().ok()? != shelf.canonicalize().ok()? {
        return None;
    }
    let text = git_stdout(
        top,
        &[
            "log",
            "--all",
            "--name-only",
            "--format=",
            "--",
            "zero/intents",
        ],
    )?;
    let mut highest = None;
    for line in text.lines() {
        if let Some(n) = lifecycle_history_id(line) {
            highest = Some(highest.map_or(n, |h: u32| h.max(n)));
        }
    }
    highest
}

fn lifecycle_history_id(path: &str) -> Option<u32> {
    let path = path.trim().trim_matches('"');
    let mut parts = path.split('/');
    if parts.next() != Some("zero") || parts.next() != Some("intents") {
        return None;
    }
    let folder = parts.next()?;
    let file = parts.next()?;
    if parts.next().is_some() || !GIT_LIFECYCLE.contains(&folder) || !file.ends_with(".md") {
        return None;
    }
    let stem = file.strip_suffix(".md")?;
    three_digit_prefix(stem)
}

fn git_stdout(dir: &Path, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn real_tree() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("intents")
    }

    fn real_dirs() -> BTreeSet<String> {
        let tree = real_tree();
        assert!(
            tree.is_dir(),
            "the intent ledger is not at {}: an absent ledger is not an empty one",
            tree.display()
        );
        fs::read_dir(&tree)
            .unwrap()
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect()
    }

    fn fixture(name: &str, files: &[&str]) -> PathBuf {
        let base =
            std::env::temp_dir().join(format!("int212-folders-{}-{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&base);
        for f in files {
            let p = base.join(f);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, "").unwrap();
        }
        base
    }

    #[test]
    fn the_lifecycle_is_the_open_shelf() {
        assert_eq!(LIFECYCLE, &["open"]);
    }

    #[test]
    fn active_and_pending_work_are_the_open_shelf() {
        assert_eq!(ACTIVE, &["open"]);
        assert_eq!(PENDING, &["open"]);
        assert!(ACTIVE.iter().all(|d| LIFECYCLE.contains(d)));
        assert!(PENDING.iter().all(|d| ACTIVE.contains(d)));
    }

    #[test]
    fn no_directory_is_named_in_two_sets() {
        let mut seen = BTreeSet::new();
        for d in LIFECYCLE.iter().chain(RECORDS).chain(OTHER) {
            assert!(seen.insert(*d), "{} is classified twice", d);
        }
    }

    #[test]
    fn every_directory_in_the_real_tree_is_classified() {
        let known: BTreeSet<&str> = LIFECYCLE
            .iter()
            .chain(RECORDS)
            .chain(OTHER)
            .copied()
            .collect();
        let unclassified: Vec<String> = real_dirs()
            .into_iter()
            .filter(|d| !known.contains(d.as_str()))
            .collect();
        assert!(
            unclassified.is_empty(),
            "directories in intents/ that no set names: {:?}",
            unclassified
        );
    }

    #[test]
    fn no_set_names_a_directory_the_tree_does_not_have() {
        let real = real_dirs();
        let stale: Vec<&str> = LIFECYCLE
            .iter()
            .chain(ACTIVE)
            .chain(RECORDS)
            .chain(OTHER)
            .copied()
            .filter(|d| !real.contains(*d))
            .collect();
        assert!(
            stale.is_empty(),
            "names in the sets that are not directories: {:?}",
            stale
        );
    }

    #[test]
    fn the_numberer_counts_the_open_shelf() {
        let base = fixture("planned", &["open/010-a.md", "open/300-b.md"]);
        assert_eq!(next_number(&base, "open"), 301);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn the_lifecycle_shares_one_counter() {
        let base = fixture("shared", &["open/010-a.md", "open/042-b.md"]);
        assert_eq!(next_number(&base, "open"), 43);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn a_record_directory_owns_its_own_sequence() {
        let base = fixture(
            "records",
            &["decisions/137-a.md", "open/200-b.md", "incidents/003-c.md"],
        );
        assert_eq!(next_number(&base, "decisions"), 138);
        assert_eq!(next_number(&base, "incidents"), 4);
        assert_eq!(next_number(&base, "experiments"), 1);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn pending_work_is_the_open_shelf() {
        assert_eq!(PENDING, &["open"]);
        assert!(PENDING.iter().all(|d| ACTIVE.contains(d)));
    }

    #[test]
    fn only_a_three_digit_prefix_is_an_id() {
        let base = fixture(
            "prefix",
            &[
                "open/012-a.md",
                "open/0123-b.md",
                "open/F-0052.md",
                "open/README.md",
            ],
        );
        assert_eq!(next_number(&base, "open"), 13);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn open_302_plus_closed_319_answers_320() {
        let base = fixture("closed319", &["open/302-live.md"]);
        fs::write(
            base.join("closed.tsv"),
            "id\tstatus\tclosed\tlast_commit\ttitle\n319\tcomplete\t2026-11-17\tabc1234\tfinished\n",
        )
        .unwrap();
        let n = next_number(&base, "open");
        let _ = fs::remove_dir_all(&base);
        assert_eq!(n, 320, "open/302 plus closed 319");
    }

    #[test]
    fn a_lifecycle_id_git_has_seen_counts_toward_the_next_number() {
        let root = std::env::temp_dir().join(format!(
            "int320-git-number-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&root);
        let intents = root.join("zero/intents");
        fs::create_dir_all(intents.join("complete")).unwrap();
        fs::write(intents.join("complete/319-old.md"), "old\n").unwrap();
        let git = |args: &[&str]| {
            let out = std::process::Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(args)
                .output()
                .expect("git");
            assert!(
                out.status.success(),
                "{args:?} {}",
                String::from_utf8_lossy(&out.stderr)
            );
        };
        git(&["init", "-q"]);
        git(&["config", "user.email", "t@example.com"]);
        git(&["config", "user.name", "test"]);
        git(&["add", "zero/intents/complete/319-old.md"]);
        git(&["commit", "-q", "-m", "archive 319"]);
        git(&["rm", "-q", "zero/intents/complete/319-old.md"]);
        git(&["commit", "-q", "-m", "drop 319"]);
        fs::create_dir_all(intents.join("open")).unwrap();
        fs::write(intents.join("open/302-live.md"), "").unwrap();
        let n = next_number(&intents, "open");
        let _ = fs::remove_dir_all(&root);
        assert_eq!(n, 320, "open/302 plus git history 319");
    }
}
