---
id: 161
date: 2026-07-15
type: future
title: "Migrate framework16 to Lanzaboote Secure Boot"
status: planned
tags: [faelight, secureboot, lanzaboote, metal, blocked]
depends_on: [160]
---


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
4. sudo sbctl enroll-keys --custom   (RESOLVED 2026-07-16 -- see below. NOT --microsoft, NOT --firmware-builtin)
5. Reboot. Want: `Secure Boot: enabled (user)`, `Measured UKI: yes`.

## RESOLVED 2026-07-16: --firmware-builtin. NOT --microsoft. (This section was WRONG.)
The heading below used to read "--microsoft IS NON-NEGOTIABLE HERE. Measured, not assumed." It was
NOT measured. It read a ROM FILE EXISTING IN SYSFS and concluded the ROM was in the boot chain.
Those are different facts, and the difference is the whole argument.

THE ACTUAL MEASUREMENT (sbctl's own FAQ names the test; run on this laptop 2026-07-16):
    sudo cp /sys/kernel/security/tpm0/binary_bios_measurements /tmp/eventlog
    sudo chown christian /tmp/eventlog
    nix shell nixpkgs#tpm2-tools -c tpm2 eventlog /tmp/eventlog | grep "EventType:" | sort | uniq -c
  EV_EFI_BOOT_SERVICES_DRIVER ................ ABSENT. Zero. <- the FAQ's marker for OpROM
  EV_EFI_BOOT_SERVICES_APPLICATION ........... 3 (bootloader + UKI, expected)
  Unknown event type ......................... 47  -> ALL PCR 0 (34) and PCR 1 (13). NONE in PCR 2.
PCR 0 is firmware code, PCR 1 is firmware configuration -- INSYDE vendor events, where they belong.
PCR 2 is where OPTION ROM CODE measures. IT IS EMPTY.
The dGPU's 128KB ROM exists on the card and NEVER ENTERS THE SECURE BOOT CHAIN on this machine.

AND THE BRICK SCENARIO HAS A CONDITION THIS INTENT MISSED. sbctl's FAQ, verbatim:
  "If you don't have any iGPU but your nvidia card has Option ROM that fails to validate, you
   might not have any way to display graphics. This would prevent you from turning off secure boot."
IF YOU DON'T HAVE ANY iGPU. This machine has one: 0000:c4:00.0 (Radeon 780M in the 7840HS, no ROM).
0000:03:00.0 (device 0x7480, Navi 33 = the RX 7700S module) is the one with the ROM. Worst case the
dGPU does not initialise and the display still reaches the firmware menu.

FIELD EVIDENCE ON THIS EXACT HARDWARE (community.frame.work):
  - DHowett (Framework): "Secure boot doesn't require Microsoft's keys."
  - Matt_Hartley (Framework): "This is the correct answer. You do not need to do anything special."
  - Quentin, Framework 16: own keys + Framework's frame.work-LaptopAMDDB key only -- "everything
    works perfectly, including the BIOS menus and even the recent firmware update."
  - Anselm_Schuler, Framework 16, thread tagged graphics-module-amd-rx7700s -- THE SAME GPU MODULE:
    custom keys, NO Microsoft, empty dbx. Took a firmware update. "The machine didn't get bricked."

WHAT --firmware-builtin ACTUALLY DOES -- DHowett again, and read the footnote:
  "You can find the Framework certificates in the NVRAM variables dbDefault, KEKDefault and
   PKDefault. I know[1] that sbctl has support for enrolling the manufacturer's default
   certificates when you set up your own key management."
   [1] having written that support
A Framework engineer wrote sbctl's --firmware-builtin. It enrols Framework's OWN certs -- the ones
signing the firmware's UEFI binaries and fwupd -- without putting Microsoft's CA in db.

SECOND CORRECTION, same day: --firmware-builtin IS ALSO WRONG. We read the actual certificates
instead of trusting the reasoning, and they say otherwise. dbDefault on THIS machine holds THREE:
    mfg_cert-0  CN=Microsoft Windows Production PCA 2011    notAfter Oct 19 2026
    mfg_cert-1  CN=Microsoft Corporation UEFI CA 2011       notAfter Jun 27 2026  <- ALREADY EXPIRED
    mfg_cert-2  CN=frame.work-LaptopAMDDB                   notAfter Oct 14 2120
Extracted with:
    sudo dd if=/sys/firmware/efi/efivars/dbDefault-8be4df61-... of=/tmp/dbDefault.esl bs=1 skip=4
    nix shell nixpkgs#efitools -c sig-list-to-certs /tmp/dbDefault.esl /tmp/mfg_cert
    nix shell nixpkgs#openssl -c openssl x509 -in /tmp/mfg_cert-N.der -inform der -noout -subject -dates
And sbctl agrees independently: `sbctl status` -> "Vendor Keys: microsoft builtin-db builtin-KEK
builtin-PK".
sbctl(8) on the flag: "-f, --firmware-builtin: Enroll signatures FROM dbDefault, KEKDefault or
PKDefault." Framework's firmware-default db CONTAINS Microsoft. So --firmware-builtin would enroll
BOTH Microsoft CAs *plus* Framework -- MORE Microsoft than --microsoft, not less. The exact opposite
of the reason it was chosen.

THE ACTUAL DECISION: `sudo sbctl enroll-keys --custom`
sbctl(8): "-c, --custom: Enroll custom KEK and db certificates from /var/lib/sbctl/keys/custom/KEK/,
/var/lib/sbctl/keys/custom/db/, respectively."
So: place ONLY frame.work-LaptopAMDDB (already extracted to /tmp/mfg_cert-2.der) into
/var/lib/sbctl/keys/custom/db/, then enroll --custom. Result: own PK/KEK/db + Framework's db cert.
ZERO Microsoft. Framework's firmware binaries and fwupd stay trusted; the cert runs to 2120; nothing
expired goes in; and with PCR 2 empty nothing needs Microsoft's CA anyway.
This is EXACTLY what Framework 16 owner Quentin described running: "only my own keys (and the
Framework frame.work-LaptopAMDDB key in db)".

