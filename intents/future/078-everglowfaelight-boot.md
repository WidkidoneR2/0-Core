---
id: 078
date: 2026-06-23
type: future
title: "Everglow/Faelight-boot"
status: planned
tags: [Boot, NixOS, faelight, gerneration-aware]
---

## Vision
A NixOS-generation-aware Secure Boot + boot manager, written as a Rust EFI application,
that understands /nix/var/nix/profiles directly. Unlike rEFInd (distro-agnostic), sbctl
(keys only), or ukify (UKI only), Everglow knows which generation is current, which
failed boot, which are GC-able, which have specialisations -- so the boot UI is clean
and generation management is bulletproof.

## The Problem
Existing boot tooling is distro-agnostic and fragmented: rEFInd scans disks (no
generation awareness); sbctl manages keys only; ukify builds UKIs but does not tie them
to the generation tree or sign atomically. No single tool does generation-aware menu +
Secure Boot signing + atomic UKI builds where a failed rebuild never breaks boot.

## The Solution (architecture)
UEFI Secure Boot -> Signed Boot Manager -> Generation Menu -> Signed UKIs -> Linux.
1. Secure Boot Manager (like sbctl): generate/enroll/rotate/backup PK,KEK,db keys; verify.
2. Generation-aware menu from /nix/var/nix/profiles: Current, Generation N..., Specialisation, Recovery.
3. UKI builder (systemd-ukify or custom Rust): each generation gets its own signed UKI.
4. Atomic update on nixos-rebuild switch: build -> create UKI -> sign -> update menu ->
   activate; any failure leaves the previous generation bootable.
Boot-time: firmware verifies menu -> menu lists generations -> user selects ->
Secure Boot verifies the chosen UKI -> kernel boots.

## Why better than rEFInd
Everglow knows current/GC-able/failed-boot generations, specialisations, and which kernel
belongs to which generation -> a far cleaner UI than a disk-scanning agnostic manager.

## Feasibility
Pieces exist: signing (sbctl/sbsigntools), UKI (systemd-ukify), EFI graphics (Rust EFI
crates/GNU-EFI/EDK2), generation metadata (Nix profiles), verification (UEFI firmware).
Hard part is NOT Secure Boot -- it is a robust EFI app with a nice UI and bulletproof
generation management.

## The central tradeoff
Every feature moved into the boot manager joins the TRUSTED boot chain -- more code
before the kernel = more attack surface + more that can brick boot. Capable menu vs.
minimal trusted base is the core design question.

## Relationship to other intents
- Depends on / absorbs INT-059 (Lanzaboote / Secure Boot foundation): needs PK/KEK/db keys; may replace that layer.
- DISTINCT from INT-049 (boot-polish) and INT-054 (greetd theme): those are the boot/login EXPERIENCE; Everglow is the boot INFRASTRUCTURE.
- Firmly POST-1.0.0. Likely the flagship post-1.0.0 epic. Multi-phase.

## Success Criteria (rough -- refine into gates when designed clear-headed)
- [ ] Secure Boot keys: generate / enroll / rotate / backup / verify
- [ ] Boot menu generated directly from /nix/var/nix/profiles (generations + specialisations + recovery)
- [ ] Each generation gets its own signed UKI
- [ ] Atomic update: failed build leaves the previous generation bootable
- [ ] Boot chain: firmware verifies menu -> menu -> verified UKI -> kernel
- [ ] Trusted-base size deliberately minimized (tradeoff resolved + documented)

## Note (2026-06-23)
Captured from Christian's design braindump before sleep. NOT a refined charter yet --
deserves a clear-headed design session (possibly a standalone design doc first) before
phases/gates lock. Fix tag typo: gerneration -> generation.
