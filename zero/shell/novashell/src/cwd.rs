//! INT-241: THE ONE PLACE THE SHELL CHANGES DIRECTORY.
//!
//! `std::env::set_current_dir` moves the process and nothing else. POSIX requires a shell to set
//! `PWD` on every directory change, and bash does; nsh never did -- so `PWD` kept whatever the
//! bash that exec'd nsh exported at login, for the whole life of the shell.
//!
//! Measured 2026-09-23, interactive session:
//!
//! ```text
//!     cd ~/0-core
//!     pwd        -> /home/christian/0-core
//!     echo $PWD  -> /home/christian          NEVER UPDATED, not merely stale
//! ```
//!
//! ⚠️ THE `-c` DOOR AGREED WITH BASH THE WHOLE TIME, because it delegates to `sh`, which inherits
//! a correct PWD from its caller. A regression test written against `-c` would have passed without
//! this fix and proved nothing -- which is why INT-241 G6 drives the REPL door.
//!
//! ⭐ PWD IS READ BACK FROM `current_dir()` AFTER THE MOVE, never derived from the argument. A
//! relative path, a path through a symlink and `~` expansion then agree with `pwd` by
//! construction, rather than by re-implementing the resolution `pwd` already performs.
//!
//! EVERY site that moves the shell calls this. TEN existed when it was written -- cd, z_jump
//! twice, the crate jump, scope enter and leave, session restore, the cwd adopted after yazi
//! exits, startup, and a restore path in main -- and setting PWD beside each would be the
//! multi-owner shape this ledger keeps removing.
//!
//! ⚠️ THE TENTH WAS FOUND ONLY BECAUSE THE CENSUS WAS RE-RUN WITHOUT `head`. INT-241 G2 asks for
//! every site ENUMERATED rather than grepped for once, and the first pass missed main.rs:3604.

use std::path::{Path, PathBuf};

/// Change the shell's working directory AND the `PWD` it advertises.
///
/// Signature matches `std::env::set_current_dir`, so a call site swaps one for the other without
/// touching its error handling.
pub fn chdir<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    std::env::set_current_dir(path)?;
    if let Ok(now) = std::env::current_dir() {
        std::env::set_var("PWD", now);
    }
    Ok(())
}

/// INT-301: where a fresh session starts, or None to stay where it was spawned.
///
/// THE ONE STARTUP DIRECTORY RULE. A terminal that opens a window or split from a shell in some
/// directory hands the new shell that directory (F-0102, F-0106), and nsh keeps it. Two cases start
/// in the core root instead, and they are the ones INT-261 was written for: a launch from HOME, which
/// is what a terminal opened from the desktop gets, and a launch directory that cannot be used
/// (deleted, or not a directory). `keep` is NSH_KEEP_CWD (INT-206): a caller that chose a working
/// directory, nsh-test, keeps it whatever it is.
///
/// Pure on purpose: the caller reads the environment and makes the move, so every case is a unit
/// test and not a terminal.
pub fn startup_dir(
    keep: bool,
    launch: Option<&Path>,
    home: Option<&Path>,
    core_root: &Path,
) -> Option<PathBuf> {
    if keep {
        return None;
    }
    let launch = match launch {
        Some(p) if p.is_dir() => p,
        _ => return Some(core_root.to_path_buf()),
    };
    // Resolved before comparing, so HOME reached through a symlink or a trailing slash is HOME.
    let same = |a: &Path, b: &Path| match (a.canonicalize(), b.canonicalize()) {
        (Ok(x), Ok(y)) => x == y,
        _ => a == b,
    };
    if home.is_some_and(|h| same(launch, h)) {
        return Some(core_root.to_path_buf());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{startup_dir, Path, PathBuf};

    const ROOT: &str = "/core-root";

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("nsh-cwd-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("make scratch");
        dir
    }

    fn pair(tag: &str) -> (PathBuf, PathBuf) {
        let base = scratch(tag);
        let work = base.join("work");
        let home = base.join("home");
        std::fs::create_dir_all(&work).expect("make work");
        std::fs::create_dir_all(&home).expect("make home");
        (work, home)
    }

    #[test]
    fn a_launch_directory_that_is_not_home_is_kept() {
        let (work, home) = pair("kept");
        let got = startup_dir(false, Some(&work), Some(&home), Path::new(ROOT));
        assert_eq!(got, None);
    }

    #[test]
    fn a_launch_from_home_goes_to_the_core_root() {
        let (_, home) = pair("home");
        let got = startup_dir(false, Some(&home), Some(&home), Path::new(ROOT));
        assert_eq!(got, Some(PathBuf::from(ROOT)));
    }

    #[test]
    fn home_through_a_trailing_slash_or_a_symlink_is_still_home() {
        let (work, home) = pair("alias");
        let slashed = PathBuf::from(format!("{}/", home.display()));
        let got = startup_dir(false, Some(&slashed), Some(&home), Path::new(ROOT));
        assert_eq!(got, Some(PathBuf::from(ROOT)), "trailing slash");
        let link = work.join("home-link");
        std::os::unix::fs::symlink(&home, &link).expect("symlink");
        let got = startup_dir(false, Some(&link), Some(&home), Path::new(ROOT));
        assert_eq!(got, Some(PathBuf::from(ROOT)), "symlink");
    }

    #[test]
    fn a_directory_that_cannot_be_used_goes_to_the_core_root() {
        let (work, home) = pair("unusable");
        let missing = work.join("gone");
        let file = work.join("a-file");
        std::fs::write(&file, "x").expect("make file");
        for (name, launch) in [
            ("missing", Some(missing)),
            ("file", Some(file)),
            ("none", None),
        ] {
            let got = startup_dir(false, launch.as_deref(), Some(&home), Path::new(ROOT));
            assert_eq!(got, Some(PathBuf::from(ROOT)), "{}", name);
        }
    }

    #[test]
    fn keep_stays_wherever_it_is_even_in_home_or_nowhere() {
        let (_, home) = pair("keep");
        for launch in [Some(home.clone()), Some(home.join("gone")), None] {
            let got = startup_dir(true, launch.as_deref(), Some(&home), Path::new(ROOT));
            assert_eq!(got, None, "{:?}", launch);
        }
    }

    #[test]
    fn with_no_home_set_the_launch_directory_is_kept() {
        let (work, _) = pair("nohome");
        let got = startup_dir(false, Some(&work), None, Path::new(ROOT));
        assert_eq!(got, None);
    }
}
