---
id: 287
title: "Faelight Forest COSMIC Direction -- fsh as authority, COSMIC as visual layer"
status: planned
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
This intent documents the direction, the study plan,
and the selective borrowing strategy.
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
WHY COSMIC SPECIFICALLY
1. Rust-native compositor (cosmic-comp via smithay)
   Same foundation as Niri. Protocol knowledge transfers.
   State ownership patterns directly applicable to faelight-term v3.
2. libcosmic (iced-based UI toolkit)
   Rust-native, Wayland-correct, modern async.
   No GTK/QML baggage.
   Right tool for: integrity dashboards, policy approval interfaces,
   forest memory explorers, ledger navigation, authorization dialogs.
3. cosmic-text already chosen for faelight-term v3
   The text stack is shared. The ecosystem is already converging.
4. COSMIC tolerates shell-as-authority
   Wayland protocols are clean enough.
   Rust tooling tends toward explicitness.
   Modern session management is modular.
   compositor boundaries are clear.
5. Pop_OS rep is watching
   The people who built COSMIC are interested in what
   Faelight Forest does with it. This is an opportunity
   to demonstrate what shell-first Rust desktop architecture
   can look like.
---
WHAT WE BORROW (SELECTIVE)
FROM cosmic-comp:
  smithay state ownership patterns
  workspace management ideas
  rendering pipeline architecture
  IPC boundary design
FROM libcosmic:
  component patterns for forest-native UIs
  async state handling
  iced-based dashboard architecture
FROM cosmic-term:
  wgpu + cosmic-text integration (directly feeds INT-286)
  PTY management patterns
  damage tracking implementation
FROM COSMIC workspace UX:
  intentional context grouping ideas
  task-state visualization concepts
  operational focus transitions
---
WHAT WE DO NOT IMPORT
Automatic background behavior -- the forest rejects things
that run without explicit human authorization.
UI-first assumptions -- COSMIC optimizes for desktop convenience.
Faelight optimizes for human intentionality.
Full desktop ownership -- the forest has its own security model,
its own orchestration, its own trust boundaries.
These do not change.
---
STUDY PLAN
Phase 1 -- cosmic-comp architecture
  Read: compositor state model
  Read: workspace management
  Read: IPC boundaries
  Goal: understand what patterns apply to faelight-term v3
Phase 2 -- libcosmic + iced
  Build: small proof of concept UI using libcosmic
  Goal: can we build a forest integrity dashboard with this?
  Gate: one faelight tool rebuilt or prototyped with libcosmic
Phase 3 -- cosmic-term source
  Read: wgpu integration
  Read: cosmic-text usage
  Read: PTY handling
  Goal: directly inform faelight-term v3 (INT-286) Phase 1 study
Phase 4 -- selective integration
  Identify: which COSMIC patterns improve which forest tools
  Build: one dashboard or visualization using libcosmic
  Gate: Pop_OS rep sees a working demo
---
TOOLS AFFECTED
faelight-term v3 (INT-286) -- cosmic-term study informs wgpu architecture
faelight-fm v2 (INT-280) -- ratatui TUI, libcosmic for any graphical layer
faelight-bar -- may evolve toward libcosmic applet model
future: integrity dashboard -- libcosmic native
future: ledger navigator -- libcosmic native
future: forest memory explorer -- libcosmic native
---
GATES
[ ] cosmic-comp architecture studied -- 3 key patterns documented
[ ] libcosmic proof of concept built -- one small UI renders
[ ] cosmic-term source read -- wgpu integration understood
[ ] INT-286 Phase 1 gate completed informed by cosmic-term study
[ ] One forest tool prototype using libcosmic architecture
[ ] Pop_OS rep demo -- something working to show
[ ] Architectural decision recorded: what we borrow, what we reject
Final:
[ ] Faelight Forest direction document updated to reflect COSMIC borrowing
[ ] The inversion is demonstrated: shell defines state, visual reflects it
[ ] The forest shows what shell-first Rust desktop architecture looks like
---
CONTEXT
Post-summer timeline.
3-4 months of study before major implementation.
Shell fixes (INT-285) and Friday Architecture (INT-246) come first.
The foundation must be solid before the visual layer matters.
Pop_OS rep is watching.
Graydon expressed interest.
The presentation is this summer.
Build the right thing. Build it correctly.
The forest does not rush.
"Most desktops ask: what app do you want to open?
Faelight Forest asks: what do you want to happen?
COSMIC becomes the surface where the answer appears.
The shell already knows the answer.
It always did." 🌲
