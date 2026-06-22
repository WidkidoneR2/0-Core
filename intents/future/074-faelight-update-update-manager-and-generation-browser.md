---
id: 074
date: 2026-06-22
type: future
title: "Faelight-Update v-next: update manager + generation browser"
status: planned
tags: [faelight-update, nix, flake-update, generations, generation-browser, tui, post-1.0.0]
---

## Why
Faelight-Update is the forest's update tool. Post-1.0.0 it grows to oversee the whole
update-and-generation story -- two capabilities folded into ONE existing tool rather than
two new silos: an update manager (deliberate, legible flake updates) and a generation
browser (the generation timeline, made legible). Both serve manual-control and
understanding-over-convenience, and compose with INT-073 (gen control), INT-034 (triad),
and INT-056 (recovery).

## What
Update manager:
- Per-input view of what `nix flake update` would change; update one input at a time.
- Closure diff shown BEFORE the switch (nvd / nix store diff-closures).
- Rebuild gated on review.
Generation browser:
- Browse generations: timeline, dates, sizes.
- Closure diff between any two generations.
- Each generation tied to its commit + intent (via INT-034 triad data).
- Roll back / boot a chosen generation (serves INT-056 recovery).

## Approach
Extend the existing Faelight-Update tool (Rust/ratatui). The update side wraps
flake-update + nvd/diff-closures with per-input granularity and a review gate. The
generation side reads the generation list + closure diffs + INT-034's commit/intent
mapping. Wrap nvd; do not reinvent the diff engine.

## Phases
Phase 0 -- survey current Faelight-Update; map where update-manager + gen-browser slot in.
Phase 1 -- update manager: per-input flake update + pre-switch closure diff + review gate.
Phase 2 -- generation browser: timeline + closure diff between gens + roll-back.
Phase 3 -- integration: tie generations to commit + intent (depends on INT-034 data).

## Gates
- [ ] Phase 0: current Faelight-Update surveyed; update-manager + gen-browser integration points recorded
- [ ] update manager: per-input flake update with pre-switch closure diff and a review gate
- [ ] generation browser: timeline + closure diff between generations + roll-back
- [ ] generations tied to commit + intent (via INT-034 triad data)

## Notes
- Consolidates two post-1.0.0 ideas into the existing tool, not two new silos.
- Composes with INT-073 (gen control), INT-034 (triad), INT-056 (recovery).
- Build on nvd / nix store diff-closures; do not reinvent the diff engine.

## The Rule
"See what changes before it changes you." 🌲
