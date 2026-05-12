---
id: 286
title: "faelight-term v3 -- wgpu + cosmic-text rebuild"
status: in-progress
date: 2026-05-08
tags: [faelight-term, wgpu, cosmic-text, gpu, rendering, unicode, wayland, rebuild]
---
faelight-term v2 has structural limits that cannot be patched away.
fontdue does not handle emoji width correctly.
Software pixel buffer redraws the entire frame every tick.
The scrollback model corrupts on repeated large output.
These are not bugs. They are architectural choices that were wrong.
v3 replaces the renderer entirely.
The terminal logic stays. The rendering pipeline is rebuilt from scratch.
---
WHY THIS IS THE RIGHT MOVE
Current stack problems:
  fontdue -- no emoji 2-cell width, no ligatures, no font fallback
  Software pixel buffer -- full redraw every frame, causes mouse drag flash
  Custom scrollback -- corrupts on repeated large output (INT-284 Bug 1)
  Custom PTY -- subtle read timing issues (partially fixed in v2.1)
v3 stack solutions:
  cosmic-text -- correct Unicode, emoji width, ligatures, font fallback by design
  wgpu -- GPU pipeline, damage regions, frame pacing, 60fps without tearing
  glyphon -- wgpu text rendering layer that wraps cosmic-text cleanly
  portable-pty -- battle-tested PTY management, replaces custom nix PTY
  smithay-client-toolkit -- keep, it is solid
  vte crate -- keep, already in codebase, handles escape sequences correctly
---
ARCHITECTURE
Keep from v2:
  smithay-client-toolkit (Wayland surface, seat, output)
  vte crate (escape sequence parsing -- do not reinvent this)
  Terminal grid model (rows x cols of Cell structs)
  fsh integration and forest signals
  Niri-aware window management
Replace in v3:
  fontdue -> cosmic-text via glyphon
  software pixel buffer -> wgpu render pipeline
  custom PTY -> portable-pty
  current scrollback -> proper ring buffer with damage tracking
New in v3:
  damage tracking -- only redraw changed cells, not full buffer
  dirty flag per cell -- set on write, cleared on render
  frame pacing -- render at display refresh rate, not event rate
  fractional scaling -- correct for Niri HiDPI
  proper emoji -- 2-cell width, color emoji, no extra space
---
DEPENDS ON
  INT-287 -- Faelight Forest COSMIC Direction -- study cosmic-term first
  Phase 0 of this intent is reading cosmic-term source before writing a line.
---
STUDY BEFORE WRITING CODE
Rio terminal (highest priority):
  Wayland-first, wgpu renderer, simpler than WezTerm
  Study: src/renderer/, src/layout/, Wayland integration
  Source: github.com/raphamorim/rio
Alacritty (grid model):
  Best reference for scrollback correctness
  Study: alacritty/src/display/content.rs, grid model
  Source: github.com/alacritty/alacritty
glyphon crate:
  wgpu text rendering that wraps cosmic-text
  This is the bridge between wgpu and cosmic-text
  Source: github.com/grovesNL/glyphon
---
DEPENDENCIES TO ADD
  wgpu = "22"
  glyphon = "0.7"
  cosmic-text = "0.12"
  portable-pty = "0.8"
Keep:
  smithay-client-toolkit
  vte
  nix (for signals, not PTY)
  calloop + calloop-wayland-source
---
BUILD PHASES
Phase 0 -- cosmic-term source study (INT-287 Phase 3)
  Read cosmic-term wgpu integration
  Read cosmic-term cosmic-text usage
  Read cosmic-term PTY handling
  Document: 3 key patterns to apply to faelight-term v3
  Gate: study notes written, patterns identified
Phase 1 -- Renderer foundation (study + spike)
  Read Rio renderer source
  Read glyphon examples
  Get a wgpu surface rendering a colored rectangle in a Wayland window
  Gate: GPU triangle renders in a Wayland window via wgpu + sctk
