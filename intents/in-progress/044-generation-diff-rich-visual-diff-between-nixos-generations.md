---
id: 044
date: 2026-06-09
type: feature
title: "generation-diff: rich visual diff between NixOS generations"
status: in-progress
tags: [nixos, generations, diff, ratatui, forest, visual]
priority: medium
---
## Why
Every `rebuild` creates a new NixOS generation.
Right now you can list generations but you cannot see what changed.
Was it a package upgrade? A service change? A config tweak?
You have to guess or dig through git logs manually.

generation-diff makes the answer instant and visual.

## Vision
  gen-diff          -- diff current vs previous generation
  gen-diff 105 107  -- diff any two generations
  gen-diff --last 3 -- show last 3 generations as a timeline

Output shows:
  + added packages (neon green)
  - removed packages (neon red)
  ~ changed packages with version delta (neon amber)
  config changes: services added/removed, options changed
  forest metadata: which intents were completed, commit range

## What Already Exists
NixOS stores generations in /nix/var/nix/profiles/system-*
Each generation has a manifest and store paths.
`nixos-rebuild list-generations --json` gives structured data.
The forest already tracks commits and health per session.

## Approach
- Parse generation manifests from /nix/var/nix/profiles/
- Diff package sets between two generations
- Cross-reference with git log for commit range
- Cross-reference with state.db for intent completions in range
- Render as ratatui TUI or plain colored output
- fsh command: gen-diff [A] [B]

## Phases

Phase 1 -- Generation metadata
  Read and parse generation manifests
  Extract package lists, NixOS version, date, commit hash
  Gate: gen-diff lists all generations with dates and commit hashes

Phase 2 -- Package diff
  Diff package sets between two generations
  Color output: + added, - removed, ~ changed with version
  Gate: gen-diff shows package changes between two generations

Phase 3 -- Forest context
  Cross-reference git log for commit range between generations
  Cross-reference state.db for intents completed in range
  Gate: gen-diff shows "3 intents completed, 47 commits" between gens

Phase 4 -- fsh integration
  Register gen-diff as fsh vocabulary command
  Tab completion for generation numbers
  Gate: gen-diff works natively in fsh with tab completion

## Gates
- [x] gen-diff lists all generations with date and commit hash -- 154 gens; commit best-effort timestamp-matched (configurationRevision Unknown; 1c flake stamp pending for exact future attribution) (Phase 1, 2026-06-18)
- [x] gen-diff shows package additions in neon green -- 142 adds on gen 25->178 (Phase 2, 2026-06-18)
- [x] gen-diff shows package removals in neon red -- niri/starship/stow on 25->178 (Phase 2, 2026-06-18)
- [x] gen-diff shows version changes in neon amber -- 8 changes incl brave/linux/mesa on 25->178 (Phase 2, 2026-06-18)
- [ ] gen-diff shows commit range between generations
- [ ] gen-diff shows intents completed in generation range
- [x] gen-diff A B diffs any two specific generations -- `gen-diff 25 178` (Phase 2, 2026-06-18)
- [ ] gen-diff --last N shows N most recent generations
- [ ] fsh tab completion for generation numbers

## Depends On
- INT-034 (Forest release v2) -- generation + commit + intent triad

## The Rule
"Every rebuild is a checkpoint.
 You should be able to see exactly what changed
 and why the forest is different today than yesterday." 🌲
