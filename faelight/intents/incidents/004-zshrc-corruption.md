---
id: 004
date: 2026-01-19
type: incident
status: resolved
title: "Shell Configuration Corruption - .zshrc Recovery"
severity: CRITICAL
affected: shell-zsh/.zshrc
---

# Incident 004: Shell Configuration Corruption

## Summary
The primary shell configuration file (shell-zsh/.zshrc) was corrupted during a file path migration in commit 1144a6c, reducing it from 781 lines to 2 lines containing only command-not-found handler output.

## Timeline
- **2026-01-15**: File migrated from shell-zsh/.config/zsh/.zshrc to shell-zsh/.zshrc (commit 1144a6c)
- **2026-01-19**: Corruption discovered during v7.6.1 release process
- **2026-01-19**: File recovered from commit 19fd3bd (v7.3.0)

## Impact
- All 221 user aliases temporarily inaccessible
- Shell configuration reduced to 2 lines
- System unusable until recovery

## Root Cause
File corruption occurred during path migration. The command-not-found handler output was written to the file instead of the actual configuration content.

## Recovery
File recovered from git history at old path using: git show 19fd3bd:shell-zsh/.config/zsh/.zshrc

## Prevention Measures
- [ ] Create pre-commit hook to check .zshrc has >100 lines
- [ ] Add .zshrc validation to dot-doctor
- [ ] Document file path history in README

## Status: RESOLVED
All 221 aliases recovered. System functional.
