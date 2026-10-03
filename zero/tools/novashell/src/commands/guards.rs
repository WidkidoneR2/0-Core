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

/// Whether `path` is protected: read from `cwd` when relative, so a path typed inside the repo
/// is judged where it really is, then matched against the markers (INT-267 step 2e).
pub fn is_protected_from(path: &str, extra: &[&str], cwd: &std::path::Path) -> bool {
    let p = std::path::Path::new(path);
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        cwd.join(p)
    };
    let abs = abs.to_string_lossy();
    protected_markers(extra)
        .iter()
        .any(|m| abs.contains(m.as_str()))
}

/// Whether `path` is protected, read from the shell's current directory.
pub fn is_protected(path: &str, extra: &[&str]) -> bool {
    let cwd = std::env::current_dir().unwrap_or_default();
    is_protected_from(path, extra, &cwd)
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

    /// INT-267 step 2c: a crate is deployed when the binary it builds runs from PATH -- novashell
    /// builds nsh. Seen red while deployed meant ~/0-core/scripts/<name>, a NixOS-era layout.
    #[test]
    fn deployed_means_its_binary_runs_from_path() {
        use std::os::unix::fs::PermissionsExt;
        let d = std::env::temp_dir().join(format!("deployed-int267-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("bin")).unwrap();
        std::fs::create_dir_all(d.join("crate/src")).unwrap();
        std::fs::write(d.join("crate/src/main.rs"), "").unwrap();
        std::fs::write(d.join("bin/nsh"), "").unwrap();
        std::fs::set_permissions(d.join("bin/nsh"), std::fs::Permissions::from_mode(0o755))
            .unwrap();
        let cargo =
            "[package]\nname = \"novashell\"\n\n[[bin]]\nname = \"nsh\"\npath = \"src/main.rs\"\n";
        let path = d.join("bin").into_os_string();
        let p = Some(path.as_os_str());
        let shell = super::super::crate_deployed(&d.join("crate"), "novashell", cargo, p);
        let lib = super::super::crate_deployed(&d.join("lib"), "zero-core", "[package]\n", p);
        let _ = std::fs::remove_dir_all(&d);
        assert!(
            shell,
            "novashell reads as not deployed although nsh is on PATH"
        );
        assert!(!lib, "a library reads as deployed");
    }

    /// INT-267 step 2e: the guard refuses the crate trees and nothing else. A relative path is
    /// read from where it was typed; a tools/ elsewhere on the machine is not the repo's. Seen red
    /// while the marker was a parent's last segment, which refused any path containing it.
    #[test]
    fn the_guard_protects_the_crate_trees_and_nothing_else() {
        let zero = zero_core::paths::source_dir();
        let parent = zero_core::paths::CRATE_PARENTS[0].trim_start_matches("zero/");
        let inside = format!("{}/zero-core/src/lib.rs", parent);
        let elsewhere = std::path::Path::new("/home/someone");
        assert!(
            is_protected_from(&inside, &[], &zero),
            "a crate path typed inside zero/ is open"
        );
        assert!(!is_protected_from(
            "/home/someone/notes/tools/a.txt",
            &[],
            &zero
        ));
        assert!(!is_protected_from("notes/tools/a.txt", &[], elsewhere));
    }
}
