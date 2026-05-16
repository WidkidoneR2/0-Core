---
id: 307
title: "power-profiles-daemon + Friday integration -- intelligent power management"
status: planned
date: 2026-05-15
type: feature
tags: [power, friday, amd, framework, daemon, intelligence]
depends_on: [246]
---
## The Vision
Friday knows what you are doing.
Friday should know how much power you need.

Compiling a Rust workspace on battery?
Friday switches to performance automatically.
Session idle for 30 minutes on battery?
Friday switches to power-saver.
Plugged in and building a release binary?
Friday ensures performance mode is active.

---
## Hardware
Framework 16 AMD Ryzen 7040
power-profiles-daemon with amd_pstate driver
Three profiles: performance, balanced, power-saver
D-Bus interface: net.hadess.PowerProfiles

---
## Friday Integration Points
### Signal-based switching
Friday observes: cargo build --release detected
Friday acts: switch to performance if on battery
Friday restores: balanced after build completes

### Battery awareness
On battery + idle > 30min → power-saver
On battery + compilation → performance
Plugged in → balanced or performance

### core command integration
core power status -- show current profile + battery
core power set <profile> -- manual override
core power auto -- re-enable Friday control

---
## Implementation
Phase 1 -- D-Bus control
  powerprofilesctl set <profile> from Rust via D-Bus
  Read current profile and battery status
  Gate: core power set performance works

Phase 2 -- Friday awareness
  Friday pattern: detect cargo build, helix, yazi sessions
  Friday decides: appropriate power profile per context
  Gate: Friday switches profile on compile detection

Phase 3 -- faelight-bar integration
  Show current power profile in bar right zone
  Icon: ⚡ performance, ⚖ balanced, 🍃 power-saver
  Gate: profile visible in faelight-bar

---
## Gates
- [x] power-profiles-daemon installed and running 2026-05-15
- [x] amd_pstate driver active — all 3 profiles working
- [ ] core power command implemented
- [ ] Friday detects compilation and switches profile
- [ ] Battery state read from D-Bus
- [ ] faelight-bar shows current profile
- [ ] Friday auto-switching validated over 1 week

---
"The forest knows when to sprint.
The forest knows when to rest.
Power is not wasted.
Power is intentional." 🌲
