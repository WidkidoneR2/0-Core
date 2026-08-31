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

/// XDG state home: `$XDG_STATE_HOME`, or `~/.local/state` when unset. The spec
/// defines it as state that persists between restarts but is not important or
/// portable enough for `$XDG_DATA_HOME` -- logs, history, current application
/// state. That is exactly what runtime_dir() holds.
///
/// Kept SEPARATE from runtime_dir so XDG policy and the Faelight namespace stay
/// independent of each other.
pub fn state_home() -> PathBuf {
    match env::var("XDG_STATE_HOME") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => home().join(".local/state"),
    }
}

/// XDG executable home: `$XDG_BIN_HOME`, or `~/.local/bin` when unset.
///
/// ⚠️ THE ONLY OWNER OF WHERE A BINARY LIVES. Before this existed there were
/// THIRTY-SIX sites building the path themselves, across FIVE different layouts:
/// `scripts/` (deleted in e733287d), `bin/`, `/run/current-system/sw/bin`,
/// `pkgs/faelight/scripts/`, and `~/.local/bin`. Six of them still EXECUTE a
/// binary at a path that has not existed for months, and they fail silently.
///
/// Nobody noticed because on NixOS none of them were load-bearing: the rebuild
/// put binaries in the store and regenerated the PATH directory itself, so no
/// tool ever had to copy one. On Arch there is no reconciler, so this path is
/// real and it needs exactly one owner -- the same lesson runtime_dir() learned
/// in INT-061, where one helper let the whole tree move and nothing recreated
/// the old location.
///
/// Kept SEPARATE from any tool that writes here, so XDG policy stays one fact
/// and deployment stays a separate act.
pub fn bin_dir() -> PathBuf {
    match env::var("XDG_BIN_HOME") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => home().join(".local/bin"),
    }
}

/// Machine-local state: state.db, logs, cache, snapshots, locks, backups,
/// journal, events.
///
/// ⚠️ THIS USED TO LIVE AT faelight_dir()/runtime, INSIDE THE REPO, under a
/// header reading "Execution & Build Artifacts". Nothing in it is a build
/// artifact -- those are in core_dir()/target -- so it was misfiled against its
/// own section. Measured 2026-08-21: starting fsh with a HOME that had no forest
/// CREATED one, because this path is derived from the repo root. No other shell
/// stores state in a source tree, and every one of them (bash HISTFILE, OSH
/// HISTFILE, YSH YSH_HISTFILE) makes the location overridable.
///
/// RESOLUTION ORDER, chosen so the code can ship before the data moves:
///   1. $FAELIGHT_STATE_DIR       -- explicit override; also lets fsh-test
///                                   isolate per-case state off the live db
///   2. state_home()/faelight     -- if it already exists, it wins
///   3. faelight_dir()/runtime    -- legacy, only while it still exists
///   4. state_home()/faelight     -- fresh install, no repo required
///
/// There is deliberately no window where the code points somewhere the data is
/// not: today (3) holds and nothing changes; after the move (2) takes over on
/// its own; on a machine with no forest (4) applies and none is created.
pub fn runtime_dir() -> PathBuf {
    if let Ok(v) = env::var("FAELIGHT_STATE_DIR") {
        if !v.is_empty() {
            return PathBuf::from(v);
        }
    }
    let xdg = state_home().join("faelight");
    if xdg.exists() {
        return xdg;
    }
    let legacy = faelight_dir().join("runtime");
    if legacy.exists() {
        return legacy;
    }
    xdg
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

// INT-061 said every runtime/* path routes through runtime_dir(). It was 95%
// true: the journal and snapshot domains built theirs with ctx.fpath("runtime/x")
// instead, and backups had BOTH owners agreeing by coincidence. Measured when
// runtime/ moved to XDG state and journal/ alone stayed behind in the repo.
/// XDG cache home. DISTINCT from cache_dir(), which is runtime_dir()/cache --
/// this is the user's ~/.cache, where derived data that is cheap to recompute
/// and costs nothing to lose belongs.
pub fn xdg_cache_home() -> PathBuf {
    match env::var("XDG_CACHE_HOME") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => home().join(".cache"),
    }
}

/// The health percentage the doctor last wrote. Derived, not authoritative.
pub fn health_status_file() -> PathBuf {
    xdg_cache_home().join("faelight/health-status")
}

/// The health percentage, or None when it has never been written or cannot be
/// parsed.
///
/// ⚠️ THE Option IS THE POINT. Four readers used to resolve this file
/// independently and disagree about what its absence meant: the prompt fell back
/// to 100 and printed "peak", the banner fell back to 95, the status line fell
/// back to the string "100%", and the GTK bar fell back to 100. A machine that
/// had never run the doctor therefore claimed PEAK HEALTH on every prompt render.
/// Returning None forces each caller to say what absent means for ITS OWN
/// display, which is the same rule INT-148 gave the doctor: could-not-determine
/// is not a passing grade.
pub fn read_health() -> Option<u8> {
    let raw = std::fs::read_to_string(health_status_file()).ok()?;
    raw.trim().trim_end_matches('%').parse().ok()
}

pub fn journal_dir() -> PathBuf {
    runtime_dir().join("journal")
}

pub fn snapshots_dir() -> PathBuf {
    runtime_dir().join("snapshots")
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

/// The fsh config file -- aliases and settings -- at its LIVE location.
///
/// WARNING: this used to return core_dir()/nix/home/dotfiles/... and had ZERO
/// callers. Under home-manager that repo path and the XDG path were one file
/// via symlink, so the distinction did not exist. After the Omarchy migration
/// they are two real files that agree only until the next edit, and seven sites
/// read the config across THREE different locations -- the repo copy, the XDG
/// copy, and 0-core/config/... which has never existed on this machine.
///
/// The XDG copy is the one the shell loads (see faelight-shell config.rs) and
/// the one that gets edited, so it is the only honest answer. Renamed from
/// aliases_file because the file is the shell config; aliases are its contents.
pub fn shell_config() -> PathBuf {
    // NSH_CONFIG FIRST, and it belongs here rather than in one consumer. config.rs honoured
    // it; the cheatsheet and the alias reporter did not, so pointing the shell at a different
    // config moved the shell and left two readers on the default -- a split nothing reports.
    if let Ok(p) = env::var("NSH_CONFIG") {
        if !p.trim().is_empty() {
            return PathBuf::from(p);
        }
    }
    match env::var("XDG_CONFIG_HOME") {
        Ok(v) if !v.is_empty() => PathBuf::from(v).join("faelight-shell/config.nsh"),
        _ => home().join(".config/faelight-shell/config.nsh"),
    }
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
        // runtime_dir no longer contains the word "runtime" once the data has
        // moved to XDG state, so assert the CONTRACT instead of the spelling:
        // an explicit override always wins.
        unsafe { std::env::set_var("FAELIGHT_STATE_DIR", "/tmp/fsh-paths-test") }
        assert_eq!(runtime_dir(), PathBuf::from("/tmp/fsh-paths-test"));
        unsafe { std::env::remove_var("FAELIGHT_STATE_DIR") }

        // With no override, it must land somewhere real -- either the legacy repo
        // path while that still exists, or XDG state.
        let r = runtime_dir();
        assert!(
            r.ends_with("runtime") || r.ends_with("faelight"),
            "unexpected runtime_dir: {}",
            r.display()
        );
    }
}

// ═══════════════════════════════════════════════════════════
