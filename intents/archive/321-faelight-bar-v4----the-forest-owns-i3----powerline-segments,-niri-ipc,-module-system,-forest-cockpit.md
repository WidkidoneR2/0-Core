---
id: 321
title: "faelight-bar v4 -- The Forest Owns i3"
status: planned
date: 2026-05-18
tags: [bar, i3, waybar, yambar, powerline, segments, niri-ipc, workspaces, modules, renderer]
---
---
THE PREMISE

i3bar is the gold standard.
Dense. Segmented. Powerline arrows. Colored blocks.
Every workspace visible. Every status readable at a glance.
It feels like a cockpit. Not a decoration.

We built faelight-bar v3 from scratch.
We proved the renderer. We proved the layer-shell.
We proved we can own the stack completely.

Now we build the bar that looks like i3bar
but knows things i3bar never could.

faelight-bar v4 is not a waybar clone.
It is not a yambar clone.
It is the forest's own cockpit.
Every segment is alive.
Every segment knows about the forest.
---
WHAT MAKES i3bar FEEL LIKE i3bar

Study these before writing a line of code.

1. SEGMENTS with per-segment background colors
   Each block has its own fg + bg color pair.
   The contrast between adjacent segment backgrounds creates visual rhythm.
   This is not text color -- it is background fill behind each segment.

2. POWERLINE ARROWS between segments
   Unicode U+E0B0 (filled right-pointing triangle from Nerd Fonts)
   Unicode U+E0B2 (filled left-pointing triangle)
   The triangle color = the bg of the segment it points away from.
   The triangle bg = the bg of the segment it points toward.
   This creates the melting-into-each-other effect.
   Can be drawn as filled triangles directly in the SHM buffer.
   No font required -- pure pixel geometry.

3. WORKSPACE INDICATORS on the left
   One block per workspace.
   Active = bright bg. Occupied = dim. Empty = darker.
   Clicking changes workspace.
   Requires Niri IPC.

4. WINDOW TITLE in center
   The focused window title -- live, updating.
   Requires Niri IPC.

5. STATUS MODULES on the right
   Independent blocks: CPU | RAM | NET | BAT | TIME
   Each polls independently. One failure does not affect others.
   Module trait: name, update(), segments() -> Vec<Segment>

6. CLICK EVENTS
   The bar is interactive.
   Each segment registers a click region.
   Left click, middle click, right click all handled.
   Scroll up / scroll down on modules.

7. DENSITY
   Tight padding. Small gaps. Information-dense.
   Every pixel earns its place.
   No wasted space.
---
REFERENCE ARCHITECTURES TO STUDY

i3bar (the original):
  Language: C
  Protocol: reads JSON blocks from i3status via stdin
  Source: github.com/i3/i3 -- i3bar/
  Key files:
    src/main.c          -- event loop, click handling
    src/draw_util.c     -- segment drawing, powerline arrows
    src/child.c         -- reads status blocks from child process
    src/workspace.c     -- workspace blocks, click-to-focus
  Study: how segments are drawn, how click regions are tracked
  Key insight: segment = {text, fg, bg, min_width, align, urgent}

