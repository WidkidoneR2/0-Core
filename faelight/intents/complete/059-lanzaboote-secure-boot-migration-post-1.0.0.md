---
id: 059
date: 2026-06-13
type: study
title: Lanzaboote Secure Boot migration post-1.0.0
status: complete
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

## STATE OF PLAY (2026-07-15) -- two corrections and the real gates
CORRECTION 1: "migrate framework16 from GRUB to systemd-boot" is STALE. Hop 1 is ALREADY DONE.
The host runs systemd-boot 260.1 today (verified: `bootctl status` -> "Current Boot Loader:
Product: systemd-boot 260.1"; /boot/loader/entries/*.conf are BLS Type #1 entries, which GRUB does
not use). This intent was written 2026-06-13; the migration happened since. Only hop 2 (Lanzaboote)
and hop 3 (key enrollment) remain.

CORRECTION 2: this intent had ZERO markdown gates -- prose only. Per INT-130, cicomplete enforces
gates, so as written it could never close. Gates added below.

METAL STATE, measured 2026-07-15 (efivars, ground truth):
  SetupMode  = 0   <- PK IS ENROLLED. The firmware is in USER MODE, not setup mode.
  SecureBoot = 0   <- Secure Boot is merely SWITCHED OFF, not absent.
  Firmware: UEFI 2.90 (INSYDE Corp. 0.773)   TPM2 Support: YES (a real TPM -- measured boot is live)
CONSEQUENCE: enrolling our own keys on metal REQUIRES first clearing the existing keys in the UEFI
menu ("Restore Secure Boot to Factory Settings" / "Erase all Secure Boot Settings") to drop the
firmware into setup mode. That is a PHYSICAL, at-the-keyboard, unscriptable step.
GAP TO NAME HONESTLY: the VM is ALREADY in setup mode (no PK), so the rehearsal will NOT exercise
the key-clearing step. The rehearsal proves enrollment, lockout, and recovery -- not that dance
with the INSYDE menu.
`sbctl` is NOT installed (nixpkgs has 0.18; it is not in systemPackages). Needed either way.

## HOW THE build-vm CHICKEN-AND-EGG RESOLVED (2026-07-15) -- upstream built this path
First `vm build` with lanzaboote DIED:
  Failed to install generation 1: Get stub name: Failed to read public key from
  /var/lib/sbctl/keys/db/db.pem: No such file or directory  -> Failed to install bootloader
  -> Kernel panic - not syncing: Attempted to kill init!
Upstream's order is `sbctl create-keys` on a RUNNING system, then rebuild. build-vm has no
running-system step -- it bakes the ESP inside a sandboxed builder VM. Setting
`autoGenerateKeys.enable = true` fixed it, and the SOURCE says why (nix/modules/lanzaboote.nix):
  allowUnsigned = ... default = cfg.autoGenerateKeys.enable;
    "Whether to allow installing unsigned artifacts to the ESP. This is useful for installing
     Lanzaboote where the key is generated during the first boot."
So autoGenerateKeys flips allowUnsigned, which passes `--allow-unsigned true` to lzbt install.
THE WORKING SEQUENCE (all three legs proven in the VM):
  1. `vm build`  -> ESP installed UNSIGNED (no keys exist yet). sbctl verify: 4 x NOT SIGNED.
  2. `vm up`     -> generate-sb-keys.service fires (wantedBy multi-user.target,
                    ConditionPathExists=!/var/lib/sbctl) -> PK/KEK/db created IN THE GUEST.
  3. `sudo /run/current-system/bin/switch-to-configuration boot` -> reinstalls, now SIGNED.
  4. `sudo sbctl enroll-keys --microsoft` -> PK enrolled, setup mode exits.
  5. reboot -> `Secure Boot: enabled (user)`. It boots.
The keys are NOT baked into the image -- generate-sb-keys ran at 20:50:48 on the guest's own
first boot, minutes after the build. Every fresh overlay makes its own keys.

## DIVERGENCES FROM METAL -- what this rehearsal does NOT prove
1. KEY CLEARING: the VM was already in setup mode (no PK). Metal is SetupMode=0 (user mode) and
   needs "Restore Secure Boot to Factory Settings" in the INSYDE menu first -- physical, manual,
   unrehearsable here.
2. OPTION ROMs: sbctl found an OptionROM in the VM's bootchain (qemu's virtio ROMs). The
   Framework 16's are DIFFERENT hardware -- dGPU module, expansion cards. --microsoft worked here;
   it may or may not be the right answer there.
