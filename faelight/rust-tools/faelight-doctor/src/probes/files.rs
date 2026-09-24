//! Probes that read the filesystem: symlinks, aliases, config files.

use crate::measurement::Measurement;
use std::fs;
use walkdir::WalkDir;

/// Dotfile symlinks that point at nothing.
///
/// ⭐ THE EXCLUSION IS A PROPERTY, NOT A NAME -- mostly. A dangling link into a runtime
/// directory means the app is not running, not that configuration is broken. Chromium and
/// PulseAudio both drop lock and socket links in ~/.config pointing into /tmp, and those targets
/// vanish the moment the process exits.
///
/// ⚠️ TWO CONDITIONS, BECAUSE NEITHER CATCHES ALL OF THEM. Replacing the name list with a target
/// test alone took the count from one to four: Chromium's SingletonLock and SingletonCookie use
/// RELATIVE targets, so asking where the target lives says nothing about them, while the socket
/// beside them points at /tmp and the name says nothing about it.
///
/// ⏭ AND THE NAME HALF WILL DRIFT, exactly like the binary list. It names three applications
/// today and grows with every program that keeps a relative lock link. Ported faithfully because
/// a migration that changes behaviour is a migration that cannot be trusted -- but the durable
/// answer is a property that covers relative targets too, and that is its own question.
pub fn broken_symlinks() -> Measurement {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(e) => return Measurement::unknown(format!("HOME is not set -- {}", e)),
    };
    let config = std::path::PathBuf::from(home).join(".config");
    if !config.exists() {
        return Measurement::unknown(format!("{} does not exist", config.display()));
    }
    let mut broken = 0usize;
    for entry in WalkDir::new(&config)
        .max_depth(6)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        let target = fs::read_link(p).unwrap_or_default();
        let runtime_target = target.starts_with("/tmp") || target.starts_with("/run");
        let name = p.to_string_lossy();
        let runtime_name = name.contains("Singleton")
            || name.contains("BraveSoftware")
            || name.contains("Notesnook");
        if p.is_symlink() && !p.exists() && !runtime_target && !runtime_name {
            broken += 1;
        }
    }
    if broken == 0 {
        Measurement::pass("no broken symlinks found")
    } else {
        Measurement::fail(format!("{} broken symlinks found", broken))
    }
}

/// The two names of each migrating directory, `zero` and `faelight`, are ONE directory.
///
/// ⭐ DIRECTION-AGNOSTIC, 2026-09-24 (INT-247 Layer 3b). Before the flip `zero` links to
/// `faelight`; after it `faelight` links to `zero`. The invariant true at EVERY moment of the
/// migration is the one checked: exactly one name is a real directory and the other links to
/// it. The pass message names the real one, so the flip is visible in `d` as it happens.
///
/// ⚠️ DISTINCT STATES, NOT COLLAPSED. Absent, unreadable, dangling, resolving elsewhere,
/// two links, and TWO REAL DIRECTORIES -- the last means writes are splitting between two places,
/// and it looks fine until you notice half your state is missing. Unreadable is NOT reported as
/// absent: an entry the probe could not read is a different finding from one that is not there.
///
/// The previous version asserted one direction and read the config side through
/// faelight_config_dir(), which returns the `zero` path once paths.rs flips -- it would have
/// compared a name with itself, a check that cannot fail. Both names are spelled out here.
pub fn zero_alias() -> Measurement {
    let chains = [
        ("state", faelight_core::paths::state_home()),
        ("config", faelight_core::paths::config_dir()),
    ];
    let mut issues: Vec<String> = Vec::new();
    let mut real: Vec<String> = Vec::new();
    for (label, base) in &chains {
        match alias_pair(&base.join("zero"), &base.join("faelight")) {
            Ok(name) => real.push(format!("{}={}", label, name)),
            Err(e) => issues.push(e),
        }
    }
    if issues.is_empty() {
        let msg = format!(
            "both names resolve to one directory -- real: {}",
            real.join(", ")
        );
        Measurement::pass(&msg)
    } else {
        Measurement::warn(issues.join("; "))
    }
}

