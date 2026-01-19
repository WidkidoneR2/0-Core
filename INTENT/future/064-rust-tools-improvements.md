---
title: "Rust Tools Improvements & Stow Structure Fix"
id: 064
date: 2026-01-19
type: future
title: "Rust Tools Improvements & Stow Structure Fix"
status: in-progress
tags: [rust, tools, stow, architecture]
---

## Vision
Comprehensive improvement of all Rust tools with proper stow integration.

## Completed (v7.6.1)

### 1. bump-system-version ✅ (v7.6.4)
- Added is_valid_version() to reject invalid versions
- Added skip_version_history to preserve README version table
- Fixed workspace build path issue

### 2. Stow Structure ✅
- Removed absolute symlinks
- All 11 packages properly stowed with --ignore=\.dotmeta
- System health: 46% → 92%

### 3. dot-doctor ✅
- Dynamic detection: 11/11 packages
- No more hardcoded checks

### 4. intent-guard ✅
- Fixed aggressive mv detection
- Only blocks moves FROM 0-core

### 5. faelight-stow v0.2.0 ✅
- Can now stow new packages

## In Progress

### 6. All 30+ Tools Review
- **Status:** Not started
- **Plan:** Systematic review across future versions
- **Scope:** Performance, UX, functionality improvements for each tool

## Success Criteria
- [x] Fix stow to manage shell-zsh properly
- [x] Fastfetch properly stowed (not manual symlink)
- [x] faelight-stow can add new packages
- [x] dot-doctor dynamically detects stowed packages
- [ ] All tools reviewed and improved
- [x] Documentation updated

---
_"Fix the foundation before building higher."_ 🌲
