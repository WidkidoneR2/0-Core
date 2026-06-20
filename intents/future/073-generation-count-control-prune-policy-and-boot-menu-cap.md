---
id: 073
date: 2026-06-20
type: future
title: "Generation count control: prune policy and boot-menu cap"
status: planned
tags: [nixos, generations, gc, nix-collect-garbage, boot, housekeeping]
---

## Why
The doctor flags 160 generations (60 older than 14 days, prunable). Left unmanaged the
boot menu and store metadata accrue clutter. But blunt automatic GC would fight two
principles: manual control over automation, and keeping generations as rollback points
(INT-056 Forest Recovery leans on them). So this is a deliberate policy decision plus a
one-time safe prune -- not auto-nix.gc by default.

## What
- Decide the ongoing generation-control mechanism (Phase 0 options).
- One-time safe prune of the old prunable generations, keeping enough recent rollback
  points for recovery.
- Put the chosen mechanism in place (declaratively if config-based).

## Approach
Phase 0 weighs three mechanisms and picks one (or a blend), explicitly honouring manual
control and 056's rollback retention:
  (a) boot.loader.systemd-boot.configurationLimit = N -- declarative cap on boot-menu
      entries; clean menu without aggressively deleting the store.
  (b) deliberate manual prune -- a documented command / fsh verb you run intentionally
      (e.g. nix-collect-garbage --delete-older-than Nd); maximal manual control.
  (c) nix.gc.automatic scheduled -- convenient but automation; likely declined per values.
Phase 1 runs the one-time prune after confirming none of the old generations are needed
recovery points. Phase 2 lands the chosen ongoing mechanism.

## Phases
Phase 0 -- decide policy (weigh manual control + 056 retention); record here.
Phase 1 -- one-time safe prune; booted generation untouched.
Phase 2 -- ongoing mechanism in place (declarative if config-based).

## Gates
- [ ] generation-control policy decided and recorded in this charter
- [ ] one-time safe prune done: prunable old generations cleared, booted gen intact, d clean
- [ ] ongoing mechanism in place and demonstrated (declarative if config-based)

## Notes
- Tension by design: manual control over automation + keep rollback points for INT-056.
  That is why the mechanism is a deliberate choice, not auto-GC out of the box.
- Current state: 160 generations, 60 older than 14d prunable; Nix store 51 GiB / 1.4%.

## The Rule
"Prune with intent -- keep what the forest may need to return to." 🌲
