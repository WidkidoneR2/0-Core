---
id: 017
date: 2026-06-03
type: improvement
title: "faelight-git NixOS audit: review paths, assumptions, improvements"
status: complete
tags: [faelight-git, nixos, audit, git, improvement]
priority: medium
---

## Why

faelight-git was built on Arch. On NixOS several assumptions may have
changed. The fg alias works but the tool needs a full review.

## What to Check

- Hardcoded paths to scripts/
- Deploy pipeline assumptions
- Intent commit linking still works
- Event bus integration on NixOS
- Any pacman/Arch-specific code

## Gate

faelight-git works cleanly on NixOS. No stale Arch assumptions.
fg alias works for all git operations.

## Audit Findings (2026-06-04)

### NixOS compatibility: CLEAN
- Only 2 HOME fallbacks in commit.rs -- safe, reads $HOME env var first
- No scripts/ dependencies anywhere in the codebase
- No core-protect references in main workflow
- No pacman/paru/arch assumptions

### Commands verified working
- fg commit -- works, intent detection correct
- fg done -- scans intents/in-progress/ correctly, sorts by filename
- fg alias points at faelight-git binary in NixOS PATH

### Old bug resolved
- INT-328/INT-349 mislabeling bug: resolved by intent renumbering
- INT-024 (was 328) now in in-progress, sorts correctly by filename

### No changes needed
faelight-git is NixOS-ready as-is. No source modifications required.

### Gate Check
✅ No Arch-specific paths
✅ No scripts/ dependencies  
✅ fg alias correct
✅ Intent detection working
✅ fg done logic correct
