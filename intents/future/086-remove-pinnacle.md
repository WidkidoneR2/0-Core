---
id: 086
date: 2026-06-23
type: future
status: planned
title: "Remove Pinnacle (installed) -- making room for Miracle"
tags: [decommission, pinnacle, compositor, flake, cleanup]
version: TBD
---
## Why
Pinnacle was the studied candidate (VM-tested, smoke-tested nested in Mango). Decision
(2026-06-23): go Miracle instead (more mature v0.8.3, Sway-IPC, WASM plugins). Pinnacle is
currently INSTALLED/built into the system; remove to make room and avoid two candidates.
## Footprint (scanned 2026-06-23)
- hosts/framework16:116 -- inputs.pinnacle.packages.${system}.pinnacle in systemPackages
- hosts/framework16:139 -- faelight.desktop.pinnacle.enable = true
- flake.nix -- pinnacle as a flake INPUT + the faelight.desktop.pinnacle module
- pkgs.protobuf:117 likely a pinnacle build dep -- VERIFY before removing
- modules/desktop/pinnacle.nix (or wherever faelight.desktop.pinnacle lives)
- registry/docs pinnacle refs
NOTE: full re-scan at start (known surface, not guaranteed complete).
## Approach
Total-scan-first, remove: flake input, module, host enable + systemPackages entry,
protobuf if pinnacle-only, registry/docs. Rebuild clean.
## Sequencing
AFTER INT-085, BEFORE INT-087.
## The Rule
"Studied, considered, not chosen -- a complete answer. Remove it cleanly." 🌲
