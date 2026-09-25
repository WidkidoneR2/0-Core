---
id: 073
date: 2026-06-20
type: future
title: "Generation count control: prune policy and boot-menu cap"
status: complete
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

## Phase 0 Decision (2026-06-22)
Policy: BLEND, confirmed.
- Declarative boot-menu cap: boot.loader.systemd-boot.configurationLimit = 25. Caps the
  systemd-boot menu to the 25 most recent generations (legible menu, ample for INT-056's
  "boot a known-good older generation" path). Hides older entries from the menu; whether it
  also reduces the profile count is confirmed in Phase 2.
- Manual prune (deliberate, never scheduled): sudo nix-collect-garbage --delete-older-than 14d.
  Data (nixos-rebuild list-generations, 2026-06-22): all 160 generations fall between
  2026-06-03 (gen 25) and 2026-06-19 (gen 184) -- the whole history is ~19 days, because this
  dev system rebuilds ~8+ times/day. A 30d threshold would delete nothing; 14d is the
  data-correct line. It removes the 64 oldest (gens ~25-88, Jun 3-7 -- exactly the doctor's
  "64 prunable") and keeps ~96 (gens 89-184, Jun 8 onward): two weeks of rollback history, far
  beyond what INT-056 needs. Booted gen 184 is never touched.
- Rejected: nix.gc.automatic (scheduled auto-deletion) -- fights manual-control-over-automation.
- Churn: at ~8+ generations/day the count regrows fast, so the ongoing rhythm is a deliberate
  14d prune run by hand when the doctor nags. The eventual one-keystroke prune verb belongs to
  Faelight-Update v-next (INT-074), not here.
- Disk is NOT the driver (store 51 GiB / 1.4%); this is boot-menu legibility + the advisory +
  intentionality, so retention stays generous.

## Results (2026-06-22)
- Gate 2 (prune): sudo nix-collect-garbage --delete-older-than 14d removed the 64 oldest
  generations and GC'd 4635 store paths -- 15.8 GiB freed (store 51 -> 35 GiB), count 160 -> 96.
  Booted gen 184 untouched. Doctor "prunable" dropped 64 -> 1 (a boundary straggler that crossed
  the 14d line minutes later -- the churn, harmless).
- Gate 3 (cap): configurationLimit was ALREADY present at 5 -- menu-vs-count verified directly
  (5 boot entries 180-184 against 96 profile gens). Bumped 5 -> 15 for at-boot recovery headroom;
  rebuilt; /boot/loader/entries now lists 15 (gens 171-185). The menu caps the picker; full
  rollback history stays reachable via nixos-rebuild --rollback regardless.

## Gates
- [x] generation-control policy decided and recorded in this charter
- [x] one-time safe prune done: prunable old generations cleared, booted gen intact, d clean
- [x] ongoing mechanism in place and demonstrated (declarative if config-based)

## Notes
- Tension by design: manual control over automation + keep rollback points for INT-056.
  That is why the mechanism is a deliberate choice, not auto-GC out of the box.
- Current state: 160 generations, 60 older than 14d prunable; Nix store 51 GiB / 1.4%.

## The Rule
"Prune with intent -- keep what the forest may need to return to." 🌲