3. THE FLAG ITSELF: the intent's Framework note says use `--firmware-builtin` (enroll the keys the
   firmware names as default-db). We used `--microsoft`. Metal will NOT be a byte-for-byte replay.
4. NO LUKS in the VM, and no boot media wired -- so the Forest Recovery Protocol cannot be
   mirrored exactly (see gate 6).

## Gates
- [x] `sbctl` in systemPackages; Lanzaboote wired as a flake input, applied to the VM ONLY <!-- evidence: 2026-07-15, VM. flake input github:nix-community/lanzaboote/v1.0.0 (e8c096a) with nixpkgs.follows; module imported in nix/hosts/vm/base.nix ONLY; systemd-boot forced off there; sbctl 0.18 merged into the existing systemPackages list. framework16/configuration.nix still says systemd-boot.enable = true -- metal untouched. v1.0.0 MEETS this intent's own decision criterion ('Lanzaboote >= 1.0'), so this is no longer a spike -->
- [x] VM: keys created + enrolled in setup mode; guest reports `Secure Boot: enabled (user)` <!-- evidence: 2026-07-15, VM. generate-sb-keys.service fired on FIRST GUEST BOOT (not image build) -- 'Created Owner UUID ef01e822-...', full PK/KEK/db in /var/lib/sbctl. Then `sbctl enroll-keys --microsoft` -> 'Enrolled keys to the EFI variables!' -> sbctl status: Setup Mode DISABLED, Vendor Keys microsoft. sbctl REFUSED the bare enroll-keys first: 'Found OptionROM in the bootchain' -- the tool working as designed. --microsoft chosen: the standard path and the likely metal choice; COST RECORDED: Microsoft's CA now sits in db, so MS-signed binaries can boot this machine -- a real widening of the trust root, in tension with 'nothing runs without explicit human authorization'. --tpm-eventlog (trust ROM checksums, not a CA) is closer to our values but sbctl marks it experimental -->
- [x] VM: the guest BOOTS with Secure Boot enforcing, through the signed UKI <!-- evidence: 2026-07-15, VM. full down/up cycle after enrollment -> 'guest is UP after 14s' -> bootctl: 'Secure Boot: enabled (user)'. Current Entry: nixos-generation-1-5e4nienm...efi -- a SIGNED UKI, not a BLS .conf. sbctl verify: BOOTX64.EFI, the UKI, and systemd-bootx64.efi all signed. The raw kernel-6.18.35-*.efi stays UNSIGNED BY DESIGN -- lanzaboote embeds the initrd hash in the signed UKI and validates from there; upstream docs say kernel- files are expected unsigned -->
- [x] VM: `Measured UKI: yes` (TPM2 measurement live, not just present) <!-- evidence: 2026-07-15, VM. guest bootctl reports 'TPM2 Support: yes' AND 'Measured UKI: yes'. NOTE: measurement went live while the UKI was still UNSIGNED -- Measured Boot and Secure Boot are separate mechanisms, and the TPM measures whatever boots, signed or not -->
- [x] VM: DELIBERATE LOCKOUT -- the firmware REFUSED, and the failure looks nothing like one
<!-- evidence: 2026-07-15. Overwrote the signed /boot/EFI/BOOT/BOOTX64.EFI with an UNSIGNED
systemd-bootx64.efi from the store (the realistic metal failure: an unsigned artifact reaches the
ESP while the firmware enforces). sbctl verify caught it BEFORE the reboot: "x BOOTX64.EFI is not
signed". Then vm down; vm up -> the guest never reached sshd, 120s timeout. vm-serial.log, 308
bytes, ALL of it:
    BdsDxe: failed to load Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x6,0x0): Not Found
    BdsDxe: No bootable option or device was found.
    BdsDxe: Press any key to enter the Boot Manager Menu.
