---
id: 015
date: 2026-06-04
type: feature
title: "faelight-fm v3.1: Nix-aware, git-first, semantic navigation"
status: complete
tags: [faelight-fm, ratatui, nix, git, semantic, broot]
priority: high
---

## Vision

faelight-fm v3.1 builds on the ratatui v3 foundation.
The forest file manager understands Nix, Git, and meaning -- not just paths.

Two phases:

### Phase 1 -- v3.1 (builds on current ratatui v3)
- broot-style tree navigation with branches
- Nix-aware file info: which package owns this file
- Detect /nix/store symlinks, show their target
- git integration: staging, diff preview, branch indicator
- Semantic search: "show Rust files modified this week"
- "Why does this file exist?" via nix-store --query

### Phase 2 -- v4.0 (future, own intent)
- Core Library with indexing + search engine
- Plugin system: Git plugin, Nix plugin, Media plugin
- Browse flakes as virtual directories
- View derivation metadata
- Detect garbage-collection roots
- Full Nix explorer experience

## Nix-Aware Features (v3.1)

Using nix-store queries wrapped in Rust:
- nix-store --query --referrers: who depends on this
- nix-store --query --deriver: what built this
- readlink on /nix/store symlinks
- Show NixOS generation each file belongs to

## Git-First Features (v3.1)

Building on existing git status badges:
- Diff preview in preview pane (right panel)
- Stage/unstage with s key
- Branch indicator in header
- History timeline per file with t key
- Visual staging area

## Semantic Search

Without external tools:
- "show Rust files modified this week"
- "show files larger than 10MB"
- Filter by extension, date, size, git status
- Forest-aware: files related to active intent

## Architecture (v3.1 additions to current structure)

Current: main.rs (single file, ~320 lines)
Target modular structure:
faelight-fm/src/
  main.rs        # entry point only
  app.rs         # AppState + main loop
  fs/            # filesystem operations
  nix/           # nix store queries
  git/           # read-only git context
  search/        # semantic search
  ui/            # ratatui layout (no logic)
  input/         # keybindings + modes

## Gate Check
✅ Tree navigation with expand/collapse (broot-style inline tree)
✅ Nix store path info shown via n key
✅ Git status badges + diff preview for modified files
✅ Stage/unstage with s key
✅ Filter with / prefix (fuzzy matching)
✅ Builds cleanly -- zero errors, zero warnings
✅ Health 100% after rebuild
✅ Single mode: faelight-fm / fm alias
✅ Dual panel mode: faelight-fm --dual / fmd alias
✅ No terminal hang on exit -- panic hook + cursor restore
✅ Forest colors throughout