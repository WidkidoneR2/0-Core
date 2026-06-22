---
id: 075
date: 2026-06-22
type: future
title: "Nix store explorer: GC roots, reverse-deps, and what keeps paths alive"
status: planned
tags: [nix, store, gc-roots, reverse-deps, reclaim, tui, post-1.0.0]
---

## Why
/nix/store is the substrate of the whole system but opaque behind arcane commands. The
valuable, forest-native angle is NOT closure browsing (nix-tree already does that well)
but the reclaim story: GC roots, reverse-deps, "what is keeping this path alive," what is
safely reclaimable -- made legible and integrated with the forest.

## What
- Browse store paths with sizes.
- "What is keeping this alive": GC roots + reverse-deps (nix why-depends, --query --roots).
- Reclaimable vs pinned.
- Forest integration (tie to generations / tools where it helps).
Explicit non-goal: do not reinvent nix-tree's closure tree; scope to the roots/reclaim gap.

## Approach
TUI (Rust/ratatui) over nix path-info / nix-store --query --roots / nix why-depends. The
useful, hard part is presenting roots + reverse-deps legibly -- "why is this 51 GiB here,
and what pins it."

## Phases
Phase 0 -- confirm the gap vs nix-tree; pin the queries (roots, reverse-deps, sizes).
Phase 1 -- store browse + sizes + "what keeps this alive" (roots / reverse-deps).
Phase 2 -- reclaimable-vs-pinned view + forest integration.

## Gates
- [ ] Phase 0: gap vs nix-tree confirmed; root/reverse-dep/size queries pinned and recorded
- [ ] store browse with sizes + "what keeps this alive" (GC roots / reverse-deps)
- [ ] reclaimable-vs-pinned view + forest integration

## Notes
- nix-tree already covers closure exploration -- scope to the GC-root / reverse-dep gap.
- Pairs with INT-073: shows what a prune would actually free.

## The Rule
"Know what the forest holds, and why it holds it." 🌲
