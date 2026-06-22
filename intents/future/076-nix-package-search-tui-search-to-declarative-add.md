---
id: 076
date: 2026-06-22
type: future
title: "Nix package search TUI: search to declarative config-add"
status: planned
tags: [nix, nixpkgs, search, declarative, config, tui, post-1.0.0]
---

## Why
The real idea is not search (search.nixos.org and nix-index already cover that) but the
last mile: search nixpkgs, then add the chosen package to the config DECLARATIVELY -- to
the right module, the Nix way, never nix-env -i. Search-to-declarative-add is the
differentiator and the main point of the tool.

## What
- Fast nixpkgs search (built on a fast index -- nix-index/nix-locate -- not slow nix search).
- Package detail (version, description, homepage).
- The core: add the selected package to the declarative config at the right place, as a
  reviewable change, ready to rebuild.
Honest scope: search alone is low marginal value over existing tools; the declarative-add
is the reason this exists.

## Approach
TUI (Rust/ratatui). Search layer wraps a fast index rather than slow nix search. The add
layer edits the declarative config (systemPackages or a focused packages module) with a
reviewable diff -- always declarative, never imperative.

## Phases
Phase 0 -- choose search backend (nix-index vs nix search); pin where declarative adds land.
Phase 1 -- search + package detail in the TUI.
Phase 2 -- the core: declarative add to config (reviewable), ready to rebuild.

## Gates
- [ ] Phase 0: search backend chosen; declarative-add target location in config pinned
- [ ] search + package detail working in the TUI
- [ ] selected package added to the declarative config (reviewable), ready to rebuild

## Notes
- Differentiator is search -> declarative-add, not search (search.nixos.org / nix-index cover search).
- Never imperative (no nix-env -i); the add is always declarative and reviewable.

## The Rule
"Find it, then let the config own it." 🌲