THE FINDING, AND IT IS THE POINT OF THIS GATE: the message NEVER MENTIONS SECURE BOOT. No "Security
Violation", no "Access Denied", no signature warning. Just "Not Found" and "No bootable option or
device". On metal at 2am you would conclude your SSD had died and go hunting for a disk failure.
The cause is one unsigned file the firmware silently refused. 308 bytes that will save an hour.
ALSO: the firmware did NOT fall back to /EFI/systemd/systemd-bootx64.efi even though it was still
present and still signed. It refused BOOTX64.EFI and stopped. One bad file, dead machine.
ALSO: this was only readable because -serial file: was wired an hour earlier (INT-159 followup).
Without it the console goes nowhere and the failure is invisible. -->
- [x] VM: RECOVERED from the lockout the metal way -- from OUTSIDE, without `vm rollback`
<!-- evidence: 2026-07-15. The open question resolved: with no ISO wired, the HOST plays the role a
rescue USB plays on metal. qemu-nbd attaches the qcow2 as a block device; from there it IS the
rescue-USB view.
  sudo modprobe nbd max_part=8
  sudo qemu-nbd --connect=/dev/nbd0 <the qcow2>      -> nbd0p1 249M (ESP), nbd0p2 16.5G (root)
  sudo fsck.vfat -a -w /dev/nbd0p1
  sudo mount /dev/nbd0p1 /mnt/vm-esp
  restore BOOTX64.EFI from the backup; sync; umount THEN disconnect
  vm up -> guest is UP (16s)
THE LAYOUT IS THE METAL LAYOUT: ESP + root, and the ESP is plain vfat.
FINDING FOR INT-056's RUNBOOK: a Secure Boot lockout needs NO LUKS UNLOCK. The ESP cannot be
encrypted -- firmware reads it before any OS exists. So recovery is: mount one vfat partition,
restore one file, reboot. No passphrase, no nixos-enter, no subvolume mounts. The runbook's Level 3
was written for "cannot get INTO the system"; this is "cannot get OUT of the firmware" -- a
different problem with a much shorter fix.
UNPLANNED, AND WORTH MORE THAN THE PLAN: the first restore failed. fsck said "File size is 156336
bytes, cluster chain length is 0 bytes" and "Fs was not properly unmounted". The guest wrote the
file over ssh; `vm down` then SIGTERMed qemu -- a power-cord yank -- and the dirty page cache died
with it. The directory entry landed, the data blocks never did: a file that exists and cannot be
read, which also forced the ESP read-only (errors=remount-ro). fsck.vfat was the only way in. That
is what a real rescue looks like, and it is why dosfstools is on INT-160's tool list.
BUG FOUND IN OUR OWN TOOLING: `vm down` SIGTERMs qemu and LOSES UNSYNCED GUEST WRITES. For a
proving ground where you edit files inside the guest, that is a real defect -- it should ACPI
powerdown first and kill only as fallback. Worth its own intent.
LIMIT NAMED HONESTLY: this rehearses the ESP-repair path, NOT the full Forest Recovery Protocol
(no LUKS in the VM), and NOT booting rescue media under enforcement -- which INT-160 shows is
impossible anyway (NixOS ISOs are unsigned; SB must be disabled first). -->
- [x] VM: `vm rollback` restores a working VM after the lockout -- THE NET IS REAL
<!-- evidence: 2026-07-15. Broke it again deliberately (this time WITH a sync, so the corruption was
real rather than the phantom truncation above), confirmed dead, then `vm rollback lanza-sb-enforcing`.
THREE checks, each testing a different claim -- "it booted" alone would have been a lie:
1. guest is UP in 16s                          -> the disk reverted to something bootable
2. BOOTX64.EFI.signed-backup is GONE           -> the disk REALLY reverted. That file was created at
   21:13, AFTER the snapshot; if it survived, the rollback was cosmetic.
3. bootctl: `Secure Boot: enabled (user)`      -> THE EFI VARS ROLLED BACK TOO.
Check 3 is the one that mattered: INT-027 claimed since day one that `vm snapshot` moves the OVMF
vars .fd with the qcow2, and it had NEVER been tested. If the vars had not reverted we would have a
pre-enrollment disk under firmware that still held keys -- a mismatch that surfaces exactly when
you are already panicking.
BONUS: rollback AUTO-SNAPSHOTTED first ('auto-pre-rollback-1784169337') and printed the undo
command. It protects you from the rollback itself, unasked. -->
- [x] Decision recorded: GO for metal -- but as a SEPARATE intent, blocked on INT-160
<!-- evidence: 2026-07-15. This intent is `type: study`. Its OWN decision criteria were: "VM
rehearsal passes incl. recovery; Lanzaboote >= 1.0 or judged stable enough; key-management approach
chosen." ALL THREE NOW MET -- so it closes with a DECISION and PROMOTES to a feature intent, which
is what the ledger's design says to do. It does not stretch across a hardware change.
DECISION: GO. Adopt Lanzaboote on framework16. Odds, as judgment not calculation: ~85% first try,
~97% recoverable, ~1% brick, WITH the prereqs in gate 9 -- without them the middle number collapses.
FOR: mechanism proven end to end tonight (enroll -> sign -> enforce -> boot); v1.0.0 is a stable
release, not a spike; Framework+NixOS+Lanzaboote is a trodden path; every prereq passes (UEFI 2.90,
systemd-boot 260.1, TPM2 yes, bootspec on); and we have SEEN the failure screen -- `BdsDxe: No
bootable option or device was found` reads like a dead SSD and would have cost an hour of chasing
hardware.
AGAINST: option ROMs on OUR hardware were the one thing the VM structurally could not test; the
INSYDE key-clearing step is unrehearsed and unrehearsable; Framework firmware updates can disturb
SB state afterward.
KEY CLEARING UNDERSTOOD: metal is SetupMode=0 (user mode, PK enrolled). Reaching setup mode needs
"Restore Secure Boot to Factory Settings" / erase Platform Keys in the INSYDE menu -- physical,
manual, at the keyboard. Then enroll. This step never happens in the VM (already setup mode). -->

