---
id: 004
date: 2026-06-03
type: feature
title: "faelight-fm v3: broot-inspired, ratatui, forest-native navigation"
status: planned
tags: [faelight-fm, ratatui, broot, file-manager, forest-native]
priority: high
---

## Why

faelight-fm v2 used libcosmic which requires rustc 1.93+ and has heavy
dependencies. A broot-inspired ratatui approach builds cleanly in the
workspace, has no external GUI dependencies, and aligns with the forest
philosophy of understanding over convenience.

## Philosophy

Navigate by meaning, not by path. The forest file manager should understand
intent relationships, show context, and integrate with Friday.

## Approach

- ratatui TUI, standalone binary
- broot-style fuzzy navigation
- Friday-aware: show related intents for selected files
- Forest color theme
- Builds cleanly in workspace without libcosmic

## Gate

faelight-fm launches, navigates filesystem, integrates with yazi handoff.

## Why Not libcosmic (2026-06-03 finding)

faelight-fm v2 uses libcosmic as a git dependency. Nix builds are hermetic
and offline -- git dependencies cannot be fetched at build time. This is not
a rustc version problem (26.05 ships 1.95 which satisfies all requirements).
The architecture itself is incompatible with Nix's build model.

The broot-style rework removes this dependency entirely. ratatui is available
in nixpkgs and builds cleanly in the workspace.
