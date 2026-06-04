---
id: 032
date: 2026-06-04
type: feature
title: "faelight-fm v4: full Nix explorer, plugin system, semantic engine"
status: planned
tags: [faelight-fm, nix, explorer, plugins, semantic, v4]
priority: medium
---

## Vision

The full file manager vision. Requires v3.1 (INT-015) first.

Core Library:
- filesystem abstraction
- indexing engine
- semantic search engine
- plugin system
- Nix integration layer

UI Layer (ratatui, not COSMIC):
- panels, previews, tabs
- shortcuts, modes
- forest visual language

Extensions/Plugins:
- Git plugin: full git GUI inside fm
- Nix plugin: flake browser, derivation viewer
- Media plugin: image/video preview
- Intent plugin: files linked to intents

## Nix Explorer Features

- Browse flakes as virtual directories
- View derivation metadata (.drv files)
- Detect garbage-collection roots
- Show which generation owns a file
- "Why does this file exist?" full dep trace
- Visualize /nix/store symlink chains
- Right-panel: package definition preview

## Gate

- [ ] Plugin system loads/unloads cleanly
- [ ] Nix plugin shows derivation for any file
- [ ] Flake browser shows flake inputs as virtual dirs
- [ ] GC root detection works
- [ ] INT-015 (v3.1) complete first
