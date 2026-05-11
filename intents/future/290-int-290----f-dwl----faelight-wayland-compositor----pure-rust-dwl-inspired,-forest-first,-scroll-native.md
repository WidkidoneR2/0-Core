---
id: 290
title: "F-DWL -- Faelight Wayland Compositor -- pure Rust, forest-first, scroll-native"
status: planned
date: 2026-05-11
tags: [compositor, wayland, rust, smithay, f-dwl, dwl, dwm, suckless, niri, spatial, forest]
---
Most window managers are tools the user configures.
F-DWL is different.
F-DWL is a surface the forest drives.
dwm and dwl proved that a compositor can be small, understandable,
and powerful without being complex.
Suckless philosophy: simplicity, clarity, source code as config.
Faelight philosophy: understanding over convenience, forest as authority.
These are the same philosophy.
F-DWL merges them.
---
WHY BUILD A COMPOSITOR
Every compositor in existence was built compositor-first.
The user configures the compositor.
The compositor manages windows.
F-DWL is built forest-first.
The shell defines intent.
The compositor reflects state.
Windows are surfaces for the forest, not the other way around.
No compositor today is built around:
  an intelligent shell as the authority
  a spatial model where position carries meaning
  a state database as the single source of truth
  an AI layer that knows what you are working on
F-DWL is the first.
---
INSPIRATION: DWM AND DWL
dwm (suckless.org):
  Written in C, under 2000 lines
  Source code IS the config
  Tiling by default, floating when needed
  Tag system instead of workspaces
  Minimal, fast, understood completely by the author
dwl (Wayland port of dwm):
  dwm philosophy on Wayland
  Uses wlroots (C)
  Tiny codebase
  Keyboard-driven
  No status bar by default -- you add what you need
F-DWL takes this philosophy and adds:
  Pure Rust (no C, no wlroots)
  smithay as the Wayland foundation
  Infinite horizontal scroll (Niri model, INT-289)
  Forest state awareness (state.db integration)
  fsh as the authority layer
  Friday as the intelligence layer
---
ARCHITECTURE
F-DWL is NOT a fork of Niri.
F-DWL is NOT a port of dwl.
F-DWL is a new compositor built from smithay up,
inspired by both, designed for Faelight Forest.
Stack:
  smithay -- Wayland compositor library (pure Rust)
  wgpu -- GPU rendering (same as faelight-term v3)
  cosmic-text -- text rendering (same stack throughout)
  calloop -- event loop (same as Niri, faelight-term)
  state.db -- forest state integration
  zbus -- desktop integration (INT-287)
Window model:
  Infinite horizontal scroll strip (INT-289)
  Windows tile within the strip
  Scroll position is persistent and meaningful
  Each workspace region has a name and intent context
  The forest remembers where everything was
Layout:
  Default: master + stack (dwm-style)
  Alt: monocle (full screen, cycle through)
  Alt: scroll-strip (Niri-style infinite)
  Keyboard-driven layout switching
  No mouse required (but mouse supported)
Config:
  Rust source file (suckless-style -- recompile to configure)
  OR: kdl config file (Niri-style, forest-preferred)
  Decision: kdl with sensible defaults, source for deep customization
---
RELATIONSHIP TO EXISTING INTENTS
INT-138 (faelight-compositor v2 -- EGL/OpenGL):
  Phase 1 of F-DWL study -- rendering foundation
  EGL experience feeds directly into F-DWL renderer
  Complete INT-138 first
INT-287 (COSMIC Direction):
  cosmic-comp study teaches smithay architecture
  F-DWL learns from cosmic-comp patterns
  Complete INT-287 study phase first
INT-286 (faelight-term v3):
  wgpu rendering experience from term v3
  Same wgpu stack used in F-DWL renderer
  Complete INT-286 first
INT-289 (Scroll-Native Desktop UX):
  F-DWL is the compositor that makes INT-289 real
  Infinite scroll is built into F-DWL from day one
  INT-289 spatial graph lives in state.db -- F-DWL reads it
INT-280/281 (faelight-fm, Forest Explorer):
  These tools run inside F-DWL
  They benefit from F-DWL spatial awareness
  Build them before or alongside F-DWL
---
SUCKLESS INSPIRATION
dwm principles worth keeping:
  Under 2000 lines of code for the core
  No feature creep -- one thing does one thing
  Keyboard-driven by default
  Patches for optional features (not built in)
  The author understands every line
