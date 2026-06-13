---
id: 024
title: "Forest R&D Environment -- VM-based sandbox, experiment graduation pipeline, hypothesis-test-gate-graduate"
status: in-progress
date: 2026-05-21
tags: [forest, vm, sandbox, rnd, research, graduation, pipeline, qemu, experimentation]
---

INT-328 -- Forest R&D Environment -- The Forest That Experiments on Itself
date: 2026-05-21

---
THE PREMISE

Every dangerous idea needs a safe place to be wrong.

The forest is now stable enough that careless experimentation
threatens real daily infrastructure. A bad mkinitcpio change
could break boot. A broken compositor patch could kill the session.
A bad IPC change could cascade into every tool.

The answer is not to stop experimenting.
The answer is to build a formal place to be dangerous.

INT-328 establishes the Forest R&D Environment:
  A QEMU-based VM running a real Arch Linux forest
  A formal pipeline: hypothesis → prototype → test → gate → graduate
  A discipline: nothing lands on production without surviving the sandbox
  A second forest node: the lab, not the workshop

The VM already exists (installed 2026-05-20).
This intent formalizes what it is and how it works.
---
WHAT THE R&D ENVIRONMENT IS

The R&D environment is not a throwaway scratch space.
It is a disciplined second forest node with its own:
  Intent tracking (shared read-only view of main forest intents)
  Health monitoring (independent, not reported to main forest health)
  Deploy pipeline (isolated -- cannot affect main forest)
  State database (separate state.db, can be wiped freely)
  Boot configuration (own GRUB, own initramfs, own kernel params)

It IS:
  A real Arch Linux install running in QEMU with KVM
  The first forest node outside the main machine
  The place where INT-325 (boot splash), INT-308 (DRM), INT-327 (self-healing) get tested
  Disposable: wipe and rebuild in 30 minutes
  Reproducible: rebuild script creates identical environment

It IS NOT:
  A production machine
  Part of the forest health score
  A place to do daily work
  A replacement for the real machine
---
THE GRADUATION PIPELINE

Every dangerous experiment follows the same path:

  HYPOTHESIS
  "I think I can replace mkinitcpio's base hook with a forest-native hook
   that shows a boot splash without breaking the initramfs."

  PROTOTYPE (in VM)
  Build the change in the VM.
  No gates required. Break things freely.
  Friday tracks what was tried and what failed.

  STRESS TEST (in VM)
  Boot the VM 5 times with the change.
  Test failure cases: bad config, missing files, wrong hooks.
  Test recovery: can we recover from a bad initramfs in the VM?
  If it breaks: learn from it, iterate, try again.

  GATE (before graduation)
  [ ] Change has been demonstrated to work, not just implemented
  [ ] Failure cases have been tested and documented
  [ ] Recovery path exists and has been tested
  [ ] Friday has logged the experiment as validated
  [ ] Architecture document updated

  GRADUATE (to main forest)
  Only after all gates pass: apply the change to the real machine.
  Checkpoint created before applying.
  Rollback plan documented.
  If something goes wrong on real machine: rollback, document, learn.

The pipeline is not bureaucracy.
It is the difference between "I think this works" and "I know this works."
---
VM SPECIFICATIONS

Current VM (installed 2026-05-20):
  Disk: ~/vms/arch-test.qcow2 (20GB qcow2)
  ISO: ~/vms/arch.iso (Arch Linux 7.0.3)
  Launch: ~/vms/start-vm.sh
  RAM: 4GB
  CPUs: 4 (host passthrough)
  KVM: enabled
  Display: GTK (Wayland backend)
  Network: SLIRP (user networking, internet access)
  Boot: GRUB (BIOS mode, sda1=BIOS boot, sda2=root)

VM capabilities:
  Full Arch Linux environment
  Internet access (pacman, AUR, git clone)
  KVM acceleration (near-native performance)
  Snapshots (qcow2 supports internal snapshots)
  Multiple VMs (create new .qcow2 for different experiments)

Snapshot workflow:
  Before dangerous experiment:
    qemu-img snapshot -c "before-initramfs-experiment" ~/vms/arch-test.qcow2
  If experiment breaks everything:
    qemu-img snapshot -a "before-initramfs-experiment" ~/vms/arch-test.qcow2
  VM restored to exact state before the experiment.
  This is the VM equivalent of forest checkpoint.
---
EXPERIMENTS QUEUED FOR THE VM

These intents are blocked or waiting for VM validation:

INT-308 Phase 4 -- DRM backend
  The compositor's DRM/KMS backend needs testing outside Niri.
  In the VM: install a minimal display server, test raw DRM output.
  Risk on real machine: could break display. VM: safe.

INT-325 -- faelight-boot (boot splash)
  Custom initramfs hooks, boot splash screen, Plymouth replacement.
  Risk on real machine: bad initramfs = unbootable system.
  VM: break it, fix it, boot again. Zero stakes.

INT-327 -- Forest Self-Healing Runtime
  Testing the degraded mode tiers requires intentionally crashing services.
  Killing the compositor, corrupting state.db, starving memory.
  Risk on real machine: lose work session. VM: just kill the VM.

