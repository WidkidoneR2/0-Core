//! INT-212 G5: the ONE finder for an intent file by id.
//!
//! Before this module the lookup was written out in intent/mod.rs, narrative and predict, and
//! the copies disagreed: some tested `starts_with(id)` with no `-` boundary (short id 21 also
//! matches 212-...), some had the boundary, and the gate check skipped silently when it found
//! nothing. The id is a NUMBER: the digits before the first `-` of a `.md` file name, compared
//! as a number, so 21, 021 and 0021 name the same file and none of them names 212.

use super::folders;
use std::fs;
use std::path::{Path, PathBuf};

/// The digits of `s` as a number, or None when `s` is empty or holds anything but digits.
fn number_of(s: &str) -> Option<u32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse::<u32>().ok()
}

/// The file of intent `id`, searching `folders` in order; the first folder holding it wins.
/// None means no such file, and the caller decides what that means: a missing file is never
/// a guess and never a silent skip.
pub fn find_intent_file(base: &Path, id: &str, folders: &[&str]) -> Option<PathBuf> {
    let want = number_of(id.trim())?;
    for folder in folders {
        let Ok(entries) = fs::read_dir(base.join(folder)) else {
            continue;
        };
        let mut hits: Vec<PathBuf> = entries
            .flatten()
            .filter_map(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                let (stem, _) = n.split_once('-')?;
                (n.ends_with(".md") && number_of(stem) == Some(want)).then(|| e.path())
            })
            .collect();
        hits.sort();
        if let Some(first) = hits.into_iter().next() {
            return Some(first);
        }
    }
    None
}

/// Every `.md` file directly in `folders`, folder by folder in the order given and by file name
/// inside each. A folder that does not exist holds nothing (INT-212 G7).
pub fn files_in(base: &Path, folders: &[&str]) -> Vec<PathBuf> {
    let mut all = Vec::new();
    for folder in folders {
        let Ok(entries) = fs::read_dir(base.join(folder)) else {
            continue;
        };
        let mut here: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
            .collect();
        here.sort();
        all.extend(here);
    }
    all
}

/// The value of the first `status:` line, trimmed: the frontmatter status. Prose further down
/// that mentions a status is never read.
fn status_of(content: &str) -> &str {
    content
        .lines()
        .find(|l| l.starts_with("status:"))
        .map(|l| l.trim_start_matches("status:").trim())
        .unwrap_or("")
}

/// The files of work in flight: every file in the ACTIVE folders whose frontmatter status line
/// says in-progress. The folder does not decide it: cistart moves a file out of future/ only,
/// so a started intent can still sit in planned/.
pub fn in_progress_files(base: &Path) -> Vec<PathBuf> {
    files_in(base, folders::ACTIVE)
        .into_iter()
        .filter(|p| {
            fs::read_to_string(p)
                .map(|c| status_of(&c) == "in-progress")
                .unwrap_or(false)
        })
        .collect()
}

#[cfg(test)]
mod files_tests {
    use super::super::folders;
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static N: AtomicU32 = AtomicU32::new(0);

    const STARTED: &str = "---\nstatus: in-progress\n---\n";

