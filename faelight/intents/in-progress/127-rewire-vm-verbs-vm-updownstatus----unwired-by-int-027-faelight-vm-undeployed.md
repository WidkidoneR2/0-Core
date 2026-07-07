---
id: 127
date: 2026-07-07
type: future
title: "Rewire vm verbs (vm up/down/status) -- unwired by INT-027, faelight-vm undeployed"
status: in-progress
tags: [vm, libvirt, virsh, int-027, regression]
---

## Root Cause (git-confirmed 2026-07-07)
`vm` failed with "No such file or directory (os error 2)". Not the VM -- the SCRIPT the
dispatcher calls. `vm_dispatch` (faelight-shell/src/commands/mod.rs:8948) hardcodes:
    format!("{}/0-core/pkgs/faelight/scripts/vm", home)
INT-061 Phase 5 (commit 66d0f82d) moved the whole tree: `{pkgs => faelight/packages}/`.
A pure rename (git shows 0 content change) -- the script is intact and hardened at
`faelight/packages/faelight/scripts/vm` (executable, 12k). INT-061 repointed the config.fsh
deploy aliases and flake.nix src paths, but MISSED this raw path string buried in Rust.
That single stale string is the entire bug.

## The VM (confirmed healthy, not broken)
- Target: `faelight-vm` (flake build-vm, tuigreet mode -- mirrors hosts/framework16).
  Chosen deliberately: the VM must match the real system (tuigreet, NOT regreet) for
  recovery testing to be valid. `faelight-vm-regreet` stays a separate future experiment.
- Script is the INT-077 build/up/ssh/down/gui/status loop, hardened by INT-079
  (flock double-launch guard, stale-state janitor, `vm debug`). All preserved.

## Fix
Update the one path string in vm_dispatch:
  pkgs/faelight/scripts/vm  ->  faelight/packages/faelight/scripts/vm
Build fsh, deploy, reload. No other change.

## Gates
- [ ] vm_dispatch path string corrected to faelight/packages/faelight/scripts/vm
- [ ] faelight-shell builds clean, zero warnings
- [ ] Deployed (rebuild + reload); running new binary (hash-verified)
- [ ] Live: `vm status` reports faelight-vm state (NOT "No such file")
- [ ] Live: `vm up` -> `vm ssh hostname` returns `faelight-vm` -> `vm down` clean
- [ ] `d` before and after

## Out of scope (recorded, not fixed here)
- The dead-code libvirt nixos-lab handlers (vm_status/snapshot/... #[allow(dead_code)])
  stay as-is -- they belong to INT-027, still planned.
- SYSTEMIC LINK: this bug is a textbook INT-115 case (a hardcoded path string that broke
  when the tree moved). The real long-term fix is routing this through paths.rs. 115 owns
  that; 127 only corrects the immediate string. Recorded so the connection isn't lost.

## The Rule
"A moved file is not a lost file -- but a hardcoded path forgets. Route it, or it breaks
 again the next time the forest rearranges itself." 🌲
