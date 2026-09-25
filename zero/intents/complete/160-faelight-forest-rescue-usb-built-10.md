---
id: 160
date: 2026-07-15
type: future
title: "Faelight Forest Rescue USB Built 1.0"
status: complete
tags: [faelight forest, usb, wtf, rescue, help]
priority: high
blocks: [161]
---


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
- [x] ISO builds from the flake: nixos-minimal + the tool list above, each tool justified
<!-- evidence: 2026-07-16. nix/hosts/rescue/configuration.nix imports
<nixpkgs>/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix (path VERIFIED by find(1)
against the store -- the first attempt used nixos/modules/installation-cd/... from memory and
failed eval: "path ... does not exist"). Wired as nixosConfigurations.rescue +
packages.rescue-iso. `nix build .#rescue-iso` -> result/iso/faelight-rescue.iso, 1.4G.
Also fixed a real deprecation nixpkgs reported at eval: isoImage.isoBaseName -> image.baseName.
BOOTED IN THE VM, not just built: reached the NixOS installer and a `nixos@faelight-rescue`
prompt -- that hostname is OURS, set in our config, so the thing that booted is our declarative
ISO and not a stock one.
SEPARATE HOST BY DESIGN: nothing here touches framework16. This intent cannot brick anything. -->
- [x] docs/recovery-runbook.md is baked INTO the image (findable without network or the host)
<!-- evidence: 2026-07-16. environment.etc."faelight/recovery-runbook.md".source points at
docs/recovery-runbook.md. Verified in the built system closure:
  /nix/store/w3wrg8qj...-nixos-system-faelight-rescue-26.05.../etc/faelight/
  recovery-runbook.md -> /nix/store/kz8jvr63...-recovery-runbook.md
