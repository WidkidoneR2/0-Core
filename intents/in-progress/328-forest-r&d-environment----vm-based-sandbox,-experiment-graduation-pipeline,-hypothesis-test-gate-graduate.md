---
id: 328
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
GATES
[ ] VM launches with vm start command
[ ] Minimal forest installed in VM (fsh, core, daemon, state.db)
[ ] d shows health inside VM for first time
[ ] First snapshot created and restore demonstrated
[ ] First experiment graduated: hypothesis → VM → gate → real machine
[ ] vm commands in fsh vocabulary
[ ] Snapshot naming convention followed for all experiments

DEPENDS ON
INT-325 (faelight-boot) -- first major VM experiment
INT-308 Phase 4 -- DRM testing in VM
INT-327 (self-healing) -- degraded mode testing in VM
INT-329 (typed pipes) -- prototype in VM first
INT-261 (fsh vocabulary) -- vm commands as forest vocabulary

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
