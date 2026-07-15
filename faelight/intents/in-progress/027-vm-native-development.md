---
id: 027
date: 2026-06-04
type: feature
title: "faelight-vm: the forest's proving ground -- full-boot-chain VM (OVMF), snapshots, Rust migration, performance"
status: in-progress
tags: [vm, nixos, qemu, ovmf, development, sandbox, friday]
priority: high
---

## Vision
The VM is the forest's PROVING GROUND: a safe, disposable machine that boots the FULL chain like
real metal, so dangerous work is rehearsed here before it ever touches the Framework 16. Simple
forest verbs on top (`vm up` / `vm down` / `vm build` / `vm status`), real power underneath.

## Why it matters (who depends on this)
- INT-049 lifecycle work (boot -> login -> session -> logout -> shutdown): a wrong early-boot or
  shutdown change on metal can brick boot or corrupt state. The VM is the ONLY safe forge.
- INT-059 (Lanzaboote) + INT-078 (Everglow): bootloader / Secure Boot work. Classic lockout risk.
  Both already require VM rehearsal; this is the VM they rehearse in.
- The deterministic (non-AI) personal assistant + Friday: developed, snapshotted, rolled back in
  disposable VMs. Friday is ~3 months out; a rough VM means friction on every experiment.

## Stack (DECIDED 2026-07-13)
Forest-native: QEMU/KVM as the engine, driven by a Rust `faelight-vm` tool. NO libvirt daemon,
NO GUI (virt-manager). Full firmware boot via OVMF/UEFI.
Rejected: the enterprise stack (libvirt + virt-manager) -- GUI + stateful daemon + XML clash with
the forest's TUI/CLI/Nix-reproducible philosophy, and it is the very layer 027 already retired
once (dead nixos-lab code still sits unwired in faelight-shell/commands/mod.rs).
Reference, not template: Quickemu (bash QEMU wrapper) -- mine it for WHAT (snapshot command shape:
create/apply/delete/info; hardware-optimal qemu flags), not HOW (we are going Rust).

## Reality today
- `vm` = a 286-line shell script at faelight/packages/faelight/scripts/vm (INT-077). fsh forwards
  ALL args to it (INT-079 G3: the script is the single source of truth for subcommands).
- Guest = nix/hosts/vm/base.nix + login-mirror.nix (tuigreet) or login-regreet.nix, built by
  `nixos-rebuild build-vm --flake .#faelight-vm`. 8 GiB / 4 cores / KVM (accel=kvm:tcg -cpu max).
- Works: build (+regreet mode), up, ssh, down, status, debug. Process guards, flock launch-lock,
  stale-state janitor, qcow2 persistence.
- DEADWOOD FOUND (2026-07-15): nix/hosts/vm/configuration.nix (269 lines) is imported by NOTHING
  -- an orphan duplicating base.nix from before the INT-024 base/login split. Cleanup candidate.

## Gates
- [x] FULL BOOT CHAIN: the VM boots OVMF -> systemd-boot -> generation entry -> kernel -> greetd,
      like metal (was kernel-direct, skipping firmware AND bootloader entirely)
      <!-- evidence: commit 5adb7134, nix/hosts/vm/base.nix useBootLoader+useEFIBoot. 2026-07-15
      DEMONSTRATED: launcher -kernel/-initrd/-append count=0, pflash lines=2 (OVMF_CODE.fd +
      writable NIX_EFI_VARS). Guest: /sys/firmware/efi EXISTS (kernel creates it only on real UEFI
      boot). Guest bootctl: "Firmware UEFI 2.70 (EDK II 1.00)", "Current Boot Loader systemd-boot
      260.1", "Loader /boot/EFI/systemd/systemd-bootx64.efi", "Current Entry
      nixos-generation-1.conf". Reached tuigreet login through the full chain. -->
