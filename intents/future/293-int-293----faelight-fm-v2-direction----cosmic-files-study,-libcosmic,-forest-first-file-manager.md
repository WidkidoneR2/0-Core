---
id: 293
title: "faelight-fm v2 -- COSMIC Files study, libcosmic, forest-first file manager"
status: planned
date: 2026-05-12
tags: [faelight-fm, cosmic, libcosmic, cosmic-files, yazi, broot, file-manager, forest, rust]
---

faelight-term v3 proved the approach:
Study the best Rust implementations.
Extract the patterns.
Build something forest-first that owns the stack.

faelight-fm v2 follows the same philosophy.

---

WHY A NEW FM

faelight-fm v1 (current) is a ratatui TUI.
It works but it is not visual-first.
The forest needs a file manager that:
  Understands forest context (active intent, git state, health)
  Runs as a Wayland surface (not inside a terminal)
  Integrates with the scroll-native UX (INT-289)
  Shows the forest spatial graph (state.db)
  Friday can surface context about what you are looking at

---

STUDY SOURCES

COSMIC Files (highest priority):
  Pure Rust, libcosmic widget toolkit
  Wayland-native, built for COSMIC Desktop
  Tabs, bookmarks, grid/list view, drag-and-drop
  Source: github.com/pop-os/cosmic-files
  Study: src/app.rs (application structure),
         src/tab.rs (tab/pane model),
         src/operation.rs (file operations)
  Key pattern: how libcosmic builds the widget tree

Yazi (secondary study):
  Already installed and used daily
  Written in Rust, TUI-based
  Excellent async file operations
  Plugin system in Lua
  Key pattern: async file ops, preview rendering, miller columns
  Source: github.com/sxyazi/yazi

Broot (reference):
  Rust TUI tree navigator
  Forest metaphor built in (it is literally called broot)
  Fuzzy search, preview, custom verbs
  Key pattern: tree model, search-first navigation
  Source: github.com/Canop/broot

---

WHAT TO BORROW FROM EACH

From COSMIC Files:
  libcosmic widget toolkit for the GUI layer
  Tab model for multiple directory views
  Drag-and-drop to Wayland compositor
  File operation progress (copy/move/delete with progress bar)

From Yazi:
  Miller columns layout (parent | current | preview)
  Async file operations (non-blocking directory reads)
  Preview system (text, image, binary detection)
  Key binding philosophy

From Broot:
  Tree model -- always show context
  Search-first navigation -- type to filter
  Forest metaphor -- the name itself

---

FAELIGHT-FM v2 ARCHITECTURE

Stack:
  libcosmic -- widget toolkit (same as COSMIC bar, COSMIC files)
  cosmic-text -- text rendering (same as faelight-term v3)
  wgpu -- GPU renderer (same stack throughout the forest)
  state.db -- forest context integration
  Friday -- intelligence layer

Layout (Miller columns, forest-aware):
  Left panel:   Bookmarks + forest locations (0-core, intents, docs)
  Center panel: Current directory (grid or list)
  Right panel:  Preview (file content, image, binary info)
  Bottom bar:   Active intent, Friday signal, forest health

Forest integration:
  Git status per file (modified, untracked, staged)
  Intent context (which intent does this file belong to?)
  Friday observations (has this file been frequently accessed?)
  Health indicators (broken symlinks, stow conflicts highlighted)

Keybindings (Yazi-inspired, forest-aware):
  h/j/k/l    -- navigate (miller columns)
  Enter       -- open file
  Space       -- select/deselect
  y           -- yank (copy path)
  p           -- paste
  d           -- delete (with forest safety guard)
  /           -- search
  f           -- Friday context for file
  i           -- intent context for directory
  g           -- go to root (0-core)

---

RELATIONSHIP TO OTHER INTENTS

INT-287 (COSMIC Direction):
  libcosmic study feeds directly into faelight-fm v2
  cosmic-files is the primary reference implementation

INT-289 (Scroll-Native UX):
  FM panels are spatial anchors in the scroll strip
  Opening FM places it at a remembered scroll position

INT-286 (faelight-term v3):
  Same stack: cosmic-text, wgpu, Wayland-native
  FM and term share renderer patterns

INT-290 (F-DWL):
  FM runs inside F-DWL as a first-class surface
  Layer-shell integration for file drop targets

---

STUDY PHASE (before writing code)

Phase 0 -- Study (runs alongside INT-287):
  Clone and read cosmic-files source
  Clone and read yazi source (already have it installed)
  Clone and read broot source
  Document: 5 key patterns to apply to faelight-fm v2
  Gate: architecture document written

Phase 1 -- libcosmic spike:
  Get a libcosmic window rendering on Wayland
  Understand the widget tree model
  Gate: libcosmic window appears with a simple widget

Phase 2 -- Directory listing:
  Read directory contents
  Display as list with cosmic-text
  Show git status per file
  Gate: ls equivalent renders in the window

Phase 3 -- Navigation:
  h/j/k/l navigation
  Miller columns layout
  Directory traversal
  Gate: can navigate the entire 0-core directory tree

Phase 4 -- Forest integration:
  Intent context per directory
  Friday observations surfaced
  Git status indicators
  State.db spatial anchors
  Gate: 0-core shows intent context for each subdirectory

Phase 5 -- File operations:
  Copy, move, delete with safety guard
  Progress bar for long operations
  Friday warning for destructive ops
  Gate: can perform all common file operations

Phase 6 -- Daily driver:
  Replace yazi as primary FM
  1 week daily use
  Gate: yazi no longer opened

---

TIMELINE

Post INT-292 (faelight-term v3 stable daily driver).
Start Phase 0 study during INT-292 daily driving period.
Phase 1-3: summer 2026.
Phase 4-6: fall 2026.

The forest builds one tool at a time.
Each tool teaches the next.
faelight-term v3 taught us wgpu + cosmic-text.
faelight-fm v2 inherits that knowledge.

---

GATES

[ ] Phase 0: COSMIC Files, Yazi, Broot studied -- patterns documented
[ ] Phase 1: libcosmic window renders on Wayland
[ ] Phase 2: directory listing with git status
[ ] Phase 3: miller columns navigation working
[ ] Phase 4: forest integration -- intent context, Friday signals
[ ] Phase 5: file operations with safety guard
[ ] Phase 6: 1 week daily driver, yazi retired

Final:
[ ] faelight-fm v2 is the forest file manager
[ ] Forest-aware: git, intents, Friday, state.db
[ ] Same stack as faelight-term v3 (wgpu, cosmic-text, libcosmic)
[ ] Spatial anchor in the scroll-native desktop (INT-289)
[ ] Yazi becomes optional, not required

---

"The file manager is the forest navigator.
It should know where you are, what you are working on,
and what Friday thinks about what it sees.
faelight-fm v2 is not a file manager.
It is a forest explorer." 🌲
