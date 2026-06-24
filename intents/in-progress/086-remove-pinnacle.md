---
id: 086
date: 2026-06-23
type: future
status: in-progress
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

## PARKED (2026-06-24) -- fold into the ReGreet login rebuild
Re-scan at 086 start revealed Pinnacle IS wired into greetd: modules/desktop/pinnacle.nix:21
creates environment.etc."greetd/sessions/pinnacle.desktop" (Exec=pinnacle --session) -- a
selectable login session (NOT the default; default stays tuigreet --cmd mango).
DECISION: do NOT remove Pinnacle piecemeal now. The login is being rebuilt with ReGreet
(relates INT-005 / INT-054 / INT-056) -- a VM-gated, login-touching effort. Pinnacle's greetd
session entry should be removed AS PART OF that login rebuild, with the VM safety net in place,
rather than touching greetd twice. Removing it now (without the VM harness, right before a
2-day absence) is the wrong risk/reward.
## Corrected sequencing
085 (Niri removal, DONE) -> [ReGreet login rebuild, VM-gated -- absorbs Pinnacle's greetd
session removal] -> 086 finishes the non-login Pinnacle bits -> 087 (Miracle).
## Full footprint (scanned 2026-06-24) -- ready for execution when unblocked
GREETD (do during login rebuild): modules/desktop/pinnacle.nix:21-24 (greetd session file).
FLAKE INPUT: flake.nix:10-11 (inputs.pinnacle.url + follows), :14 (outputs param `pinnacle`)
  -- removing requires flake.lock reconciliation (like the palette Cargo.lock fix).
MODULE: modules/desktop/pinnacle.nix (whole file -- options + config + greetd session).
HOSTS: framework16:5 (imports pinnacle.nix), :122 (systemPackages pinnacle), :123 (pkgs.protobuf
  -- VERIFY pinnacle-only: protobuf appears ONLY here in nix; rust 'protobuf' matches in
  stress/mod.rs + events/signal.rs need checking but likely not prost/tonic build deps),
  :145 (faelight.desktop.pinnacle.enable = true). hosts/vm:81 (systemPackages pinnacle).
CODE: doctor/checks.rs:1147 (comment), :1152 (("pinnacle","Pinnacle") process detection tuple).
LOGOUT: pkgs/faelight-logout/main.py:40 (PINNACLE_SOCKET branch).
NON-login parts (code/logout/systemPackages/module-minus-greetd/flake-input) COULD be done
independently, but cleanest is one pass during the login rebuild.
## Also reconcile
INT-067 (faelight-bar secondary compositor) assumes "Pinnacle primary, Miracle fallback" --
that premise is now inverted (Pinnacle removed, Miracle is THE second compositor). Update 067
when 087 lands.
