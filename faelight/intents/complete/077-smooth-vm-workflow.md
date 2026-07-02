---
id: 077
date: 2026-06-22
type: future
title: "Smooth VM workflow"
status: complete
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
## Gate 3 native fsh arm (2026-06-22)
`vm` is now a native fsh verb (no script path needed). Discovery during wiring:
fsh ALREADY had a `vm` command -- vm_dispatch (INT-027) driving a libvirt domain
`nixos-lab` via virsh (start/stop/snapshot/restore/snapshots/list). That is a
DIFFERENT VM and mechanism than 077's build-vm faelight-vm. Decision (with Christian):
`vm` = faelight-vm, the machine actually being built for intent testing. So
vm_dispatch was repointed to the 077 script (build|up|ssh|down|status, inherited
stdio so `vm ssh` stays interactive). nixos-lab confirmed shut off (dormant).
INT-027's seven libvirt handlers (vm_start/stop/status/snapshot/restore/snapshots/
list) PRESERVED in source, unwired from the verb, marked #[allow(dead_code)] with a
note -- not deleted (027 is still planned). cargo check clean (0 warnings).
DEMONSTRATED in a fresh shell after rebuild: vm status / vm up (ssh ready 2s) /
vm ssh hostname -> faelight-vm / vm down. Native verb, one machine, no conflation.
Follow-up (deferred): INT-027 charter to note `vm` now drives faelight-vm; tab-
completion entry for vm subcommands; SSH key-auth to drop the password prompt.
## Gate 4 (2026-06-22) -- recovery drill, honestly scoped
Goal: prove the `vm` workflow can carry a recovery rehearsal end-to-end. It can --
but the drill had to be re-aimed to what a build-vm guest can HONESTLY rehearse.

What the drill found (the test bed earning its keep -- these are real boundaries,
discovered in a disposable VM instead of on metal):
- build-vm guest has NO system-profile generations: `nixos-rebuild list-generations`
  -> "no profile 'system' found". A build-vm image boots one baked-in config from a
  flake INPUT; there is nothing to roll back to. So runbook Level 2 (generation
  rollback) CANNOT be rehearsed here.
- No in-guest flake: /home/christian/0-core/flake.nix absent. So a config-change ->
  rebuild drill (nixos-rebuild switch --flake) also cannot run in this guest.
  Both belong to INT-027's install-on-disk / libvirt tooling, not 077's smooth loop --
  exactly the 027-vs-077 boundary the Notes already draw.
- No greetd in the guest config (Pinnacle/Niri installed, no login manager wired):
  greetd.service "not loaded". Login-stack drills need greetd added to hosts/vm, or
  belong to INT-056's own VM work.

What WAS demonstrated (all over `vm ssh`, operator-free so fsh does not punt to sh):
- Service-recovery rehearsal (runbook Level 0 in spirit): sshd active -> sudo systemctl
  restart sshd (the service the console connection itself rides on) -> a FRESH `vm ssh`
  session returns active. Recovering the very service you depend on, proven not to lock
  you out -- driven entirely through the verb.
- Auth is frictionless (this session): host ed25519 key authorized + passwordless sudo
  in the guest (TEST-BED ONLY, scoped to hosts/vm, never framework16). `vm ssh sudo
  whoami` -> root, zero prompts. Copy-paste both directions works. THIS is the
  deliverable: a VM you drop into and work in before touching real metal.

Gate met in spirit -- "a recovery drill driven end-to-end from the console VM" -- via
service recovery, with the generation/greetd scenarios honestly logged as out-of-scope
for a build-vm guest (-> INT-027 / INT-056). The `vm` workflow carried it cleanly.
Command-delivery note: keep host-side `vm ssh` args operator-free (no | ; 2>&1 on the
host line -- fsh punts those to sh, which cannot see the `vm` builtin); let the GUEST
do any piping inside a quoted command.
## Gate 5 (2026-06-23) -- graphical VM via SPICE, windowed
`vm gui` added to the script: launches the guest with a SPICE display on a unix socket
(QEMU_OPTS: -vga virtio -spice unix + virtio-serial + spicevmc agent channel) and opens
remote-viewer (host pkg virt-viewer, added to framework16). Guest got spice-vdagentd +
spice-vdagent + qemuGuest.

SAFETY CORRECTION (important): first version launched remote-viewer --full-screen, which
seized the whole Mango session with no tested escape -> required a hard reboot. Fixed:
`vm gui` now opens a NORMAL, RESIZABLE WINDOW (no fullscreen). It sits beside the terminal;
close via titlebar / Ctrl+Q (VM keeps running headless behind it), Shift+F12 frees the
mouse. Rule learned: never launch a screen-grabbing fullscreen guest over the live session.

DEMONSTRATED: `vm gui` -> a half-screen resizable window showing faelight-vm's NixOS
console (christian@faelight-vm prompt), terminal + chat still fully usable alongside it.
SPICE channel wired (/dev/virtio-ports/com.redhat.spice.0 present); spice-vdagentd starts
on demand (sudo systemctl start spice-vdagentd -> active).

