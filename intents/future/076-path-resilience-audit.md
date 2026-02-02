# Intent 076: Path Resilience Audit

## Problem
After numbered gravity restructuring, we fixed paths reactively as tools crashed. This creates technical debt and potential failures. We need a systematic audit of all 40 tools to ensure path resilience.

## Goals
1. **Audit all 40 tools** for hardcoded paths
2. **Centralize path configuration** (single source of truth)
3. **Create path constants** that all tools import
4. **Document path assumptions** in each tool
5. **Add path validation** in critical tools

## Hardcoded Paths to Eliminate

### Current Problematic Patterns:
```rust
// ❌ BAD: Hardcoded paths
"/home/christian/0-core/stow"
"0-core/VERSION"
"~/0-core/INTENT"

// ✅ GOOD: Use faelight-core constants
use faelight_core::paths;
paths::STOW_DIR
paths::VERSION_FILE
paths::INTENTS_DIR
```

### Tools Fixed Tonight (Partial):
- [x] dot-doctor (stow, VERSION, themes)
- [x] bump-system-version (VERSION, stow, README, CHANGELOG)
- [x] get-version (stow)
- [x] latest-update (stow)
- [x] faelight-link (stow)
- [x] alias-audit (stow)
- [x] faelight-bar (VERSION)
- [x] faelight-dashboard (VERSION)
- [x] entropy-check (VERSION)
- [x] profile (VERSION)
- [x] faelight (VERSION)
- [x] faelight-fetch (VERSION)

### Tools Needing Full Audit (28 remaining):
- [ ] archaeology-0-core
- [ ] core-diff
- [ ] core-protect
- [ ] dotctl
- [ ] faelight-bootstrap
- [ ] faelight-daemon
- [ ] faelight-dmenu
- [ ] faelight-fm
- [ ] faelight-git
- [ ] faelight-hooks
- [ ] faelight-launcher
- [ ] faelight-lock
- [ ] faelight-menu
- [ ] faelight-notify
- [ ] faelight-snapshot
- [ ] faelight-stow
- [ ] faelight-term
- [ ] faelight-zone
- [ ] intent
- [ ] intent-guard
- [ ] keyscan
- [ ] recent-files
- [ ] safe-update
- [ ] teach
- [ ] workspace-view
- [ ] faelight-core (the library itself!)

## Solution: Path Registry in faelight-core

Create `rust-tools/faelight-core/src/paths.rs`:
```rust
use std::path::PathBuf;
use std::env;

/// Get home directory
pub fn home() -> PathBuf {
    PathBuf::from(env::var("HOME").expect("HOME not set"))
}

/// Core paths (numbered gravity structure)
pub const CORE_NAME: &str = "0-core";

pub fn core_dir() -> PathBuf {
    home().join(CORE_NAME)
}

// 00-meta/
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

// 01-registry/
pub fn registry_dir() -> PathBuf {
    core_dir().join("01-registry")
}

pub fn tools_registry() -> PathBuf {
    registry_dir().join("tools.toml")
}

pub fn aliases_registry() -> PathBuf {
    registry_dir().join("aliases.toml")
}

// 02-rules/
pub fn rules_dir() -> PathBuf {
    core_dir().join("02-rules")
}

pub fn hooks_dir() -> PathBuf {
    rules_dir().join("hooks")
}

pub fn security_dir() -> PathBuf {
    rules_dir().join("security")
}

// 03-interfaces/
pub fn interfaces_dir() -> PathBuf {
    core_dir().join("03-interfaces")
}

pub fn stow_dir() -> PathBuf {
    interfaces_dir().join("stow")
}

pub fn profiles_dir() -> PathBuf {
    interfaces_dir().join("profiles")
}

// 04-runtime/
pub fn runtime_dir() -> PathBuf {
    core_dir().join("04-runtime")
}

pub fn logs_dir() -> PathBuf {
    runtime_dir().join("logs")
}

pub fn backups_dir() -> PathBuf {
    runtime_dir().join("backups")
}

// Root directories
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

// Zone paths
pub fn src_dir() -> PathBuf {
    home().join("1-src")
}

pub fn projects_dir() -> PathBuf {
    home().join("2-projects")
}

pub fn archive_dir() -> PathBuf {
    home().join("3-archive")
}
```

## Implementation Plan

### Phase 1: Create Path Registry (Week 1)
- [ ] Create `faelight-core/src/paths.rs`
- [ ] Export from `faelight-core/src/lib.rs`
- [ ] Add path validation functions
- [ ] Document usage in README

### Phase 2: Critical Tools (Week 2)
Update tools that interact with structure:
- [ ] bump-system-version (uses 5+ paths)
- [ ] dot-doctor (uses 4+ paths)
- [ ] faelight-bootstrap (uses many paths)
- [ ] faelight-stow (manages stow dir)
- [ ] faelight-link (symlink manager)

### Phase 3: Systematic Tool Audit (Weeks 3-4)
Go through each of 40 tools:
- [ ] Search for hardcoded strings: "0-core", "stow", "VERSION", "INTENT"
- [ ] Replace with `faelight_core::paths::*`
- [ ] Add to tool's Cargo.toml: `faelight-core = { path = "../faelight-core" }`
- [ ] Test tool works after changes
- [ ] Document in tool's README what paths it uses

