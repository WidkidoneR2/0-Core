# INT-286 Phase 0 -- Study Notes
## Date: 2026-05-11
## Sources: cosmic-term (pop-os/cosmic-term)

---

## KEY DECISION: Use alacritty_terminal crate

cosmic-term uses alacritty_terminal for everything grid-related:
- Terminal grid model (rows x cols of Cell structs)
- VTE escape sequence parsing (built in, no separate vte crate needed)
- PTY management via alacritty_terminal::tty
- Scrollback via Term<T> with Boundary enum
- Selection model built in
- TermDamage for damage tracking

This is better than our current approach:
- We have custom grid + vte crate separately
- alacritty_terminal gives us both in one battle-tested package
- Alacritty has been solving these problems for years

REVISED DEPENDENCY LIST:
  alacritty_terminal = "0.24"  -- replaces custom grid + vte + portable-pty
  cosmic_text = "0.12"         -- replaces fontdue, correct emoji/unicode
  glyphon = "0.7"              -- wgpu text rendering wrapping cosmic_text
  wgpu = "22"                  -- GPU renderer, replaces software pixel buffer
  smithay-client-toolkit       -- keep, Wayland surface management
  calloop                      -- keep, event loop

---

## PATTERN 1: alacritty_terminal grid model

```rust
use alacritty_terminal::{
    Term,
    event::{Event, EventListener, Notify, OnResize},
    event_loop::{EventLoop, Msg, Notifier},
    sync::FairMutex,
    term::{Config, TermDamage, TermMode},
    tty::{self, Options},
};
```

Term<T> is the central struct. T is your EventListener.
FairMutex wraps Term for thread-safe access.
EventLoop drives the PTY read loop.
Notifier sends input to the PTY.
TermDamage tracks what changed for efficient redraws.

---

## PATTERN 2: cosmic_text for text shaping

```rust
use cosmic_text::{
    Attrs, AttrsList, Buffer, BufferLine,
    Family, Metrics, Shaping, Weight, Wrap,
};
```

Buffer holds the text to render.
BufferLine is one line of terminal output.
Attrs defines font family, weight, style.
Metrics defines font size and line height.
Shaping::Advanced enables full Unicode shaping + emoji.

Emoji works because cosmic_text uses fontconfig for fallback.
2-cell emoji is handled by the shaping engine, not us.

---

## PATTERN 3: Damage tracking

cosmic-term uses TermDamage from alacritty_terminal:
- TermDamage::Full -- redraw everything
- TermDamage::Partial(lines) -- redraw specific lines

Combined with per-cell dirty flags, this enables:
- Only re-upload changed glyphs to GPU
- Frame pacing without full redraws
- Smooth scrolling without flashing

---

## PATTERN 4: Architecture overview

cosmic-term architecture:
  main.rs -> iced Application
  terminal.rs -> Term<EventListener> + PTY + cosmic_text buffers
  terminal_box.rs -> iced Widget that draws the terminal

For faelight-term v3:
  main.rs -> calloop event loop + sctk Wayland surface
  terminal.rs -> Term<EventListener> + PTY + cosmic_text buffers
  renderer.rs -> wgpu pipeline + glyphon text rendering
  grid_bridge.rs -> maps Term grid cells to glyphon glyph positions

---

## WHAT WE KEEP FROM V2

- smithay-client-toolkit (Wayland surface, seat, output)
- calloop event loop pattern
- fsh integration logic
- forest signal emission
- Friday panel concept

## WHAT WE REPLACE

- Custom grid -> alacritty_terminal::Term
- Custom PTY -> alacritty_terminal::tty
- vte crate (standalone) -> built into alacritty_terminal
- fontdue -> cosmic_text via glyphon
- software pixel buffer -> wgpu render pipeline
- custom scrollback -> Term scrollback with TermDamage

---

## NEXT STEPS (Phase 1)

1. Add alacritty_terminal to faelight-term Cargo.toml
2. Implement EventListener trait for our event type
3. Spawn PTY with tty::Options
4. Start EventLoop to drive PTY reads
5. Wrap Term in FairMutex
6. Spike: get wgpu surface rendering in a Wayland window
7. Connect glyphon to render a fixed string
8. Bridge: map Term grid cells to glyphon positions

Phase 0 gate: COMPLETE
- Study notes written
- Key patterns identified
- Dependency list revised
- Architecture decision made: alacritty_terminal is the foundation

---

## RIO STUDY -- sugarloaf wgpu renderer

### Wayland -> wgpu bridge (raw-window-handle)

Rio's SugarloafWindow implements HasWindowHandle + HasDisplayHandle.
This is how wgpu gets the Wayland surface.

