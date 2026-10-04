//! INT-271: the one owner of a crate's version write.
//!
//! bump-versions (novashell) and cicomplete (engine) both call this. Before INT-271 each carried
//! its own copy, engine_apply_bump and apply_version_bump, and the engine copy existed only to
//! avoid depending on NovaShell. zero-core sits below both, so neither depends on the other.
//!
//! plan_bump runs every check and touches nothing; apply_bump is plan_bump plus one write. A close
//! that must validate every crate before writing any calls plan_bump for all of them first.

use std::path::Path;

/// A level a human chose. The three words are spelled here and nowhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Patch,
    Minor,
    Major,
}

impl Level {
    pub fn parse(s: &str) -> Option<Level> {
        match s {
            "patch" => Some(Level::Patch),
            "minor" => Some(Level::Minor),
            "major" => Some(Level::Major),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Level::Patch => "patch",
            Level::Minor => "minor",
            Level::Major => "major",
        }
    }
}

/// Bump an `x.y.z` version. Anything that is not exactly three numeric parts is refused.
pub fn bump(ver: &str, level: Level) -> Result<String, String> {
    let parts: Vec<&str> = ver.split('.').collect();
    let nums: Vec<u64> = parts.iter().filter_map(|p| p.parse().ok()).collect();
    if parts.len() != 3 || nums.len() != 3 {
        return Err(format!("version '{}' is not x.y.z semver", ver));
    }
    Ok(match level {
        Level::Patch => format!("{}.{}.{}", nums[0], nums[1], nums[2] + 1),
        Level::Minor => format!("{}.{}.0", nums[0], nums[1] + 1),
        Level::Major => format!("{}.0.0", nums[0] + 1),
    })
}

/// What a bump would do, computed without touching disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    pub old: String,
    pub new: String,
    pub content: String,
}

/// Every check, no write. The manifest must hold exactly one line that starts `version = `
/// after leading whitespace, and that line must be uniquely matchable for the replace.
pub fn plan_bump(content: &str, level: Level) -> Result<Planned, String> {
    let lines: Vec<&str> = content
        .lines()
        .filter(|l| l.trim_start().starts_with("version = "))
        .collect();
    if lines.len() != 1 {
        return Err(format!(
            "expected exactly 1 `version = ` line, found {}",
            lines.len()
        ));
    }
    let old_line = lines[0];
    let old = old_line
        .trim()
        .trim_start_matches("version = ")
        .trim_matches('"')
        .to_string();
    let new = bump(&old, level)?;
    if content.matches(old_line).count() != 1 {
        return Err("version line not uniquely matchable".to_string());
    }
    let new_line = old_line.replace(&old, &new);
    Ok(Planned {
        content: content.replacen(old_line, &new_line, 1),
        old,
        new,
    })
}

/// plan_bump, then one write. Returns (old, new). Nothing is written if any check fails.
pub fn apply_bump(path: &Path, level: Level) -> Result<(String, String), String> {
    let shown = path.display();
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", shown, e))?;
    let planned = plan_bump(&content, level)
        .map_err(|e| format!("{}: {} -- aborting (no write)", shown, e))?;
    std::fs::write(path, &planned.content).map_err(|e| format!("cannot write {}: {}", shown, e))?;
    Ok((planned.old, planned.new))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOML: &str = "[package]\nname = \"demo\"\nversion = \"1.4.9\"\nedition = \"2021\"\n\n[dependencies]\nserde = { version = \"1\" }\n";

    #[test]
    fn levels_bump_the_right_digit() {
        assert_eq!(bump("1.4.9", Level::Patch).unwrap(), "1.4.10");
        assert_eq!(bump("1.4.9", Level::Minor).unwrap(), "1.5.0");
        assert_eq!(bump("1.4.9", Level::Major).unwrap(), "2.0.0");
    }

    #[test]
    fn a_version_that_is_not_three_numbers_is_refused() {
        // Before INT-271 the parse dropped parts that were not numbers, so 1.2.x.3 bumped to 1.2.4.
        for bad in ["1.2", "1.2.3.4", "1.2.x.3", "1.2.3-beta", "", "a.b.c"] {
            assert!(
                bump(bad, Level::Patch).is_err(),
                "{} should be refused",
                bad
            );
        }
    }

    #[test]
    fn level_words_round_trip_and_nothing_else_parses() {
        for l in [Level::Patch, Level::Minor, Level::Major] {
            assert_eq!(Level::parse(l.as_str()), Some(l));
        }
        for bad in ["Patch", "skip", "", "ptach", "novashell"] {
            assert_eq!(Level::parse(bad), None);
        }
    }

    #[test]
    fn plan_rewrites_only_the_package_version_line() {
        let p = plan_bump(TOML, Level::Minor).unwrap();
        assert_eq!((p.old.as_str(), p.new.as_str()), ("1.4.9", "1.5.0"));
        assert_eq!(
            p.content,
            TOML.replace("version = \"1.4.9\"", "version = \"1.5.0\"")
        );
    }

    #[test]
    fn plan_refuses_zero_or_two_version_lines() {
        assert!(plan_bump("[package]\nname = \"x\"\n", Level::Patch).is_err());
        assert!(plan_bump(
            "version = \"1.0.0\"\n[x]\n  version = \"2.0.0\"\n",
            Level::Patch
        )
        .is_err());
    }

    #[test]
    fn apply_writes_on_success_and_leaves_the_file_alone_on_refusal() {
        let dir = std::env::temp_dir().join(format!("zero-core-version-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let good = dir.join("good.toml");
        std::fs::write(&good, TOML).unwrap();
        assert_eq!(
            apply_bump(&good, Level::Patch).unwrap(),
            ("1.4.9".to_string(), "1.4.10".to_string())
        );
        assert!(std::fs::read_to_string(&good)
            .unwrap()
            .contains("version = \"1.4.10\""));

        let bad = dir.join("bad.toml");
        let two = "version = \"1.0.0\"\n  version = \"2.0.0\"\n";
        std::fs::write(&bad, two).unwrap();
        assert!(apply_bump(&bad, Level::Patch).is_err());
        assert_eq!(std::fs::read_to_string(&bad).unwrap(), two);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
