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

## OVERLAP-WITH-059 DECISION + VM NOTE (2026-07-13)
078 already flags it "may replace" INT-059's Secure Boot layer. Make that an EXPLICIT decision to
resolve BEFORE boot work starts: Everglow (build our OWN generation-aware Rust EFI boot manager +
Secure Boot) vs INT-059 (ADOPT existing Lanzaboote). These are two approaches to the SAME boot/
Secure-Boot layer -- likely NOT both. Decide deliberately (build-own = maximal control + huge
scope; adopt-Lanzaboote = faster, proven, less control). Do not drift into building both.
VM: all of this (own boot manager OR Lanzaboote) is rehearsed in the INT-027 faelight-vm with
OVMF + Secure Boot emulation BEFORE metal -- boot-manager work is the classic lockout risk.

## DEC-140 (2026-07-15): DEFERRED, and shrunk to the EFI-app tier only
Resolved: Everglow does NOT compete with INT-059. The value separates into tiers (see DEC-140).
- The METADATA tier (boot notes, commit awareness, provenance, timeline, labels) does NOT need an
  EFI app -- BLS Type #1 entries are plain text and `title` is just a string. It renders in the
  systemd-boot running today, works on top of Lanzaboote, and is testable in userspace. Build early,
  separately.
- The TRUST tier is Lanzaboote (INT-059). Adopted. Everglow does not replace it.
- THIS intent = the EFI-app tier only: boot UI, search, diff, diagnostics page,
  recovery-without-booting, theme engine. Months. Post-1.0. Deferred, NOT cancelled -- adopting
  Lanzaboote does not foreclose it.
Christian's own crate split already drew this line: faelight-generation / faelight-uki /
faelight-secureboot / faelight-cli are ALL userspace; only faelight-boot is EFI. Those userspace
crates may be built early and independently of the EFI app.
REJECTED (DEC-140): the EFI plugin architecture -- plugins inside a trusted boot core either sit in
the verification path (the TCB moved, it did not shrink) or load unsigned code pre-OS. Contradicts
this intent's own "minimize the trusted computing base" note.
NOTE: automatic rollback via boot counting is NOT free -- verified 2026-07-15, no such option exists
anywhere under boot.loader in NixOS. systemd-boot supports +tries; NixOS never emits them.
