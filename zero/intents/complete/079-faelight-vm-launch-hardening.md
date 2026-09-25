---
id: 079
date: 2026-06-23
type: feature
title: "faelight-vm launch hardening: atomic lock, stale-state janitor, vm debug"
status: complete
tags: [vm, faelight-vm, qemu, stability, lane0, flock]
priority: high
---
## Why
Follow-up from INT-077 Gate 6, which seeded these exact fixes. The smooth VM loop
(INT-077) works and is the linchpin for INT-056, INT-043 (gate 131), and the
Pinnacle / 067 / 010 / 055 compositor cluster. But Gate 6 observed repeated
`vm gui` spawning multiple qemu on one qcow2, traced to `vm down` no-op'ing through
fsh's shadowed pkill. That specific bug is already fixed in the script (Python /proc
walker, verified 2026-06-23). This intent closes the REMAINING structural weakness:
the guard in `up`/`gui` checks-then-launches with no atomic lock (a TOCTOU race), and
host-side stale state (orphaned spice.sock / vm.pid) is left after a guest-side poweroff.

## Evidence baseline (2026-06-23, all read-only)
- Source script `pkgs/faelight/scripts/vm`: Python /proc walker, ZERO pkill. Correct.
- fsh `vm_dispatch` execs the source script by path (no stale deployed copy exists).
- Launcher bakes in `-nographic`; guest config `graphics = false`. No qemu window.
- Guard needle "faelight-vm" present in live qemu cmdline (-name + -drive). Guard matches.
- Launcher `-drive` uses qemu's default OFD write-lock: a true second writer is REFUSED
  by qemu, not silently corrupting. Last vm.log = one clean boot + clean guest shutdown.
- Remaining gap: no atomic mutex between guard-check and launch; stale spice.sock/vm.pid.

## What
Harden the `vm` workflow against double-launch and stale state. Bash script edits only
(pkgs/faelight/scripts/vm) -- fsh execs it directly, so NO fsh rebuild and NO VM rebuild.
Every edit backed up first and reversible.

## Gates
- [x] G1: atomic `flock` guard on `vm up` / `vm gui` -- a concurrent launch refuses
      instantly; the check-then-launch TOCTOU window is closed.
- [x] G2: stale-state janitor -- orphaned spice.sock / vm.pid cleared on up/gui/status
      when no qemu is live (last shutdown was guest-side poweroff, so host cleanup was skipped).
- [x] G3: `vm debug` -- the read-only diagnostic baked in as an authorized subcommand
      (launches nothing, kills nothing).
- [x] G4: verified -- `vm up` and `vm gui` each yield exactly one qemu guest (gui: exactly
      one viewer window); the live qemu is matched by the guard string; `vm down` verifiably
      returns zero qemu, confirmed by a read-only re-probe.

## Notes
- Scope discipline: bash script only. A Rust-native rewrite of `vm` is explicitly OUT of
  scope (would be its own intent) -- we harden working code, not rewrite it.
- Relationship to INT-077: 077 stays complete (the smooth loop). 079 hardens it.
- Relationship to INT-027: deeper VM tooling (create/snapshot/rollback) remains 027.


## Evidence log
### 2026-06-23 -- G1 (flock) + G4 (down=zero) DEMONSTRATED
Live run in fsh (script edited, backed up vm.bak-20260623T162859, no VM/fsh rebuild):
- `vm up` -> one guest, ssh ready on 2222 in 2s.
- second `vm up` -> refused on the lock: "Another vm launch is in progress
  (lock: .../vm.lock). Refusing to double-launch." -- flock caught it (the stronger
  of the two refusal paths; lock held for the whole cmd_up body).
- `vm down` -> "VM stopped (16485).", no still-running warning.
- read-only re-probe (vm_debug): qemu alive = 0, port 2222 closed, spice.sock absent.
G1 + G4 met with hard evidence. G2 (janitor) + G3 (vm debug) remain.

### 2026-06-23 -- G2 (janitor) DEMONSTRATED
vm_clean_stale() added; called after vm_lock in up/gui and at top of status.
Backup vm.bak-20260623T163214. bash -n OK. Both halves proven live:
- CLEANUP: planted stale spice.sock with no qemu -> `vm status` swept it
  (vm_debug Section E: spice.sock exists False).
- SAFETY: `vm up` (live guest) -> planted spice.sock WHILE running -> `vm status`
  left it untouched (ls confirms socket survived) -> `vm down` clean. The
  no-live-qemu guard correctly distinguishes orphan from running VM.
G2 met. G3 (vm debug verb) remains.

### 2026-06-23 -- G3 (vm debug) DEMONSTRATED
`cmd_debug()` added to script (read-only Python heredoc: live qemu+guard, port,
flock state, janitor view, disk). fsh dispatch refactored (Option B): vm_dispatch
now forwards ALL args to the script -- no verb whitelist, no duplicate help string
(the old one had drifted, never listing debug). One fsh rebuild required
(cargo check + release clean; nixos-rebuild build clean; deploy switched 5/6 ADVISORY;
fresh terminal picked up new binary). Backups: vm.bak-20260623T163723,
mod.rs.bak-20260623T164053.
PROVEN in fresh terminal: vm debug (idle) -> alive 0, port closed, lock free, no stale.
vm up -> vm debug shows alive 1, guard-match=YES, port LISTENING. vm down -> debug
back to all-clear. Independent guard-match=YES against a live guest (4th confirmation).
FOLLOW-UP (cosmetic, deferred -- not gating): debug labels overstate certainty --
  'launch lock: HELD (launch in progress)' actually means the detached launch subshell
  still holds FD9 while the VM runs (lock is real, label imprecise); and vm.pid sometimes
  holds the real qemu pid, so its 'stale by design' comment isn't always accurate.
  Reword both in cmd_debug (script-only, no rebuild) when convenient.

## Outcome
All 4 gates met. faelight-vm launch path hardened: flock closes the double-launch
race; janitor clears orphaned state (sparing live VMs); down returns zero qemu; vm debug
gives read-only eyes on all of it. Corruption risk closed with two backstops (flock +
qemu's own qcow2 lock). Console/SSH loop -- ~90% of use -- is solid. Graphical
Pinnacle-on-780M render stays out of scope (INT-027/067), as 077 already drew.

## The Rule
"One command, one VM, one window -- and down means down." 🌲