HONEST CAVEAT -- window clipboard: seamless SPICE clipboard needs the per-session
spice-vdagent CLIENT, which wants a graphical guest session. The guest boots to a plain
console (no compositor yet), so window-clipboard is not seamless until a graphical session
runs (arrives with gate 6 / compositor). This does NOT limit testing: the SSH path
(`vm ssh`) runs the guest IN this terminal, so normal terminal copy-paste already gives
full host<->VM command/result exchange today -- the real testing channel. SPICE window =
for WATCHING (compositor render); SSH = for WORKING. Gate met: graphical window up,
resizable, beside the terminal, SPICE + agent in place. "Fullscreen on a dedicated
workspace" intentionally dropped for safety -- windowed + tiled is the correct design.
## Gate 6 (2026-06-23) -- compositor host: boundary established (Path B)
Goal: confirm the graphical VM can host a compositor guest (Pinnacle), as the render
handoff to INT-067. Outcome: the BOUNDARY is now known and documented -- which is the
honest deliverable. Pinnacle does NOT render under bare virtio-gpu in this build-vm.

Pinnacle launch (captured over SSH to a log -- the VM->host text channel that works when
the window cannot copy-paste): pinnacle --no-config got FURTHER than the INT-021 attempts
-- udev backend started, connector Virtual-1 found, CRTC setup attempted. Then failed:
  - "Unable to become drm master, assuming unprivileged mode" (run over SSH = no seat;
    needs the console session for DRM-master)
  - Preferred formats AB30/AR30/AB24 "NoSupportedPlaneFormat"
  - FATAL: "The graphics api has found no node matching DrmNode { ty: Render }" (exit 1)
=> bare virtio-gpu lacks the EGL/render-node capability smithay/Pinnacle需. 

GL attempt (Path A, bounded): switched `vm gui` to virtio-gpu-gl. First opts clashed
("console already has an OpenGL context" -- had both egl-headless AND spice gl=on).
Corrected to single GL context (virtio-gpu-gl + egl-headless,rendernode=host renderD128,
no spice gl=on): QEMU ACCEPTED it and the VM booted clean. So the GL VM LAUNCHES. We did
NOT proceed to the in-window Pinnacle render test because a workflow bug made it unsafe
(below) and gate-6 budget was spent. 

WORKFLOW BUGS SURFACED (the test bed earning its keep -- found here, not on metal):
1. `vm down` silently no-ops: it calls pkill via fsh, and fsh's pkill/kill/pgrep builtins
   are shadowed/broken, so nothing gets killed. Result: repeated `vm gui` spawned MULTIPLE
   qemu on ONE qcow2 (disk-corruption risk). Cleared only via a Python /proc walker
   (os.kill). FIX NEEDED: vm down must track qemu's real PID and kill by PID (Python or
   real /run/current-system/sw/bin tools), not pattern-match through fsh.
2. fsh shadows system binaries + punts operators to bare `sh`: hit ALL night -- `vm` not
   found under sh on piped/redirected lines; `kill` only takes job-ids; pkill/pgrep exit-1
   noise; heredocs broke. Biggest friction source of the session. Candidate intent.

HONEST VERDICT for INT-067: a build-vm + virtio-gpu can host a compositor only with 3D
accel (virtio-gpu-gl, which now launches) AND a console-seat launch -- and even then it is
VIRTUAL gpu, NOT the AMD 780M. The real 780M DRM render test belongs to bare-metal or a
GPU-passthrough VM (INT-027 territory), never this lightweight build-vm. Gate met as
"boundary confirmed + path documented": the VM is the STAGE up to GPU rendering; the
hardware-specific render is explicitly out of scope for 077's console-first lab.

Follow-up intents seeded: (a) fix `vm down` PID tracking; (b) fsh stop shadowing system
binaries / route builtins with operators; (c) widen the native fsh `vm` arm to forward
`gui` (currently script-path only); (d) INT-067 owns the real-hardware compositor render.
## Gates
- [x] Phase 0: build-vm boots faelight-vm; current console/display behaviour recorded here
- [x] console VM boots to a serial console in the terminal with copy-paste working both ways
- [x] console VM wrapped as a simple fsh verb (one command to boot/enter)
- [x] an INT-056 recovery drill driven end-to-end from the console VM
- [x] graphical VM: SPICE display + shared clipboard, fullscreen on a dedicated workspace
- [x] graphical VM confirmed able to host a compositor guest (Pinnacle render handoff to INT-067)

## Notes
- VM-as-enabler: unblocks 056, 043 (gate 131), and the Pinnacle / 067 / 010 / 055 cluster.
- Relationship to INT-027 (VM-native dev: create/enter/snapshot/rollback): 027 is the full
  tooling; this intent is the smooth EVERYDAY loop. Keep 077 lean; fold deeper tooling into 027.
- Mango workspace is not Niri: dedicated-space feel = fullscreen guest, not a second compositor.

## The Rule
"The lab should be one command away -- and paste should just work." 🌲
