//! Probes that read the tool registry, and the one parse they share.

use crate::measurement::Measurement;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Registry {
    #[serde(default)]
    tool: Vec<Tool>,
}

#[derive(Debug, Deserialize)]
struct Tool {
    name: String,
    #[serde(default)]
    expected_usage: String,
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    deployable: bool,
    #[serde(default)]
    retired: bool,
}

/// ⭐ ONE PARSE, AND IT IS A REAL ONE.
///
/// Three checks each hand-scanned this file line by line, looking for `[[tool]]` and
/// `name = "` prefixes. A LINE SCANNER CANNOT SEE A DUPLICATE KEY.
///
/// ⚠️ MEASURED 2026-09-19: `tools.toml` HAD BEEN INVALID TOML for an unknown period -- a stray
/// `retired = false` at end of file, duplicating the key in the last tool's block. Every scanner
/// read it happily. The first attempt to parse it properly, for this port, refused it. The
/// registry the doctor trusts was malformed and nothing in the system could notice.
///
/// Returns the error rather than an empty list: AN UNREADABLE REGISTRY IS NOT AN EMPTY ONE, and
/// that distinction is the whole reason these checks were wrong before.
fn read_registry() -> Result<Registry, String> {
    let path = faelight_core::paths::tools_registry();
    if !path.exists() {
        return Err(format!("tool registry not found at {}", path.display()));
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {} -- {}", path.display(), e))?;
    toml::from_str::<Registry>(&text)
        .map_err(|e| format!("{} is not valid TOML -- {}", path.display(), e))
}

fn on_path(name: &str) -> bool {
    which::which(name).is_ok()
}

/// Every tool the registry expects is installed.
///
/// ⚠️ AN UNREADABLE REGISTRY IS NOT AN EMPTY ONE. The old chain ended `unwrap_or_default`, so a
/// missing tools.toml produced an empty list, an empty list has nothing missing, and the check
/// printed a green "All 0 key tools installed". FOUND IN THE VM 2026-08-23 on a guest with
/// THIRTY-TWO tools actually deployed -- so it was not merely silent, it described a machine
/// confidently and wrongly.
pub fn tool_installation() -> Measurement {
    let reg = match read_registry() {
        Ok(r) => r,
        Err(e) => {
            return Measurement::unknown(format!("cannot say what should be installed -- {}", e))
        }
    };
    let want: Vec<&str> = reg
        .tool
        .iter()
        .filter(|t| {
            t.deployable
                && !t.retired
                && (t.expected_usage == "high" || t.expected_usage == "medium")
        })
        .map(|t| t.name.as_str())
        .collect();
    if want.is_empty() {
        return Measurement::unknown("the registry declares no tools to install");
    }
    let missing: Vec<&str> = want.iter().copied().filter(|n| !on_path(n)).collect();
    if missing.is_empty() {
        Measurement::pass(format!("all {} key tools installed", want.len()))
    } else {
        Measurement::warn(format!(
            "{} of {} missing: {}",
            missing.len(),
            want.len(),
            missing.join(", ")
        ))
    }
}

/// Every Rust tool in the registry resolves on PATH.
///
/// ⚠️ A PERCENTAGE OF NOTHING IS NOT ZERO PERCENT. The old chain ended `unwrap_or_default`, so an
/// unreadable registry gave total = 0, and the divide-by-zero guard chose 0 as the answer --
/// printing `0/0 tools deployed (0%)`, WHICH READS AS A MEASUREMENT. The guard was right that
/// zero cannot be divided; it was wrong that the answer is zero.
///
/// ⭐ AND THE QUESTION IS "CAN I RUN THIS", WHICH `which` ANSWERS ON EVERY SYSTEM. This once
/// branched on /etc/NIXOS and, off NixOS, looked inside a source directory -- one deployment
/// shape hardcoded as if it were the only one. On Omarchy with 29 tools on PATH it reported
/// 0/29 while the installation check, which used `which`, reported 24/25. Two checks over the
/// same tools disagreeing completely, and the one that assumed a location was wrong.
pub fn path_resilience() -> Measurement {
    let reg = match read_registry() {
        Ok(r) => r,
        Err(e) => {
            return Measurement::unknown(format!("cannot say what should be deployed -- {}", e))
        }
    };
    let want: Vec<&str> = reg
        .tool
        .iter()
        .filter(|t| t.deployable && !t.retired && t.kind == "rust")
        .map(|t| t.name.as_str())
        .collect();
    let total = want.len();
    if total == 0 {
        return Measurement::unknown("the registry declares no deployable rust tools");
    }
    let deployed = want.iter().filter(|n| on_path(n)).count();
    let pct = (deployed * 100) / total;
    if deployed == total {
        Measurement::pass(format!("{}/{} tools deployed (100%)", deployed, total))
    } else {
        let missing: Vec<&str> = want.iter().copied().filter(|n| !on_path(n)).collect();
        Measurement::warn(format!(
            "{}/{} tools deployed ({}%) -- absent: {}",
            deployed,
            total,
            pct,
            missing.join(", ")
        ))
    }
}

/// The registry files are present and their declared fields check out.
///
/// ⚠️ A FILE THIS CHECK COULD NOT REACH IS NOT A FILE IT VALIDATED. The count in the message was
/// once the size of the list it MEANT to check, so with tools.toml deleted the loop skipped it,
/// issues stayed empty, and the check reported "All 4 registry files valid" having looked at
/// three. PROVEN LIVE 2026-09-05: the registry was moved aside and this check stayed green.
///
/// ⭐ AND "VALID" OVERSTATED IT. This confirms a .schema.json EXISTS and then searches the TOML
/// for empty fields. It parses neither. Real schema validation is separate work, so the message
/// says what it did: present, and field checks clean.
///
/// ⚠️ ONE THING CORRECTED IN THE PORT: the old warn read `"{} schema issue(s): {}", len, [0]` --
/// the count plus THE FIRST ISSUE ONLY. With three problems you were told about one. This
/// intent's own rule is that a count is not a finding, applied here only halfway.
pub fn schema_validation() -> Measurement {
    let schema_dir = faelight_core::paths::schema_dir();
    let registry_dir = faelight_core::paths::registry_dir();
    if !schema_dir.exists() {
        return Measurement::unknown(format!("{} not found", schema_dir.display()));
    }
    const PAIRS: [(&str, &str); 4] = [
        ("tools.toml", "tools.schema.json"),
        ("zones.toml", "zones.schema.json"),
        ("profiles.toml", "profiles.schema.json"),
        ("sandbox-policies.toml", "policies.schema.json"),
    ];
    let mut issues: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let mut validated = 0usize;
    for (registry_file, schema_file) in PAIRS {
        let registry_path = registry_dir.join(registry_file);
        let schema_path = schema_dir.join(schema_file);
        if !registry_path.exists() {
            skipped.push(format!("{} not found", registry_file));
            continue;
        }
        if !schema_path.exists() {
            // A MISSING SCHEMA IS A FINDING, NOT A SKIP. The registry file is right there and
            // unvalidatable, which is a fact about the system rather than an absence of one.
            issues.push(format!("missing schema: {}", schema_file));
            continue;
        }
        match std::fs::read_to_string(&registry_path) {
            Ok(content) => {
                if content.contains("name = \"\"") {
                    issues.push(format!("{}: empty name field", registry_file));
                }
                if content.contains("description = \"\"") {
                    issues.push(format!("{}: empty description", registry_file));
                }
                validated += 1;
            }
            // Exists but unreadable: not missing, not valid.
            Err(err) => skipped.push(format!("{}: {}", registry_file, err)),
        }
    }
    if !skipped.is_empty() {
        return Measurement::unknown(format!(
            "checked {} of {} registry files; could not check: {}",
            validated,
            PAIRS.len(),
            skipped.join(", ")
        ));
    }
    if issues.is_empty() {
        Measurement::pass(format!(
            "{} registry files present, field checks clean",
            validated
        ))
    } else {
        Measurement::warn(issues.join(", "))
    }
}

/// Every deployed tool has an alias.
///
/// ⚠️ BOTH HALVES OF THIS CHECK USED TO COLLAPSE, IN OPPOSITE DIRECTIONS.
///
/// The expectation half returned an empty Vec when the registry could not be read, so nothing
/// was expected, so nothing could be missing, and the check reported clean -- THE SILENT HALF.
///
/// The alias half ended `unwrap_or_default`, turning an unreadable shell config into an EMPTY
/// alias map, so every expected tool read as missing -- THE LOUD HALF. And the Fail arm that
/// said "could not read aliases file" was UNREACHABLE, because the function it called could not
/// return an error. A failure branch that cannot fire is this intent's thesis one layer down.
///
/// ⭐ BOTH ARE `unknown` HERE, AND FOR THE SAME REASON: the aliases are not known to be missing,
/// they are unknown. Reporting Fail would have said twenty-one tools lack aliases when the truth
/// is that nobody could look.
pub fn alias_coverage() -> Measurement {
    let reg = match read_registry() {
        Ok(r) => r,
        Err(e) => return Measurement::unknown(format!("cannot say what needs an alias -- {}", e)),
    };
    let path = faelight_core::paths::shell_config();
    let text = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            return Measurement::unknown(format!("could not read {} -- {}", path.display(), e))
        }
    };

    // The alias VALUES are what matter: an alias is covered when some alias runs the tool.
    let values: Vec<String> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.strip_prefix("alias "))
        .filter_map(|rest| rest.split_once('='))
        .map(|(_, target)| {
            target
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .to_string()
        })
        .collect();
    if values.is_empty() {
        return Measurement::unknown(format!("no aliases found in {}", path.display()));
    }

    let expected: Vec<&str> = reg
        .tool
        .iter()
        .filter(|t| {
            t.deployable
                && !t.retired
                && (t.expected_usage == "high" || t.expected_usage == "medium")
        })
        .map(|t| t.name.as_str())
        // A library crate and a background service are not things anyone types.
        .filter(|n| *n != "faelight-core" && *n != "faelight-daemon")
        .collect();

    let missing: Vec<&str> = expected
        .iter()
        .copied()
        .filter(|tool| !values.iter().any(|v| v.contains(tool)))
        .collect();

    if missing.is_empty() {
        Measurement::pass(format!(
            "all {} tools have aliases ({} aliases total)",
            expected.len(),
            values.len()
        ))
    } else {
        Measurement::warn(format!(
            "{} tools missing aliases: {}",
            missing.len(),
            missing.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Status;

    #[test]
    fn the_registry_parses_and_declares_tools() {
        // ⭐ THIS TEST WOULD HAVE CAUGHT THE MALFORMED FILE. It parses the real registry, so a
        // duplicate key or any other TOML error fails here rather than being read past.
        let reg = read_registry().expect("the tool registry must be valid TOML");
        assert!(
            !reg.tool.is_empty(),
            "the registry declares at least one tool"
        );
    }

    #[test]
    fn tool_installation_never_passes_on_an_empty_expectation() {
        let m = tool_installation();
        assert!(!m.message.is_empty());
        if m.status == Status::Pass {
            assert!(
                !m.message.contains("all 0 "),
                "a pass over zero tools is the defect, not a result: {}",
                m.message
            );
        }
    }

    #[test]
    fn alias_coverage_never_blames_tools_for_its_own_blindness() {
        // ⭐ THE LOUD HALF. An unreadable shell config once made EVERY tool read as missing.
        // If this probe cannot read the config it must say so, not accuse the tools.
        let m = alias_coverage();
        assert!(!m.message.is_empty());
        if m.status == Status::Warn {
            assert!(
                m.message.contains("missing aliases"),
                "a warn must be about aliases, not about reading the config: {}",
                m.message
            );
        }
    }

    #[test]
    fn schema_validation_names_every_issue_not_just_the_first() {
        let m = schema_validation();
        assert!(!m.message.is_empty());
        if m.status == Status::Warn {
            // ⭐ The old message was "N schema issue(s): <the first one>". A warn here must
            // carry what it found, not a count plus a sample.
            assert!(
                !m.message.contains("issue(s):"),
                "a count plus one example is not a finding: {}",
                m.message
            );
        }
    }

    #[test]
    fn path_resilience_never_reports_a_percentage_of_nothing() {
        let m = path_resilience();
        assert!(!m.message.is_empty());
        if matches!(m.status, Status::Pass | Status::Warn) {
            assert!(
                !m.message.starts_with("0/0"),
                "0/0 reads as a measurement and is not one: {}",
                m.message
            );
        }
    }
}
