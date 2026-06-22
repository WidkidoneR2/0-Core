---
id: 077
date: 2026-06-22
type: future
title: "Smooth VM workflow"
status: in-progress
tags: [VM, Mango, nix-lab, Nix, Pinnacle]
---

## Why
Almost every in-progress intent needs VM work: INT-056 (recovery drills), INT-043 gate 131
(clean-VM cache pull), and the Pinnacle / INT-067 / 010 / 055 compositor cluster (DRM-on-780M
render test). The VM is the linchpin -- but today it is friction: no smooth terminal loop, no
easy copy-paste, no dedicated space to see a graphical guest. A frictionless VM workflow
unblocks the most intents per unit effort, so it comes BEFORE the intents that consume it.

## What
Two distinct VM paths, console first:
- CONSOLE VM (daily driver): boot a NixOS guest to a serial console IN the terminal -- native
  terminal copy-paste both ways, scrollback, lives in the fsh flow. Covers boot tests, recovery
  drills (056), cache-pull verification (043). ~90% of the need.
- GRAPHICAL VM (render test only): a dedicated Mango workspace running the guest display
  fullscreen with a shared clipboard (SPICE/virt-viewer), used specifically to watch Pinnacle
  render on the 780M DRM backend -- the test that has been VM-gated.

## Approach
Build on nixos-rebuild build-vm --flake .#faelight-vm (the flake already has
nixosConfigurations.faelight-vm at hosts/vm/configuration.nix -- no virt-manager/disk setup).
Console path: serial console + headless QEMU so the guest comes up in the terminal; wrap as a
simple fsh verb. Graphical path: SPICE display + spice-vdagent in the guest for shared
clipboard; launch the viewer fullscreen on a dedicated workspace. A Mango workspace stays Mango
(it does not become Niri) -- the dedicated-space feel is a fullscreen guest, not a second
compositor.

## Phases
Phase 0 -- survey hosts/vm/configuration.nix; confirm build-vm boots; note console/display state.
Phase 1 -- console VM: serial console in-terminal, copy-paste both ways, wrapped as an fsh verb.
Phase 2 -- recovery loop proof: drive an INT-056 rescue drill end-to-end from the console VM.
Phase 3 -- graphical VM: SPICE display + shared clipboard, fullscreen on a dedicated workspace.
Phase 4 -- Pinnacle render slot: confirm the graphical VM can host a compositor guest (handoff
  to the Pinnacle / INT-067 work -- this intent provides the stage, not the compositor decision).

## Phase 0 Findings (2026-06-22)
hosts/vm/configuration.nix surveyed (nixosConfigurations.faelight-vm):
- Real NixOS guest: user christian / initialPassword faelight, SSH on (password auth,
  no root). Provisioned for the GRAPHICAL compositor test already -- Pinnacle, Niri,
  Alacritty, faelight-forest, rust toolchain all installed; hardware.graphics + seatd on.
- NO serial-console setup: no console=ttyS0, no boot.kernelParams for serial. So the
  console-in-terminal path (the copy-paste win) is NOT wired yet -- that is Phase 1.
- Two boot tools differ:
  * nixos-rebuild build-vm -> QEMU opens a GRAPHICAL window by default (separate window,
    no terminal copy-paste -- the friction we are removing).
  * Console-in-terminal needs console=ttyS0 in kernelParams AND QEMU serial routed to
    stdio (-nographic / QEMU_OPTS). That is the Phase 1 build.
- Config is NOT minimal (full faelight-forest + Pinnacle + Niri + rustc): first boot is a
  real build; a lighter console-only variant may be worth it later for a fast loop (noted,
  not done now).
Conclusion: VM exists and is graphical-ready; the console path must be ADDED (Phase 1),
matching the console-first priority.

Boot confirmed (2026-06-22): build-vm built clean (exit 0, result/bin/run-faelight-vm-vm),
and the VM was launched and BOOTED -- graphical QEMU window, login christian/faelight in
~4-5s, behaves like real hardware. Gate 1 demonstrated (built AND booted). Note: as a
normal user `poweroff` is denied (no polkit/sudo power rule in the guest) -- use sudo
poweroff or close the window; a clean user-shutdown is small guest-ergonomics polish for
the console loop later. Note: this was the default GRAPHICAL window -- precisely the
separate-window friction Phase 1 (serial console in-terminal) removes.