```rust
// faelight-term v3 equivalent:
struct FaelightWindow {
    raw_window: RawWindowHandle,   // WlSurface from sctk -> RawWaylandWindowHandle
    raw_display: RawDisplayHandle, // WlDisplay from sctk -> RawWaylandDisplayHandle
}
impl HasWindowHandle for FaelightWindow { ... }
impl HasDisplayHandle for FaelightWindow { ... }
let surface = instance.create_surface(faelight_window)?;
```

Crate needed: raw-window-handle = "0.6"

### wgpu initialization sequence (from webgpu.rs)

```rust
let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
    backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
    ..Default::default()
});
let surface = instance.create_surface(window).unwrap();
let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
    power_preference: wgpu::PowerPreference::HighPerformance,
    compatible_surface: Some(&surface),
    force_fallback_adapter: false,
})).expect("adapter");
let (device, queue) = block_on(adapter.request_device(
    &wgpu::DeviceDescriptor::default()
)).expect("device");
```

Always use HighPerformance -- Rio removed the preference knob, everyone picks it.
Fallback to downlevel_webgl2_defaults if first request fails.

### Rio renderer structure (sugarloaf/src/renderer/)

- mod.rs -- main renderer, orchestrates all passes
- compositor.rs -- composites layers (text, background, selection)
- batch.rs -- batches draw calls for efficiency
- quad.vert/frag.glsl -- GPU shaders for quads (backgrounds, selections)
- renderer.wgsl -- main render shader (wgsl format, preferred for wgpu)

Key insight: Rio uses separate render passes per layer.
faelight-term v3 can start simpler: one pass for background, one for text.

### Rio renderer approach for text

Rio uses their own text rendering via sugarloaf fonts.
We use glyphon (wraps cosmic-text for wgpu) instead -- cleaner for our case.

glyphon gives us:
- TextAtlas -- GPU texture atlas for glyphs
- TextRenderer -- renders text to wgpu render pass
- Buffer -- from cosmic_text, holds shaped text

### REVISED DEPENDENCY LIST (final after both studies)

```toml
[dependencies]
# Terminal model + PTY + VTE (from cosmic-term study)
alacritty_terminal = "0.24"

# Text shaping + Unicode + emoji (from cosmic-term study)
cosmic-text = "0.12"

# wgpu text rendering bridge (connects cosmic-text to wgpu)
glyphon = "0.7"

# GPU renderer
wgpu = "22"

# Wayland surface bridge to wgpu
raw-window-handle = "0.6"

# Wayland (keep from v2)
smithay-client-toolkit = "0.18"
calloop = "0.12"
calloop-wayland-source = "0.2"

# Signals (keep from v2)
nix = "0.27"  # for signals only, not PTY
```

REMOVED vs INT-286 original plan:
- portable-pty (alacritty_terminal::tty is better)
- vte crate standalone (built into alacritty_terminal)
- fontdue (cosmic-text replaces it)

---

## PHASE 0 GATE: COMPLETE

- [x] cosmic-term source studied -- alacritty_terminal pattern identified
- [x] Rio source studied -- wgpu initialization + Wayland bridge pattern identified
- [x] Dependency list revised -- alacritty_terminal replaces 3 separate crates
- [x] Architecture decision documented
- [x] Phase 1 roadmap clear

Phase 1 first commit: get wgpu to render a colored rectangle in a Wayland window.
That is the spike. Everything else builds on it.

---

## PHASE 2 FINDINGS (2026-05-12)

### Wayland deadlock bug -- FIXED

Symptom: window never appeared in Niri when launched from inside faelight-term.
Root cause: classic Wayland deadlock
  - Niri waits for us to commit a buffer before mapping the window
  - We were waiting for Niri to send an event before calling render()
  - Neither side moves -- deadlock

Fix: call render() ONCE immediately after initialization, before blocking_dispatch.
  event_queue.flush() after the initial render to push the commit to Niri.

### Nested Wayland client limitation

faelight-term-v3 cannot be launched from inside faelight-term v2.
When run inside faelight-term, blocking_dispatch returns immediately instead of blocking.
Root cause: PTY interaction with the Wayland event loop.
Workaround: launch from foot or zsh directly.
This is a non-issue for v3 as a standalone terminal -- it launches from the compositor.

### What "looks horrible" means

The text renders but needs:
  - Proper font size calibration (cell size matching font metrics)
  - Correct grid positioning (character cells aligned to pixel grid)
  - Background color per-cell (not just a solid window background)
  - Cursor rendering
These are all Phase 3+ concerns. The pipeline is correct.