    /// A scratch ledger: (folder, file name, content) triples.
    fn ledger(files: &[(&str, &str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "zero-files-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        for (folder, name, content) in files {
            fs::create_dir_all(dir.join(folder)).unwrap();
            fs::write(dir.join(folder).join(name), content).unwrap();
        }
        dir
    }

    /// Each path as folder/file, so an assertion reads like the ledger.
    fn names(paths: Vec<PathBuf>) -> Vec<String> {
        paths
            .iter()
            .map(|p| {
                let folder = p.parent().unwrap().file_name().unwrap().to_string_lossy();
                format!("{}/{}", folder, p.file_name().unwrap().to_string_lossy())
            })
            .collect()
    }

    #[test]
    fn files_in_lists_md_files_in_folder_order_then_by_name() {
        let b = ledger(&[
            ("future", "b.md", "x"),
            ("future", "a.md", "x"),
            ("planned", "c.md", "x"),
            ("future", "notes.txt", "x"),
        ]);
        assert_eq!(
            names(files_in(&b, &["planned", "future"])),
            vec!["planned/c.md", "future/a.md", "future/b.md"]
        );
    }

    #[test]
    fn files_in_skips_a_folder_that_does_not_exist() {
        let b = ledger(&[("future", "a.md", "x")]);
        assert_eq!(
            names(files_in(&b, &["in-progress", "future"])),
            vec!["future/a.md"]
        );
    }

    #[test]
    fn a_started_intent_is_found_in_every_active_folder() {
        let b = ledger(&[
            ("planned", "3-c.md", STARTED),
            ("future", "2-b.md", STARTED),
            ("in-progress", "1-a.md", STARTED),
        ]);
        assert_eq!(
            names(in_progress_files(&b)),
            vec!["in-progress/1-a.md", "future/2-b.md", "planned/3-c.md"]
        );
        assert_eq!(folders::ACTIVE, &["in-progress", "future", "planned"]);
    }

    #[test]
    fn an_intent_that_is_not_started_is_not_in_flight() {
        let b = ledger(&[
            ("future", "1-a.md", "---\nstatus: future\n---\n"),
            ("planned", "2-b.md", "---\nstatus: planned\n---\n"),
        ]);
        assert!(in_progress_files(&b).is_empty());
    }

    #[test]
    fn a_finished_folder_is_never_in_flight() {
        let b = ledger(&[
            ("complete", "1-a.md", STARTED),
            ("cancelled", "2-b.md", STARTED),
        ]);
        assert!(in_progress_files(&b).is_empty());
    }

    #[test]
    fn a_prose_mention_of_the_status_is_not_a_status() {
        let b = ledger(&[(
            "future",
            "1-a.md",
            "---\nstatus: future\n---\nthe line status: in-progress is how a started intent reads\n",
        )]);
        assert!(in_progress_files(&b).is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static N: AtomicU32 = AtomicU32::new(0);

    /// A scratch ledger: (folder, file name) pairs, each file created empty.
    fn ledger(files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "zero-finder-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        for (folder, name) in files {
            fs::create_dir_all(dir.join(folder)).unwrap();
            fs::write(dir.join(folder).join(name), "x").unwrap();
        }
        dir
    }

    fn name(p: Option<PathBuf>) -> Option<String> {
        p.map(|p| p.file_name().unwrap().to_string_lossy().to_string())
    }

    const ORDER: &[&str] = &["in-progress", "future", "planned"];

    #[test]
    fn an_exact_id_finds_its_file() {
        let b = ledger(&[("planned", "212-a.md")]);
        assert_eq!(
            name(find_intent_file(&b, "212", ORDER)),
            Some("212-a.md".into())
        );
    }

    #[test]
    fn padded_and_unpadded_input_name_the_same_file() {
        let b = ledger(&[("planned", "021-a.md")]);
        for id in ["21", "021", "0021", " 21 "] {
            assert_eq!(
                name(find_intent_file(&b, id, ORDER)),
                Some("021-a.md".into()),
                "{id}"
            );
        }
    }

    #[test]
    fn a_short_id_never_matches_a_longer_ids_file() {
        let b = ledger(&[("planned", "212-b.md"), ("planned", "100-c.md")]);
        for id in ["21", "1", "10", "2"] {
            assert_eq!(find_intent_file(&b, id, ORDER), None, "{id}");
        }
    }

    #[test]
    fn a_collision_resolves_to_the_right_file_whatever_the_folder_order() {
        let b = ledger(&[("in-progress", "212-b.md"), ("planned", "021-a.md")]);
        assert_eq!(
            name(find_intent_file(&b, "21", ORDER)),
            Some("021-a.md".into())
        );
        assert_eq!(
            name(find_intent_file(&b, "212", ORDER)),
            Some("212-b.md".into())
        );
    }

    #[test]
    fn the_first_folder_in_the_order_wins() {
        let b = ledger(&[("planned", "050-p.md"), ("in-progress", "050-i.md")]);
        assert_eq!(
            name(find_intent_file(&b, "50", ORDER)),
            Some("050-i.md".into())
        );
        assert_eq!(
            name(find_intent_file(&b, "50", &["planned", "in-progress"])),
            Some("050-p.md".into())
        );
    }

    #[test]
    fn only_a_markdown_file_with_a_dash_is_an_intent_file() {
        let b = ledger(&[
            ("planned", "212-a.txt"),
            ("planned", "212.md"),
            ("planned", "212"),
        ]);
        assert_eq!(find_intent_file(&b, "212", ORDER), None);
    }

    #[test]
    fn a_missing_file_or_folder_is_none_never_a_guess() {
        let b = ledger(&[("planned", "212-a.md")]);
        assert_eq!(find_intent_file(&b, "213", ORDER), None);
        assert_eq!(find_intent_file(&b, "212", &["cancelled"]), None);
        assert_eq!(find_intent_file(&b.join("absent"), "212", ORDER), None);
    }

    #[test]
    fn an_id_that_is_not_a_number_finds_nothing() {
        let b = ledger(&[("planned", "212-a.md")]);
        for id in ["", " ", "21x", "-212", "212-a", "all", "2 12"] {
            assert_eq!(find_intent_file(&b, id, ORDER), None, "{id:?}");
        }
    }

    #[test]
    fn every_lifecycle_file_in_the_real_tree_is_found_by_its_own_number() {
        let tree = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("intents");
        assert!(
            tree.is_dir(),
            "the intent ledger is not at {}",
            tree.display()
        );
        let mut checked = 0;
        for folder in crate::domains::intent::folders::LIFECYCLE {
            for e in fs::read_dir(tree.join(folder)).unwrap().flatten() {
                let n = e.file_name().to_string_lossy().to_string();
                let Some((stem, _)) = n.split_once('-') else {
                    continue;
                };
                let Some(num) = number_of(stem) else { continue };
                if !n.ends_with(".md") {
                    continue;
                }
                let got = find_intent_file(
                    &tree,
                    &num.to_string(),
                    crate::domains::intent::folders::LIFECYCLE,
                )
                .unwrap_or_else(|| panic!("{n}: not found by its own number {num}"));
                let got_name = got.file_name().unwrap().to_string_lossy().to_string();
                let got_num = number_of(got_name.split_once('-').unwrap().0);
                assert_eq!(got_num, Some(num), "{n}: found {got_name} instead");
                checked += 1;
            }
        }
        assert!(
            checked > 100,
            "only {checked} files checked: the tree is not the ledger"
        );
    }
}
