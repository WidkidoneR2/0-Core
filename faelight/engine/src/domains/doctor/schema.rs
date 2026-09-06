// doctor/schema.rs — registry schema validation check
use super::{CheckResult, Status, Tier};
use std::fs;

pub fn check_schema_validation(_core_root: &str) -> CheckResult {
    let schema_dir = faelight_core::paths::schema_dir();
    let registry_dir = faelight_core::paths::registry_dir();

    if !schema_dir.exists() {
        return CheckResult {
            tier: Tier::System,
            id: "schema_validation".into(),
            name: "Schema Validation".into(),
            status: Status::Warn,
            message: "schema/ directory not found".into(),
            fix: Some("Create schema/ directory".into()),
        };
    }

    // Validate each registry file against its schema
    let pairs = [
        ("tools.toml", "tools.schema.json"),
        ("zones.toml", "zones.schema.json"),
        ("profiles.toml", "profiles.schema.json"),
        ("sandbox-policies.toml", "policies.schema.json"),
    ];

    let mut issues = Vec::new();
    // INT-222: a file this check could not reach is NOT a file it validated. The count in the
    // message below used to be pairs.len() -- the size of the list it MEANT to check -- so with
    // tools.toml deleted the loop skipped it, issues stayed empty, and the check reported
    // "All 4 registry files valid" having looked at three. Proven live 2026-09-05 while
    // verifying INT-192: the registry was moved aside and this check stayed green.
    let mut skipped: Vec<String> = Vec::new();
    let mut validated = 0usize;

    for (registry_file, schema_file) in &pairs {
        let registry_path = registry_dir.join(registry_file);
        let schema_path = schema_dir.join(schema_file);

        if !registry_path.exists() {
            skipped.push(format!("{} not found", registry_file));
            continue;
        }

        if !schema_path.exists() {
            // A MISSING SCHEMA IS A FINDING, NOT A SKIP. The registry file is right there and
            // unvalidatable, which is a fact about the system rather than an absence of knowledge.
            issues.push(format!("missing schema: {}", schema_file));
            continue;
        }

        // Basic field checks. NOTE: this does not parse the schema -- see the message wording.
        match fs::read_to_string(&registry_path) {
            Ok(content) => {
                if content.contains("name = \"\"") {
                    issues.push(format!("{}: empty name field", registry_file));
                }
                if content.contains("description = \"\"") {
                    issues.push(format!("{}: empty description", registry_file));
                }
                validated += 1;
            }
            // Exists but unreadable: the third collapse in this loop. `if let Ok` discarded the
            // error and the file contributed no issues, which read as clean.
            Err(err) => skipped.push(format!("{}: {}", registry_file, err)),
        }
    }

    if !skipped.is_empty() {
        // A check that could not reach every subject has not passed on all of them. Unknown
        // rather than Warn: nothing here is known to be WRONG, it is unknown (INT-148, INT-192).
        return CheckResult {
            tier: Tier::System,
            id: "schema_validation".into(),
            name: "Schema Validation".into(),
            status: Status::Unknown,
            message: format!(
                "checked {} of {} registry files; could not check: {}",
                validated,
                pairs.len(),
                skipped.join(", ")
            ),
            fix: Some("Restore the missing registry file, or run: core registry validate".into()),
        };
    }

    if issues.is_empty() {
        CheckResult {
            tier: Tier::System,
            id: "schema_validation".into(),
            name: "Schema Validation".into(),
            status: Status::Pass,
            // ⚠️ WORDING IS DELIBERATE. This said "All N registry files valid" for a check that
            // confirms a .schema.json EXISTS and then substring-searches the TOML for two empty
            // fields. It never parses either file, so "valid" overstated it. Real schema
            // validation is a separate piece of work; the message now says what it does.
            message: format!("{} registry files present, field checks clean", validated),
            fix: None,
        }
    } else {
        CheckResult {
            tier: Tier::System,
            id: "schema_validation".into(),
            name: "Schema Validation".into(),
            status: Status::Warn,
            message: format!("{} schema issue(s): {}", issues.len(), issues[0]),
            fix: Some("Run: core registry validate".into()),
        }
    }
}