THE DRY RUN -- do this BEFORE enrolling for real (Christian's rule: all green, tested several times):
    sbctl enroll-keys --custom --export esl
sbctl(8): "--export: Export the keys we intend to enroll as EFI Signature Lists (esl), or EFI
Authenticated Variables (auth) into the current working directory." UNRESOLVED: whether --export
exports INSTEAD of enrolling or exports AND enrolls. READ THE SOURCE BEFORE RUNNING IT. (The machine
is in user mode, so the firmware should reject a real enrollment anyway -- but "should" is not a
plan.) Then decode the exported esl with sig-list-to-certs + openssl and confirm: own certs +
frame.work-LaptopAMDDB, no Microsoft.

VERIFY AFTER ENROLLING: `sbctl status` -> Vendor Keys should NOT say microsoft. CachyOS reports
--firmware-builtin producing DUPLICATE db entries on some ASUS/Gigabyte boards, rejected by the
firmware as a Secure Boot Violation. Not INSYDE, not reported on Framework, and we are not using
that flag now -- but check rather than assume.
FALLBACKS, in order: --firmware-builtin (Framework + Microsoft, still recoverable), then --microsoft.
FALLBACK IF IT REFUSES: --microsoft still works and is still recoverable. It is the second choice
now, not the only one.

## (superseded) the original reasoning, kept for the record
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
RESOLVED 2026-07-16 -- see the section above. BOTH were wrong. The answer is --custom: own keys
plus frame.work-LaptopAMDDB alone, because Framework's dbDefault CONTAINS Microsoft's CAs, so
--firmware-builtin would enroll them too. Confirmed by measurement (PCR 2 empty) and by Framework
owners running the same RX 7700S module on custom keys only.

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

## RECOGNISE THE FAILURE (corrected 2026-07-15 -- an earlier version of this section was WRONG)
MEASURED under OVMF with Secure Boot enforcing. A VALID but UNSIGNED bootloader gives:
    BdsDxe: failed to load Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x6,0x0):
            Access Denied -- rejected probably by Secure Boot
    BdsDxe: No bootable option or device was found.
OVMF NAMES SECURE BOOT. An earlier version of this section claimed the failure never mentions
Secure Boot and reads like a dead SSD -- that came from a TRUNCATED file (our own `vm down` SIGTERM
bug), which gives "Not Found" instead. Two different failures:
    valid but UNSIGNED   -> "Access Denied -- rejected probably by Secure Boot"
    TRUNCATED / corrupt  -> "Not Found"
