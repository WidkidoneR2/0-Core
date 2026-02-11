//! Path management for 0-Core numbered gravity structure
//!
//! Philosophy: Paths should be explicit, never hidden.
//! All filesystem assumptions are documented here.
//!
//! If HOME is not set, we FAIL LOUDLY - no silent fallbacks.

use std::env;
use std::path::{Path, PathBuf};

/// Get home directory - PANICS if HOME not set (fail fast philosophy)
pub fn home() -> PathBuf {
    PathBuf::from(
        env::var("HOME")
            .expect("HOME environment variable must be set - this is a critical assumption"),
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

/// Get the applications directory (~/.local/share/applications)
pub fn applications_dir() -> PathBuf {
    home().join(".local/share/applications")
}

pub fn archive_dir() -> PathBuf {
    home().join("3-archive")
}

// SYSTEM FONTS (Common font locations)
// ═══════════════════════════════════════════════════════════

/// System fonts directory
pub fn system_fonts_dir() -> PathBuf {
    PathBuf::from("/usr/share/fonts")
}

/// TTF fonts directory
pub fn ttf_fonts_dir() -> PathBuf {
    system_fonts_dir().join("TTF")
}

/// Hack Nerd Font (commonly used in 0-Core)
pub fn hack_nerd_font() -> PathBuf {
    ttf_fonts_dir().join("HackNerdFont-Regular.ttf")
}

/// JetBrains Mono Nerd Font (used in faelight-bar)
pub fn jetbrains_mono_nerd_font() -> PathBuf {
    ttf_fonts_dir().join("JetBrainsMonoNerdFont-Regular.ttf")
}

/// Helper: Check if a font file exists
pub fn font_exists(font_path: &Path) -> bool {
    font_path.exists()
}

// ═══════════════════════════════════════════════════════════
// USER DATA (XDG-like directories)
// ═══════════════════════════════════════════════════════════

/// User's local data directory (~/.local/share)
pub fn local_data_dir() -> PathBuf {
    home().join(".local/share")
}

/// faelight-link backups directory
pub fn faelight_link_backups() -> PathBuf {
    local_data_dir().join("faelight-link/backups")
}

/// faelight-core state directory
pub fn faelight_state_dir() -> PathBuf {
    home().join(".local/state/0-core")
}

// ═══════════════════════════════════════════════════════════
// GIT REPOSITORY PATHS
// ═══════════════════════════════════════════════════════════

/// Git directory within 0-core
pub fn git_dir() -> PathBuf {
    core_dir().join(".git")
}

/// Git hooks directory (for faelight-git governance)
pub fn git_hooks_dir() -> PathBuf {
    git_dir().join("hooks")
}

/// Git config directory
pub fn git_config_dir() -> PathBuf {
    config_dir().join("git")
}

/// Core lock file (prevents simultaneous operations)
pub fn core_lock_file() -> PathBuf {
    home().join(".0-core-locked")
}

/// Check if 0-core is locked
pub fn is_core_locked() -> bool {
    core_lock_file().exists()
}

// ═══════════════════════════════════════════════════════════
// GIT CONFIGURATION FILES
// ═══════════════════════════════════════════════════════════

/// Gitleaks configuration file in 0-core root
pub fn gitleaks_config() -> PathBuf {
    core_dir().join(".gitleaks.toml")
}

/// Git attributes file
pub fn git_attributes() -> PathBuf {
    core_dir().join(".gitattributes")
}

/// Git ignore file  
pub fn git_ignore() -> PathBuf {
    core_dir().join(".gitignore")
}

// ═══════════════════════════════════════════════════════════
// VERSION MANAGEMENT PATHS

// ═══════════════════════════════════════════════════════════
// ADDITIONAL VERSION MANAGEMENT PATHS
// ═══════════════════════════════════════════════════════════

/// Workspace Cargo.toml
pub fn cargo_toml() -> PathBuf {
    core_dir().join("Cargo.toml")
}

/// Zshrc file in stow
pub fn zshrc_file() -> PathBuf {
    stow_dir().join("shell-zsh/.zshrc")
}

/// Changelog draft for a specific version
pub fn changelog_draft(version: &str) -> PathBuf {
    logs_dir().join(format!("CHANGELOG-v{}-DRAFT.md", version))
}

// ═══════════════════════════════════════════════════════════
// INTENT SYSTEM PATHS
// ═══════════════════════════════════════════════════════════

/// Intent future directory
pub fn intents_future() -> PathBuf {
    intents_dir().join("future")
}

/// Intent complete directory
pub fn intents_complete() -> PathBuf {
    intents_dir().join("complete")
}

/// Intent cancelled directory
pub fn intents_cancelled() -> PathBuf {
    intents_dir().join("cancelled")
}

/// Intent deferred directory
pub fn intents_deferred() -> PathBuf {
    intents_dir().join("deferred")
}

/// Intent decisions directory
pub fn intents_decisions() -> PathBuf {
    intents_dir().join("decisions")
}

/// Intent experiments directory
pub fn intents_experiments() -> PathBuf {
    intents_dir().join("experiments")
}

/// Intent philosophy directory
pub fn intents_philosophy() -> PathBuf {
    intents_dir().join("philosophy")
}

/// Intent incidents directory
pub fn intents_incidents() -> PathBuf {
    intents_dir().join("incidents")
}

// ═══════════════════════════════════════════════════════════
// SHELL CONFIGURATION PATHS
// ═══════════════════════════════════════════════════════════

/// ZSH aliases file
pub fn aliases_file() -> PathBuf {
    stow_dir().join("shell-zsh/.config/zsh/aliases.zsh")
}

// ═══════════════════════════════════════════════════════════
// ENTROPY MONITORING PATHS
// ═══════════════════════════════════════════════════════════

/// Entropy baseline file
pub fn entropy_baseline_file() -> PathBuf {
    faelight_config_dir().join("entropy-baseline.json")
}

/// Entropy history file
pub fn entropy_history_file() -> PathBuf {
    faelight_config_dir().join("entropy-history.json")
}

// ═══════════════════════════════════════════════════════════
// PROFILE STATE FILES
// ═══════════════════════════════════════════════════════════

/// Current active profile file
pub fn current_profile_file() -> PathBuf {
    faelight_state_dir().join("current-profile")
}

/// Profile operation log
pub fn profile_log_file() -> PathBuf {
    faelight_state_dir().join("profile.log")
}

// ═══════════════════════════════════════════════════════════
// SWAY WINDOW MANAGER
// ═══════════════════════════════════════════════════════════

/// Sway config file
pub fn sway_config() -> PathBuf {
    home().join(".config/sway/config")
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

// ═══════════════════════════════════════════════════════════
