---
id: 049
date: 2026-06-09
type: polish
title: "boot-polish: clean quiet boot, fix display handoff, greetd stability"
status: planned
tags: [boot, plymouth, greetd, display, tuigreet, polish, amd, framework]
priority: medium
---
## Why
Generation 92/93 caused blank screen after LUKS unlock.
The 2026-06-09 tuigreet incident proved live boot changes without VM
pre-flight cost 24 hours of lost work.
This intent fixes boot cosmetics and hardens the boot path.
Nothing lands on the real machine without VM validation first.

## Known Facts
- Generation 91: stable (Plymouth enabled, no quiet params)
- Generation 92: broke (Plymouth + quiet + splash added)
- Plymouth bgrt theme conflicts with AMD Radeon 780M display handoff
- libvirtd onBoot=start was resuming nixos-lab VM -- caused blank screen (FIXED gen 93)
- Ctrl+Alt+F2 did not work during blank screen -- full display lock
- INT-056 must complete first (TTY2 hardening, greetd fallback)

## The Problem Stack
1. Plymouth bgrt reads firmware ACPI logo -- conflicts with AMD KMS handoff
2. quiet + splash suppress output during failure -- no recovery signal
3. No fallback TTY during blank screen -- full lockout
4. greetd appears too slowly after LUKS unlock

## Options (test in VM via INT-024 first)

Option A -- Disable Plymouth entirely (recommended first)
  Remove boot.plymouth.enable
  Use systemd-boot silent mode only
  Cleanest approach, no display handoff risk
  Risk: none -- cosmetic only

Option B -- Replace Plymouth bgrt with text/spinner theme
  bgrt uses ACPI BGRT (firmware logo) -- known AMD conflict
  spinner or text theme avoids the handoff issue
  Risk: low -- theme swap only

Option C -- quiet without Plymouth
  boot.kernelParams = ["quiet"] only
  No splash, no Plymouth
  systemd suppresses most output naturally
  Risk: low

Option D -- Fix AMD KMS handoff
  Add amdgpu.dc=1 or drm.modeset kernel params
  Force KMS earlier in boot sequence
  Risk: medium -- kernel param changes

## Phases

Phase 1 -- INT-056 pre-flight (HARD DEPENDENCY)
  TTY2 hardened, greetd fallback session defined
  Ctrl+Alt+F2 verified working before any boot changes
  Gate: INT-056 Phase 1 and 2 complete

Phase 2 -- VM boot testing (INT-024 required)
  Snapshot VM: before-INT-049
  Test Option A in VM: disable Plymouth
  Boot VM 5 times, verify clean greetd handoff
  Test recovery: intentionally break, verify TTY2 escape
  Gate: clean boot in VM, recovery demonstrated

Phase 3 -- Graduate to real machine
  VM gates passed
  Generation checkpoint before applying
  Apply chosen option to framework16 flake
  Boot and verify: greetd within 3 seconds of LUKS unlock
  Gate: clean boot on Framework 16, no blank screen

Phase 4 -- Boot time optimization
  systemd-analyze to measure boot time
  Identify slow units
  Target: boot to greetd in under 10 seconds
  Gate: systemd-analyze shows < 10s to greetd

## Gates
- [ ] INT-056 Phase 1+2 complete before any boot changes
- [ ] VM snapshot created: before-INT-049
- [ ] Option A (disable Plymouth) tested in VM -- 5 clean boots
- [ ] Recovery from broken boot demonstrated in VM
- [ ] Clean boot on Framework 16 -- no blank screen
- [ ] tuigreet appears within 3 seconds of LUKS unlock
- [ ] Ctrl+Alt+F2 works as emergency escape at all boot stages
- [ ] No white text flash during password entry
- [ ] Boot time under 10 seconds to greetd (systemd-analyze)

## Depends On
- INT-056 (Forest Recovery Protocol) -- MUST complete first
- INT-024 (VM graduation pipeline) -- all changes tested in VM

## The Rule
"The boot screen is cosmetic.
 Stability is priority.
 Test in the VM. Graduate to the machine.
 Never the other way around." 🌲
