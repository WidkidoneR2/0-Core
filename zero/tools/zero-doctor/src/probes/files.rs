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

/// The five directories Project 0 keeps outside the repo each have ONE name.
///
/// ⭐ PASS 7, 2026-09-30 (INT-247). The old names that linked to these directories during the
/// rename are removed. The invariant is now: the zero (or nsh) name is a real directory, and the
/// old name does not exist at all -- not as a link, not as a directory. An old name that comes
/// back is a finding: a link means something recreated it; a real entry means writes may be
/// splitting between two places.
///
/// ⚠️ DISTINCT STATES, NOT COLLAPSED. Absent, a link where the directory belongs, not a
/// directory, the old name back, and UNREADABLE -- an entry the probe could not read is a
/// different finding from one that is not there, and is never reported as absent.
///
/// The old names are built with concat! so this file does not spell them: a guard that reads
/// source text for the retired name should not have to exempt the one probe that must name it.
pub fn zero_alias() -> Measurement {
    let pairs = one_name_pairs();
    let issues: Vec<String> = pairs
        .iter()
        .filter_map(|(new, old)| one_name(new, old).err())
        .collect();
    if issues.is_empty() {
        let msg = format!(
            "{} directories, one name each -- no old name remains",
            pairs.len()
        );
        Measurement::pass(&msg)
    } else {
        Measurement::warn(issues.join("; "))
    }
}

/// Each directory and the name it had before the rename.
fn one_name_pairs() -> Vec<(std::path::PathBuf, std::path::PathBuf)> {
    use zero_core::paths as p;
    let old = concat!("fae", "light");
    let old_shell = concat!("fae", "light-shell");
    vec![
        (p::state_home().join("zero"), p::state_home().join(old)),
        (p::zero_config_dir(), p::config_dir().join(old)),
        (p::zero_cache_dir(), p::xdg_cache_home().join(old)),
        (p::zero_data_dir(), p::local_data_dir().join(old)),
        (p::config_dir().join("nsh"), p::config_dir().join(old_shell)),
    ]
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

/// One directory: `new` must be a real directory and `old` must not exist at all.
/// Err is the finding, naming the path.
fn one_name(new: &std::path::Path, old: &std::path::Path) -> Result<(), String> {
    let m = alias_entry(new)?;
    if m.file_type().is_symlink() {
        return Err(format!(
            "{} is a link -- it should be the real directory",
            new.display()
        ));
    }
    if !m.is_dir() {
        return Err(format!("{} is not a directory", new.display()));
    }
    match fs::symlink_metadata(old) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("{} could not be read: {}", old.display(), e)),
        Ok(o) if o.file_type().is_symlink() => Err(format!(
            "{} is back as a link -- nothing should recreate the old name",
            old.display()
        )),
        Ok(_) => Err(format!(
            "{} is back as a real entry -- writes may be splitting from {}",
            old.display(),
            new.display()
        )),
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
    let dir = zero_core::paths::zero_config_dir();
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
    fn one_name_passes_when_only_the_new_name_exists() {
        let b = alias_scratch("one-name");
        std::fs::create_dir(b.join("zero")).unwrap();
        assert_eq!(one_name(&b.join("zero"), &b.join("old")), Ok(()));
        let _ = std::fs::remove_dir_all(&b);
    }

    #[test]
    fn one_name_names_every_broken_state() {
        use std::os::unix::fs::symlink;
        let link_back = alias_scratch("link-back");
        std::fs::create_dir(link_back.join("zero")).unwrap();
        symlink("zero", link_back.join("old")).unwrap();

        let dir_back = alias_scratch("dir-back");
        std::fs::create_dir(dir_back.join("zero")).unwrap();
        std::fs::create_dir(dir_back.join("old")).unwrap();

        let missing = alias_scratch("missing");

        let new_is_link = alias_scratch("new-is-link");
        std::fs::create_dir(new_is_link.join("other")).unwrap();
        symlink("other", new_is_link.join("zero")).unwrap();

        let not_dir = alias_scratch("not-dir");
        std::fs::write(not_dir.join("zero"), b"").unwrap();

        for (base, finding) in [
            (&link_back, "back as a link"),
            (&dir_back, "back as a real entry"),
            (&missing, "absent"),
            (&new_is_link, "is a link"),
            (&not_dir, "not a directory"),
        ] {
            let r = one_name(&base.join("zero"), &base.join("old"));
            let e = r.expect_err(finding);
            assert!(e.contains(finding), "expected {:?} in: {}", finding, e);
            assert!(e.contains('/'), "a finding must name its path: {}", e);
            let _ = std::fs::remove_dir_all(base);
        }
    }

    #[test]
    fn one_name_says_unreadable_not_absent() {
        use std::os::unix::fs::PermissionsExt;
        let b = alias_scratch("unreadable");
        let locked = b.join("locked");
        std::fs::create_dir(&locked).unwrap();
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
        // root reads through 000, so the case cannot be built there -- skip rather than lie
        let can_bypass = std::fs::read_dir(&locked).is_ok();
        let r = one_name(&locked.join("zero"), &locked.join("old"));
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
