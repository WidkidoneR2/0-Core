---
id: 099
date: 2026-03-03
updated: 2026-03-03
type: future
title: "Niri Migration & faelight-compositor — The Forest Grows Its Own Roots"
status: planned
tags: [compositor, wayland, rust, niri, smithay, faelight-comp, v12, architecture]
version: 12.0.0
depends_on: [098]
---

## Vision

**v10 gave the forest intelligence. v11 gives it discipline. v12 gives it roots.**

From v1 to v10, 0-Core has been built on borrowed substrate — first Hyprland,
then Sway. Both are excellent compositors. Neither is ours.

Every tool in Faelight Forest is understood completely, built intentionally,
and integrated into the system's self-awareness. The compositor has been the
single exception — an opaque dependency at Layer 0 that doesn't know the
forest exists.

That changes with v12.

The goal is not aesthetics. Not switching for the sake of switching.
The goal is philosophical completion:

- 100% Rust across the entire stack
- The compositor as a first-class participant in the event ledger
- A self-aware, event-sourced, capability-gated compositor that feeds
  the orchestrator
- The forest knowing itself all the way down to the display server

---

## The Family Model

The tools of Faelight Forest are not isolated binaries. They are a family.
Each one was built to protect and inform the others:

- `faelight-daemon` watches over the system
- `core` coordinates the family
- `doctor` checks on everyone's health
- `faelight-git` guards the history
- `faelight-bar` shows the family's pulse

`faelight-compositor` is the next sibling — the one who stands at the
boundary between the forest and the outside world. Every application,
every window, every keystroke passes through the compositor first.

Right now that role is filled by Sway — someone else's child.
v12 brings that role home.

---

## The Three Phases

### Phase 1 — Live on Niri (Study by Using)

Before building, understand. Niri is:

- Written in Rust on Smithay — the same foundation faelight-compositor
  will use
- A one-person project that proved a production compositor can be built
  by a single developer on Smithay
- Architecturally modular: core state, layout, animation, input, rendering
  all cleanly separated — the same domain model as 0-Core's engine
- A different mental model: infinite horizontal strip vs discrete workspaces

Migration steps:
```
1. Install Niri alongside Sway (both available at login)
2. Port keybindings from Sway config → Niri config
3. Wire faelight-bar into Niri session
4. Wire faelight-notify, faelight-menu, faelight-launcher into Niri
5. Run Niri as primary session for minimum 30 days
6. Document what works, what doesn't, what's missing
```

Source reading order:
```
1. src/niri.rs        — central state container (maps to engine State)
2. src/layout/        — tiling + workspace logic (the unique part)
3. src/input/         — keyboard/mouse handling
4. src/animation/     — spring physics, bezier curves
5. src/render_helpers/— GPU rendering, damage tracking
```

Niri IPC feeds directly into faelight-daemon:
```
niri msg --json event-stream → EventBroadcast → event ledger
```

Workspace switches, window focus, layout changes — all become
first-class events in the causality engine.

### Phase 2 — Study Smithay + Contribute to Niri

Understanding Niri from the outside is not enough.
Reading the source is not enough.
Contributing forces true comprehension.

Study order:
```
1. Smithay anvil/     — the reference compositor (hello world)
2. cosmic-comp src/   — production reference, same Smithay base
3. Niri src/          — the target model, single-developer proof of concept
```

Contribution goals:
```
- Fix a small bug or improve documentation
- Propose or implement a protocol improvement
- Understand the Smithay event loop deeply enough to replicate it
- Understand the IPC socket architecture
```

This phase has no fixed timeline. It ends when the source is understood
well enough to write faelight-compositor's first lines with confidence.

### Phase 3 — faelight-compositor on Smithay

The compositor that joins the family.

**What makes faelight-compositor different from every other compositor:**

It is not just a display server. It is a participant in the forest's
self-awareness. It knows about the event ledger. It emits structured
events. It integrates with core's capability model.
```
faelight-compositor
  └── Smithay (protocol handling, DRM/KMS, input)
  └── Layout engine (tiling model — informed by Niri study)
  └── Event emitter → faelight-daemon broadcast channel
  └── Capability gate → ControlWM domain in core
  └── Health reporter → doctor compositor check
  └── Config reader → 03-interfaces/compositor/
```