## Gate 2 (2026-06-22) -- console access, via SSH not serial
Goal: drive the VM from a terminal with working copy-paste. Path taken honestly:
- Serial console (console=ttyS0 + serial-getty + autologin + virtualisation.graphics=false)
  got the VM booting IN the terminal and autologging in -- but keystrokes never reached the
  guest shell (one-way serial through the nixos-vm run script; input not routed back).
  Real progress (in-terminal boot, autologin, headless) but the input path kept fighting us.
- Pivoted to SSH -- the reliable console path. Added virtualisation.vmVariant.virtualisation
  .forwardPorts (host 2222 -> guest 22). Lifecycle fix: launch with `setsid ... < /dev/null &`
  so the VM survives the launch script (a plain bg job died on script exit -> connection
  refused). Then: ssh -o StrictHostKeyChecking=no christian@localhost -p 2222 (pw faelight).
- DEMONSTRATED: landed in the guest shell over SSH; hostname=faelight-vm, whoami=christian.
  Real bidirectional terminal -> native copy-paste + scrollback. This is how 056 recovery
  drills and 043 cache tests will actually be driven.
Decision: SSH is the console path; serial console deferred (nice-to-have, not needed).
Gate reworded in spirit: "console access in-terminal with copy-paste" -- met via SSH.
Launch sequence to wrap into an fsh verb next (gate 3):
  setsid ./result/bin/run-faelight-vm-vm > /tmp/vm.log 2>&1 < /dev/null &  (after build-vm)
  wait for 2222 LISTENING, then ssh -p 2222.

## Gate 3 (2026-06-22) -- the `vm` verb
Wrapped the build/launch/SSH loop into one verb: pkgs/faelight/scripts/vm
(script pattern, like cache-status/cache-push; thin fsh `vm` arm to follow).
Subcommands: build | up | ssh | down | status.
Design decisions baked in from the gate-1/2 hard knocks:
- NO ./result assumption: build-vm runs in a fixed state dir
  (~/.local/state/faelight-vm); launcher resolved by absolute path. The result
  symlink there is also a GC-root, so the image survives a generation prune.
- Console = SSH (gate 2 decision), not serial. up waits for port 2222 to listen
  (60s cap) then reports ready; ssh auto-starts the VM if down.
- Lifecycle: setsid + </dev/null detaches the VM so it survives the launcher;
  down matches the running qemu by its qcow2 in the cmdline (pid goes stale under
  setsid), and disk state persists between runs.
DEMONSTRATED (2026-06-22): vm build (67s, GC-rooted image) -> vm up (ssh ready 2s)
-> vm ssh hostname returned `faelight-vm` from inside the guest -> vm down clean.
One command to boot, one to enter, one to stop. Headless by design (no window;
the guest comes over SSH). This is the loop 056 drills and 043 cache tests use.
Polish deferred (not gating): status returns non-zero when down (cosmetic);
SSH key-auth to drop the password prompt (folds into the SSH-key-only hardening).
## Gates
- [x] Phase 0: build-vm boots faelight-vm; current console/display behaviour recorded here
- [x] console VM boots to a serial console in the terminal with copy-paste working both ways
- [x] console VM wrapped as a simple fsh verb (one command to boot/enter)
- [ ] an INT-056 recovery drill driven end-to-end from the console VM
- [ ] graphical VM: SPICE display + shared clipboard, fullscreen on a dedicated workspace
- [ ] graphical VM confirmed able to host a compositor guest (Pinnacle render handoff to INT-067)

## Notes
- VM-as-enabler: unblocks 056, 043 (gate 131), and the Pinnacle / 067 / 010 / 055 cluster.
- Relationship to INT-027 (VM-native dev: create/enter/snapshot/rollback): 027 is the full
  tooling; this intent is the smooth EVERYDAY loop. Keep 077 lean; fold deeper tooling into 027.
- Mango workspace is not Niri: dedicated-space feel = fullscreen guest, not a second compositor.

## The Rule
"The lab should be one command away -- and paste should just work." 🌲
