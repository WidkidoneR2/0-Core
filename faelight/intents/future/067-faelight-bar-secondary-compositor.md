---
id: 067
date: 2026-06-17
type: feature
title: "faelight-bar under the secondary compositor (Pinnacle primary, MiracleWM fallback)"
status: planned
tags: [bar, wayland, layer-shell, compositor, pinnacle, miraclewm, workspaces, ipc]
priority: medium
---
## Why
Split out of INT-053. faelight-bar v2 is demonstrated under MangoWM (dwl-ipc).
The forest runs a deliberate two-compositor design -- a primary/secondary split
for a change of scenery while working: Pinnacle as the secondary daily driver for
Rust projects, Mango retained for building Friday once the planned intents land.
The bar must therefore render under the secondary compositor too. That work is
blocked on the secondary compositor actually standing up as a session, so it does
not belong in 053.

## Decision -- 2026-06-17: Pinnacle primary, MiracleWM fallback
Pinnacle is the target secondary compositor:
  -- Rust-native (Smithay), on-brand for a ~98%-Rust forest; same family as Niri.
  -- Lua/Rust gRPC config (AwesomeWM-style), programmable.
  -- Already installed and nested-smoke-tested on this NixOS box (EGL works); the
     open question is real-DRM/login behaviour on the 780M.
If Pinnacle will not hold as a real session on the hardware, uninstall it and
stand up MiracleWM as the secondary instead:
  -- Mir-based (Canonical), more battle-tested real-hardware DRM than a solo
     Smithay project -- the maturity hedge.
  -- v0.9 (Apr 2026): WebAssembly plugin system + Rust plugin API, cursor themes.
  -- i3/Sway-compatible IPC (swaymsg/Waybar) -- a cleaner workspace source than
     dwl-ipc, and well documented.
  -- Trade-off: core is C++/Mir (only plugins are Rust); Mir-on-NixOS is unproven,
     heavier packaging.

## Bar portability (already designed for this)
The bar is compositor-agnostic: it reads ~/.cache/faelight/workspaces as JSON and
knows nothing of the compositor. Per INT-053's 2026-06-17 dwl-ipc decision, a new
compositor just needs a side helper writing that same JSON:
  -- Pinnacle: an ext-workspace-v1 (or Pinnacle API) helper.
  -- MiracleWM: a sway-IPC helper (swaymsg) -- the easier of the two.
The bar itself does not change.

## Pre-flight -- INT-056 (this one DOES apply)
Unlike the bar (a client surface), switching the session compositor is a
login-surface change with real lockout risk. INT-056 pre-flight applies here:
VM-test via INT-024 first, snapshot, TTY2 verified, greetd fallback, recovery
demonstrated -- before it lands on the real machine. Sequenced after INT-024.

## Gates
- [ ] secondary compositor stands up as a real session on the 780M (Pinnacle via INT-038, or MiracleWM)
- [ ] faelight-bar renders anchored top under the secondary compositor, no crash
- [ ] workspace indicators live under the secondary compositor (helper writes the same JSON the bar reads)
- [ ] INT-056 pre-flight passed in the VM for the compositor switch

## Depends On
- INT-038 (Pinnacle Lua config) -- stands up Pinnacle as a session
- INT-024 (Forest R&D VM) -- pre-flight test bed
- INT-056 (Forest Recovery Protocol) -- pre-flight gate for the login surface
- INT-053 (faelight-bar v2) -- source of the moved Pinnacle/workspace gate

## The Rule
"The bar follows the forest, not the compositor.
 Whichever window manager paints the screen, the pulse reads the same." 🌲
