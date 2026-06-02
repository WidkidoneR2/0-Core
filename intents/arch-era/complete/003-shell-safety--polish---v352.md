---
id: 003
date: 2025-12-31
type: future
title: "Shell Safety & Polish - v3.5.2"
status: complete
tags: [shell, safety, v3.5.2, zsh, starship]
version: v3.5.2
packages: [shell-zsh, prompt-starship]
---

## Why

The shell is our primary interface with 0-Core.
Safety + Polish = Better daily experience.

After building Intent Ledger (v3.5.0) and Git Guardrails (v3.5.1),
v3.5.2 focuses on making the shell safer and more elegant.

## What

Four improvements:

1. **Dangerous Command Highlighting**
   - Highlight `rm -rf`, `chmod 777`, `dd if=` in RED
   - Cognitive safety layer
   - Prevents muscle-memory disasters

2. **Alias Autocompletion**
   - Completions for core-diff, dot-doctor, dotctl, intent
   - Discoverability (188+ aliases!)
   - Professional polish

3. **0-Core Path Highlighting**
   - Make `~/0-core` paths visually distinct
   - Consistency with theme
   - Quick visual identification

4. **Starship Lock Status Integration** (from experiment 001)
   - Show 🔒/🔓 in prompt inline
   - Cleaner than verbose warnings
   - Elegant solution

## Timeline

v3.5.2 (early January 2026)
After v3.5.0 (Intent Ledger) and v3.5.1 (Git Guardrails)

## Why These Specific Improvements

All four align with philosophy:

- Safety without blocking
- Understanding through visibility
- Polish without complexity
- Intentional defaults

## Impact

Daily quality of life improvements.
Safer operations.
More professional feel.
Sets tone for v4.0+ work.

# ### 2026-01-04

Status changed to complete.
v3.5.2 shipped with all four features.

---

_Part of the 0-Core Intent Ledger_ 🌲
