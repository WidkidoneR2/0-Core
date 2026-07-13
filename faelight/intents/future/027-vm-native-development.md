---
id: 027
date: 2026-06-04
type: feature
title: "VM-native development: faelight-vm tooling (build/up/ssh/down + snapshot/rollback)"
status: planned
tags: [vm, nixos, qemu, development, sandbox]
priority: high
---

## Vision
VMs as first-class forest objects -- a smooth `vm` workflow for building, entering, and
(eventually) snapshotting disposable NixOS VMs, so high-risk work can be proven in a VM before
touching the real system. The vocabulary is forest-native: `vm` is a first-class fsh domain.

## Reality (rewritten 2026-07-13 -- the original libvirt plan was retired; this is what exists)
The `vm` tool is a 286-line shell script at `faelight/packages/faelight/scripts/vm` (INT-077),
driving the `nix/hosts/vm/` NixOS host (hostname faelight-vm, headless, serial-on-stdio, qcow2
disk). fsh wires it via the `vm` verb, forwarding ALL args to the script (INT-079 G3: the script
is the single source of truth for subcommands -- no duplicate verb list in fsh).

IMPORTANT history correction:
- The ORIGINAL 027 plan (libvirt/QEMU + a "nixos-lab" domain, vm_start/stop/snapshot/restore)
  was BUILT, then RETIRED. That code still sits in faelight-shell/commands/mod.rs but is
  UNWIRED and dormant -- superseded by the faelight-vm script approach. (Cleanup candidate:
  the dead nixos-lab functions could be removed.)
- INT-021 (Pinnacle VM study -- "prove the compositor in a VM before metal") is COMPLETE.
  Pinnacle now runs on real metal (greetd session launches; only the keybind/config-load fix
  in INT-142 remains). So the old "INT-021 Pinnacle VM uses this system" gate is RESOLVED and
  retired -- VMs are no longer needed to prove Pinnacle.

## What works today (the tool is real -- "works, needs work")
- `vm build` -- nixos-rebuild build-vm from nix/hosts/vm/ (slow; run when hosts/vm/ changes).
  Also a `regreet` variant (candy-neon ReGreet testbed).
- `vm up` -- launch headless, wait for ssh on the port.
- `vm ssh [cmd]` -- interactive guest shell (password: faelight), optional remote command.
- `vm down` -- stop the VM; qcow2 disk state persists.
- `vm debug` -- read-only diagnostic: live qemu via /proc, port check, launch-lock, stale-state
  janitor view (what it would clean vs leave).
- Engineering already present: process guards, launch lock (flock), stale-state janitor,
  qcow2 disk persistence.

## The real remaining work (honest gates)
- [ ] `vm snapshot <tag>` -- snapshot the faelight-vm qcow2 disk (the code flags this as
      "a later decision, not wired" -- THIS is the main missing capability vs the vision).
- [ ] `vm rollback <tag>` -- restore a named qcow2 snapshot.
- [ ] `vm snapshots` / list -- show available snapshots (the old libvirt vm_snapshots did this;
      reimplement for qcow2).
- [ ] USER PAIN POINTS (fill on the VM day -- Christian to name what feels rough in real use;
      e.g. build/up/ssh as separate steps vs a one-shot, speed, RAM, the password flow, etc.)
- [ ] Optional cleanup: remove the dead libvirt/nixos-lab functions from mod.rs (dormant since
      INT-077).
- [ ] (Consider) snapshots mapped to a tag/intent so a VM snapshot ties to the active intent,
      per the original vision.

## Approach notes
- qcow2 snapshots: `qemu-img snapshot` (-c create, -a apply, -l list, -d delete) against the VM
  disk while the VM is DOWN -- simplest reliable path. Or qemu's live-snapshot via monitor if
  hot snapshots are wanted (more complex). Decide on the VM day.
- Keep the script as the single source of truth (INT-079); add snapshot subcommands THERE, fsh
  forwards automatically.

## Gate (superseded -- see "real remaining work" above)
- [x] vm build/up/ssh/down exist and work (via faelight-vm script, INT-077)
- [x] INT-021 Pinnacle VM need RESOLVED (Pinnacle on metal -- gate retired, not applicable)

## The Rule
"The VM is the safe forest -- disposable ground to prove risky work. Make it smooth, make it
snapshot." 🌲