/// One entry, read without following a link. NotFound is "absent"; any other error is
/// "could not be read" -- the two are different findings and must not collapse.
fn alias_entry(p: &std::path::Path) -> Result<fs::Metadata, String> {
    fs::symlink_metadata(p).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!("{} absent", p.display())
        } else {
            format!("{} could not be read: {}", p.display(), e)
        }
    })
}

/// One chain: `zero` and `faelight` must be ONE directory under two names, in EITHER direction.
/// Ok carries the name that is the real directory; Err is the finding, naming the path.
fn alias_pair(zero: &std::path::Path, old: &std::path::Path) -> Result<String, String> {
    let (mz, mo) = match (alias_entry(zero), alias_entry(old)) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(a), Err(b)) => return Err(format!("{}; {}", a, b)),
        (Err(e), _) | (_, Err(e)) => return Err(e),
    };
    let (real, link) = match (mz.file_type().is_symlink(), mo.file_type().is_symlink()) {
        (false, true) => (zero, old),
        (true, false) => (old, zero),
        (false, false) => {
            return Err(format!(
                "{} and {} are BOTH REAL, neither is a link -- writes are splitting between them",
                zero.display(),
                old.display()
            ))
        }
        (true, true) => {
            return Err(format!(
                "{} and {} are both links -- neither is the real directory",
                zero.display(),
                old.display()
            ))
        }
    };
    if !real.is_dir() {
        return Err(format!("{} is not a directory", real.display()));
    }
    if !link.exists() {
        return Err(format!("{} dangles", link.display()));
    }
    match (fs::canonicalize(link), fs::canonicalize(real)) {
        (Ok(a), Ok(b)) if a == b => Ok(real
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| real.display().to_string())),
        (Ok(a), Ok(b)) => Err(format!(
            "{} resolves to {}, not {}",
            link.display(),
            a.display(),
            b.display()
        )),
        _ => Err(format!("{} could not be resolved", link.display())),
    }
}

