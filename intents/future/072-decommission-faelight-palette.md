---
id: 072
date: 2026-06-20
type: future
title: "Decommission faelight-palette (unused since Niri 11.0.0)"
status: planned
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

## Gates
- [ ] dependency scan: no live consumers of faelight-palette (or each listed and handled)
- [ ] faelight-palette removed from workspace, flake outputs, aliases, registry/metadata
- [ ] rebuild clean with palette gone; not on PATH; tool count down one; d clean

## Notes
- Clears one of the 8 sub-70 audit tools.
- Unused since Faelight Forest Niri 11.0.0.
- Decommission pattern: see INT-058 (yazi).

## The Rule
"What the forest no longer uses, it lets go -- cleanly." 🌲
