---
id: 072
date: 2026-06-20
type: future
title: "Decommission faelight-palette (unused since Niri 11.0.0)"
status: in-progress
tags: [decommission, faelight-palette, cleanup, tools, audit, nixos]
---

## Why
faelight-palette has been unused since Faelight Forest Niri 11.0.0 (confirmed by
research) and is one of the 8 tools the welcome screen flags with audit score < 70.
Retiring it removes dead weight and clears one low-score tool -- fewer things to
understand, in line with "every tool is understood; nothing is installed blindly."

## What
- Confirm nothing live still depends on faelight-palette.
- Remove it: workspace crate, flake/package entry, aliases, registry/metadata refs.
- Rebuild clean with it gone; tool count drops by one.

## Approach
Phase 0 scans the repo (rg faelight-palette) for live consumers -- aliases, configs,
other tools, docs -- and lists them (expected: none). Phase 1 removes the crate from the
workspace and flake outputs, deletes the source, strips aliases/registry entries. Phase 2
rebuilds and verifies it is gone from PATH and the inventory, with d clean. Mirrors the
decommission pattern from INT-058 (yazi).

## Phases
Phase 0 -- dependency scan; record consumers.
Phase 1 -- removal from workspace, flake outputs, aliases, registry/metadata.
Phase 2 -- rebuild + verify (off PATH, tool count down one, d clean).

## Phase 0 Findings (2026-06-23) -- NOT a simple decommission
The scan found faelight-palette is NOT orphaned -- it is the LIVE launcher, wired into
the engine dispatcher. Scope is bigger than the charter assumed. Recorded honestly;
removal DEFERRED to a clear-headed session (do not cut dispatch-wired code tired).

LIVE consumers (must be handled BEFORE removal):
- engine/src/app/dispatcher.rs:360-372 -- dispatches palette / dmenu / Launch into
  domains::launcher (LIVE routing, compiled into core).
- engine/src/domains/launcher/mod.rs -- all 3 fns (palette/dmenu/launcher) exec
  scripts/faelight-palette. The whole domain points at palette.
- rust-tools/faelight/src/main.rs:307 -- faelight umbrella find_tool("faelight-palette").
- engine/src/domains/doctor/checks.rs:164 + aliases.rs:21 -- doctor expects it.

The launcher TANGLE (the real finding -- three sources disagree):
- registry/aliases.toml:49 -- `launch` alias -> `faelight-launcher`.
- faelight-launcher is a PHANTOM: zero code (no rust-tools/faelight-launcher crate),
  only the alias + historical docs. CHANGELOG:1219 says it was "fully removed,
  superseded by palette." So the `launch` alias currently points at a MISSING binary
  (broken), while the ENGINE dispatches launching to palette. palette is the de-facto
  live launcher; faelight-launcher is dead; the alias and docs contradict each other.
- registry/tools.toml:162 also MISLABELS palette as "Color palette and theme management"
  (it is actually the command-palette/app-launcher).

REVISED scope for removal (next session, daylight):
1. Decide the launcher future: keep palette (then DO NOT decommission -- it is in use),
   OR replace it (with what?), OR remove the launcher dispatch commands entirely.
2. If removing: strip dispatcher.rs launcher arms + the launcher domain + faelight umbrella
   ref + doctor checks/aliases, fix the broken `launch` alias, reconcile registry/docs,
   THEN delete the crate, THEN rebuild.
This is launcher archaeology, not a one-tool delete. Pause here with Phase 0 complete.

Leave-alone refs (history / other intents): CHANGELOG, ARCHITECTURE.md, intents 016/025,
assets/fonts/README.md.
## Gates
- [x] dependency scan: no live consumers of faelight-palette (or each listed and handled)
- [ ] faelight-palette removed from workspace, flake outputs, aliases, registry/metadata
- [ ] rebuild clean with palette gone; not on PATH; tool count down one; d clean

## Notes
- Clears one of the 8 sub-70 audit tools.
- Unused since Faelight Forest Niri 11.0.0.
- Decommission pattern: see INT-058 (yazi).

## The Rule
"What the forest no longer uses, it lets go -- cleanly." 🌲
