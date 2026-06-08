---
id: 049
date: 2026-06-08
type: feature
title: "boot-polish: clean quiet boot, fix display handoff, greetd stability"
status: planned
tags: [boot, plymouth, greetd, display, tuigreet, polish]
priority: medium
---

## Problem

Generation 92/93 caused blank screen after LUKS unlock.
Root cause: Plymouth + quiet boot params + AMD GPU driver
display handoff to greetd fails on Framework 16.

Two separate bugs:
1. libvirtd onBoot="start" resumed nixos-lab VM -- caused blank screen (FIXED gen 93)
2. Plymouth + quiet splash -- breaks display handoff on AMD Radeon 780M

## Known Facts
- Generation 91 is stable (Plymouth enabled, no quiet params)
- Generation 92 broke (Plymouth + quiet + splash params added)
- Plymouth bgrt theme may conflict with AMD GPU driver
- Ctrl+Alt+F2 did not work during blank screen -- full display lock

## Options to Investigate (in VM first)

### Option A -- Disable Plymouth entirely
- Remove boot.plymouth.enable
- Use systemd-boot silent mode only
- Cleanest approach, no display handoff risk

### Option B -- Replace Plymouth with different theme
- Try Plymouth spinner or text theme instead of bgrt
- bgrt uses ACPI BGRT (firmware logo) which may conflict

### Option C -- Use quiet without Plymouth
- boot.kernelParams = [ "quiet" ] only
- No splash, no Plymouth
- systemd suppresses most output naturally

### Option D -- Keep Plymouth, fix AMD handoff
- Add amdgpu.dc=1 or drm.modeset kernel params
- Force KMS earlier in boot

## Gate
- [ ] Boot tested in VM first -- never directly on real system
- [ ] Clean boot without blank screen
- [ ] tuigreet appears within 3 seconds of LUKS unlock
- [ ] No white text during password entry
- [ ] Ctrl+Alt+F2 works as emergency escape

## Note
Do NOT apply boot changes directly to framework16.
Always test in faelight-vm first.
The boot screen is cosmetic -- stability is priority.