Day one feature set (minimum viable):
```
- Basic tiling (column-based, informed by Niri)
- 5 workspaces with keyboard switching
- Single monitor (AMD laptop)
- Input handling (keyboard + touchpad)
- Lock integration with core-protect
- Event emission: workspace.switch, window.focus, window.open
- doctor health check: compositor state
- faelight-bar compatibility
```

Year one additions:
```
- Floating + tiling hybrid
- Multi-monitor
- Animation (deterministic, declarative, config-driven)
- Full event taxonomy feeding causality engine
- core simulate: workspace topology
- core why: visual attention history
- HiDPI / AMD-specific tuning
```

---

## What faelight-compositor Unlocks for Core

When the compositor joins the family, the event ledger gains a new domain:
```
core why                    # now includes visual topology
core why workspace 3        # why am I on workspace 3?
cew                         # now shows window events live
cdt                         # health trend includes compositor state
core simulate workspace     # predict workspace congestion
```

The causality engine can answer questions it never could before:

- What visual topology correlates with git churn?
- Does focus instability precede health drift?
- When did attention fragment across workspaces?

The forest becomes self-aware all the way to the display server.

---

## The WM Abstraction Layer (Bridge Phase)

Between Sway today and faelight-compositor eventually, the engine
needs a WM abstraction:
```rust
trait WindowManager {
    fn active_workspace(&self) -> u8;
    fn switch_workspace(&self, id: u8);
    fn subscribe_events(&self) -> EventStream;
    fn lock(&self);
    fn focused_window(&self) -> Option<WindowInfo>;
    fn topology(&self) -> WorkspaceTopology;
}
```

Implementations:
- `SwaAdapter` — current, maps swaymsg IPC
- `NiriAdapter` — Phase 1, maps niri msg IPC
- `FaelightAdapter` — Phase 3, native Rust types, no translation

`ControlWM` replaces `ControlSway` in the capability model.
Core never knows which compositor is running.

---

## Philosophy Alignment

This intent does not violate 0-Core philosophy — it completes it.

| Principle | How This Intent Honors It |
|-----------|--------------------------|
| Understanding over convenience | We study before we build |
| Manual over automation | Compositor never acts without explicit command |
| Fail loudly | Compositor failures surface in doctor |
| Design for recovery | Compositor state is checkpointed (Core v4) |
| We control our tools | The compositor is finally ours |
| Explicit structure | Compositor config lives in 03-interfaces/ |

---

## What This Is NOT

- Not switching for aesthetics
- Not chasing Niri because it's new
- Not competing with COSMIC or Hyprland
- Not a rewrite of Sway in Rust
- Not rushed — each phase has no fixed deadline

This is philosophical completion. The last opaque dependency
in Faelight Forest becomes a sibling.

---

## The Smithay Ecosystem Context

Smithay is the foundation of both Niri and COSMIC — the two most
important Rust compositors. By building on Smithay:

- faelight-compositor shares a foundation with production systems
- Improvements to Smithay benefit the whole ecosystem
- The Smithay community becomes a resource
- Niri's developer (Ivan Molodetskikh) is a Smithay contributor —
  collaboration potential is real

---

## Gate Check
```
✅ v10.3.0 released — Core v3 complete
✅ Intent 098 written — Core v4 planned
✅ Sway config fully understood and documented
✅ Philosophy alignment confirmed
✅ Niri installed and configured — Phase 1 start
✅ Core v4 Phase 1 complete — checkpoint foundation
✅ WM abstraction layer implemented
✅ 30 days daily driving Niri
⬜ Smithay anvil studied
⬜ First Niri contribution
```

---

## Stats Context (at time of writing)
```
System:      v10.4.0 — Niri Version
Compositor:  Niri 25.11 (Rust) — Phase 1 active, daily driver
Rust %:      ~95%
Goal:        100% Rust
Tools:       43 aliases, 36 deployed, 42 custom binaries
Health:      95%
Commits:     1314
```

---

## The Version Arc
```
v1-v8    Learning, tools, structure
v9       Production-ready tools, 100% path resilience
v10      Core v2/v3 — self-aware system
v11      Core v4 — reliable, disciplined (INT-098)
v12      faelight-compositor — 100% Rust (this intent)
v13      Faelight Forest — complete self-aware environment
```

---

## The Phrase

**"The tools protect each other like siblings. The compositor is the last
one to come home."**

*"We didn't set out to build a desktop environment.
We set out to understand one. And then we built it anyway."*
