---
id: 111
date: 2026-03-03
type: incident
title: "Niri keybind conflict — Mod+Alt+b locked system for 2 days"
status: complete
tags: [incident, niri, keybinds, debug]
version: 10.3.0
---

## Incident Summary

During Niri Phase 2 migration, a keybind conflict on `Mod+Alt+b` caused Brave
to fail to launch and the config reload to error silently. The system appeared
broken for 2 days until resolved via TTY.

## Root Cause

Two bindings assigned to `Mod+Alt+b`:
1. `bump-system-version` (existing Faelight tool)
2. `faelight-browser` (newly added during migration)

Niri rejected the duplicate silently — the red flash on screen was the only
indicator. No fallback, no error in terminal.

## Resolution

Accessed TTY (Ctrl+Alt+F2), opened config manually, removed line 129
(`Mod+Alt+b → bump-system-version`). Niri hot-reloaded cleanly.

## What Was Learned

- Niri config errors show as a brief red flash — easy to miss
- Always validate config before reloading: `niri validate --config <path>`
- TTY is the escape hatch — know how to use it
- `bump-system-version` does not need a keybind (deliberate action)

## Prevention

- Run `niri validate` before every config change
- Add validation step to the niri config update workflow
- Consider adding a pre-reload validation alias

## Status

Resolved. INT-099 observation logged.