yambar (modern Wayland bar in C):
  Language: C
  Source: codeberg.org/dnkl/yambar
  Key files:
    module/*.c          -- each module is self-contained
    particle/*.c        -- rendering primitives
    bar/bar.c           -- main layout engine
  Study: module lifecycle (init/poll/destroy), particle rendering
  Key insight: modules produce tags (key-value), particles consume them

i3status-rust (status generator in Rust):
  Language: Rust
  Source: github.com/greshake/i3status-rust
  Key files:
    src/blocks/         -- each block is a self-contained Rust module
    src/protocol/i3bar.rs -- outputs i3bar JSON protocol
    src/widget.rs       -- the block output type
  Study: Block trait, async update model, error propagation per block
  Key insight: blocks are async, each has its own update interval

waybar (the reference Wayland bar):
  Language: C++ with GTK
  Source: github.com/Alexays/Waybar
  Key files:
    src/modules/        -- each module inherits AModule
    src/bar.cc          -- layer-shell setup, click dispatch
    include/AModule.hpp -- the module interface
  Study: module interface, how workspaces attach to compositor IPC
  Key insight: modules register signals, bar dispatches input events

lemonbar (pixel-perfect minimal bar):
  Language: C
  Source: github.com/LemonBoy/bar
  Study: the simplest possible bar implementation
  Key insight: reads formatted text from stdin, outputs click events to stdout
  This is the do-one-thing-perfectly reference

Niri IPC (our compositor):
  Niri exposes socket at $NIRI_SOCKET
  Events: workspace-activated, window-focus-changed, etc.
  Commands: focus-workspace, etc.
  Source: github.com/YaLTeR/niri/wiki/IPC
  Crate: niri-ipc
  Key events: WorkspacesChanged, WorkspaceActivated, WindowFocusChanged
---
ARCHITECTURE

The Segment Model:

  struct Segment {
      text: String,
      fg: [u8; 4],            // ARGB text color
      bg: [u8; 4],            // ARGB background fill
      padding: u32,           // pixels either side of text
      min_width: Option<u32>,
      click_left:   Option<String>,
      click_right:  Option<String>,
      click_middle: Option<String>,
      scroll_up:    Option<String>,
      scroll_down:  Option<String>,
      urgent: bool,
  }

The Module Trait:

  trait BarModule: Send + Sync {
      fn name(&self) -> &'static str;
      fn update(&mut self);
      fn segments(&self) -> Vec<Segment>;
  }

The Powerline Renderer:

  fn draw_powerline_right(canvas, x, from_bg, to_bg, bar_height)
    Geometric formula:
      arrow_width = bar_height / 2
      for each row y (0..bar_height):
        extent = arrow_width - |y - bar_height/2|
        fill pixels x..(x + extent) with from_bg color
    No font needed. Pure triangle geometry in the SHM buffer.

The Click Region Tracker:

  struct ClickRegion {
      x_start: u32,
      x_end:   u32,
      module_index: usize,
      click_left:   Option<String>,
      click_right:  Option<String>,
  }
  Built during render. Checked on pointer button events.

Zones:
  LEFT:   [workspaces...] [window_title]
  CENTER: [active_intent | friday_signal]
  RIGHT:  [cpu] [ram] [network] [battery] [clock]
---
MODULES TO BUILD

Phase 1 (core visual -- no IPC needed):
  LockModule        -- lsattr, [L] green / [U] red
  HealthModule      -- /etc/faelight/HEALTH, colored
  IntentModule      -- /etc/faelight/INTENT, truncated
  BatteryModule     -- /sys/class/power_supply/BAT1/
  ClockModule       -- chrono, amber background
  WifiModule        -- /sys/class/net/wl*/operstate

Phase 2 (Niri IPC):
  WorkspaceModule   -- live workspace indicators, click-to-focus
  WindowTitleModule -- focused window name, live

Phase 3 (system stats):
  CpuModule         -- /proc/stat delta, colored by load
  RamModule         -- /proc/meminfo, colored by usage
  NetworkModule     -- /sys/class/net/, bytes in/out rates

Phase 4 (forest-aware, depends on INT-294):
  FridayModule      -- D-Bus subscription, confidence display
  DeployModule      -- flashes on deploy complete
  GitModule         -- branch + dirty state + commit count
---
VISUAL DESIGN

Bar height: 32px
Arrow width: 16px (half bar height)
Segment padding: 8px each side
Font: JetBrainsMono or similar monospace 13.5px

Color palette:
  Bar BG:            #0F1410  (near-black forest)
  Segment default:   #1A2318  fg #CCCCCC
  Workspace active:  bg #1E3A2A  fg #00E580
  Workspace occupied: bg #162418  fg #88BB99
  Workspace empty:   bg #0F1410  fg #445544
  Lock green:        bg #0D2015  fg #00E580
  Lock red:          bg #2A0F0F  fg #FF5555
  Battery green:     bg #0D2015  fg #00E580   (>=95%)
  Battery cyan:      bg #001A2A  fg #00BFFF   (>=50%)
  Battery amber:     bg #1A1800  fg #F0A500   (>=20%)
  Battery red:       bg #2A0F0F  fg #FF5555   (<20%)
  Clock:             bg #1A1800  fg #F0A500
  CPU hot:           bg #2A0F0F  fg #FF5555
  Friday:            bg #001A2A  fg #00BFFF
  Intent:            bg #1A2318  fg #CCCCCC
---
RENDERING PIPELINE

1. Each module updates on its own interval (async or threaded)
2. Main loop wakes every 50ms
3. If any module dirty: rebuild segment list
4. Fill canvas with bar BG color
5. Draw segment background rectangles
6. Draw powerline arrows between adjacent segments
7. Draw text on top via cosmic-text
8. Record click regions alongside pixel positions
9. Commit SHM buffer
10. Dispatch click events to module handlers on pointer input

Result: 50ms max latency to any state change.
Workspaces update the instant Niri sends the event.
Friday signal appears before you finish reading the terminal.
---
PHASES

