---
id: 160
date: 2026-07-15
type: future
title: "Faelight Forest Rescue USB Built 1.0"
status: planned
tags: [faelight forest, usb, wtf, rescue, help]
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

## Why this exists
INT-056's Forest Recovery Protocol is COMPLETE and its runbook (docs/recovery-runbook.md) opens
Level 3 with: "Boot the NixOS installer USB, then: nixos-enter --root /mnt". That USB was never
built. A recovery protocol whose first step is an artifact you do not have is documentation, not
recovery. No intent for rescue media existed before this one.

Christian's framing (2026-07-15): this is the "what if" media -- general disaster recovery, not a
Secure Boot chore. It earns its place against a panicking kernel, a generation that will not boot,
a botched disko change, LUKS header damage, or imaging a dying NVMe. INT-059 made it urgent; it was
always missing.

## Scope: minimal + tools that earn their place -- NOT the whole forest
Base is nixos-minimal, built declaratively from this flake (installation-cd-minimal.nix as a
module). Reproducible, git-tracked, same discipline as everything else.
DELIBERATELY NOT the full system. A rescue USB carrying MangoWM, Friday, fsh, faelight-bar,
home-manager and framework16 hardware config is a WORSE rescue USB: rescue media's one job is
working when everything else does not, and every package added is failure surface on the artifact
you reach for when already in trouble. The daily system also assumes things a live ISO has not got
-- persistent state, LUKS, disko partitioning, a real ESP.

Tools, each with a reason:
- cryptsetup, btrfs-progs   -- the runbook's Level 3 needs both (LUKS unlock, subvolume mounts)
- dosfstools                -- fsck.vfat. On 2026-07-15 a truncated ESP file (dir entry present,
                               cluster chain length 0) made a VM unbootable AND made the ESP
                               remount read-only. fsck was the only way in.
- sbctl, sbsigntool, efibootmgr -- ESP repair under Secure Boot: verify, re-sign, fix boot entries
- The recovery runbook ITSELF baked into the image -- docs belong ON the media, not on the machine
  that will not boot

## THE OPEN QUESTION THIS MUST ANSWER FIRST
Will a stock NixOS ISO boot on a machine with Secure Boot ENFORCING our own keys?
Unknown. NixOS ISOs are not signed by our PK, and whether they carry a Microsoft-signed shim is
unverified. PROVEN 2026-07-15 in the VM: one unsigned BOOTX64.EFI under enforcement produced
"BdsDxe: No bootable option or device was found" -- the firmware refuses unsigned binaries
absolutely. If the ISO is unsigned by anything the firmware trusts, THE RESCUE USB IS AS
UNBOOTABLE AS THE BROKEN SYSTEM.
Consequence if so: metal recovery gains a MANDATORY first step -- enter firmware setup, disable
Secure Boot, THEN boot the USB. Firmware access becomes the real escape hatch, and the BIOS
password becomes the thing standing between a lockout and a stranded laptop.
TESTABLE NOW: the faelight-vm sits in exactly that firmware state (SB enforcing, our keys). Attach
an ISO via the QEMU_OPTS `-cdrom` seam and watch. This also closes INT-059 gate 6's open question
("the VM has no ISO/boot-media path wired").

## Gates
- [ ] ISO builds from the flake: nixos-minimal + the tool list above, each tool justified
- [ ] docs/recovery-runbook.md is baked INTO the image (findable without network or the host)
- [ ] TESTED UNDER SECURE BOOT: boot the ISO in the VM while SB enforces our keys (-cdrom via
      QEMU_OPTS). Record what happens -- this answers whether the USB survives what INT-059 does
      to metal, and it is the whole reason this intent blocks metal day
- [ ] Written to a real USB and BOOTED on the Framework 16 -- before it is needed, not during
- [ ] From the USB: mount /dev/nvme0n1p1 (the ESP) and read it. That is the ONLY step a Secure
      Boot lockout needs (see below)
- [ ] Level 3 walked end to end from the USB: LUKS unlock -> mount @root/@nix/@home ->
      nixos-enter -> rollback a generation. The runbook's own path, demonstrated once.

## FINDING FOR INT-056's RUNBOOK (2026-07-15, proven in the VM)
A Secure Boot lockout does NOT need a LUKS unlock. The ESP CANNOT be encrypted -- firmware must
read it before any OS exists -- so /dev/nvme0n1p1 is plain vfat. Recovery is: boot media -> mount
one partition -> restore/re-sign one file -> reboot. No passphrase, no nixos-enter, no subvolume
mounting. The runbook's Level 3 was written for "cannot get INTO the system"; a Secure Boot
lockout is "cannot get OUT of the firmware" -- a different problem with a much shorter fix.
This was proven from the host with qemu-nbd (attach the qcow2, fsck.vfat, restore BOOTX64.EFI from
a backup, boot -- worked). On metal the USB plays the role qemu-nbd played there.
The runbook deserves this as its own short section, above Level 3.

## Reference
- INT-056 Forest Recovery Protocol / docs/recovery-runbook.md -- this is its missing physical half
- INT-059 Lanzaboote -- blocked on this intent's Secure-Boot-enforcement test
- INT-027 faelight-vm -- the proving ground that can test the ISO before the USB is written
