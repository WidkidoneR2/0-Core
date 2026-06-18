---
id: 025
date: 2026-06-03
type: future
title: "core-protect retirement: remove 19-file dependency chain, NixOS-native replacement"
status: planned
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