INT-329 -- Forest Native Runtime (typed pipes)
  Prototype the typed pipe IPC system.
  New binary format, new transport layer.
  Risk on real machine: breaks fsh. VM: test it freely.

INT-330 -- Forest Package Philosophy
  Build the `adopt` command prototype.
  Test package trust scoring against real AUR packages.
  Risk on real machine: none, but needs iteration. VM: faster.
---
VM FOREST INSTALLATION PLAN

The current VM has bare Arch Linux.
The next step is installing a minimal forest environment:
  fsh (the shell) -- so we feel at home
  faelight-daemon -- so Friday can observe
  core -- so health monitoring works
  state.db -- so learning persists across VM sessions

This turns the VM from "a random Arch install" into
"a second, minimal forest node."

Installation sequence (when starting INT-328 work):
  1. Install base tools in VM: git, rust, base-devel
  2. Clone 0-Core into VM: git clone https://github.com/WidkidoneR2/0-Core
  3. Build fsh in VM: cargo build -p faelight-shell
  4. Initialize state.db in VM
  5. Set fsh as login shell
  6. Run d -- see forest health in VM for first time

At that point: the forest runs in two places.
The lab and the workshop.
---
SNAPSHOT NAMING CONVENTION

Snapshots follow the intent naming pattern:

  clean-install          -- bare Arch, before any forest tools
  forest-minimal         -- fsh + daemon + core installed
  before-INT-NNN         -- before a specific experiment
  INT-NNN-working        -- experiment in working state
  INT-NNN-broken         -- intentionally broken for recovery testing

Commands:
  Create: qemu-img snapshot -c "name" ~/vms/arch-test.qcow2
  List:   qemu-img snapshot -l ~/vms/arch-test.qcow2
  Apply:  qemu-img snapshot -a "name" ~/vms/arch-test.qcow2
  Delete: qemu-img snapshot -d "name" ~/vms/arch-test.qcow2
---
FOREST COMMANDS FOR VM MANAGEMENT

Add to fsh vocabulary (INT-261 style):

  vm start          -- launch the R&D VM
  vm stop           -- graceful shutdown
  vm snapshot NAME  -- create a named snapshot
  vm restore NAME   -- restore to snapshot
  vm list           -- list all snapshots
  vm status         -- is VM running?
  vm ssh            -- SSH into running VM (when network configured)

These become first-class forest commands.
The VM is not a separate tool -- it is part of the forest workflow.
---
PHASES

Phase 0 -- Formalize existing VM:
  Document VM specs in state.db
  Create start-vm.sh (done 2026-05-20)
  Create first snapshot: clean-install
  Gate: VM launches from vm start command

Phase 1 -- Install minimal forest in VM:
  git + rust + base-devel in VM
  Clone 0-Core, build fsh + core + daemon
  Initialize state.db
  Set fsh as login shell
  Gate: d shows forest health inside VM

Phase 2 -- Snapshot discipline:
  Snapshot before every experiment
  Name snapshots by intent
  Gate: restore from snapshot demonstrated (intentionally break VM, restore)

Phase 3 -- First real experiment (INT-325 boot splash):
  Prototype initramfs hook in VM
  Test boot, test failure, test recovery
  Gate: boot splash works in VM, recovery from bad initramfs demonstrated

Phase 4 -- Graduate INT-325 to real machine:
  All VM gates passed
  Checkpoint created on real machine
  Change applied
  Gate: boot splash works on real machine, rollback tested