- [x] KEY-MANAGEMENT APPROACH CHOSEN (the third study criterion): sbctl-managed local PKI at
      /var/lib/sbctl, enrolled with --microsoft
<!-- evidence: MEASURED on the host, 2026-07-15 (lspci is not installed; read sysfs directly):
  0000:03:00.0  VGA controller  vendor 0x1002 (AMD)  device 0x7480  driver amdgpu
                rom size: 131072 bytes   <- a 128KB PCI OPTION ROM, present
  0000:c4:00.0  VGA controller  vendor 0x1002 (AMD)  driver amdgpu  (no rom)
The NixOS wiki warns that removing OEM keys "may brick some devices which use Microsoft-signed
OpROMS ... It may be impossible to fix if, for example, the GPU relies on these OpROMS." We have a
GPU with an option ROM. So --microsoft is NON-NEGOTIABLE on metal -- not a preference, the
difference between a laptop and a brick.
THE COST, STATED PLAINLY: Microsoft's CA sits in db, so anything MS-signed can boot this machine.
That is a real widening of the trust root and it is in tension with "Nothing runs without explicit
human authorization." We are choosing recoverability over purity, deliberately, and recording it.
--tpm-eventlog (trust ROM checksums, not a CA) is closer to our values; sbctl marks it experimental.
Revisit if it stabilises.
NOTE: the intent's older Framework note says use --firmware-builtin. UNRESOLVED which is correct for
this machine -- --firmware-builtin enrolls what the firmware itself names as default-db, which may
be the better answer. Resolve BEFORE metal, not during. -->
- [x] Metal prerequisites DOCUMENTED (this gate is to document them; doing them is the next intent)
<!-- evidence: 2026-07-15. The checklist metal day must satisfy before touching the ESP:
1. BIOS/SUPERVISOR PASSWORD -- mandatory. Upstream: "There must be a BIOS password or a similar
   restriction that prevents unauthorized changes to the Secure Boot policy." Without one an
   attacker just switches Secure Boot off and the entire effort is decoration.
   AND THE TRAP (INT-160): that same password is what WE need to reach the menu that disables
   Secure Boot to boot a rescue USB. LOSE IT WHILE LOCKED OUT AND THERE IS NO PATH BACK. It must
   live somewhere that survives the laptop being unbootable -- not in a password manager on it.
2. RESCUE USB -- INT-160. And know its limit: a stock NixOS ISO will NOT boot under Secure Boot
   enforcement (media unsigned, no shim). The USB is not the escape hatch; the FIRMWARE MENU is.
   Recovery = firmware -> disable SB -> boot USB -> fix -> re-enable.
3. BACK UP THE CURRENT EFI VARS, OFF-MACHINE, BEFORE ERASING ANYTHING:
   `for var in PK KEK db dbx; do efi-readvar -v $var -o old_${var}.esl; done`
   We are about to erase Framework/Insyde's PK. Keep a copy.
4. BACK UP /var/lib/sbctl OFF-MACHINE. Lose those keys and no new generation can ever be signed --
   every rebuild produces an unbootable system. This is the quiet one that bites months later.
5. RESOLVE --firmware-builtin vs --microsoft for THIS machine (see gate above). Research, not
   experiment.
6. The ESP already satisfies lanzaboote's umask=0077 requirement: /dev/nvme0n1p1 is mounted
   fmask=0077,dmask=0077 by disko. Verified 2026-07-15.
7. Recognise the failure: `BdsDxe: failed to load Boot0002 ... Not Found` / "No bootable option or
   device was found". It does NOT mention Secure Boot. It looks like a dead disk. It is not. -->

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

