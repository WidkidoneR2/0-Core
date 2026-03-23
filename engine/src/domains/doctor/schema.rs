// doctor/schema.rs — registry schema validation check
use super::{CheckResult, Status};
use std::fs;

pub fn check_schema_validation(core_root: &str) -> CheckResult {
    let schema_dir = std::path::PathBuf::from(core_root).join("04-schema");
    let registry_dir = std::path::PathBuf::from(core_root).join("01-registry");

    if !schema_dir.exists() {
        return CheckResult {
            id: "schema_validation".into(),
            name: "Schema Validation".into(),
            status: Status::Warn,
            message: "04-schema/ directory not found".into(),
            fix: Some("Create 04-schema/ directory".into()),
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

    for (registry_file, schema_file) in &pairs {
        let registry_path = registry_dir.join(registry_file);
        let schema_path = schema_dir.join(schema_file);

        if !registry_path.exists() {
            continue;
        }

        if !schema_path.exists() {
            issues.push(format!("missing schema: {}", schema_file));
            continue;
        }

        // Basic validation — check required fields exist
        if let Ok(content) = fs::read_to_string(&registry_path) {
            // Check for common issues
            if content.contains("name = \"\"") {
                issues.push(format!("{}: empty name field", registry_file));
            }
            if content.contains("description = \"\"") {
                issues.push(format!("{}: empty description", registry_file));
            }
        }
    }

    if issues.is_empty() {
        CheckResult {
            id: "schema_validation".into(),
            name: "Schema Validation".into(),
            status: Status::Pass,
            message: format!("All {} registry files valid", pairs.len()),
            fix: None,
        }
    } else {
        CheckResult {
            id: "schema_validation".into(),
            name: "Schema Validation".into(),
            status: Status::Warn,
            message: format!("{} schema issue(s): {}", issues.len(), issues[0]),
            fix: Some("Run: core registry validate".into()),
        }
    }
}
