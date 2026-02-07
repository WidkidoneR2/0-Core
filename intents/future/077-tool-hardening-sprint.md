---
id: 077
date: 2026-02-03
type: future
title: "Tool Hardening Sprint - Centralized Paths"
status: in-progress
tags: [hardening, paths, infrastructure, technical-debt]
progress: 10%
---

# Intent 077: Tool Hardening Sprint - COMPLETED SESSION 1

## Status: IN PROGRESS (4/40 tools completed)

## Status: IN PROGRESS (4/40 tools completed)

## Goal
Systematically audit and harden all 40 Rust tools to use centralized paths and eliminate hardcoded references that break when structure evolves.

## Session 1 Complete - 2026-02-03 (4 tools hardened)

### ✅ Completed Tools

**1. bump-system-version v7.0.0**
- [x] Created centralized paths:: module (6 functions)
- [x] Fixed all numbered gravity path references
- [x] Improved CHANGELOG format with emoji sections
- [x] Enhanced verify_release_health() with actual doctor parsing
- [x] All 16+ hardcoded paths replaced
- **Result:** Release automation is now bulletproof

**2. dot-doctor v2.0.0**
- [x] Fixed TOML validation (now uses actual parser, not just bracket check)
- [x] Added recursive symlink detection with walkdir (depth 5)
- [x] Created CheckResult builder helpers (.pass(), .warn(), .fail())
- [x] Added toml and walkdir dependencies
- **Result:** Critical correctness bugs fixed

**3. core-diff v2.0.0**
- [x] Migrated to clap (deleted 80 lines of manual parsing!)
- [x] Created type-safe DiffMode enum (no more string comparisons)
- [x] Created Commands enum for subcommands
- [x] Automatic --help and --version generation
- [x] Better error messages via clap validation
- **Result:** Type-safe, maintainable, extensible

**4. faelight-bar v2.0.0**
- [x] Created src/paths.rs module
- [x] Replaced 5 hardcoded path references
- [x] Zero warnings, clean compilation
- **Result:** Always-visible UI tool is resilient

---

## Remaining Tools to Audit (36 tools)

### High Priority (Critical/Visible Tools)
- [ ] faelight-git (Git operations - likely has paths)
- [ ] faelight-stow (Manages stow - definitely has paths)
- [ ] faelight-link (Symlink manager - has paths)
- [ ] faelight-update (Update system - may have paths)
- [ ] faelight-zone (Zone detection - has paths)
- [ ] faelight-dashboard (System overview - may display paths)

### Medium Priority (Supporting Tools)
- [ ] archaeology-0-core (Likely has many paths)
- [ ] core-protect (Lock/unlock - has paths)
- [ ] dotctl (Stow controller - has paths)
- [ ] faelight-bootstrap (Setup - definitely has paths)
- [ ] faelight-hooks (Git hooks - likely has paths)
- [ ] faelight-snapshot (Btrfs snapshots - has paths)
- [ ] get-version (Reads VERSION file - has path)
- [ ] latest-update (Checks updates - may have paths)
- [ ] profile (Profile management - has paths)
- [ ] safe-update (Update wrapper - may have paths)
- [ ] teach (Educational tool - may reference paths)

### Lower Priority (Less Path-Dependent)
- [ ] alias-audit
- [ ] entropy-check
- [ ] faelight (CLI router)
- [ ] faelight-daemon (Service runner)
- [ ] faelight-dmenu (Menu UI)
- [ ] faelight-fetch (Downloader)
- [ ] faelight-fm (File manager)
- [ ] faelight-launcher (App launcher)
- [ ] faelight-lock (Screen locker)
- [ ] faelight-menu (Power menu)
- [ ] faelight-notify (Notifications)
- [ ] faelight-term (Terminal)
- [ ] intent (Intent management)
- [ ] intent-guard (Command safety)
- [ ] keyscan (Keybinding scanner)
- [ ] recent-files (File tracker)
- [ ] workspace-view (Workspace display)

### Library (Special Case)
- [ ] faelight-core (Shared library - should PROVIDE paths, not use hardcoded ones)

---

## Strategy for Remaining Tools

### Phase 1: Create faelight-core/paths.rs (Week 2)
- [ ] Move path constants into faelight-core library
- [ ] Create comprehensive path module:
  - `core_dir()` - ~/0-core
  - `meta_dir()` - ~/0-core/00-meta
  - `registry_dir()` - ~/0-core/01-registry
  - `rules_dir()` - ~/0-core/02-rules
  - `interfaces_dir()` - ~/0-core/03-interfaces
  - `runtime_dir()` - ~/0-core/04-runtime
  - `stow_dir()` - ~/0-core/03-interfaces/stow
  - `version_file()` - ~/0-core/00-meta/VERSION
  - `changelog_file()` - ~/0-core/00-meta/CHANGELOG.md
  - `intents_dir()` - ~/0-core/intents
  - Zone paths (src, projects, archive)
- [ ] Export from faelight-core/src/lib.rs
- [ ] Document usage in README

### Phase 2: Systematic Tool Audit (Weeks 3-4)
For each tool:
1. [ ] Search for hardcoded strings: "0-core", "stow", "VERSION", "INTENT"
2. [ ] Replace with `faelight_core::paths::*`
3. [ ] Add to tool's Cargo.toml: `faelight-core = { path = "../faelight-core" }`
4. [ ] Test tool works after changes
5. [ ] Document in tool's README what paths it uses

### Phase 3: Validation (Week 5)
- [ ] Run all tools to verify functionality
- [ ] Update doctor to validate path usage
- [ ] Document path architecture in ARCHITECTURE.md

---

## Success Criteria
- [ ] Zero hardcoded paths in any tool (0/40 currently)
- [ ] All tools use `faelight_core::paths` (4/40 have local paths modules)
- [ ] New tools can't be created with hardcoded paths
- [ ] bump-system-version never crashes on path changes
- [ ] System can be renamed with one config change

---

## Session 1 Statistics
- **Tools hardened:** 4/40 (10%)
- **Critical bugs fixed:** 2 (TOML validation, symlink detection)
- **Lines changed:** ~500+
- **Time invested:** ~4 hours
- **System health:** 100% maintained
- **Version bumps:** 2 major releases

---

## Notes from Session 1
- Created local paths modules in each tool (temporary)
- Next step: consolidate into faelight-core for ecosystem-wide use
- Pattern established: audit → paths module → replace → test → version
- clap migration was HIGHLY successful (core-diff)
- Builder patterns reduce boilerplate (dot-doctor)

---

## Timeline
- **Now:** Session 1 complete (4 tools)
- **This week:** Rest, celebrate, plan Phase 1
- **Next 2 weeks:** Create faelight-core paths module
- **Weeks 3-4:** Systematic audit of remaining 36 tools
- **Week 5:** Validation and documentation
- **Summer:** Show Linus the resilient ecosystem

---

## Related Intents
- Intent 076: Path Resilience Audit (parent intent)
- Intent 078: Nix Migration (deferred until post-summer)

**The forest evolves methodically.** 🌲
