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
    check: Vec<toml::Value>,
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

/// One check as declared: readable, or readable except for a probe name this build lacks.
enum Declared {
    Known(Definition),
    UnknownProbe { id: String, probe: String },
}

/// Read one check on its own, so a check this build cannot fully read does not take the whole
/// file down with it.
///
/// ⚠️ A check that fails to read but reads once its `probe` key is removed has exactly one
/// problem: a probe name from a newer core than this binary. That check is refused by name and
/// the rest of the set still runs. Any other fault stays a fatal `Malformed`, as before.
fn read_definition(v: toml::Value, path: &str) -> Result<Declared, LoadError> {
    match v.clone().try_into::<Definition>() {
        Ok(d) => Ok(Declared::Known(d)),
        Err(e) => {
            let id = v.get("id").and_then(|i| i.as_str()).map(str::to_string);
            let probe = v.get("probe").and_then(|p| p.as_str()).map(str::to_string);
            let mut without = v;
            if let Some(table) = without.as_table_mut() {
                table.remove("probe");
            }
            match (id, probe, without.try_into::<Definition>()) {
                (Some(id), Some(probe), Ok(_)) => Ok(Declared::UnknownProbe { id, probe }),
                _ => Err(LoadError::Malformed {
                    path: path.into(),
                    why: e.to_string(),
                }),
            }
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
        let mut declared = Vec::new();
        for v in file.check {
            declared.push(read_definition(v, path)?);
        }
        let mut seen = std::collections::HashSet::new();
        for d in &declared {
            let id = match d {
                Declared::Known(d) => &d.id,
                Declared::UnknownProbe { id, .. } => id,
            };
            if !seen.insert(id.clone()) {
                return Err(LoadError::DuplicateId { id: id.clone() });
            }
        }
        let mut checks = Vec::new();
        let mut refused = Vec::new();
        for d in declared {
            match d {
                Declared::Known(d) => match d.validate() {
                    Ok(()) => checks.push(d),
                    Err(e) => refused.push((d.id.clone(), e)),
                },
                Declared::UnknownProbe { id, probe } => {
                    refused.push((id.clone(), DefinitionError::UnknownProbe { id, probe }))
                }
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
        assert_eq!(r.checks.len(), 28, "all 28 registered checks are declared");
    }

    #[test]
    fn labels_are_excluded_from_the_denominator() {
        let r = Registry::parse(REAL, "checks.toml").unwrap();
        let mut labels: Vec<&str> = r.labels().map(|d| d.id.as_str()).collect();
        labels.sort();
        // Both measure truly and never judge: the cache size and the VM count are facts
        // a human interprets, not verdicts.
        assert_eq!(labels, vec!["package_cache", "vm_state"]);
        assert_eq!(r.judging().count(), 26);
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
    fn a_probe_this_build_does_not_know_refuses_that_check_and_not_the_file() {
        let text = r#"
[[check]]
id = "old"
name = "Old"
section = "S"
tier = "user"
probe = "disk_space"
severities = ["pass", "warn"]

[[check]]
id = "from_the_future"
name = "Future"
section = "S"
tier = "user"
probe = "a_probe_from_a_newer_core"
severities = ["pass", "warn"]
"#;
        let r = Registry::parse(text, "t").expect("one unknown probe must not fail the load");
        assert_eq!(r.checks.len(), 1);
        assert_eq!(r.checks[0].id, "old");
        assert_eq!(r.refused.len(), 1);
        assert_eq!(r.refused[0].0, "from_the_future");
        assert!(matches!(
            r.refused[0].1,
            DefinitionError::UnknownProbe { .. }
        ));
    }

    #[test]
    fn a_newer_checks_toml_never_blanks_an_older_binary() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../registry/doctor/checks.toml"
        );
        let text = std::fs::read_to_string(path).expect("the real check set is readable");
        let future: String = text
            .lines()
            .map(|l| match l.starts_with("probe = \"") {
                true => l.replacen("probe = \"", "probe = \"newer_core_", 1),
                false => l.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n");
        let total = text.lines().filter(|l| l.starts_with("[[check]]")).count();
        let probed = text.lines().filter(|l| l.starts_with("probe = \"")).count();
        assert!(probed > 0, "the real set declares probes");
        let r = Registry::parse(&future, "future").expect("unknown probes must not fail the load");
        assert_eq!(r.refused.len(), probed);
        assert_eq!(r.checks.len(), total - probed);
        assert!(r
            .refused
            .iter()
            .all(|(_, e)| matches!(e, DefinitionError::UnknownProbe { .. })));
    }

    #[test]
    fn an_unknown_probe_does_not_excuse_another_fault_in_the_same_check() {
        let text = r#"
[[check]]
id = "both_wrong"
name = "Both"
section = "S"
tier = "nonsense"
probe = "a_probe_from_a_newer_core"
severities = ["pass"]
"#;
        assert!(matches!(
            Registry::parse(text, "t"),
            Err(LoadError::Malformed { .. })
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
