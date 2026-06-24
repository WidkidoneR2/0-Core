---
id: 088
date: 2026-06-23
type: future
status: planned
title: "Nix Inspector: why did this value win? (option-resolution debugger)"
tags: [nix, debugging, tool, understanding, forest-native]
version: TBD
---
## Why
Biggest Nix pain point: WHY did an option end up this way? Set services.openssh.enable in
three files -- which wins, and why? Nix has the data (module system, priorities); the UX is
missing. "Understanding over convenience" made into a tool. (Christian's idea, 2026-06-23.)
## Vision
  inspect services.openssh.enable
  -> Value: true | Type: bool
     Defined in: hosts/framework16:108 ; modules/security/ssh.nix:12 (mkDefault)
     Winner: framework16:108 (explicit overrides mkDefault)
## How (research first)
- nixos-option already shows definitions + value; the tool is a forest-native, themed,
  readable layer over it (ratatui TUI or fsh builtin). Also: definitionsWithLocations,
  nix eval of the module system. Integration: `inspect <opt>` builtin / `core nix inspect`.
## Phases
P0 research (nixos-option, definitionsWithLocations). P1 MVP (value/type/definitions/winner).
P2 the "why" (mkDefault/mkForce/mkOverride priority). P3 forest UX (themed TUI/builtin).
## Related (captured, NOT this intent)
- Nix Time Machine (gen diff/browse): OVERLAPS INT-074 (gen browser) + INT-044 (done). Fold
  into 074.
- Nix Control Center (GUI -> generate Nix): TENSION with hand-authored philosophy. Capture as
  inspect/visualize, not auto-generate. Separate future.
## The Rule
"If you cannot say why the value won, you do not understand your system." 🌲
