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

## ANSWERED (2026-07-15, upstream docs -- no test needed, and it changes the plan)
A stock NixOS ISO will NOT boot under Secure Boot enforcement. The NixOS wiki states plainly that
the installation media's EFI bootloader is not signed and does not use a signed shim, so Secure
Boot must be disabled to boot it. This is a known gap, not an oversight: the nixos-shim project
(RaitoBezarius/nixos-shim) exists specifically to build NixOS installer images that boot on Secure
Boot systems with only Microsoft keys -- still pending shim-review approval. Not solved yet.

CONSEQUENCE -- the USB is NOT the escape hatch, the FIRMWARE MENU is:
  metal recovery = enter firmware -> DISABLE Secure Boot -> boot USB -> fix -> re-enable
The USB is useless while Secure Boot enforces. It is still worth building -- every non-SB failure
(panicking kernel, unbootable generation, botched disko, LUKS header damage, dying NVMe) is a case
where SB is irrelevant and the USB is the only tool. But it does NOT rescue a Secure Boot lockout
on its own.

THE TRAP THIS EXPOSES -- the BIOS password cuts BOTH ways:
Upstream is blunt that a BIOS password is mandatory ("There must be a BIOS password or a similar
restriction that prevents unauthorized changes to the Secure Boot policy") -- without one, an
attacker simply switches Secure Boot off and the entire effort is decoration.
BUT: we need that same password to reach the menu that disables Secure Boot to boot the rescue USB.
LOSE THE BIOS PASSWORD WHILE LOCKED OUT AND THERE IS NO PATH BACK. That is the real brick scenario,
and it is not the one we were worried about. The password must be recorded somewhere that survives
the laptop being unbootable -- not in a password manager on the laptop.

## ORIGINAL OPEN QUESTION (kept for the record -- answered above)
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
- [x] VERIFIED with the REAL media: the USB is refused under Secure Boot enforcement
<!-- evidence: 2026-07-15. Christian's actual nixos-minimal-25.11 stick. Inspected first: its EFI
partition holds ONLY an unsigned GRUB BOOTX64.EFI + refind_x64.efi + grub.cfg -- NO shimx64.efi,
nothing Microsoft-signed to chain through. Confirms the wiki on his own media.
Then booted it: dd'd the stick to a file (he is NOT in the `disk` group, so qemu cannot read
/dev/sda directly -- see the config note below), attached read-only via the QEMU_OPTS seam as
`-drive file=$ISO,format=raw,readonly=on,if=none,id=usbstick -device usb-storage,drive=usbstick,
bootindex=0` so the firmware would try the USB FIRST. Serial log:
    BdsDxe: loading Boot0003 "UEFI QEMU QEMU USB HARDDRIVE 1-0000:00:1d.7-2" ...
    BdsDxe: failed to load Boot0003 ...: Access Denied -- rejected probably by Secure Boot
    BdsDxe: loading Boot0002 "UEFI Misc Device" ...
    BdsDxe: starting Boot0002 "UEFI Misc Device" ...
REFUSED -- and then it FELL THROUGH to the signed disk and booted normally. That fall-through is the
finding that matters: on metal, plugging in this USB with SB enforcing does NOTHING VISIBLE. The
machine boots as if the port were dead. Recovery MUST start at the firmware menu.
This also closes INT-059's gate 6 open question ("the VM has no ISO/boot-media path wired") -- it
does now, via QEMU_OPTS + usb-storage, no launcher change needed. -->
- [ ] Firmware-menu recovery rehearsed in the VM: enter OVMF setup, DISABLE Secure Boot, boot the
      ISO, mount the ESP. This is the ACTUAL metal recovery path now -- the USB alone is not one.
      (Needs `vm gui`, which was fixed by inspection on 2026-07-15 and never tested.)
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
