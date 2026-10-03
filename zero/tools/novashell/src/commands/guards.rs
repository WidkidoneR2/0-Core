//! INT-267 L2: the guards that protect the source tree, built from the owners in
//! zero_core::paths instead of spelled by hand, so they protect the tree where it is.

use std::path::PathBuf;

/// Substring markers the rename, copy, move and write guards refuse: the crate parents the
/// owner names, then the markers each command adds.
pub fn protected_markers(extra: &[&str]) -> Vec<String> {
    let mut out = zero_core::paths::crate_parent_markers();
    out.extend(extra.iter().map(|e| e.to_string()));
    out
}

/// The directories delete asks about before it removes anything: every crate, and the source
/// directories the owners name. Absolute, so they hold whatever the working directory is.
/// zero/scripts is joined by hand because paths::scripts_dir() names a directory that does
/// not exist (INT-267, found 2026-10-02).
pub fn delete_guard_dirs() -> Vec<PathBuf> {
    use zero_core::paths;
    let core = paths::core_dir();
    let mut out: Vec<PathBuf> = paths::CRATE_PARENTS
        .iter()
        .chain(paths::SINGLE_CRATES.iter())
        .map(|p| core.join(p))
        .collect();
    out.push(paths::intents_dir());
    out.push(paths::source_dir().join("scripts"));
    out.push(paths::meta_dir());
    out.push(paths::docs_dir());
    out
}

/// The entries of a guard list that name no directory: a guard that protects nothing.
#[cfg(test)]
pub fn missing(dirs: &[PathBuf]) -> Vec<PathBuf> {
    dirs.iter().filter(|d| !d.is_dir()).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// INT-267 SAFETY: every directory delete guards exists, so its warning can fire. Seen red
    /// while the list named directories the tree left when it moved under zero/.
    #[test]
    fn every_delete_guard_dir_exists() {
        let gone = missing(&delete_guard_dirs());
        assert!(
            gone.is_empty(),
            "delete guards directories that do not exist: {gone:?}"
        );
    }

    /// The check itself: a guard entry naming a missing directory is caught.
    #[test]
    fn a_missing_guard_dir_is_caught() {
        let planted = vec![std::env::temp_dir().join("int267-no-such-directory")];
        assert_eq!(missing(&planted), planted);
    }

    /// The markers refuse a path inside a crate, as the hand-spelled lists did.
    #[test]
    fn markers_refuse_a_path_inside_a_crate() {
        let p = zero_core::paths::core_dir()
            .join(zero_core::paths::CRATE_PARENTS[0])
            .join("novashell/src/main.rs")
            .display()
            .to_string();
        assert!(protected_markers(&[])
            .iter()
            .any(|m| p.contains(m.as_str())));
    }

    /// INT-267 step 2b: rm -rf guards every crate home and the ledger, and each entry exists --
    /// a guard naming a missing directory protects nothing. Seen red while the engine entry
    /// named ~/0-core/engine, which the tree left when it moved under zero/.
    #[test]
    fn rm_guard_names_every_crate_home_and_each_exists() {
        let guarded = crate::core_integration::rm_guarded_dirs();
        let core = zero_core::paths::core_dir();
        let homes = zero_core::paths::CRATE_PARENTS
            .iter()
            .chain(zero_core::paths::SINGLE_CRATES.iter());
        for home in homes {
            let want = core.join(home).to_string_lossy().to_string();
            assert!(
                guarded.contains(&want),
                "rm -rf does not guard {want}: {guarded:?}"
            );
        }
        let gone: Vec<&String> = guarded
            .iter()
            .filter(|d| !std::path::Path::new(d.as_str()).is_dir())
            .collect();
        assert!(
            gone.is_empty(),
            "rm -rf guards what does not exist: {gone:?}"
        );
    }
}
