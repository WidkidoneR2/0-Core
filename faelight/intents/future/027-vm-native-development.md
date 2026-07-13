---
id: 027
date: 2026-06-04
type: feature
title: "faelight-vm: Friday-infrastructure VM tooling -- Rust migration (organic) + performance tuning + snapshot/rollback"
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

## Strategic layer (2026-07-13): the VM is FRIDAY INFRASTRUCTURE
Context from Christian: the VM is not a side utility -- it is core proving-ground infrastructure
for building Friday (the real work, ~3-4 months out). It is part of the project/labs triad
together with the debug-shell (INT-153) and the labs/ directory. Friday work is high-risk and
iterative; it happens in disposable VMs FIRST, before touching the real system. So the quality of
this tool directly determines how smoothly Friday gets built. That is why it earns real
investment now, during the prerequisite phase -- a rough VM tool means friction on every Friday
experiment later.

Sequencing: VM improvements are PREREQUISITE work -- done before the real Friday build starts.
Christian's stated plan: focus on the VM when back from break.

## Two SEPARATE efforts (do not conflate -- different payoffs)
Christian named both architecture AND performance as the pain. They are distinct fixes:

### 1. Architecture -- migrate to a Rust `faelight-vm` crate (ORGANICALLY, not big-bang)
Decision: the VM tool should become a proper Rust tool like every other forest tool, NOT stay a
286-line bash script with embedded Python (the embedded Python parsing /proc + managing state is
the tell that it has outgrown shell). Rationale: consistency with the forest, real types + error
handling, testability, first-class fsh domain -- and crucially, Friday work will demand
programmatic VM orchestration (snapshots tied to intents, state you can reason about) that bolts
poorly onto bash.
BUT avoid the trap: DO NOT big-bang-rewrite the working build/up/ssh/down bash into Rust that
does the same thing -- that is motion, not progress (zero new capability for a day of porting).
Instead: build NEW capabilities (snapshots) in Rust as the START of faelight-vm, and port the old
bash pieces over as you touch them. The Rust tool grows in while every step also ships a real
feature. Architecture migrates through value-add, not through a rewrite sprint.

### 2. Performance -- profile + tune the Nix/qemu layer (NOT a language fix)
Honest caveat: a Rust rewrite will NOT make the VM faster. Build/boot time is Nix + qemu, not the
script. The real levers (do these while finishing prerequisite intents -- daily-friction payoff):
- Profile `vm build`: is the time Nix EVALUATION or the BUILD/realize? Different fixes. Time eval
  vs realize separately.
- Build caching: if `vm build` rebuilds when nix/hosts/vm/ is UNCHANGED, that is pure waste -- a
  content/hash check to skip the rebuild is a big cheap win.
- Guest resources: verify KVM acceleration is actually on (-enable-kvm), CPU cores, RAM (a prior
  hand-tune exists for the Mir compositor), disk cache mode.
- Boot time: lighter guest config (fewer services, faster boot target) since it is a test-bed.

## Priorities for the VM day (Christian to confirm order on the day)
- Performance tuning likely FIRST (immediate daily payoff while finishing prerequisites), then
- Snapshots-in-Rust (new capability + starts the faelight-vm crate), then
- Organic port of the rest as touched.
(Christian to set the actual order when the VM day begins.)
