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

## VM PROVING GROUND + 078 OVERLAP (2026-07-13)
The MANDATORY VM-rehearsal gate above now points at the CONCRETE tool: the forest-native
faelight-vm being built in INT-027 (QEMU/KVM + Rust, OVMF/UEFI + Secure Boot emulation). Rehearse
the full GRUB->systemd-boot->Lanzaboote path + the deliberate lockout + Forest Recovery Protocol
recovery THERE before bare metal. (027 is being reshaped into the canonical VM proving ground; its
foundational capability -- boot through OVMF, snapshot, rollback -- is exactly what this rehearsal
needs.)
OVERLAP: INT-078 (Everglow, build-own boot manager) contemplates REPLACING this Lanzaboote layer.
059 (adopt Lanzaboote) vs 078 (build own) is an either/or to resolve before boot work -- see 078.
TIMING: Christian -- ~1-2 weeks out. Sequenced AFTER the 027 VM foundation exists (need the OVMF
proving ground first).

## DEC-140 (2026-07-15): CLEARED TO PROCEED -- this is the adopted trust tier
Resolved: INT-078 (Everglow) does NOT replace this layer. 078 is deferred to its EFI-app tier
post-1.0; Lanzaboote is the trust chain. Proceed.
The MANDATORY VM-rehearsal gate above is now ACTIONABLE: as of 2026-07-15 the INT-027 faelight-vm
boots the real chain (OVMF -> systemd-boot -> nixos-generation-1.conf -> kernel -> greetd),
verified by the guest itself (/sys/firmware/efi exists; bootctl reports "Firmware UEFI 2.70
(EDK II 1.00)", "Current Boot Loader systemd-boot 260.1").
TWO VM PREREQUISITES remain, found by booting it (guest bootctl, 2026-07-15):
- `Secure Boot: disabled (unsupported)` -- the VM loads plain OVMF_CODE.fd. Key enrollment
  rehearsal needs the OVMF_CODE.secboot.fd variant wired into the launcher.
- `TPM2 Support: no` / `Measured UKI: no` -- Measured Boot needs swtpm emulation.
Note: the VM's OVMF already has WRITABLE EFI vars (NIX_EFI_VARS pflash, unit 1), and INT-027's
`vm snapshot` captures that .fd alongside the disk -- so an enroll-keys-then-lock-yourself-out
rehearsal can be rolled back honestly. That was designed for exactly this.
Also in the option tree: boot.loader.limine -- the other route this intent wants to compare.

## CORRECTION (2026-07-15, same day): the above is WRONG -- Secure Boot IS reachable
The finding below ("the module CANNOT do Secure Boot, therefore faelight-vm must own the qemu
invocation") was based on a BAD TEST. That attempt changed efi.firmware to OVMF_CODE.ms.fd while
leaving the machine type at the module's default i440fx -- which has no SMM. The .ms firmware is
built SMM_REQUIRE=TRUE, so of course it could not come up. The test proved we forgot a flag, not
that the module has a limit.
WHAT IS ACTUALLY TRUE (proven by boot, 2026-07-15):
- The generated launcher ends with `$QEMU_OPTS \` then `"$@"` -- TWO pass-through seams the module
  hands you. QEMU_OPTS is unquoted, so it word-splits into real args.
- qemu MERGES a second -machine over the built-in one. PROVEN by /proc on a live guest:
    -machine accel=kvm:tcg          (from the module)
    -machine q35,smm=on             (from QEMU_OPTS)
    /dev/kvm OPEN  -> KVM SURVIVED
  The machine type is reachable WITHOUT owning the launch, and virtualization is not lost.
- efi.firmware = "${pkgs.OVMFFull.fd}/FV/OVMF_CODE.ms.fd" + efi.variables = ".../OVMF_VARS.fd"
  (plain vars = NO keys = SETUP MODE) + QEMU_OPTS="-machine q35,smm=on -global
  driver=cfi.pflash01,property=secure,value=on"
  -> guest bootctl: `Secure Boot: disabled (setup)`.  Was `disabled (unsupported)`.
  SETUP MODE IS THE GATE -- it is the state sbctl needs to enroll our OWN PK/KEK/db (INT-059).
- Still true and worth keeping: OVMFFull.fd.firmware/.variables resolve to the PLAIN files, so the
  .ms paths must be named EXPLICITLY; and there is no `useSecureBoot` option. But neither blocks us.
- Also learned from the guest's own bootctl: systemd-boot advertises "Enroll SecureBoot keys" and
  "Boot counting" as features -- the loader can enroll keys itself (relevant to INT-059).
CONSEQUENCE: INT-159 loses its main justification. Owning the launch is NOT required for INT-059's
rehearsal. What remains real is child-ownership (the zombie swtpm), which is a much smaller job.
Leaving the original text below verbatim -- the wrong turn is part of the record.

## VM prerequisite is BIGGER than expected (2026-07-15, tested)
The two prereqs are NOT symmetric:
- TPM2: solved. `virtualisation.tpm.enable = true` works (module spawns swtpm). One line.
- SECURE BOOT: NOT solvable in `nixos-rebuild build-vm`. The module has no useSecureBoot, plain
  OVMF is built without SB support, and the generated launcher uses i440fx (`-machine
  accel=kvm:tcg`) with no SMM -- which .ms/SB firmware requires (SMM_REQUIRE=TRUE, needs
  `-machine q35,smm=on`). Tried .ms CODE + plain VARS: the VM did not boot at all.
So this intent's MANDATORY VM-rehearsal gate now DEPENDS on INT-027 building a real qemu launcher
(q35, smm=on, .ms pflash). That is the sequencing: faelight-vm launcher -> SB-capable VM ->
rehearsal (enroll own keys in setup mode, deliberate lockout, recovery) -> metal.
Caution banked: a zombie swtpm survived `vm down` and held the launch lock invisibly (see 027).
Any SB rehearsal will lean on swtpm -- the launcher must own its children.

