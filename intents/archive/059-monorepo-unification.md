---
id: 059
date: 2026-01-15
type: future
title: "Monorepo Unification - Rust Workspace"
status: complete
tags: [v7.5, architecture, rust, structure]
version: 7.5.0
blocks:
  - 047  # Need faelight-core first
relates:
  - 048  # Tool migrations
  - 061  # Code quality
---

## Vision
Transform 0-Core into a proper Rust workspace monorepo with shared dependencies and unified builds.

**Philosophy:** "Many tools, one forest."

## The Problem

**Current Structure:**
```
~/0-core/rust-tools/
├── faelight-launcher/  (14 separate Cargo.toml)
├── faelight-bar/       (14 duplicate dependencies)
├── faelight-menu/      (14 separate builds)
└── ...                 (No code sharing)
```

**Pain Points:**
- Duplicate dependencies (wayland-client × 14)
- No shared code (each reimplements config loading)
- Slow builds (no workspace caching)
- Version drift (bar v0.5, menu v0.6, launcher v2.0)
- Large disk usage (target/ × 14)

## The Solution

**New Structure:**
```
~/0-core/
├── Cargo.toml          # Workspace root
├── Cargo.lock          # Unified lockfile
├── crates/
│   ├── faelight-core/      # Shared library
│   │   ├── config/         # Config loading
│   │   ├── health/         # Health checks
│   │   ├── ipc/            # Sway IPC
│   │   └── wayland/        # Wayland helpers
│   │
│   ├── faelight-bar/       # Bar (uses core)
│   ├── faelight-launcher/  # Launcher (uses core)
│   ├── faelight-menu/      # Menu (uses core)
│   ├── faelight-notify/    # Notify (uses core)
│   └── faelight-lock/      # Lock (uses core)
│
├── tools/
│   ├── intent/             # Intent system
│   ├── archaeology/        # System archaeology
│   └── workspace-view/     # Workspace intelligence
│
├── target/             # Single build directory
└── Justfile            # Build orchestration
```

**Root Cargo.toml:**
```toml
[workspace]
members = [
    "crates/*",
    "tools/*",
]
resolver = "2"

[workspace.dependencies]
wayland-client = "0.31"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }
# ... shared deps

[workspace.package]
version = "7.5.0"
edition = "2021"
authors = ["Christian <christian@faelight.forest>"]
```

## Migration Strategy

### Phase 1: Create Workspace (v7.5.0)
```bash
# 1. Create root Cargo.toml
# 2. Move rust-tools/* → crates/
# 3. Update all Cargo.toml to use workspace deps
# 4. Test: cargo build --workspace
```

### Phase 2: Extract Shared Code (v7.6.0)
```bash
# Move common code to faelight-core:
- Config loading → core::config
- Wayland setup → core::wayland
- Sway IPC → core::ipc
- Health checks → core::health
```

### Phase 3: Unified Versioning (v7.7.0)
```bash
# All tools version together:
bump-system-version 7.7.0
# Updates ALL crate versions at once
```

## Benefits

**Build Performance:**
- ✅ Shared target/ (90% faster rebuilds)
- ✅ Workspace-level caching
- ✅ Parallel builds

**Code Quality:**
- ✅ Shared linting config
- ✅ Shared formatting
- ✅ Shared test infrastructure

**Developer Experience:**
- ✅ `cargo build` builds everything
- ✅ `cargo test` tests everything
- ✅ Single dependency update

**Disk Usage:**
- ✅ One target/ instead of 14
- ✅ Deduplicated dependencies
- ✅ ~2GB → ~300MB

## Success Criteria
- [ ] Workspace Cargo.toml created
- [ ] All tools moved to crates/
- [ ] All tools use workspace dependencies
- [ ] faelight-core library functional
- [ ] Shared code extracted
- [ ] `cargo build --workspace` works
- [ ] `cargo test --workspace` passes
- [ ] Justfile for common tasks
- [ ] Documentation updated
- [ ] Zero functionality loss

## Breaking Changes
- Directory structure changes
- Build commands change
- Import paths change (use faelight_core::*)

## Timeline
**v7.5.0** - Create workspace, migrate tools
**v7.6.0** - Extract shared code
**v7.7.0** - Unified versioning

## Dependencies
- Blocks: Intent 047 (need faelight-core)
- Relates: Intent 061 (code quality)

## Completion Notes
- Rust workspace unified across all core tools
- Shared dependencies and build output
- Disk usage and compile time reduced significantly
- Foundation for future cross-tool intelligence



---

_"A forest united grows stronger."_ 🌲🦀
