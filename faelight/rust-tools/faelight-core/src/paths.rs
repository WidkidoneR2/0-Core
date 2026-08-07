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

/// INT-061 v2: the Faelight platform domain under the repo root. Dirs owned by
/// the platform (policy, registry, intents, engine state, etc.) live here so the
/// tree encodes the OS/platform seam. Relocating the platform half = editing this
/// one helper + the per-dir accessors that build on it.
pub fn faelight_dir() -> PathBuf {
    core_dir().join("faelight")
}

// ═══════════════════════════════════════════════════════════
// 00-META: System Identity & Documentation
// ═══════════════════════════════════════════════════════════

pub fn meta_dir() -> PathBuf {
    faelight_dir().join("meta")
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
    faelight_dir().join("registry")
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

pub fn policy_dir() -> PathBuf {
    faelight_dir().join("policy")
}

pub fn hooks_dir() -> PathBuf {
    policy_dir().join("hooks")
}

pub fn security_dir() -> PathBuf {
    policy_dir().join("security")
}

// ═══════════════════════════════════════════════════════════
// 03-INTERFACES: User-Facing Configs
// ═══════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════
// 04-RUNTIME: Execution & Build Artifacts
// ═══════════════════════════════════════════════════════════

pub fn runtime_dir() -> PathBuf {
    faelight_dir().join("runtime")
}

/// The single canonical state database. Every tool MUST resolve state.db
/// through this function -- never hardcode "runtime/state.db". INT-061: this is
/// the seam that makes moving runtime/ a one-line change here.
pub fn state_db() -> PathBuf {
    // INT-204: one escape hatch, and it lives HERE because this function is the seam. A second place
    // resolving the database is what the note above forbids, so the override is read once, at the
    // decider, and everything else keeps calling this.
    //
    // WHY IT EXISTS: fsh-test spawns a shell per case against the user's real database, so a case
    // that creates an alias leaves it behind. The suite has been getting its isolation by accident --
    // config::apply prunes aliases absent from config.fsh at every startup, which happens to sweep
    // test pollution -- and that stops the moment config is overridden.
    //
    // NOT fsh-prefixed on purpose: this crate is shared, so the name has to describe the reach.
    //
    // ⚠️ A LEAKED VALUE HERE IS SERIOUS -- a shell pointed at a scratch file has no history, no
    // aliases and no session memory. That is why callers announce a non-canonical database rather
    // than resolving it silently; see the startup notice in faelight-shell.
    if let Ok(p) = env::var("FAELIGHT_STATE_DB") {
        if !p.trim().is_empty() {
            return PathBuf::from(p);
        }
    }
    runtime_dir().join("state.db")
}

/// Schema directory (registry JSON schemas read by engine doctor/bootstrap).
pub fn schema_dir() -> PathBuf {
    faelight_dir().join("schema")
}

/// core_root as a String, for consumers (e.g. engine AppContext) that store the
/// root as a String rather than a PathBuf. Single source of truth: derived from
/// core_dir(), NOT re-computed via format!("{}/0-core", home).
pub fn core_root_string() -> String {
    core_dir().to_string_lossy().to_string()
}

pub fn target_dir() -> PathBuf {
    core_dir().join("target")
}

pub fn logs_dir() -> PathBuf {
    runtime_dir().join("logs")
}

pub fn backups_dir() -> PathBuf {
    runtime_dir().join("backups")
}

// INT-061: runtime subdirectories -- every runtime/* path routes through here so
// relocating runtime/ (e.g. to faelight/runtime/) is a one-line change above.
pub fn checkpoints_dir() -> PathBuf {
    runtime_dir().join("checkpoints")
}

pub fn events_dir() -> PathBuf {
    runtime_dir().join("events")
}

pub fn reactions_dir() -> PathBuf {
    runtime_dir().join("reactions")
}

pub fn cache_dir() -> PathBuf {
    runtime_dir().join("cache")
}

pub fn health_cache() -> PathBuf {
    cache_dir().join("health.txt")
}

pub fn forecast_cache() -> PathBuf {
    cache_dir().join("forecast.txt")
}

pub fn reactions_config() -> PathBuf {
    reactions_dir().join("custom.toml")
}

pub fn capabilities_log() -> PathBuf {
    logs_dir().join("capabilities.jsonl")
}

// ═══════════════════════════════════════════════════════════
// ROOT DIRECTORIES (Unnumbered)
// ═══════════════════════════════════════════════════════════

pub fn intents_dir() -> PathBuf {
    faelight_dir().join("intents")
}

pub fn docs_dir() -> PathBuf {
    core_dir().join("docs")
}

pub fn rust_tools_dir() -> PathBuf {
    faelight_dir().join("rust-tools")
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
pub fn scratch_dir() -> PathBuf {
    home().join("scratch")
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

/// fsh config file (aliases live here)
pub fn aliases_file() -> PathBuf {
    core_dir().join("nix/home/dotfiles/faelight-shell/.config/faelight-shell/config.fsh")
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
        assert!(path.to_string_lossy().contains("meta/VERSION"));
    }

    #[test]
    fn test_numbered_gravity() {
        // Verify flat NixOS-era structure (numbered gravity retired -- INT-105)
        assert!(meta_dir().to_string_lossy().contains("meta"));
        assert!(registry_dir().to_string_lossy().contains("registry"));
        assert!(policy_dir().to_string_lossy().contains("policy"));
        assert!(runtime_dir().to_string_lossy().contains("runtime"));
    }
}

// ═══════════════════════════════════════════════════════════
