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

/// The `zero` aliases resolve to the `faelight` directories.
///
/// ⭐ THIS CHECK EXISTS TO WATCH A MIGRATION IN PROGRESS. INT-247 Layer 3b created
/// `~/.local/state/zero -> faelight` and `~/.config/zero -> faelight`, and the week between
/// creating them and flipping the writers is exactly when something could quietly disturb them.
///
/// ⚠️ FOUR DISTINCT STATES, NOT TWO. Absent, a REAL DIRECTORY where a link should be (which
/// means writes are splitting between two places), dangling, and resolving somewhere else are
/// all different problems with different fixes. Collapsing them to "broken" would lose the one
/// that matters most -- a real directory looks fine until you notice half your state is missing.
pub fn zero_alias() -> Measurement {
    let pairs = [
        (
            faelight_core::paths::state_home().join("zero"),
            faelight_core::paths::state_home().join("faelight"),
        ),
        (
            faelight_core::paths::config_dir().join("zero"),
            faelight_core::paths::faelight_config_dir(),
        ),
    ];
    let mut issues: Vec<String> = Vec::new();
    for (link, target) in &pairs {
        let name = link.display().to_string();
        match fs::symlink_metadata(link) {
            Err(_) => issues.push(format!("{} absent", name)),
            Ok(meta) => {
                if !meta.file_type().is_symlink() {
                    issues.push(format!("{} is a REAL DIRECTORY, not a link", name));
                } else if !link.exists() {
                    issues.push(format!("{} dangles", name));
                } else {
                    match (fs::canonicalize(link), fs::canonicalize(target)) {
                        (Ok(a), Ok(b)) if a == b => {}
                        (Ok(a), Ok(b)) => issues.push(format!(
                            "{} resolves to {}, not {}",
                            name,
                            a.display(),
                            b.display()
                        )),
                        _ => issues.push(format!("{} could not be resolved", name)),
                    }
                }
            }
        }
    }
    if issues.is_empty() {
        Measurement::pass("state and config aliases resolve to the faelight directories")
    } else {
        Measurement::warn(issues.join("; "))
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
