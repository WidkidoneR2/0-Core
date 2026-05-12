---
id: 287
title: "Faelight Forest COSMIC Direction -- fsh as authority, COSMIC as visual layer"
status: in-progress
date: 2026-05-08
tags: [cosmic, architecture, fsh, shell, direction, vision, rust, wayland, niri]
---
This is not a migration intent.
This is an architectural direction intent.
Faelight Forest stays on Niri.
fsh stays as the operational brain.
state.db stays as the single source of truth.
What changes: we study COSMIC deeply, borrow selectively,
and build toward a world where the shell defines reality
and the visual layer reflects it.

Most desktops assume GUI is primary, terminal is secondary.
Faelight Forest inverts this.
The shell owns state. The visual layer reflects state.
COSMIC is modern enough to tolerate this inversion.

---
THE ARCHITECTURE

fsh + core + Friday
    down (state authority)
state.db -- single source of truth
    down (reflection)
COSMIC visual layer -- workspace, notifications, surfaces
    down (surfaces)
faelight-bar, dashboards, integrity views, ledger navigation

The shell is not a tool inside the desktop.
The desktop is a surface for the shell.

---
PROGRESS AS OF 2026-05-12

Phase 3 is COMPLETE. cosmic-term source was studied and directly
informed faelight-term v3. The wgpu + cosmic-text + glyphon stack
is now proven in production on AMD RX 7700S (RADV NAVI33).

faelight-term v3 IS the COSMIC proof of concept:
  - wgpu (Vulkan) rendering -- GPU-accelerated
  - cosmic-text for font shaping -- Unicode correct, emoji correct
  - glyphon for per-cell spans -- ANSI color pipeline
  - alacritty_terminal ring buffer -- correct PTY state
  - Running as daily driver -- Super+Enter opens v3

POP_OS rep has noted the COSMIC integration progress.
Graydon Hoare has expressed interest in the project.
The story is being told through working code, not slides.

---
WHAT WE BORROW (SELECTIVE) -- STATUS

FROM cosmic-term:
  [x] wgpu + cosmic-text integration -- DONE (faelight-term v3)
  [x] PTY management patterns -- DONE (alacritty_terminal)
  [x] damage tracking -- DONE (dirty flag + 60fps loop)

FROM cosmic-comp:
  [ ] smithay state ownership patterns -- study planned
  [ ] workspace management ideas -- future
  [ ] IPC boundary design -- INT-294 (zbus event bus)

FROM libcosmic:
  [ ] component patterns for forest-native UIs -- future
  [ ] iced-based dashboard architecture -- future
  [ ] faelight-fm v2 (INT-293) -- libcosmic file manager

FROM COSMIC workspace UX:
  [ ] intentional context grouping -- future
  [ ] task-state visualization -- future

---
ZBUS INTEGRATION

zbus is the Rust-native D-Bus library.
INT-294 (Forest Event Bus v2) uses zbus for D-Bus IPC.
zbus already in workspace (faelight-notify uses it).
The IPC layer between the forest and desktop is being built.

---
GATES

Phase 3 -- cosmic-term source study:
[x] cosmic-term wgpu integration understood
[x] cosmic-text + glyphon pipeline implemented in faelight-term v3
[x] PTY handling patterns applied (alacritty_terminal)
[x] faelight-term v3 deployed and running as daily driver

Phase 1 -- cosmic-comp architecture:
[ ] compositor state model studied -- 3 key patterns documented
[ ] workspace management patterns documented
[ ] IPC boundary design documented

Phase 2 -- libcosmic + iced:
[ ] small proof of concept UI using libcosmic
[ ] one faelight tool prototyped with libcosmic (INT-293 faelight-fm)

Phase 4 -- selective integration:
[ ] faelight-fm v2 using libcosmic (INT-293)
[ ] faelight-bar v3 using libcosmic (INT-295)
[ ] Pop_OS rep demo -- working COSMIC-stack forest tools

Final:
[ ] Forest direction document updated to reflect COSMIC borrowing
[ ] The inversion demonstrated: shell defines state, visual reflects it
[ ] The forest shows what shell-first Rust desktop architecture looks like

---
CONTEXT

Pre-presentation timeline (summer 2026).
Phase 3 already complete via faelight-term v3.
Shell fixes (INT-291) and Friday Architecture (INT-246) continuing.
The foundation is solid. The visual layer is next.
Pop_OS rep is watching. Graydon expressed interest.
The presentation is this summer.

"Most desktops ask: what app do you want to open?
Faelight Forest asks: what do you want to happen?
COSMIC becomes the surface where the answer appears.
The shell already knows the answer.
It always did." 🌲
