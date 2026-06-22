---
id: 077
date: 2026-06-22
type: future
title: "Smooth VM workflow"
status: planned
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

## Gates
- [ ] Phase 0: build-vm boots faelight-vm; current console/display behaviour recorded here
- [ ] console VM boots to a serial console in the terminal with copy-paste working both ways
- [ ] console VM wrapped as a simple fsh verb (one command to boot/enter)
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
