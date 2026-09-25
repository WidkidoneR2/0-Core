//! Loading a check set from TOML, and refusing what does not hold up.
//!
//! ⚠️ EVERY DEFINITION IS VALIDATED AT LOAD, AND A BAD ONE IS NAMED. This is what makes TOML
//! acceptable: a malformed definition would otherwise be a runtime surprise where a Rust array
//! was a compile error. Refusing it loudly, by id, with the reason, is the trade.
//!
//! The load does NOT stop at the first bad definition. It collects every refusal, because
//! "fix one, run again, find the next" is a worse tool than "here is everything wrong".

use crate::definition::{Definition, DefinitionError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct File {
    #[serde(default)]
    check: Vec<Definition>,
}

/// What a load produced: the definitions that hold up, and every one that did not.
#[derive(Debug)]
pub struct Registry {
    pub checks: Vec<Definition>,
    pub refused: Vec<(String, DefinitionError)>,
}

/// Why a check set could not be read at all -- as opposed to individual definitions failing.
#[derive(Debug)]
pub enum LoadError {
    Unreadable {
        path: String,
        why: String,
    },
    Malformed {
        path: String,
        why: String,
    },
    /// ⚠️ An EMPTY check set is an error, not an empty success. A doctor with no checks would
    /// report 0/0 and render as a clean pass -- the free pass INT-222 exists to remove.
    Empty {
        path: String,
    },
    /// Two definitions claiming the same id: one of them is silently unreachable.
    DuplicateId {
        id: String,
    },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Unreadable { path, why } => {
                write!(f, "cannot read the check set at {}: {}", path, why)
            }
            LoadError::Malformed { path, why } => {
                write!(f, "the check set at {} is not valid TOML: {}", path, why)
            }
            LoadError::Empty { path } => write!(
                f,
                "the check set at {} declares no checks -- a doctor with nothing to measure \
                 reports a clean pass, which is the defect this format exists to remove",
                path
            ),
            LoadError::DuplicateId { id } => write!(
                f,
                "two definitions claim the id {} -- one of them would never be reachable",
                id
            ),
        }
    }
}

impl Registry {
    /// Parse a check set and validate every definition in it.
    ///
    /// Definitions that fail validation are moved to `refused`, NOT dropped: the caller decides
    /// whether a refusal is fatal, and can report it by name either way.
    pub fn parse(text: &str, path: &str) -> Result<Registry, LoadError> {
        let file: File = toml::from_str(text).map_err(|e| LoadError::Malformed {
            path: path.into(),
            why: e.to_string(),
        })?;
        if file.check.is_empty() {
            return Err(LoadError::Empty { path: path.into() });
        }
        let mut seen = std::collections::HashSet::new();
        for d in &file.check {
            if !seen.insert(d.id.clone()) {
                return Err(LoadError::DuplicateId { id: d.id.clone() });
            }
        }
        let mut checks = Vec::new();
        let mut refused = Vec::new();
        for d in file.check {
            match d.validate() {
                Ok(()) => checks.push(d),
                Err(e) => refused.push((d.id.clone(), e)),
            }
        }
        Ok(Registry { checks, refused })
    }

    /// Read and parse a check set from disk.
    pub fn load(path: &std::path::Path) -> Result<Registry, LoadError> {
        let p = path.to_string_lossy().to_string();
        let text = std::fs::read_to_string(path).map_err(|e| LoadError::Unreadable {
            path: p.clone(),
            why: e.to_string(),
        })?;
        Registry::parse(&text, &p)
    }

    /// The definitions that JUDGE -- the denominator. Labels are excluded by construction.
    pub fn judging(&self) -> impl Iterator<Item = &Definition> {
        self.checks.iter().filter(|d| !d.is_label())
    }

    /// The definitions that only report.
    pub fn labels(&self) -> impl Iterator<Item = &Definition> {
        self.checks.iter().filter(|d| d.is_label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ THE REAL CHECK SET, NOT A FIXTURE. The file this loads is the one the doctor
    /// will use, and it is compiled in so the test cannot drift from it.
    const REAL: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../registry/doctor/checks.toml"
    ));

    #[test]
    fn the_real_check_set_is_entirely_valid() {
        let r = Registry::parse(REAL, "checks.toml").expect("the check set parses");

        // ⭐ THIS TEST USED TO ASSERT THE OPPOSITE, AND THAT IS THE POINT.
        //
        // It was written as `the_real_check_set_loads_and_refuses_vm_state`, because
        // vm_state was transcribed into TOML exactly as the old code had it -- info tier,
        // declaring warn -- so the engine refused it by name on every load. That refusal
        // was gate 3 proven on live data rather than on a fabricated example.
        //
        // The probe has since been ported and returns pass or unknown, never warn, so the
        // declaration was made true and the refusal correctly stopped. What is asserted now
        // is the INVARIANT, not the snapshot: EVERY DEFINITION HOLDS UP.
        assert!(
            r.refused.is_empty(),
            "the shipped check set must not contain a definition the engine refuses: {:?}",
            r.refused
        );
        assert_eq!(r.checks.len(), 27, "all 27 registered checks are declared");
    }

    #[test]
    fn labels_are_excluded_from_the_denominator() {
        let r = Registry::parse(REAL, "checks.toml").unwrap();
        let mut labels: Vec<&str> = r.labels().map(|d| d.id.as_str()).collect();
        labels.sort();
        // Both measure truly and never judge: the cache size and the VM count are facts
        // a human interprets, not verdicts.
        assert_eq!(labels, vec!["package_cache", "vm_state"]);
        assert_eq!(r.judging().count(), 25);
        assert_eq!(r.judging().count() + labels.len(), r.checks.len());
    }

    #[test]
    fn an_empty_check_set_is_an_error_not_a_pass() {
        assert!(matches!(
            Registry::parse("", "empty.toml"),
            Err(LoadError::Empty { .. })
        ));
    }

    #[test]
    fn a_duplicate_id_is_refused() {
        let t = r#"
            [[check]]
            id = "a"
            name = "A"
            section = "Test"
            tier = "user"
            probe = "friday"
            severities = ["pass"]
            [[check]]
            id = "a"
            name = "Again"
            section = "Test"
            tier = "user"
            probe = "network"
            severities = ["pass"]
        "#;
        assert!(matches!(
            Registry::parse(t, "dup.toml"),
            Err(LoadError::DuplicateId { .. })
        ));
    }
}