/// The config files parse.
///
/// ⚠️ A FILE THAT EXISTS BUT CANNOT BE READ ONCE COUNTED AS NO ISSUE. The old chain matched only
/// `Ok(content)`, so a permissions error fell through and the file was silently treated as valid.
/// Present-and-unreadable is a THIRD state: not missing, not valid.
///
/// ⭐ AND A COUNT IS NOT A FINDING. The message used to say "N config issues", which tells you
/// how many and nothing about which -- so the warning could not be acted on without repeating
/// the check by hand. Each issue names its file and what is wrong with it.
pub fn zero_config() -> Measurement {
    let dir = faelight_core::paths::faelight_config_dir();
    let mut issues: Vec<String> = Vec::new();
    for file in ["config.toml", "profiles.toml", "themes.toml"] {
        let path = dir.join(file);
        if !path.exists() {
            issues.push(format!("{} missing", file));
            continue;
        }
        match fs::read_to_string(&path) {
            Ok(content) => {
                if let Err(e) = toml::from_str::<toml::Value>(&content) {
                    issues.push(format!("{} invalid: {}", file, e));
                }
            }
            Err(e) => issues.push(format!("{} unreadable: {}", file, e)),
        }
    }
    if issues.is_empty() {
        Measurement::pass("all config files valid")
    } else {
        Measurement::warn(issues.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Status;

    #[test]
    fn broken_symlinks_reports_or_says_it_could_not() {
        let m = broken_symlinks();
        assert!(!m.message.is_empty());
        assert!(matches!(
            m.status,
            Status::Pass | Status::Fail | Status::Unknown
        ));
    }

    #[test]
    fn zero_alias_names_every_problem_it_finds() {
        // ⚠️ A warn here must NAME the path and the problem. "aliases broken" would be a count
        // dressed as a finding -- unactionable without repeating the check by hand.
        let m = zero_alias();
        assert!(!m.message.is_empty());
        if m.status == Status::Warn {
            assert!(
                m.message.contains('/'),
                "a warn must name the path it is about: {}",
                m.message
            );
        }
    }

    /// A pid-unique scratch base, recreated empty so a rerun starts clean.
    fn alias_scratch(name: &str) -> std::path::PathBuf {
        let d =
            std::env::temp_dir().join(format!("doctor-zero-alias-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn alias_pair_passes_in_both_directions() {
        use std::os::unix::fs::symlink;
        // BEFORE the flip: faelight is real, zero links to it
        let b = alias_scratch("before");
        std::fs::create_dir(b.join("faelight")).unwrap();
        symlink("faelight", b.join("zero")).unwrap();
        assert_eq!(
            alias_pair(&b.join("zero"), &b.join("faelight")),
            Ok("faelight".to_string())
        );
        // AFTER the flip: zero is real, faelight links to it
        let a = alias_scratch("after");
        std::fs::create_dir(a.join("zero")).unwrap();
        symlink("zero", a.join("faelight")).unwrap();
        assert_eq!(
            alias_pair(&a.join("zero"), &a.join("faelight")),
            Ok("zero".to_string())
        );
        let _ = std::fs::remove_dir_all(&b);
        let _ = std::fs::remove_dir_all(&a);
    }

    #[test]
    fn alias_pair_names_every_broken_state() {
        use std::os::unix::fs::symlink;
        let two_real = alias_scratch("two-real");
        std::fs::create_dir(two_real.join("zero")).unwrap();
        std::fs::create_dir(two_real.join("faelight")).unwrap();

        let two_links = alias_scratch("two-links");
        symlink("faelight", two_links.join("zero")).unwrap();
        symlink("zero", two_links.join("faelight")).unwrap();

        let dangling = alias_scratch("dangling");
        std::fs::create_dir(dangling.join("faelight")).unwrap();
        symlink("nowhere", dangling.join("zero")).unwrap();

        let absent = alias_scratch("absent");
        std::fs::create_dir(absent.join("faelight")).unwrap();

        let elsewhere = alias_scratch("elsewhere");
        std::fs::create_dir(elsewhere.join("faelight")).unwrap();
        std::fs::create_dir(elsewhere.join("other")).unwrap();
        symlink("other", elsewhere.join("zero")).unwrap();

        for (base, finding) in [
            (&two_real, "BOTH REAL"),
            (&two_links, "both links"),
            (&dangling, "dangles"),
            (&absent, "absent"),
            (&elsewhere, "resolves to"),
        ] {
            let r = alias_pair(&base.join("zero"), &base.join("faelight"));
            let e = r.expect_err(finding);
            assert!(e.contains(finding), "expected {:?} in: {}", finding, e);
            assert!(e.contains('/'), "a finding must name its path: {}", e);
            let _ = std::fs::remove_dir_all(base);
        }
    }

    #[test]
    fn alias_pair_says_unreadable_not_absent() {
        use std::os::unix::fs::PermissionsExt;
        let b = alias_scratch("unreadable");
        let locked = b.join("locked");
        std::fs::create_dir(&locked).unwrap();
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
        // root reads through 000, so the case cannot be built there -- skip rather than lie
        let can_bypass = std::fs::read_dir(&locked).is_ok();
        let r = alias_pair(&locked.join("zero"), &locked.join("faelight"));
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
        let _ = std::fs::remove_dir_all(&b);
        if can_bypass {
            return;
        }
        let e = r.expect_err("an unreadable parent must not pass");
        assert!(
            e.contains("could not be read"),
            "unreadable reported as: {}",
            e
        );
        assert!(
            !e.contains("absent"),
            "unreadable collapsed to absent: {}",
            e
        );
    }

    #[test]
    fn zero_config_names_the_file_not_a_count() {
        let m = zero_config();
        assert!(!m.message.is_empty());
        if m.status == Status::Warn {
            assert!(
                m.message.contains(".toml"),
                "a warn must name the file: {}",
                m.message
            );
        }
    }
}