- [x] `vm snapshot <tag>` -- snapshots the qcow2 AND the OVMF EFI vars (both or neither; atomic) <!-- evidence: commit 63b31e57 (crate+wiring), deployed gen 368. 2026-07-15 DEMONSTRATED: 'bootchain-works' created, CROSS-CHECKED via qemu-img snapshot -l directly (ID 1, 07:40:42); EFI vars copy landed 541k. Guards proven: reserved 'auto-' prefix REFUSED, bad tag REFUSED, duplicate REFUSED, live-VM REFUSED ('VM is RUNNING (PIDs: 42857)... image would tear') -- all exit 1 -->
- [x] `vm rollback <tag>` -- restores disk + EFI vars; auto-snapshots current state first <!-- evidence: commit 63b31e57 (crate+wiring), deployed gen 368. 2026-07-15 DEMONSTRATED: wrote /etc/faelight-damage in the guest -> vm down -> vm rollback bootchain-works -> vm up -> 'cat: /etc/faelight-damage: No such file or directory'. Damage GONE. auto-pre-rollback-1784119319 created first (disk + EFI vars), undo command printed -->
- [x] `vm snapshots` list / `vm delete` / `vm prune` (auto-* only, >14d default, --all, --dry-run) <!-- evidence: commit 63b31e57 (crate+wiring), deployed gen 368. 2026-07-15 DEMONSTRATED: deployed `vm snapshots` lists both, correctly typed manual vs auto; prune --all --dry-run found nothing (manual tags protected -- deliberate, never auto-pruned) -->
- [x] `vm up` reports the GUEST is up, not just that the port bound <!-- evidence: commit a3be3943, deployed gen 369. 2026-07-15 MEASURED cold start: port_open said 'ssh ready after 2s' while the guest's sshd did not answer until 11s -- a NINE-SECOND LIE (qemu's user-mode net binds the host forward port the instant qemu starts, so the loop matched on iteration 1 while the guest was still in OVMF). Rust `faelight-vm wait-ready` reads the SSH BANNER instead (only a live sshd sends 'SSH-2.0-...'; qemu with no guest accepts and closes, 0 bytes). DEPLOYED PROOF: 'vm down ; vm up' -> 'guest is UP (sshd answered on port 2222 after 12s)'. Negative case: with no VM, wait-ready refused to claim ready (exit 1). Second organic port to Rust. -->
- [ ] Performance: profile build eval-vs-realize; skip rebuild when nix/hosts/vm/ is unchanged
- [x] Rust `faelight-vm` crate exists, entered ORGANICALLY -- snapshots built in Rust; zero working bash rewritten <!-- evidence: commit 63b31e57 (crate+wiring), deployed gen 368. 2026-07-15 DEMONSTRATED: faelight/rust-tools/faelight-vm/{Cargo.toml,src/main.rs}; auto-registered via the rust-tools/* workspace glob; crane ships the binary with faelight-forest -- ZERO nix edits. Script forwards 5 verbs via fvm() (INT-079 G3 holds). which faelight-vm -> /run/current-system/sw/bin/faelight-vm -->
- [ ] (consider) snapshots tagged with the active intent, per the original vision

## Findings from the first full-chain boot (2026-07-15) -- prerequisites others need
- INT-059 Secure Boot: guest reports `Secure Boot: disabled (unsupported)`. This OVMF is plain
  OVMF_CODE.fd -- Secure Boot rehearsal needs the `OVMF_CODE.secboot.fd` variant wired in.
- INT-059 Measured Boot: guest reports `TPM2 Support: no`. Needs swtpm (software TPM) emulation.
- `vm up` says "ssh ready after 2s" -- FALSE SIGNAL. qemu binds the host forward port on launch,
  before the guest exists. The up-check tests the wrong thing. Fix in the Rust tool.
- Baseline: `vm build` 324.9s (5.4 min). First boot writes the bootloader to the ESP (slower once).

## Approach: architecture and performance are SEPARATE efforts
1. ARCHITECTURE -- organic Rust migration. Build NEW capabilities (snapshots) in Rust as the START
   of a faelight-vm crate; port bash pieces as they are touched. NEVER big-bang-rewrite working
   bash into Rust that does the same thing -- that is motion, not progress.
2. PERFORMANCE -- Nix/qemu tuning, NOT a language fix. A Rust rewrite makes nothing faster.
   Levers: eval-vs-realize profiling, skip-rebuild-when-unchanged, guest weight. (KVM is already
   on -- verified in the launcher, so that box is ticked.)

## Honest constraint: VM-for-logic, metal-for-visual
The VM proves lifecycle LOGIC and CORRECTNESS (systemd ordering, boot sequence, shutdown ordering,
hangs). Final VISUAL flicker-tuning needs REAL hardware -- the VM's virtio GPU is not the
Framework 16 AMD 780M display path. INT-049 must plan around this split.

## Canonical
027 is THE VM intent. Scattered VM references (INT-024 mode flavours, INT-077 script, INT-079
launch hardening) point here. INT-157 (nixosTest regression testing) is a SEPARATE tool for a
separate job: ephemeral test VMs, kernel-direct by design. The test driver is not the dev VM and
must not become it.

## Lessons banked (2026-07-15)
- BOOT LOG WENT EMPTY after useEFIBoot. vm.log was 401 bytes pre-boot-chain; now 0. The serial
  console no longer reaches qemu stdout -- OVMF owns the console early and the handoff changed.
  This is boot-stage OBSERVABILITY we lost, and INT-049's lifecycle work will need it back
  (watching each stage is the whole point). Fix candidate: -serial on the qemu line, or
  console= kernel params reconciled with the firmware path. Not chased today.
- NIX FLAKES ONLY SEE GIT-TRACKED FILES. The first dep after creating the crate shipped no binary --
  the crate dir was untracked, so the flake never saw it ("Git tree is dirty" was the tell).
  `git add` BEFORE `dep` for new files. The script's PATH guard caught it cleanly.
- Snapshots are tied to the CURRENT backing file. `vm build` after a guest-config change makes a new
  store path -- old snapshots become meaningless. Warn-on-mismatch: TODO.
- Friday misreads deliberate guard-test failures as problems ("failed 3 times today") and suggested
  clap/derive fixes for a crate with no clap. Noise, not signal.

## The Rule
"The VM is the safe forest. Boot it like metal, break it freely, roll it back." 🌲