F-DWL adaptations:
  Rust modules instead of C patches
  kdl config instead of config.h recompile
  Forest integration instead of status scripts
  Friday instead of dwmstatus
The suckless tools that could eventually be reimplemented
as Faelight Forest tools:
  dmenu -> faelight-palette (already done)
  st -> faelight-term (in progress)
  dwm/dwl -> F-DWL (this intent)
  tabbed -> F-DWL pane system
  slock -> faelight-lock (already done)
---
DEPENDENCIES (MUST COMPLETE FIRST)
[ ] INT-287 -- COSMIC study -- smithay patterns understood
[ ] INT-286 -- faelight-term v3 -- wgpu rendering experience
[ ] INT-289 -- Scroll-Native UX -- spatial model defined
[ ] INT-138 -- faelight-compositor v2 -- rendering spike
[ ] INT-280 -- faelight-fm v2 -- tool to run inside F-DWL
[ ] INT-284 -- faelight-term rendering bugs -- term must be solid
---
BUILD PHASES
Phase 0 -- Study (prerequisite, runs with INT-287)
  Read smithay source: backend, compositor, shell protocols
  Read cosmic-comp: how smithay is used in production
  Read Niri source: the scroll model implementation
  Read dwl source: the simplicity model
  Gate: architecture document written, key decisions made
Phase 1 -- First window (milestone)
  smithay compositor renders one window on screen
  Keyboard input works
  Window can be moved and resized
  Gate: a terminal window appears and is usable
Phase 2 -- Tiling
  Master/stack layout
  Keyboard-driven window navigation (hjkl)
  Window focus management
  Gate: faelight-term tiles correctly in master/stack
Phase 3 -- Scroll strip (the innovation)
  Infinite horizontal scroll model
  Windows placed along the strip
  Super+Left/Right navigates the strip
  Scroll position persists between sessions (state.db)
  Gate: navigate 5 windows spatially, position remembered
Phase 4 -- Forest integration
  state.db spatial graph populated by F-DWL
  fsh knows which workspace region is active
  Friday reads spatial context from F-DWL
  faelight-bar shows position in the scroll strip
  Gate: fsh prompt shows spatial context from F-DWL
Phase 5 -- Layer shell (panels, notifications)
  faelight-bar runs as a layer-shell surface
  faelight-lock integrates with F-DWL session lock
  Notifications appear correctly
  Gate: full faelight-bar experience in F-DWL
Phase 6 -- Replace Niri
  F-DWL becomes the daily driver
  All keybindings migrated from Niri config
  All autostart tools work in F-DWL
  Gate: 1 week daily driver without Niri fallback
---
GATES
[ ] Phase 0: architecture document complete
[ ] Phase 1: first window renders
[ ] Phase 2: tiling layout works
[ ] Phase 3: scroll strip navigable, position persists
[ ] Phase 4: forest integration live -- fsh knows where you are
[ ] Phase 5: faelight-bar and faelight-lock work in F-DWL
[ ] Phase 6: Niri replaced, daily driver for 1 week
Final:
[ ] F-DWL is the Faelight Forest compositor
[ ] Under 5000 lines of core compositor code
[ ] Every line understood by the author
[ ] The suckless philosophy lives in Rust
[ ] Scroll-native desktop UX is real
---
TIMELINE
Post-summer 2026.
After INT-286, INT-287, INT-289 are complete.
After faelight-term v3 is the daily driver.
After COSMIC study informs the architecture.
This is a 6-12 month project.
It is not rushed.
It is not started before the foundation is solid.
The forest builds what it needs, when it is ready.
---
NOTE ON F-DWL vs F-DWM
F-DWM would imply X11 (dwm is X11).
F-DWL is Wayland-native (dwl is the Wayland port).
Faelight Forest is Wayland-first.
The name is F-DWL.
If a future version supports XWayland for legacy apps,
it is still F-DWL -- XWayland is a compatibility layer,
not the core model.
---
"dwm asked: what is the smallest a window manager can be?
Faelight Forest asks: what is the most intentional?
F-DWL answers both questions at once.
Under 5000 lines.
Pure Rust.
Forest-first.
Scroll-native.
The compositor the forest deserves." 🌲
