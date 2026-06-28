---
id: 076
date: 2026-06-22
type: future
title: "Nix package search TUI: search to declarative config-add"
status: in-progress
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
- [x] Phase 0: search backend chosen (nix search --json); add-target pinned (home.packages)
- [x] search + package detail working in the TUI (candy-neon, driven live)
- [x] selected package added to declarative config (reviewable + backed up); PROVEN: hello added -> rebuilt -> runs

## Notes
- Differentiator is search -> declarative-add, not search (search.nixos.org / nix-index cover search).
- Never imperative (no nix-env -i); the add is always declarative and reviewable.

## The Rule
"Find it, then let the config own it." 🌲


## Progress (2026-06-28): All 3 phases complete -- proven in production

- Phase 0: scaffold + decisions. faelight-nix crate (ratatui), backend = `nix
  search nixpkgs --json` (no new deps), add-target = users/christian/home.nix
  home.packages. Commit f23e56ee.
- Phase 1: search data layer (search.rs parses nix-search JSON -> Vec<Package>)
  + candy-neon interactive TUI (theme.rs lifted from faelight-fm; search box,
  results list, live detail pane, j/k nav). Driven live. Commits 49bf7484, 5920a161.
- Phase 2: config_edit.rs plan_add engine (insert-first, duplicate-guard,
  non-attr rejection) + TUI 'a' -> Confirm mode (diff in detail pane) -> y writes
  (timestamped .bak then declarative write) / n cancels. Never imperative, never
  silent, always backed up. Commit 70821a62.

PROVEN END-TO-END: searched 'hello' -> selected -> reviewed diff -> y -> written to
real home.nix (with backup) -> `dep` rebuild -> `hello` runs. The whole
search->declarative-add->rebuild loop works on the real machine.

All three gates MET. Tool is functionally complete and in production use.

### Future polish (not blocking)
- Remove the --test-add scratch mode from main.rs (superseded by the real 'a' flow).
- Background-thread the search so the UI doesn't freeze ~3s during nix eval.
- Optional: system-vs-user target toggle (currently home.packages only).
