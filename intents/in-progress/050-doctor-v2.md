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
- [x] System Services rescoped to notify-only -- faelight-bar removed from the service array (no business running pre-053); message surfaces "(faelight-bar pending INT-053)" so the deferral shows in the output, not just the code. FINDING: bar's unit is a hand-placed 233-byte file at ~/.config/systemd/user/faelight-bar.service, NOT in the flake -- zero Nix-managed systemd user services exist. notify's Nix-native service + graphical-session autostart deferred to INT-053/Fridayd (053 rebuilds bar ground-up; Mango owns session integration), so wiring it now is throwaway scaffolding. Honest WARN (notify down) stays a true signal until then.

## Gates
- [x] Boot health check: kernel errors since last boot
- [x] Boot time check: systemd-analyze warn > 15s
- [x] Generation drift detection: current vs booted system link; WARN if rebuilt-not-rebooted (verified live: PASS on gen 138 booted==current; WARN branch fires on next rebuild)
- [x] Generation count: age-aware -- WARN only on generations older than 14d (prunable); PASS at 138/none-old, agrees with a live nix-collect-garbage that pruned 0
- [x] flake.lock age warning: > 30 days (stat flake.lock mtime; PASS at 2 days, fix=nix flake update)
- [x] Update Readiness: synthesis go/no-go -- WARN unless booted==current AND tree clean; lists exactly what to fix (verified: held off on a live dirty tree)
- [x] Nix store size: SUM(narSize) from Nix DB in-process (rusqlite RO, ~160ms, no fs walk) + statvfs disk %. WARN > 250 GiB (50GB draft was ~2% of the 3.6TB disk -- cry-wolf; 250 is a rare actionable GC nudge). Always shows size+%. Verified PASS: 57.4 GiB (1.5% of 3.6 TiB)
- [x] VM state check: no accidental running VMs (pgrep -f -c qemu-system; PASS-with-count -- VM-first dev means running VMs are normal, so reported not warned; verified PASS at 0 VMs, count path surfaces on next VM up)
- [x] Compositor detection: shows mango/pinnacle/niri (pgrep -x; reports first running -- verified "MangoWM running"; none-detected is PASS-info for TTY/headless, not a WARN)
- [x] Friday health: patterns + facts + avg confidence from state.db (same tables as footer; matched 13 patterns / 480 facts live). WARN on stall only (patterns < 10, or no new fact > 7d); confidence shown for trend, NOT warned (low confidence = honest uncertainty). Verified PASS: 13 patterns, 480 facts, 0.92 conf
- [x] Network: TCP connect 1.1.1.1:443 (1s cap, no DNS) + DNS resolve github.com in a 1s-bounded worker thread. WARN on offline / DNS-down (offline is not the normal workstation state, so not cry-wolf). Bounded so a dead network cannot blow the 2s gate. Verified PASS: Online -- DNS resolving
- [x] All existing 22 checks preserved and passing (33 total, all rendered -- Sandbox surfaced, Archaeology orphan dropped; rendered rows == counted total; originals all PASS on deployed binary)
- [x] Total check time under 2 seconds (deployed release binary: 732ms for all 33 checks incl. 2 DB reads + bounded network probe; debug ~1s)

## The Rule
"The forest knows its own health.
 Not just files and symlinks --
 boot state, generation drift, VM safety,
 compositor status, Friday learning.
 d answers all of it." 🌲