Both end with the SAME tail ("No bootable option or device was found. Press any key to enter the
Boot Manager Menu."). The diagnostic is the line ABOVE it -- read it, do not glance at it.

*** THIS IS OVMF / EDK II. THIS MACHINE RUNS INSYDE Corp. 0.773. ***
Different firmware. It may word this differently, or say nothing at all. There is no serial console
on this laptop -- you get whatever INSYDE paints on screen, if anything. DO NOT ARRIVE AT METAL DAY
EXPECTING THIS EXACT STRING. Expect a refusal; the wording is unknown until we see it.

THE ONE THAT WILL ACTUALLY BITE YOU (measured with the real nixos-minimal-25.11 USB):
With Secure Boot enforcing, the firmware tried the USB FIRST (bootindex=0), refused it with Access
Denied, and SILENTLY FELL THROUGH to the signed disk -- booting normally. So on metal: plug in the
rescue USB with SB on, select it, and NOTHING VISIBLE HAPPENS. Your machine just boots. You will
reseat it, try another port, suspect the stick. The cause is the firmware declining unsigned media
and moving to the next boot entry.
THEREFORE: DISABLE SECURE BOOT IN THE FIRMWARE MENU **BEFORE** EXPECTING THE USB TO DO ANYTHING.

DISCIPLINE: run `sudo sbctl verify` BEFORE every reboot. In the VM it flagged the broken file every
single time, before the reboot that would have bricked it. The diagnosis is available in advance,
for free.

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
- [x] INT-160 complete: rescue USB built AND booted on this machine
<!-- evidence: 2026-07-16. INT-160 CLOSED 7/7, commit 12d3fe14. Not "an ISO exists" -- the whole
chain demonstrated on this laptop's own INSYDE firmware:
  - dd to the stick: 1423278080 bytes, 127s. Confirmed two ways (label nixos-minimal-25.11 ->
    26.05, our nixpkgs pin; sda1 1.5G -> 1.3G).
  - Booted the Framework 16 to [nixos@faelight-rescue:~]$ -- our hostname, our declarative image.
  - THE SHORT PATH (what a Secure Boot lockout actually needs): mount -o ro /dev/nvme0n1p1 ->
    BOOTX64.EFI, 154112 bytes. The live systemd-boot, read from rescue media, no LUKS unlock.
  - THE LONG PATH (runbook Level 3): luksOpen -> mount @root/@home/@nix + ESP -> nixos-enter ->
    nixos-rebuild switch --rollback. Entered on generation 378, came back running 377 (current).
    The rollback LANDED.
And the runbook it carries was BROKEN and got fixed first (commit 1fa30b2f): two pre-061 paths
pointed at ~/0-core/hosts/framework16/, which no longer exists. Found by reading it before walking
it.
WHAT THIS CHANGES FOR THIS INTENT: 059 recorded ~97% recoverable, but that number rested on a
rescue USB that had never been built, let alone booted. It is now rehearsed on this hardware, and
the recovery leg of the odds is real rather than aspirational. The remaining risk lives in the
gates below -- the unresolved --microsoft question and the five unmet prerequisites. -->
- [x] --firmware-builtin vs --microsoft resolved by research for THIS hardware
<!-- evidence: 2026-07-16. VERDICT: --custom (own keys + frame.work-LaptopAMDDB only). Resolved by MEASUREMENT plus field evidence,
not by argument -- see the RESOLVED section above for the full case.
The short version: the TPM eventlog on this laptop has ZERO EV_EFI_BOOT_SERVICES_DRIVER events (the
sbctl FAQ's own marker for option ROM in the bootchain), and all 47 undecoded events sit in PCR 0/1
(firmware code + config); PCR 2, where OpROM measures, is EMPTY. The dGPU's ROM never enters the
Secure Boot chain. Separately, the FAQ's brick scenario is conditioned on having NO iGPU -- this
machine has one. And a Framework 16 owner with the SAME RX 7700S module runs custom keys with no
Microsoft through firmware updates without incident, with Framework staff on record that Microsoft
keys are not required.
This gate CORRECTED the intent rather than confirming it: the "--microsoft IS NON-NEGOTIABLE.
Measured, not assumed." section was not measured -- it inferred a bootchain fact from a sysfs file.
The gate did its job. -->
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
