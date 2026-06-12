---
id: 050
date: 2026-06-09
type: feature
title: "doctor v2: NixOS-aware health checks"
status: in-progress
tags: [doctor, health, nixos, boot, generations, monitoring, friday, compositor]
priority: high
---
## Why
The current health check (d) was built during the Arch era.
It checks 22 things but knows nothing about NixOS specifics:
-- generations, flake drift, store health, boot errors, VM state.
After migrating to NixOS the doctor must be smarter.

d should answer: "Is the forest in a healthy NixOS state?"
Not just "are files in the right place."

## What Already Exists
22 checks across 5 domains: System, Git, Tools, Forest, Security
CheckResult pattern in faelight-core
Health score calculation with weighted checks
Advisory/Warning/Critical thresholds

## New Check Domains to Add

### Boot Health (3 checks)
- Last boot clean: journalctl -b -p err -- count kernel errors
- Boot time: systemd-analyze -- warn if > 15 seconds to greetd
- greetd handoff: did greetd start cleanly last boot

### Generation Health (4 checks)
- Generation drift: current gen matches booted gen (warn if rebuilt but not rebooted)
- Generation count: warn if > 10 generations (disk space)
- flake.lock age: warn if > 30 days since last flake update
- Last rollback: note if rollback performed in last 7 days

### NixOS-Specific (3 checks)
- Nix store size: warn if > 50GB
- GC opportunity: nix-store --gc --print-dead to count dead paths
- Binary cache: warn if PKG_CONFIG_PATH not set in dev shell

### VM State (2 checks)
- No VMs accidentally running: pgrep qemu returns nothing
- nixos-lab disk space: ~/vms/*.qcow2 total size healthy (< 40GB)

### Compositor Health (2 checks)
- Compositor running: detect mango/pinnacle/niri from process list
- faelight-bar registered: pgrep faelight-bar

### Friday Health (3 checks)
- Pattern count: warn if < 10 patterns (Friday not learning)
- Confidence trend: warn if average confidence < 0.7
- Last learning: warn if no new facts in > 7 days

### Network Health (2 checks)
- Internet connectivity: ping 1.1.1.1 with 1s timeout
- DNS resolving: resolve github.com

## Implementation
Each new check follows existing CheckResult pattern in faelight-core.
New domains added to the doctor section of the health output.
Keep fast: all checks must complete in parallel where possible.
Target: d completes in under 2 seconds total (currently ~600ms).

## Phases

Phase 1 -- Boot and generation checks
  Add Boot Health and Generation Health domains
  Gate: d shows boot errors and generation drift

Phase 2 -- NixOS-specific checks
  Add Nix store, GC, flake.lock checks
  Gate: d warns when flake.lock > 30 days old

Phase 3 -- VM and compositor checks
  Add VM State and Compositor Health domains
  Gate: d shows which compositor is running

Phase 4 -- Friday and network checks
  Add Friday Health and Network Health domains
  Gate: d shows Friday pattern count and confidence trend

Phase 5 -- Performance
  Parallelize checks where safe
  Gate: d completes in under 2 seconds with all new checks

## Phase 0 Gates (cleanup -- honest existing checks, reshape 2026-06-11)
- [x] Core Protection check removed (retired with INT-025; LUKS covers at-rest)
- [x] Broot check un-orphaned (cockpit name mismatch had hidden it)
- [x] System Services fix advice de-Arched (systemd user services, not & backgrounding)
- [x] keybinds check compositor-aware: reads mango config first, niri fallback, shows compositor name (full pgrep detection -> gate 7)
- [x] duplicate check roster collapsed into all_checks() (single source of truth); cockpit dashboard renders clean
- [x] every rendered check is truthful -- no phantom/orphaned/dead/Arch-era checks; System Services 0/2 verified honest (pgrep -f detects running svcs; bar->INT-053, notify->Fridayd)

## Gates
- [ ] Boot health check: kernel errors since last boot
- [ ] Boot time check: systemd-analyze warn > 15s
- [x] Generation drift detection: current vs booted system link; WARN if rebuilt-not-rebooted (verified live: PASS on gen 138 booted==current; WARN branch fires on next rebuild)
- [x] Generation count: age-aware -- WARN only on generations older than 14d (prunable); PASS at 138/none-old, agrees with a live nix-collect-garbage that pruned 0
- [x] flake.lock age warning: > 30 days (stat flake.lock mtime; PASS at 2 days, fix=nix flake update)
- [ ] Nix store size warning: > 50GB
- [ ] VM state check: no accidental running VMs
- [ ] Compositor detection: shows mango/pinnacle/niri
- [ ] Friday pattern count and confidence trend
- [ ] Network connectivity check
- [ ] All existing 22 checks preserved and passing
- [ ] Total check time under 2 seconds

## The Rule
"The forest knows its own health.
 Not just files and symlinks --
 boot state, generation drift, VM safety,
 compositor status, Friday learning.
 d answers all of it." 🌲