Phase 2 -- Text rendering
  Integrate glyphon + cosmic-text
  Render a fixed string at correct cell positions
  Verify emoji renders 2 cells wide
  Gate: "Hello 🌲 World" renders correctly, emoji takes 2 cells
Phase 3 -- Grid model + PTY
  Integrate portable-pty
  Connect vte parser to terminal grid
  Grid cells map to wgpu glyph positions
  Gate: bash runs in the terminal, text appears correctly
Phase 4 -- Scrollback + damage
  Implement ring buffer scrollback (fixed max lines)
  Implement per-cell dirty flag
  Only re-upload changed glyphs to GPU
  Gate: large output (35+ lines) renders completely every time
Phase 5 -- fsh integration + forest signals
  fsh as default shell
  Forest signal emission (deploy detected, health check)
  friday voice output in terminal
  Gate: full fsh session works in v3 terminal
Phase 6 -- Polish
  Fractional scaling for Niri HiDPI
  Frame pacing at display refresh rate
  Mouse selection without flashing
  Gate: faelight-term v3 used as daily driver for 1 week
---
GATES
Phase 1:
[x] wgpu surface renders in Wayland window via sctk -- AMD RX 7700S, Vulkan, Bgra8UnormSrgb
[x] No software pixel buffer -- pure wgpu GPU pipeline from day one
Phase 2:
[x] cosmic-text renders text correctly -- Hello World, box drawing verified
[x] Emoji renders -- tree emoji visible in window
[x] Box drawing characters render via cosmic-text shaping
Phase 3:
[ ] portable-pty manages shell process
[ ] vte parser feeds terminal grid
[ ] fsh runs and output appears
Phase 4:
[ ] Scrollback ring buffer holds 10000 lines
[ ] Per-cell dirty tracking implemented
[ ] Second consecutive d shows complete output (INT-284 Bug 1 fixed)
Phase 5:
[ ] fsh is default shell in v3
[ ] Forest signals emit correctly
[ ] Friday voice appears in terminal
Phase 6:
[ ] No mouse drag flashing (INT-284 Bug 3 fixed)
[ ] Fractional scaling correct on Niri
[ ] 1 week daily driver without foot fallback
Final:
[ ] INT-284 all three bugs resolved by architecture not patches
[ ] faelight-term v3 replaces v2 completely
[ ] foot is no longer needed
"The terminal is the forest mouth.
It should speak clearly.
It should render truthfully.
It should never drop a line.
v3 builds the mouth the forest deserves." 🌲

---
COSMIC TERMINAL PATTERNS (added 2026-05-09, from Pop_OS email)
These patterns from COSMIC Terminal apply directly to faelight-term v3:
Shell-aware UI (not AI gimmicks -- real context):
  git state visible in terminal chrome
  command duration tracked and displayed
  task grouping -- related commands visually connected
  scrollback indexing -- search by intent, not just text
  semantic prompts -- fsh context surfaces in the terminal UI
GPU text pipeline for Niri specifically:
  fractional scaling handled correctly (Niri HiDPI)
  smooth scroll animation -- the terminal scrolls like the compositor
  low-latency glyph caching -- glyphs uploaded once, reused
  frame pacing at display refresh rate -- no tearing on Niri
  Target: outperform portable terminals on Niri specifically
Wayland-native clipboard + drag model:
  Most terminals still feel X11-era for clipboard
  wl-clipboard-rs already in the stack (wl-clipboard-rs 0.9.3)
  Drag and drop between terminal and file manager
  Primary selection works correctly
  No X11 clipboard assumptions anywhere
Compositor awareness:
  Terminal knows it is running inside Niri
  Responds to compositor events correctly
  Fractional scale changes handled at runtime
  Explicit sync when available
Pane semantics (future, post-v3):
  Terminal panes persist spatially (INT-289 scroll-native UX)
  Each pane remembers its scroll position
  Panes are spatial anchors, not just tabs
STUDY ALSO (added from Pop_OS email):
  cosmic-term source: github.com/pop-os/cosmic-term
  Focus on: pane/workspace semantics, PTY handling, compositor awareness
  This is now Phase 0 alongside Rio study
