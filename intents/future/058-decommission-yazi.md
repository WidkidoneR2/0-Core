
---
id: 058
date: 2026-06-13
type: fix
title: Decommission Yazi
status: planned
tags: [fix, bugfix]
version: TBD
---

## Vision
Yazi fully retired; broot is the sole file manager across config, tooling, and docs. No dead yazi integration left behind.

## Why Now
The keybind was already swapped to broot, but yazi persists: still declared in two host configs (so still installed) and still referenced by several Forest tools. Half-removed integration is exactly the drift the forest shouldn't carry.

## Approach
Decommission by category, not by blind grep:
- Install: remove `pkgs.yazi` from hosts/framework16 and hosts/vm configuration.nix; rebuild.
- Tooling (the real work): retire faelight-update's yazi_checker module + its main.rs wiring; remove yazi from faelight-shell (completion.rs, main.rs, commands/mod.rs) and faelight-term; sweep the inactive yazi code in doctor/checks.rs. Build + verify each tool.
- Metadata: update registry/packages.txt, meta/packages.txt, meta/CHANGELOG.md, docs (forest-resilience.md, ALIASES.md).
- Do NOT touch: intents/complete/*.md — historical record.
- Verify, don't assume: Cargo.lock "yazi" is almost certainly the unrelated Rust compression crate; confirm and leave it.

## Success Criteria
- `which yazi` empty after rebuild; not installed.
- All Forest tools build clean with no yazi references (except the verified-unrelated crate).
- `core doctor` honest; no yazi.
- broot confirmed as sole file manager.

## Gate Check
- [ ] pkgs.yazi removed from both host configs + rebuilt
- [ ] faelight-update yazi_checker retired + builds
- [ ] faelight-shell yazi refs removed + builds
- [ ] faelight-term yazi ref removed + builds
- [ ] doctor/checks.rs inactive yazi code swept
- [ ] metadata + docs updated
- [ ] intents/complete left untouched
- [ ] Cargo.lock yazi confirmed unrelated crate
