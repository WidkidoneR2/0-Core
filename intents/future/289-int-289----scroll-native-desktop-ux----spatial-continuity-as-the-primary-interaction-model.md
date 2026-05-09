---
id: 289
title: "Scroll-Native Desktop UX -- spatial continuity as the primary interaction model"
status: planned
date: 2026-05-09
tags: [niri, spatial, ux, desktop, scroll, innovation, cosmic, wayland, paradigm]
---
This is not an incremental improvement on existing desktop environments.
This is a new interaction paradigm.
Every major desktop environment -- GNOME, KDE, COSMIC, Hyprland --
thinks in discrete workspaces. You switch between them.
They are buckets. You pick one.
Niri thinks differently.
Niri has infinite horizontal scroll.
Workspaces are not buckets. They are a continuous spatial strip.
You do not switch -- you navigate.
No desktop environment has built its tools around this model.
Faelight Forest will be the first.
---
THE CORE INSIGHT
Traditional desktop UX assumes:
  workspaces are discrete, numbered, finite
  switching is instantaneous (no spatial memory)
  context lives in windows, not positions
  the overview is a birds-eye of buckets
Scroll-native UX assumes:
  the workspace strip is infinite and continuous
  position carries meaning (left = older, right = newer/active)
  spatial memory is real -- you remember where things live
  the overview is a zoom-out of the strip, not a grid of buckets
This is not a visual preference.
It is a fundamentally different cognitive model.
The human brain is good at spatial memory.
Traditional desktops throw that away.
Scroll-native UX uses it.
---
WHAT CHANGES WHEN EVERYTHING IS SCROLL-NATIVE
1. THE TERMINAL
   Terminal panes persist spatially in workspace history.
   You do not close a terminal -- you scroll away from it.
   When you need it again -- scroll back.
   The terminal remembers where it was.
   The shell knows its spatial position.
   fsh context includes: which workspace region, what was nearby.
2. THE FILE MANAGER
   Places become spatial anchors, not bookmarks.
   ~/0-core lives at a consistent scroll position.
   ~/1-src is always to the right of it.
   Navigation is literal movement, not teleportation.
   You develop muscle memory for where things are spatially.
3. THE LAUNCHER
   The launcher is not a popup over the current view.
   It is a spatial tool -- launch here, launch to the right,
   launch at a named anchor.
   "Open terminal to the right of current work" is a natural command.
4. NOTIFICATIONS
   Notifications are not interruptions from nowhere.
   They are spatially anchored to the work that generated them.
   A build notification appears near the workspace where the build ran.
   You navigate to it naturally.
5. THE OVERVIEW
   Not a grid of workspace thumbnails.
   A zoom-out of the entire strip.
   You see the whole spatial history.
   You navigate by scrolling the overview.
   Spatial continuity is preserved at every zoom level.
6. FRIDAY INTEGRATION
   Friday knows where you are spatially.
   "You usually work on 0-core in this region."
   "This region is for builds -- want to move this window?"
   The intelligence layer understands spatial context,
   not just temporal context.
---
THE SHELLD CONCEPT
The email that inspired this intent described:
shelld
  launcher
  notifications
  settings
  file-indexer
  clipboard-history
  session-state
Rust async services communicating over zbus or Unix sockets.
Niri renders windows. shelld coordinates everything else.
For scroll-native UX, shelld needs one more service:
  spatial-graph -- tracks what lives where in the scroll strip
                   persists spatial context across sessions
                   provides position awareness to all other services
The forest already has state.db.
The spatial graph lives in state.db.
Every window, every terminal, every context --
recorded with its scroll position.
The forest remembers where everything was.
---
WHAT ALREADY EXISTS IN FAELIGHT FOREST
The shell already knows about intent and context.
Friday already tracks patterns and predicts behavior.
The intent ledger already captures what was being worked on.
state.db already stores everything.
What is missing:
  spatial coordinates attached to all of this
  tools that understand scroll position as context
  an overview that zooms the strip instead of switching buckets
  a launcher that places things spatially
---
INNOVATION SPACE
Nobody is doing this.
COSMIC still thinks in traditional workspace metaphors.
Hyprland has special workspaces but not spatial continuity.
Sway had numbered workspaces.
Even Niri -- which invented the scroll model -- has not built
its ecosystem tools around it.
Faelight Forest has the opportunity to define what
scroll-native desktop UX actually looks like in practice.
This is not a fork of COSMIC.
This is not a better GNOME.
This is a different answer to the question:
"How should a human navigate their digital workspace?"
---
TECH STACK
Niri -- the compositor that makes this possible
fsh -- the shell that knows spatial context
state.db -- the spatial graph lives here
zbus -- desktop integration (INT-287)
iced + libcosmic -- UI for scroll-native tools
wgpu -- GPU rendering for smooth scroll animation
smithay -- Wayland protocol integration
---
COMPONENTS TO BUILD
Phase 1 -- Spatial Graph (foundation)
  spatial_context table in state.db
  Records: window_id, workspace_position, intent_context, timestamp
  fsh emits spatial events when you navigate
  Friday reads spatial context for predictions
  Gate: state.db knows where you are in the scroll strip
Phase 2 -- Scroll-Native Launcher
  Launches apps at a specific scroll position
  "to the right" "at this anchor" "near this intent"
  Not a popup -- a spatial placement tool
  Gate: launch faelight-term at a named scroll position
Phase 3 -- Spatial File Manager (faelight-fm v2)
  Places are scroll anchors, not bookmarks
  Navigation feels like movement, not teleportation
  Gate: ~/0-core always at the same scroll region
Phase 4 -- Scroll-Native Overview
  Zoom out of the entire strip
  Navigate by scrolling the overview
  Spatial continuity preserved
  Gate: overview shows the full strip, navigate by scroll
Phase 5 -- Friday Spatial Intelligence
  Friday understands where you are, not just when
  Predictions include spatial context
  "You usually work on this here" not just "you usually do this next"
  Gate: Friday suggestion includes spatial recommendation
Phase 6 -- The Demo
  Show a complete workflow that could not exist on any other desktop
  Terminal persists spatially
  File manager anchored to scroll positions
  Launcher places things spatially
  Friday predicts based on where you are
  Overview zooms the strip
  Gate: Pop_OS rep and Graydon see something genuinely new
---
GATES
[ ] Spatial graph schema designed and in state.db
[ ] fsh emits scroll position events
[ ] Scroll-native launcher prototype works
[ ] faelight-fm v2 treats places as spatial anchors
[ ] Overview prototype zooms the strip
[ ] Friday reads spatial context for predictions
[ ] Full demo workflow documented and working
[ ] The demo shows something no other desktop can do
Final:
[ ] Faelight Forest defines scroll-native desktop UX
[ ] Other projects reference this work
[ ] The paradigm has a name beyond "that Niri thing"
---
TIMELINE
Post-summer. After:
  INT-285 (fsh friction fixed)
  INT-246 (Friday Architecture complete)
  INT-287 (COSMIC study done)
  INT-286 (faelight-term v3 built)
The foundation must be solid.
Then the spatial layer goes on top.
---
"Every desktop environment asks:
which workspace do you want to switch to?
Faelight Forest asks:
where do you want to go?
The answer is always: scroll.
The forest remembers where everything lives.
You never lose context.
You never switch -- you navigate.
This is what spatial memory was always for." 🌲