---
RE-SCOPE -- INT-024 living purpose (added 2026-06-13, branch: nixos)
The Arch lab and the NixOS migration testbed both graduated (migration
gates N0-N5 complete; the Framework runs from the flake). Those sections
stay below as history. From here 024 is narrow: the NixOS lab VM
(nixos-lab, qemu:///system) and the vm runtime verbs that drive it --
the sandbox where login/compositor changes are proven before bare metal.
Canonical image: /home/christian/vms/nixos-lab.qcow2 (BIOS/SeaBIOS, qcow2,
virtio disk). Control plane: virsh. Snapshots: internal live (memory+disk);
restore returns to the exact running instant.
Known follow-up (NOT a gate-zero blocker): video is QXL (no GL) -- swap to
virtio-gpu + accel3d + spice gl before any graphical-login test.
Boundary: 024 owns runtime verbs for this one VM; 027 owns multi-VM
lifecycle (create/enter/rollback). Snapshot naming stays a documented
convention (before-INT-NNN, INT-NNN-working/-broken), not a gate.

GATES (NixOS lab -- re-scoped 2026-06-13)
[x] vm start    -- boots nixos-lab via virsh; reaches login/console
[x] vm stop     -- graceful shutdown via virsh shutdown
[x] vm status   -- reports domain state
[x] vm snapshot NAME / vm restore NAME -- internal live snapshot + revert
[x] restore proven -- running snapshot survived stop->restore back to running (RAM+disk, exact instant)
[ ] vm verbs registered in fsh vocabulary + dispatcher

DEPENDS ON (re-scoped)
INT-021 (Pinnacle VM study) -- COMPLETE 2026-06-03; satisfied the prior defer
(Arch-era deps INT-325/308/327/329/261 dropped -- do not carry to NixOS ledger)
ENABLES: 056 (recovery pre-flight), then 038/006/005 (login-zone testing)

TIMELINE
Phase 0-1: next session (VM already installed)
Phase 2: alongside first experiment
Phase 3-4: when INT-325 work begins
Full graduation pipeline: before NY presentation

"The forest experiments on itself.
Not recklessly -- deliberately.
The VM is not a sandbox.
It is a laboratory.
And the laboratory has rules." 🌲

═══════════════════════════════════════════════════
NIXOS MIGRATION TESTBED -- INT-328 charter extension
added: 2026-05-30 (branch: nixos)
═══════════════════════════════════════════════════

WHY THIS LIVES IN 328
The lab exists so dangerous ideas have a safe place to be wrong.
Replacing the forest's own foundation -- Arch to NixOS -- is the most
dangerous idea the forest has ever had. It is the textbook case for the
laboratory. So the migration gets no ad-hoc process; it follows the same
discipline: hypothesis -> VM -> gate -> graduate.

This extends 328. It does not replace it.
The Arch experiments (INT-325, 308, 327, 329, 330) remain valid and keep
their VM. The NixOS migration gets a second, separate VM node.
Two laboratories, one discipline.

WHAT IS DIFFERENT FOR THIS EXPERIMENT
  The testbed VM runs NixOS, not Arch.
  Graduation is not "apply a patch to the Arch machine."
  Graduation is "reinstall the Framework as NixOS from the proven flake."
  It happens once. It must be right.
  The Arch forest on `main` stays the live system until that moment.

NIXOS VM SPECIFICATIONS
  New disk:  ~/vms/nixos-test.qcow2 (separate from arch-test.qcow2)
  ISO:       NixOS minimal (not the graphical installer)
  Same QEMU/KVM host and snapshot discipline as the Arch lab.
  Snapshots: nixos-clean, nixos-flake-applied, nixos-forest-built, before-INT-NNN

INPUTS ALREADY IN HAND
  r-and-d/dependency-manifest.md -- 121 packages mapped to NixOS targets
  nixos branch -- migration work isolated from production main
  committed Cargo.lock -- deterministic builds of the 49 forest tools
  AUR surface cleared -- 9 of 10 in nixpkgs, paru drops

PHASES
Phase N0 -- Readiness  [DONE]
  Repo audited (source-first, lockfile committed).
  Dependency manifest built (121 pkgs categorized).
  AUR surface vetted clean. nixos branch created.
  Gate: manifest complete, branch live.

Phase N1 -- NixOS minimal VM
  Install NixOS minimal into nixos-test.qcow2 (no GUI).
  Gate: NixOS VM boots to a console.

Phase N2 -- flake.nix scaffold
  Write flake.nix + hosts/framework16 from the manifest:
  systemPackages, services.*, fonts.packages, devShell.
  De-absolutize the ~8 live configs in 03-interfaces (no /home/christian paths).
  Gate: nixos-rebuild applies the flake in the VM; declared packages present.

Phase N3 -- Forest tools as derivations
  Package the 49 rust-tools + engine + fsh under pkgs/faelight/,
  built from the committed Cargo.lock.
  Register fsh in /etc/shells and as the login shell.
  (Watch: faelight-compositor's smithay git-dep needs cargoLock.outputHashes.)
  Gate: fsh, core, and daemon build and run in the NixOS VM; fsh is login shell.

Phase N4 -- Health inside NixOS
  Get the forest healthy in the NixOS VM.
  Gate: `d` shows forest health 100% inside the NixOS VM.   <- the proof

Phase N5 -- Graduate to the Framework
  Only after N4 passes. Fresh NixOS install on the Framework:
  disko + LUKS2 declarative partitioning, nixos-install --flake .#framework16.
  state.db carried forward intact. Fresh intent ledger from INT-001.
  Pull in the nixos-hardware Framework 16 module for AMD/firmware/quirks.
  Photos at every milestone.
  Gate: forest runs on the Framework from the flake; rollback (Arch on main) intact.

GATES (NixOS testbed)
[x] dependency manifest complete, AUR surface cleared, nixos branch live
[x] NixOS minimal VM boots -- Framework IS the NixOS install
[x] flake.nix applies in VM; declared packages present
[x] live configs de-absolutized (no hardcoded /home/christian)
[x] forest tools build as derivations in VM; fsh is login shell
[x] d shows 100% inside the NixOS VM -- achieved 2026-06-03
[x] Framework reinstalled from flake; state.db carried; INT-001 ledger begins

"The forest's last Arch experiment is learning how to stop being Arch.
 It will be wrong in the VM many times.
 It will be right on the Framework once." 🌲
## Gate Check
✅ OVERRIDE reverted -- 2026-06-13 -- test override from 2026-06-04 cleared as intended
✅ DEFER lifted -- 2026-06-13 -- INT-021 completed 2026-06-03; VM-infrastructure dependency satisfied