### Phase 4: Validation (Week 5)
- [ ] Create test suite that validates all paths exist
- [ ] Add to CI/CD (if we add it)
- [ ] Add to doctor checks
- [ ] Document in ARCHITECTURE.md

## Success Criteria
- [ ] Zero hardcoded paths in any tool
- [ ] All tools use `faelight_core::paths`
- [ ] New tools can't be created with hardcoded paths (lint rule?)
- [ ] bump-system-version never crashes on path changes
- [ ] System can be renamed (e.g., "0-core" → "faelight-core") with one config change

## Timeline
- **Now**: Create intent, don't start yet
- **This week**: Rest, celebrate v8.8.0
- **Next month**: Phase 1 + 2 (critical tools)
- **Month 2-3**: Phase 3 (systematic audit)
- **Month 4**: Phase 4 (validation)
- **Month 5**: Buffer for Linus prep

## Notes
- This is CRITICAL for summer Linus demo
- Shows maturity: "I took your feedback, restructured, AND made it resilient"
- Demonstrates intentional computing: not just fixing, but preventing
- Sets foundation for future evolution

## The Bigger Picture: Tool Ecosystem Evolution

### Current State: 40 Independent Tools
Each tool reimplements:
- ❌ Path resolution
- ❌ Config reading
- ❌ Error handling
- ❌ UI patterns (colors, formatting)
- ❌ Git operations
- ❌ Health checks
- ❌ Version reading

**This creates:**
- Code duplication (same logic in 10 tools)
- Inconsistent behavior (different error messages)
- Maintenance burden (fix bug in 10 places)
- Fragility (one tool works, another breaks)

### Future State: Integrated Ecosystem
Tools share common infrastructure:
```
faelight-core/
├── paths.rs          # Path constants (this intent)
├── config.rs         # Config reading/writing
├── errors.rs         # Unified error types
├── ui.rs             # Color schemes, formatting
├── git.rs            # Git operations
├── health.rs         # Health check framework
├── version.rs        # Version management
└── zones.rs          # Zone system
```

**Benefits:**
- ✅ Fix once, all tools benefit
- ✅ Consistent UX across all tools
- ✅ Tools can call each other (composability)
- ✅ New tools are easy to build
- ✅ System evolves as one unit

### Tool Dependency Layers

**Layer 1: Core Infrastructure** (faelight-core)
- Provides: paths, config, errors, UI
- Used by: everyone

**Layer 2: Domain Services**
- faelight-git: Git operations
- faelight-link: Symlink management
- dot-doctor: Health framework
- All use Layer 1

**Layer 3: User-Facing Tools**
- bump-system-version: uses core + git
- faelight-update: uses core + git + doctor
- faelight-bar: uses core + health
- All use Layer 1 + Layer 2

**Layer 4: Meta Tools**
- faelight: CLI router (calls other tools)
- teach: Educational wrapper
- Use all layers

### Natural Interdependency Examples

**Example 1: bump-system-version calls doctor**
```rust
// Instead of: running `doctor` as subprocess
// Do: use dot_doctor::run_health_check()

use faelight_core::{paths, version};
use dot_doctor::health;

let health = health::check_all()?;
if health.percent < 100 {
    return Err("Cannot bump with system warnings");
}
```

**Example 2: faelight-update uses faelight-git**
```rust
// Instead of: running `fg status` as subprocess
// Do: use faelight_git::status()

use faelight_git::repository;

let repo = repository::open(paths::core_dir())?;
if repo.has_unpushed_commits()? {
    println!("⚠️  Git: Commits need push");
}
```

**Example 3: All tools use shared config**
```rust
// Instead of: each tool reading ~/.config/faelight/config.toml
// Do: use faelight_core::config

use faelight_core::config::Config;

let config = Config::load()?;
let theme = config.theme(); // Consistent colors across all tools
```

### Implementation Strategy

**Phase 1: Foundation (Months 1-2)**
- Create robust faelight-core with paths, config, errors, UI
- Migrate 5 critical tools to use it
- Prove the pattern works

**Phase 2: Domain Services (Month 3)**
- Refactor faelight-git, faelight-link, dot-doctor into libraries
- Make them usable by other tools (not just binaries)
- Create clean APIs

**Phase 3: Integration (Month 4)**
- Migrate all 40 tools to use shared infrastructure
- Tools call each other directly (not via subprocess)
- Remove code duplication

**Phase 4: Polish (Month 5)**
- Documentation
- Testing
- Linus demo prep

### Success Metrics
- [ ] Lines of duplicated code: 0
- [ ] Tools sharing faelight-core: 40/40
- [ ] Tools with hardcoded paths: 0/40
- [ ] Inter-tool dependencies: 20+ (natural composition)
- [ ] Time to build new tool: <1 hour (because infrastructure exists)

### The Linus Pitch

**"I didn't just build 40 tools. I built an ecosystem."**

Show him:
1. **Numbered gravity** - Structure teaches
2. **Shared infrastructure** - Tools compose naturally
3. **Intentional evolution** - Not just adding features, architecting systems
4. **Rust benefits** - Zero-cost abstractions make this possible

This demonstrates **systems thinking** - the difference between a collection of scripts and a coherent platform.
