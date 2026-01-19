---
id: 064
date: 2026-01-19
type: future
title: "Rust Tools Improvements & Stow Structure Fix"
status: planned
tags: [rust, tools, stow, architecture]
---

## Vision
Comprehensive improvement of all Rust tools with proper stow integration.

## Problems to Fix

### 1. Stow Structure Issues
- `.zshrc` is absolute symlink (not stow-managed)
- Can't add new directories to shell-zsh via stow
- Fastfetch manually symlinked as workaround

### 2. Tool Improvements Needed
- VERSION file corruption: Commands writing to VERSION when they should not
- bump-system-version: Replaces old version in version history table (should skip that section)
- faelight-stow: Only verifies, doesn't actually stow new packages
- dot-doctor: Hardcoded package list (7/7 instead of dynamic detection)
- All 30+ tools: Need review for improvements
- intent-guard: Too aggressive on INTENT/ moves (normal operations should not require confirmation)

## Success Criteria
- [ ] Fix stow to manage shell-zsh properly
- [ ] Fastfetch properly stowed (not manual symlink)
- [ ] faelight-stow can add new packages
- [ ] dot-doctor dynamically detects stowed packages
- [ ] All tools reviewed and improved
- [ ] Documentation updated

## Deferred From
v7.6.0 - Noted during release process

---

_"Fix the foundation before building higher."_ 🌲
