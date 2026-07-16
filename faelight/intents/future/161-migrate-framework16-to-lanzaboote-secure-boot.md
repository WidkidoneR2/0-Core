---
id: 161
date: 2026-07-15
type: future
title: "Migrate framework16 to Lanzaboote Secure Boot"
status: planned
tags: [faelight, secureboot, lanzaboote, metal, blocked]
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

## BLOCKED ON INT-160 (rescue USB). Do not start this without one.
Not superstition. INT-160 established that a stock NixOS ISO will NOT boot under Secure Boot
enforcement (installer media is unsigned, uses no signed shim). So the recovery path here is
FIRMWARE MENU -> disable Secure Boot -> boot USB -> fix -> re-enable. The USB is not the escape
hatch; firmware access is. But without the USB there is no "fix" step at all.

## What INT-059 proved (complete 2026-07-15, 10/10) -- this intent is the metal replay
The mechanism works end to end. Lanzaboote v1.0.0 (e8c096a) installs, signs a UKI, the firmware
enforces, the machine boots. A deliberate lockout was performed and recovered. `vm rollback`
restores disk AND EFI vars. Odds recorded there: ~85% first try, ~97% recoverable, ~1% brick, WITH
the prerequisites below. Read 059 before starting this -- especially its DIVERGENCES section.

## THE SEQUENCE (proven in the VM; metal differs at step 0 and 4)
0. METAL ONLY -- unrehearsable: metal is SetupMode=0 (USER MODE, a PK is already enrolled by
   Framework/Insyde). Enter the INSYDE firmware menu and "Restore Secure Boot to Factory Settings"
   / erase Platform Keys to reach SETUP MODE. Physical, manual, cannot be scripted, and the VM
   never exercised it (it was already in setup mode). Set the supervisor password while here.
1. Wire lanzaboote into nix/hosts/framework16/configuration.nix exactly as 059 did for the VM:
   import inputs.lanzaboote.nixosModules.lanzaboote; boot.loader.systemd-boot.enable = mkForce
   false; boot.lanzaboote = { enable = true; pkiBundle = "/var/lib/sbctl"; }. The flake input and
   pkgs.sbctl already exist from 059.
   NOTE: autoGenerateKeys was needed in the VM because build-vm bakes the ESP with no keys present.
   On metal you have a RUNNING system, so upstream's order works directly: sbctl create-keys FIRST,
   then rebuild. Decide deliberately which path to use; do not cargo-cult the VM's config.
2. sudo sbctl create-keys   (writes /var/lib/sbctl -- BACK IT UP, see prereqs)
3. sudo nixos-rebuild switch, then `sudo sbctl verify`. Expect BOOTX64.EFI, the UKI, and
   systemd-bootx64.efi SIGNED. The raw kernel-*.efi stays UNSIGNED -- that is correct and expected
   (059 proved from source: lanzaboote SHA-256s the kernel into a .linuxh PE section inside the
   SIGNED UKI, so the kernel needs no signature of its own; sbctl only looks for PE signature
   tables and cannot see it).
4. sudo sbctl enroll-keys --microsoft   (see NON-NEGOTIABLE below)
5. Reboot. Want: `Secure Boot: enabled (user)`, `Measured UKI: yes`.

## --microsoft IS NON-NEGOTIABLE HERE. Measured, not assumed.
Read from sysfs on this host, 2026-07-15 (lspci is not installed):
  0000:03:00.0  VGA controller  vendor 0x1002 (AMD)  device 0x7480  driver amdgpu
                rom size: 131072 bytes    <- a 128KB PCI OPTION ROM, present
  0000:c4:00.0  VGA controller  vendor 0x1002 (AMD)  driver amdgpu   (no rom)
The NixOS wiki: removing OEM keys "may brick some devices which use Microsoft-signed OpROMS ... It
may be impossible to fix if, for example, the GPU relies on these OpROMS." This machine has a GPU
with an option ROM. sbctl will also REFUSE a bare enroll-keys ("Found OptionROM in the bootchain")
-- the tool working as designed.
THE COST, RECORDED HONESTLY: Microsoft's CA lives in db, so anything MS-signed can boot this
machine. That is a real widening of the trust root and it is in tension with 0-Core's own
"Nothing runs without explicit human authorization." We are choosing RECOVERABILITY OVER PURITY,
deliberately. --tpm-eventlog (trust ROM checksums rather than a CA) is philosophically closer;
sbctl marks it experimental. Revisit if it stabilises.
UNRESOLVED: 059's older Framework note says use --firmware-builtin (enroll what the firmware itself
names as default-db). That may be the better answer for this machine. RESOLVE BY RESEARCH BEFORE
METAL DAY, NOT DURING.

