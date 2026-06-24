---
id: 075
date: 2026-06-22
type: future
title: "Nix store explorer: GC roots, reverse-deps, and what keeps paths alive"
status: in-progress
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

## Phase 0 Findings (2026-06-23) -- queries pinned, gap confirmed
Gap CONFIRMED: nix-tree installed (closure browsing -- do NOT duplicate); nix-du NOT
installed (size/reclaim angle is open). 075 niche = roots + reverse-deps + reclaim/size.
PINNED QUERIES (all verified live, read-only):
- GC roots:        nix-store --gc --print-roots   (548 roots, "<root> -> <storepath>")
- reverse-deps:    nix-store --query --referrers <path>   (direct referrers)
- why-alive chain: nix why-depends <root> <path>   (Phase 1 wiring)
- size (closure):  nix path-info -Sh <path>   (e.g. current-system: 15.5 GiB closure)
- size (self):     nix path-info -sh <path>   (current-system self: 195.6 KiB)
- reclaimable set: nix-store --gc --print-dead   (2359 dead paths)
- reclaim preview: nix-collect-garbage --dry-run  ("2359 store paths would be deleted")
KEY INSIGHT (the value prop, demonstrated): current-system is 195.6 KiB SELF but pins
15.5 GiB CLOSURE -- the "tiny root holds huge closure alive" story is exactly 075's gap.
Store: 54G; 2359 reclaimable dead paths.
DESIGN NOTES surfaced by Phase 0:
- PERF: --print-dead + --gc --print-roots took ~86s (walks whole store). The reclaim view
  must compute on-demand with a spinner / cache the result -- NOT recompute per keystroke.
- DATA DISCREPANCY (075 should reconcile this): the naive ls system-*-link glob counted
  2771, but `d` reports 126 generations. The counts disagree -- a good store explorer
  resolves "how many generations, really, and what's the difference." Logged as a real
  use-case 075 should answer, not just a probe artifact.

## Approach
TUI (Rust/ratatui) over nix path-info / nix-store --query --roots / nix why-depends. The
useful, hard part is presenting roots + reverse-deps legibly -- "why is this 51 GiB here,
and what pins it."

## Phase 1 Evidence (2026-06-23) -- `store why` verb DEMONSTRATED
Added `store why <path|name>` to fsh (mod.rs store_cmd + store_resolve + nix_query +
nix_query_lines + size_tail). Read-only: nix path-info (-sh/-Sh) for self/closure size,
nix-store --query --roots (pinning) + --referrers (reverse-deps). NO --print-dead (the
86s store walk) -- stays fast (~280ms). Name resolution: full path, hash prefix, or
unique name; ambiguous -> lists candidates (e.g. faelight-forest -> 138 matches).
Built via the fast loop (081 reload, no terminal reopen). One live-caught bug fixed:
size parse took .last() = unit only ("KiB"); fixed with size_tail() (last two tokens).
PROVEN: `store why 506yp26` -> self 195.6 KiB, closure 15.5 GiB, pinned by
system-215-link, no referrers -- matching Gate 0 numbers. Correctly reflects live
pinning changes (dropped result/current-system roots after redeploy to gen 216).
Gate met: store browse with sizes + what-keeps-this-alive.
FOLLOW-UP (Phase 1.5, logged not done): when a name is ambiguous (138 matches),
summarize total size + how many are dead/reclaimable -- turn the ambiguity into the
reclaim insight. Pairs with the Phase 2 reclaimable-vs-pinned view.

## Phases
Phase 0 -- confirm the gap vs nix-tree; pin the queries (roots, reverse-deps, sizes).
Phase 1 -- store browse + sizes + "what keeps this alive" (roots / reverse-deps).
Phase 2 -- reclaimable-vs-pinned view + forest integration.

## Gates
- [x] Phase 0: gap vs nix-tree confirmed; root/reverse-dep/size queries pinned and recorded
- [x] store browse with sizes + "what keeps this alive" (GC roots / reverse-deps)
- [ ] reclaimable-vs-pinned view + forest integration

## Notes
- nix-tree already covers closure exploration -- scope to the GC-root / reverse-dep gap.
- Pairs with INT-073: shows what a prune would actually free.

## The Rule
"Know what the forest holds, and why it holds it." 🌲
