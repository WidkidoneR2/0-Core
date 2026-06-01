---
id: 349
date: 2026-06-01
type: future
title: "Faelight Forest NixOS -- parity, then clean architecture"
status: in-progress
tags: [nixos, migration, architecture, parity]
version: TBD
---

## Vision

Restore Faelight Forest to full functional parity on NixOS, then re-base it onto a
clean, declarative, testable architecture. Arch was a scramble where one bad update
could brick the machine; NixOS gives reproducible Rust builds, kernel rollback, and
declarative service hardening. The goal is a system I can reason about and recover.
Restore before improve -- never both at once.

## Why Now

The migration just happened and the forest is not yet back to pre-switch capability.
Eighteen pre-migration intents are stalled until the base is whole. The ledger,
structure, and tooling decisions are cheapest to make coherently now, before piling
new feature work onto an unrestored foundation.

## Approach

Two phases plus reconciliation.

Phase 1 -- Parity: retire core-protect, repoint rollback, build deploy and refine
faelight-update for Nix, enable Friday's services, verify GUI tools, finish fonts,
clear integrity alerts, set fsh as login shell once proven solid.

Phase 2 -- Structure: the declarative layout (profiles / hosts / modules / users /
pkgs / labs / tests), Friday in a hardened systemd cage, and NixOS VM tests wired
into CI as anti-brick insurance.

Decisions already made: retire core-protect (LUKS + read-only store + generations
supersede chattr); keep one monorepo with clean internal boundaries rather than
splitting now; recreate the scripts/ dir declaratively rather than refactor fsh.

## Success Criteria

- [ ] core-protect retired (lock-core / unlock-core aliases dropped)
- [ ] rollback alias points to fg rollback
- [ ] deploy built (nixos-rebuild -> core doctor gate -> core deploy record)
- [ ] faelight-update refined (flake update -> rebuild -> doctor -> commit)
- [ ] Friday services enabled (faelight-daemon + security-audit, 0/2 -> 2/2)
- [ ] GUI tools verified (menu, palette, term, ade)
- [ ] fonts complete (color emoji + remaining nerd fonts + pastel)
- [ ] integrity alerts cleared (core integrity fix)
- [ ] fsh set as login shell, after proven solid
- [ ] python3 added to packages (restores the write-to-/tmp workflow)
- [ ] ledger triaged (merge 321+344, collapse 323/343, re-scope 330/325, close 340 as superseded)
- [ ] structure built (profiles / modules / users / pkgs / labs / tests)
- [ ] Friday hardened (ProtectSystem=strict, ReadWritePaths=state.db, NoNewPrivileges)
- [ ] VM boot + service test green in CI
- [ ] boot config (configurationLimit 5 + nix.gc.automatic)

## Gate Check

```
⬜ Phase 1 -- Parity
⬜ Phase 2 -- Structure
⬜ Ledger reconciled
⬜ VM test green
```

---
*"The forest grows with intention."* 🌲