Phase 0 -- Study (1 session):
  Read i3bar/src/draw_util.c -- powerline triangle algorithm
  Read i3status-rust/src/blocks/ -- module architecture in Rust
  Read yambar/module/*.c -- module lifecycle
  Read niri-ipc crate docs -- event types
  Gate: powerline algorithm understood and sketched
        segment model finalized on paper

Phase 1 -- Segment renderer:
  Replace v3 text-only zones with segment model
  Colored rectangle backgrounds per segment
  Powerline arrows between segments
  Gate: bar visually matches i3bar aesthetic
        powerline arrows present between all segments

Phase 2 -- Niri IPC:
  WorkspaceModule live
  WindowTitleModule live
  Click regions built during render
  Workspace click handler sends Niri IPC command
  Gate: clicking workspace segment switches workspace
        window title updates on every focus change

Phase 3 -- System modules:
  CpuModule, RamModule, NetworkModule
  Independent update intervals
  Gate: right zone shows CPU% RAM% NET BAT% TIME
        all colored by threshold

Phase 4 -- Forest modules (after INT-294):
  FridayModule -- D-Bus, confidence + message
  DeployModule -- flashes green segment on deploy
  IntentModule -- shows progress if available
  Gate: bar shows forest intelligence no other bar shows
        FridayModule alone makes this bar unique

Phase 5 -- Interaction:
  Click handlers on all modules
  Scroll on clock -- nothing (reserved)
  Scroll on volume -- future faelight-audio
  Right-click -- show module detail popup (future)
  Gate: all click and scroll events handled cleanly

Phase 6 -- Polish + daily driver:
  Tune colors, arrow sizes, padding to pixel perfection
  Memory stable over 24h (no growth)
  Gate: v3 replaced, running 1 week as daily driver
        someone who knows i3bar sees it and asks
        why does it know your intent
---
GATES
[ ] Phase 0: powerline algorithm documented, segment model finalized
[ ] Phase 1: colored segments with powerline arrows -- i3bar aesthetic matched
[ ] Phase 2: Niri IPC live -- workspaces + window title + click-to-focus
[ ] Phase 3: system modules -- CPU RAM NET BAT all live
[ ] Phase 4: forest modules -- Friday Deploy Intent in segments
[ ] Phase 5: fully interactive -- click and scroll all handled
[ ] Phase 6: daily driver -- v3 replaced, 1 week clean
Final:
[ ] Someone who knows i3bar recognizes the aesthetic instantly
[ ] The bar shows something no i3bar ever showed: Friday confidence
[ ] The forest cockpit is complete -- the desktop has a brain
---
DEPENDS ON
INT-295 (faelight-bar v3) -- renderer foundation -- COMPLETE
INT-294 (Forest Event Bus v2) -- for Friday/Deploy D-Bus modules

TIMELINE
Phase 0-3: can start any time, independent of INT-294
Phase 4: after INT-294 complete
Target Phase 3: before NY presentation (mid-July 2026)
Phase 4-6: post-presentation or concurrent

"i3bar showed us what a bar should feel like.
Faelight Forest shows what a bar should know.
We take the aesthetic.
We add the intelligence.
The cockpit is ours." 🌲

---

## The Visual Vision -- Forest Candy + i3 Precision

This is not a theme. This is an identity.

### Forest Candy
- Soft glows on active window borders -- forest green (#11140f base) with warm amber
  accents on focus. Not harsh. Not flat. Depth.
- Status elements have subtle layering -- the bar feels like it exists in space,
  not painted on the screen.
- Inactive borders fade to dark moss. Active borders breathe with a soft glow.
- The palette: forest dark (#11140f), forest green (#1a2f1a), amber focus (#c8a96e),
  ice blue accent (#00bfff), warm white text (#d7e0da).

### i3 Precision
- Every action has a keybind. Mouse is optional, never required.
- Workspaces are intentional contexts, not window dumps.
- Layout is deterministic -- you always know where a window will appear.
- Super is the forest key. Every Super+* binding is documented in the cheatsheet.

### One Visual Language
- faelight-bar, faelight-term, faelight-compositor borders, faelight-notify --
  all share the same palette, the same font weight, the same corner radius (0 -- sharp).
- No element feels like it came from a different system.
- If you screenshot any part of the forest, you know it's the forest.
- The terminal background is the same dark as the compositor background.
  The bar is a continuation of the screen, not a separate widget.

### The Thesis
Most tiling WMs are cold. Precise but cold.
Most DEs are warm. Soft but imprecise.
The forest is both.
Precision with warmth. Keyboard with depth. Green with amber.
It looks like it was built by one person who thought carefully about every pixel.
Because it was.

"The forest does not decorate. It grows.
Every visual element has a reason.
The glow on the active border is not decoration --
it is the forest telling you where you are." 🌲
