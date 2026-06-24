---
id: 025
date: 2026-06-03
type: future
title: "core-protect retirement: remove 19-file dependency chain, NixOS-native replacement"
status: in-progress
tags: [faelight]
version: TBD
---
## Vision
Retire `core-protect`, the Arch-era immutability guard, and let NixOS's immutable
Nix store be the protection. core-protect uses `chattr +i` to lock 0-core's stowed
live files (an unlock/edit/relock cycle gated on `.dotmeta` blast-radius). None of
that fits NixOS: the running system is built into the read-only, content-addressed
Nix store -- kernel-enforced and unmodifiable by design; the repo is now source that
gets built, not stowed live; and the `.dotmeta` files the edit workflow needs were
already removed during stow-conflict resolution. Its doctor `core_protect` check is
already excluded from health. core-protect is a 19-file dependency chain protecting a
model that no longer exists, and `chattr +i` on the repo actively fights the
edit-source-then-rebuild flow.

## Why Now
The NixOS migration made the Nix store the real immutability boundary, the `.dotmeta`
metadata it depends on is gone, and its health check is already sidelined. Carrying a
dedicated crate plus references across doctor/stress/fetch and four tools is pure
drag. Retiring it removes dead weight and one more Arch-era assumption from the forest.

## Phase 0 Findings (2026-06-23) -- consumer scan COMPLETE, removal de-risked
Fresh scan confirms the charter (no hidden live wiring, unlike INT-072's palette tangle).
SAFETY (all green for removal):
- Flake/Nix refs: 0 -- core-protect does NOT build into the system anymore (old bar that
  held it is gone). Deleting the crate cannot break a build.
- Service/boot/login invocations: NONE -- not lockout-class. No 24h-hell risk.
- Health: core_protect already excluded (doctor/mod.rs:254-255 is just the filter comment);
  removing core-protect needs only deleting that now-pointless exclusion.
CODE CONSUMERS to handle (~6 sites, 4 tools + engine):
- rust-tools/faelight/src/main.rs (umbrella: lock/unlock/status, ~main.rs:69,273-287)
- rust-tools/faelight-palette/src/main.rs (Lock/Unlock Core menu items :358-359,480-488)
  [INTERSECTS INT-072 -- removing this shrinks palette surface for the later launcher decision]
- rust-tools/teach/src/main.rs (teaches core-protect :293-299)
- rust-tools/faelight-shell/src/commands/mod.rs (lock-core/unlock-core/core-protect :8277-8280)
- engine/src/domains/fetch/mod.rs:98 (reads core-protect state file)
- engine/src/domains/stress/mod.rs:754-779 (test asserting core_protect excluded -- remove test)
- engine/src/domains/doctor/mod.rs:254-255 (the exclusion filter -- remove once gone)
DATA/REGISTRY:
- registry/aliases.toml (alias), registry/tools.toml:35-38 (tool entry)
DOCS (update): docs/ALIASES, THEORY_OF_OPERATION, PHILOSOPHY, POLICIES, NEW-CHAT-DIRECTIVES.
LEAVE ALONE (history): meta/CHANGELOG.md (21), intents/* (complete/in-progress/decisions/
  incidents, ~34 refs) -- historical record, not live.
ORDERING: do 025 before 072 (strips core-protect from palette, shrinking 072's surface).
Verdict: clean bounded removal -- no flake, no boot/login, health pre-handled. Safe to
execute carefully across 4 tools + engine. Phase 0 gate met.

## Approach
- Delete the `rust-tools/core-protect/` crate.
- Remove the `core_protect` doctor check (engine doctor) and confirm health computes
  cleanly without the exclusion hack.
- Strip core-protect references from the engine `stress` and `fetch` domains.
- Remove core-protect from the `faelight` umbrella, `teach`, `faelight-palette`, and
  `fsh` (any lock/unlock/edit commands or aliases).
- Remove from the flake (systemPackages / wrappers) so it stops building.
- Verify no service or boot path invokes `core-protect lock`/`unlock`.
- Rebuild, confirm health is sane, clean up README/CHANGELOG mentions.

## Success Criteria
- [ ] `rust-tools/core-protect/` removed; flake no longer builds or installs it
- [ ] doctor `core_protect` check removed; health computed without the exclusion hack
- [ ] no engine domain (doctor/stress/fetch) references core-protect
- [ ] faelight/teach/palette/fsh references and aliases removed
- [ ] no service or boot path invokes core-protect; login unaffected
- [ ] clean rebuild, health at or above pre-retirement baseline
- [ ] docs (README/CHANGELOG) updated to reflect retirement

## Gate Check
⬜ Not started

---
*"The forest grows with intention."* 🌲
