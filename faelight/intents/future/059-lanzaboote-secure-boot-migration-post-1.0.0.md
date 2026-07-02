---
id: 059
date: 2026-06-13
type: study
title: Lanzaboote Secure Boot migration post-1.0.0
status: planned
tags: [study, research, learning]
version: TBD
---

## Objective
Evaluate and (if warranted) migrate framework16 from GRUB to systemd-boot + Lanzaboote,
bringing UEFI Secure Boot + Measured Boot. Post-1.0.0 — research/spike only for now.

## Why it fits 0-Core
Extends "nothing runs without authorization" to the boot chain: a cryptographic chain of
trust where each stage validates the next before handoff. Measured Boot (TPM2) adds attestation.

## Current status (2026-06)
- Lanzaboote ~0.4.x, pre-1.0, actively maintained but "in development" — track stable
  release tags, not master.
- Two NixOS Secure Boot routes: Lanzaboote (established) and Limine (newer) — compare both.
- Key management is out of scope for the project; we own it (sbctl, local PKI bundle).

## Migration path (two hops, each a gate)
1. GRUB -> systemd-boot. Boot, verify clean.
2. systemd-boot -> Lanzaboote: import module, systemd-boot.enable = mkForce false,
   boot.lanzaboote.enable = true, pkiBundle = /var/lib/sbctl, add pkgs.sbctl.
3. Enroll keys (sbctl, --microsoft for OptionROM); enable Secure Boot via UEFI Setup Mode.

## MANDATORY GATE: VM rehearsal first
Rehearse the full path in faelight-vm.qcow2 — INCLUDING a deliberate lockout + Forest
Recovery Protocol recovery — before bare metal.

## Risks
- Bootloader migration is the historical lockout failure mode. Mitigated by the recovery
  protocol (USB -> nixos-enter -> LUKS unlock -> mount @root/@nix/@home).
- Secures the boot chain only; nix store/userspace stay unverified (LUKS covers at-rest).
- Set a BIOS password — Secure Boot is hollow without one.

## Framework16-specific
- Use --firmware-builtin when enrolling keys.
- Firmware updates can disrupt SB state; recovery = "Restore Secure Boot to Factory Settings"
  in UEFI, then re-enroll.

## Decision criteria (promote to feature/arch intent if...)
- VM rehearsal passes incl. recovery; Lanzaboote >= 1.0 or judged stable enough;
  key-management approach chosen.

## References
- nix-community/lanzaboote (docs/getting-started)
- NixOS wiki: Lanzaboote, Secure Boot
- Framework community: "secureboot setup mode" thread