Plus a users.motd printed at login that names the path AND inlines both recovery paths: the
SHORT one (Secure Boot lockout -- mount /dev/nvme0n1p1, no LUKS needed) and the LONG one
(runbook Level 3 -- cryptsetup open -> mount subvols -> nixos-enter), with the disk layout.
A rescue tool you have to remember how to use is not a rescue tool. -->
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
- [x] The recovery MODEL proven in the VM, and its limit stated honestly
<!-- evidence: 2026-07-16. ORIGINAL GATE WAS BAD and is rewritten here rather than quietly ticked.
It read: "enter OVMF setup, DISABLE Secure Boot, boot the ISO, mount the ESP." That tests OVMF.
OVMF is not this laptop's firmware. What it actually measured: in OVMF with `Secure Boot Mode:
Standard` and a PK enrolled, clearing "Attempt Secure Boot" (Device Manager -> Secure Boot
Configuration) is INERT -- the X clears and PERSISTS across reboots, but `Current Secure Boot
State` stays Enabled, bootctl still reports "Secure Boot: enabled (user)", and the firmware still
refused the USB on three identical cycles. The toggle is not greyed out; it simply does not take
effect. That is an EDK II behaviour and it does NOT transfer: the Framework 16 runs INSYDE Corp.
0.773, and Framework 16s disable Secure Boot from the INSYDE menu routinely -- it is how people
boot Linux installers on them.
WHAT THE VM DID PROVE (the controlled experiment above): same ISO, same bootindex=0, ONE variable.
SB enforcing -> "Access Denied -- rejected probably by Secure Boot" -> fell through to the signed
disk. SB not enforcing -> "starting Boot0003" -> NixOS installer. The MODEL holds:
    firmware menu -> disable Secure Boot -> boot USB -> fix -> re-enable
WHAT NO VM CAN PROVE: that INSYDE's menu toggles Secure Boot. That belongs to INT-161, where the
firmware is real, and is recorded there as a prereq to verify BEFORE enrolling any keys.
Useful method notes banked: `systemctl reboot --firmware-setup` beats catching an ESC window
(bootctl already reports "Boot into FW: supported"); ESC on the OVMF front page EXITS and boots
rather than backing up; `vm ssh` cannot answer while the guest is in firmware (qemu binds the
forward port regardless -- INT-027's known false signal); and a serial log from a firmware TUI is
full of ANSI escapes, so `strings <log> | tail -30`, never `tail`. -->
- [x] Written to a real USB and BOOTED on the Framework 16 -- before it is needed, not during
<!-- evidence: 2026-07-16, REAL HARDWARE. Written with
  sudo dd if=/nix/store/...-faelight-rescue.iso/iso/faelight-rescue.iso of=/dev/sda \
          bs=4M status=progress conv=fsync
-> 1423278080 bytes, 128s, 11.1 MB/s. TWO independent confirmations the write landed rather than
one claim: the stick's label went nixos-minimal-25.11 -> nixos-minimal-26.05 (our nixpkgs pin,
not the old installer), and sda1 went 1.5G -> 1.3G (our smaller image).
Then it BOOTED the Framework 16 -- INSYDE firmware, real hardware, not qemu -- to
`[nixos@faelight-rescue:~]$`. That hostname is set in nix/hosts/rescue/configuration.nix: the
thing that booted is OUR declarative ISO. The motd printed at login as designed, naming both
recovery paths and the disk layout.
INT-056's Level 3 opened with "Boot the NixOS installer USB." As of this gate that sentence
refers to an artifact that exists and has demonstrably booted this machine. -->
- [x] From the USB: mount /dev/nvme0n1p1 (the ESP) and read it. That is the ONLY step a Secure
      Boot lockout needs (see below)
<!-- evidence: 2026-07-16, from the rescue USB running on the Framework 16:
    sudo mount -o ro /dev/nvme0n1p1 /mnt/esp
    ls -la /mnt/esp/EFI/BOOT/    -> BOOTX64.EFI, 154112 bytes
That is the LIVE systemd-boot on the real ESP -- same 154112 bytes as the file used throughout the
INT-059/162 work. Mounted READ-ONLY (read first, touch nothing) and with NO LUKS unlock, which is
the finding this gate exists to prove: the ESP cannot be encrypted because firmware must read it
before any OS exists.
Every link of the Secure Boot lockout recovery is now demonstrated on Christian's own firmware:
media boots -> ESP mounts -> the file is readable. The repair step (restore/re-sign BOOTX64.EFI)
was separately proven from the host with qemu-nbd during INT-059. -->
- [x] Level 3 walked end to end from the USB: LUKS unlock -> mount @root/@nix/@home ->
      nixos-enter -> rollback a generation. The runbook's own path, demonstrated once.
<!-- evidence: 2026-07-16, REAL HARDWARE, from the rescue USB. Booted the Framework 16 from the
stick and walked docs/recovery-runbook.md Level 3 exactly as written: cryptsetup luksOpen
/dev/nvme0n1p2 cryptroot -> mount -o subvol=@root/@home/@nix (compress=zstd,noatime) +
/dev/nvme0n1p1 -> nixos-enter --root /mnt -> nixos-rebuild switch --rollback.
PROOF, measured after returning to the real system:
    377   2026-07-15 19:05:30   (current)
    378   2026-07-15 23:07:47
He entered rescue on generation 378. He came back running 377. The rollback LANDED.
The endpoint proves the chain: `nixos-rebuild switch --rollback` cannot run inside nixos-enter
unless the LUKS unlock took, all four mounts succeeded, and the chroot came up. Every link held.
WHAT THIS GATE IS FOR, and why it is not a duplicate of gate 6: gate 6 proved the ESP is READABLE
from rescue media (plain vfat, no passphrase) -- that recovers a Secure Boot lockout, which is
"cannot get OUT of the firmware". Gate 7 is the other failure: "cannot get INTO the system". It
needs the LUKS unlock, the btrfs subvolumes, and a working chroot, and it ends in a REPAIR rather
than a read. Different failure, different fix, both now demonstrated.
THE RUNBOOK IT WALKED WAS FIXED FIRST (commit 1fa30b2f): lines 71 and 99 pointed at
~/0-core/hosts/framework16/ -- a path INT-061 moved to nix/hosts/ -- so the media whose whole job
is working when nothing else does was telling him, at 3am, to edit a file that does not exist.
Found by READING the runbook before walking it. Walking a runbook with a dead path proves nothing.
INT-056's Level 3 opened with "Boot the NixOS installer USB, then: nixos-enter --root /mnt". As of
this gate, that is no longer documentation. It is a rehearsed procedure on media that exists. -->

## THE CONTROLLED EXPERIMENT (2026-07-16) -- the recovery model, isolated and proven
Same ISO. Same attach (`-drive ...,readonly=on` + `-device usb-storage,bootindex=0` via the
QEMU_OPTS seam). ONE variable changed: whether Secure Boot was enforcing.

SB ENFORCING (snapshot lanza-sb-enforcing), Christian's real nixos-minimal-25.11 stick:
    BdsDxe: loading Boot0003 "UEFI QEMU QEMU USB HARDDRIVE 1-0000:00:1d.7-2" ...
    BdsDxe: failed to load Boot0003 ...: Access Denied -- rejected probably by Secure Boot
    BdsDxe: loading Boot0002 "UEFI Misc Device" ...
    BdsDxe: starting Boot0002 "UEFI Misc Device" ...        <- FELL THROUGH to the signed disk

SB NOT ENFORCING (snapshot lanza-signed-not-enrolled), our faelight-rescue.iso:
    BdsDxe: loading Boot0003 "UEFI QEMU QEMU USB HARDDRIVE 1-0000:00:1d.7-2" ...
    BdsDxe: starting Boot0003 "UEFI QEMU QEMU USB HARDDRIVE 1-0000:00:1d.7-2" ...   <- BOOTS
    Loading graphical boot menu...
    -> NixOS installer -> nixos@faelight-rescue

"failed to load ... Access Denied" -> "starting". One variable. This IS the recovery model this
intent rests on, demonstrated rather than argued:
    firmware menu -> DISABLE Secure Boot -> boot the USB -> fix -> re-enable
And the fall-through is the trap: with SB on, plugging in the USB does NOTHING VISIBLE. The
machine boots normally, as if the port were dead.

HONEST LIMIT: this used `vm rollback` to reach the non-enforcing state, NOT the firmware menu.
Navigating an actual firmware menu to toggle Secure Boot is STILL UNTESTED (see the next gate) --
and on metal that menu is the whole escape hatch. The serial log ends at "Loading graphical boot
menu" because GRUB hands off to VGA there; the rest went to the screen.

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
