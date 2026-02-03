//! Path management for 0-Core numbered gravity structure
//! 
//! Philosophy: Paths should be explicit, never hidden.
//! All filesystem assumptions are documented here.
//! 
//! If HOME is not set, we FAIL LOUDLY - no silent fallbacks.

use std::env;
use std::path::PathBuf;

/// Get home directory - PANICS if HOME not set (fail fast philosophy)
fn home() -> PathBuf {
    PathBuf::from(
        env::var("HOME").expect("HOME environment variable must be set - this is a critical assumption")
    )
}

// ═══════════════════════════════════════════════════════════
// 0-CORE ROOT
// ═══════════════════════════════════════════════════════════

/// Path to 0-core root directory
pub fn core_dir() -> PathBuf {
    home().join("0-core")
}

// ═══════════════════════════════════════════════════════════
// 00-META: System Identity & Documentation
// ═══════════════════════════════════════════════════════════

pub fn meta_dir() -> PathBuf {
    core_dir().join("00-meta")
}

pub fn version_file() -> PathBuf {
    meta_dir().join("VERSION")
}

pub fn changelog_file() -> PathBuf {
    meta_dir().join("CHANGELOG.md")
}

pub fn readme_file() -> PathBuf {
    meta_dir().join("README.md")
}

// ═══════════════════════════════════════════════════════════
// 01-REGISTRY: Tool & Alias Registry
// ═══════════════════════════════════════════════════════════

pub fn registry_dir() -> PathBuf {
    core_dir().join("01-registry")
}

pub fn tools_registry() -> PathBuf {
    registry_dir().join("tools.toml")
}

pub fn aliases_registry() -> PathBuf {
    registry_dir().join("aliases.toml")
}

pub fn zones_registry() -> PathBuf {
    registry_dir().join("zones.toml")
}

// ═══════════════════════════════════════════════════════════
// 02-RULES: Governance & Security
// ═══════════════════════════════════════════════════════════

pub fn rules_dir() -> PathBuf {
    core_dir().join("02-rules")
}

pub fn hooks_dir() -> PathBuf {
    rules_dir().join("hooks")
}

pub fn security_dir() -> PathBuf {
    rules_dir().join("security")
}

// ═══════════════════════════════════════════════════════════
// 03-INTERFACES: User-Facing Configs
// ═══════════════════════════════════════════════════════════

pub fn interfaces_dir() -> PathBuf {
    core_dir().join("03-interfaces")
}

pub fn stow_dir() -> PathBuf {
    interfaces_dir().join("stow")
}

pub fn profiles_dir() -> PathBuf {
    interfaces_dir().join("profiles")
}

pub fn themes_dir() -> PathBuf {
    interfaces_dir().join("themes")
}

// ═══════════════════════════════════════════════════════════
// 04-RUNTIME: Execution & Build Artifacts
// ═══════════════════════════════════════════════════════════

pub fn runtime_dir() -> PathBuf {
    core_dir().join("04-runtime")
}

pub fn target_dir() -> PathBuf {
    runtime_dir().join("target")
}

pub fn logs_dir() -> PathBuf {
    runtime_dir().join("logs")
}

pub fn backups_dir() -> PathBuf {
    runtime_dir().join("backups")
}

// ═══════════════════════════════════════════════════════════
// ROOT DIRECTORIES (Unnumbered)
// ═══════════════════════════════════════════════════════════

pub fn intents_dir() -> PathBuf {
    core_dir().join("intents")
}

pub fn docs_dir() -> PathBuf {
    core_dir().join("docs")
}

pub fn rust_tools_dir() -> PathBuf {
    core_dir().join("rust-tools")
}

pub fn scripts_dir() -> PathBuf {
    core_dir().join("scripts")
}

// ═══════════════════════════════════════════════════════════
// USER CONFIG (Outside 0-core)
// ═══════════════════════════════════════════════════════════

pub fn config_dir() -> PathBuf {
    home().join(".config")
}

pub fn faelight_config_dir() -> PathBuf {
    config_dir().join("faelight")
}

pub fn profile_file() -> PathBuf {
    faelight_config_dir().join("profile")
}

// ═══════════════════════════════════════════════════════════
// ZONES (Numbered Gravity Beyond 0-core)
// ═══════════════════════════════════════════════════════════

pub fn src_dir() -> PathBuf {
    home().join("1-src")
}

pub fn projects_dir() -> PathBuf {
    home().join("2-projects")
}

pub fn archive_dir() -> PathBuf {
    home().join("3-archive")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_file_path() {
        let path = version_file();
        assert!(path.to_string_lossy().contains("00-meta/VERSION"));
    }

    #[test]
    fn test_numbered_gravity() {
        // Verify numbered structure is preserved
        assert!(meta_dir().to_string_lossy().contains("00-meta"));
        assert!(registry_dir().to_string_lossy().contains("01-registry"));
        assert!(rules_dir().to_string_lossy().contains("02-rules"));
        assert!(interfaces_dir().to_string_lossy().contains("03-interfaces"));
        assert!(runtime_dir().to_string_lossy().contains("04-runtime"));
    }
}
