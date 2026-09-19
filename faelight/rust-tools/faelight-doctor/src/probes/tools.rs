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
