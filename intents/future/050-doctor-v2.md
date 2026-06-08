---
id: 050
date: 2026-06-08
type: feature
title: "doctor v2: NixOS-aware health checks"
status: planned
tags: [doctor, health, nixos, boot, generations, monitoring]
priority: high
---

## Why

The current health check (d) was built during Arch era.
It checks 22 things but knows nothing about NixOS specifics.
After migrating to NixOS the doctor needs to be smarter.

## Vision

`d` should answer: "Is the forest in a healthy NixOS state?"
Not just "are files in the right place."

## New Checks to Add

### Boot Health
- Last boot clean? (journalctl -b -p err)
- Boot time (systemd-analyze)
- LUKS unlock succeeded cleanly
- Plymouth/greetd handoff worked
- Kernel errors since last boot

### Generation Health
- Current generation matches booted generation?
- Generation drift warning (rebuilt but not rebooted)
- How many generations exist (disk space)
- Last rollback recorded

### NixOS-Specific
- flake.lock age (warn if > 30 days)
- Nix store size and fragmentation
- Garbage collection opportunity
- Binary cache hit rate

### VM State
- No VMs accidentally running
- nixos-lab disk space healthy
- VM configs match flake

### Compositor Health
- Which compositor running (niri/pinnacle/mango)
- Layer shell services active
- faelight-bar registered with compositor

### Friday Health
- Friday patterns count and quality
- Confidence scores trending up/down
- Last learning session
- Prediction accuracy

### Network Health
- Internet connectivity
- DNS resolving correctly
- SSH keys valid

## Implementation

Build on top of existing doctor infrastructure.
Add NixOS domain to doctor checks.
Each new check follows existing CheckResult pattern.
Keep fast -- doctor must complete in < 1 second.

## Gate
- [ ] Boot health check added
- [ ] Generation drift detection working
- [ ] flake.lock age warning
- [ ] VM state check
- [ ] Friday health summary
- [ ] All existing 22 checks preserved
- [ ] Completes in under 1 second