## PREREQUISITES -- all of them, before touching the ESP
1. BIOS/SUPERVISOR PASSWORD SET. Upstream is blunt: without one an attacker simply switches Secure
   Boot off and this entire effort is decoration.
   AND THE TRAP: that same password is what YOU need to reach the menu that disables Secure Boot to
   boot a rescue USB. LOSE IT WHILE LOCKED OUT AND THERE IS NO PATH BACK. Store it somewhere that
   survives this laptop being unbootable -- NOT in a password manager on this laptop.
2. INT-160 rescue USB present AND booted once, before it is needed.
3. EFI VARS BACKED UP OFF-MACHINE, before erasing anything:
     for var in PK KEK db dbx; do efi-readvar -v $var -o old_${var}.esl; done
   Step 0 erases Framework/Insyde's PK. Keep a copy.
4. /var/lib/sbctl BACKED UP OFF-MACHINE. Lose those keys and no future generation can ever be
   signed -- every rebuild produces an unbootable system. This is the quiet one that bites months
   later, not on the day.
5. The ESP already meets lanzaboote's umask=0077 requirement: /dev/nvme0n1p1 is mounted
   fmask=0077,dmask=0077 by disko. Verified 2026-07-15.

## RECOGNISE THE FAILURE -- this is what a Secure Boot rejection looks like
    BdsDxe: failed to load Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x6,0x0): Not Found
    BdsDxe: No bootable option or device was found.
    BdsDxe: Press any key to enter the Boot Manager Menu.
IT DOES NOT MENTION SECURE BOOT. No "Security Violation", no "Access Denied", no signature warning.
It reads exactly like a dead SSD -- you would go hunting for a disk failure. It is one unsigned
file the firmware silently refused. Recorded from the VM lockout, 2026-07-15.
Also observed: the firmware did NOT fall back to a still-signed /EFI/systemd/systemd-bootx64.efi.
One bad file, dead machine.

## RECOVERY, if it happens (proven in the VM; the USB plays the host's role)
A Secure Boot lockout needs NO LUKS UNLOCK. The ESP cannot be encrypted -- firmware must read it
before any OS exists -- so /dev/nvme0n1p1 is plain vfat. The fix is:
  firmware menu -> disable Secure Boot -> boot the USB -> mount /dev/nvme0n1p1 -> restore or
  re-sign the file -> reboot -> re-enable Secure Boot
No passphrase, no nixos-enter, no subvolume mounting. Much shorter than INT-056's Level 3, which
was written for "cannot get INTO the system"; this is "cannot get OUT of the firmware".
If the ESP filesystem itself is damaged, fsck.vfat first -- that is what actually happened in the
VM rehearsal, and it is why dosfstools is on INT-160's tool list.
DISCIPLINE: run `sudo sbctl verify` BEFORE every reboot. In the VM it flagged the broken file
before the reboot that would have bricked it. The diagnosis is available in advance, for free.

## Gates
- [ ] INT-160 complete: rescue USB built AND booted on this machine
- [ ] --firmware-builtin vs --microsoft resolved by research for THIS hardware
- [ ] All 5 prerequisites above satisfied, each verified not assumed
- [ ] Supervisor password set AND recorded somewhere that survives an unbootable laptop
- [ ] EFI vars + /var/lib/sbctl backed up OFF-MACHINE, restore path tested
- [ ] Firmware in setup mode (SetupMode=1 in efivars -- verify, do not assume the menu worked)
- [ ] sbctl verify clean before the reboot that enforces
- [ ] Boots with `Secure Boot: enabled (user)` and `Measured UKI: yes`
- [ ] A generation rebuild + reboot AFTER enrollment still boots (the real test is the second one,
      not the first -- signing must keep working)

## Reference
- INT-059 (complete) -- the rehearsal, the divergences, the decision
- INT-160 -- rescue USB. This intent is blocked on it.
- INT-056 / docs/recovery-runbook.md -- and its finding: a Secure Boot lockout needs no LUKS unlock
- DEC-140 -- why Lanzaboote is the trust tier and INT-078 Everglow is deferred
