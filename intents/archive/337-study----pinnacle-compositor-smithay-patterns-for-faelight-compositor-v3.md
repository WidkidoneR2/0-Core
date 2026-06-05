---
id: 337
title: "Study -- Pinnacle compositor Smithay patterns for faelight-compositor v3"
status: complete
date: 2026-05-25
tags: [study, pinnacle, smithay, compositor, wayland, lua, rust, awesomewm]
---

## What Is Pinnacle

Pinnacle (https://github.com/pinnacle-comp/pinnacle) is a Smithay-based Wayland
compositor inspired by AwesomeWM. Configured in Lua or Rust. Actively maintained.

This is the most relevant external project to faelight-compositor because:
1. Same foundation: Smithay
2. Same goal: custom Wayland compositor in Rust
3. Further along: they have solved problems we are still working through
4. AwesomeWM inspiration: keyboard-driven, programmable -- aligns with Forest Candy + i3 vision

## Why Study It

faelight-compositor v2 is running on real hardware. The next step (INT-323,
faelight-compositor v3) is full session authority -- replacing Niri permanently.

Pinnacle has already solved:
- Full XDG shell protocol implementation
- Window management with tiling and floating layouts
- Lua configuration API (reconfigure without recompile)
- Input handling and keybind system
- Multi-monitor support
- Layer shell (for bar, notifications)

Studying Pinnacle prevents us from reinventing what they have already proven.

## What To Study

1. **How Pinnacle structures state** -- the main compositor state struct and
   how it organizes windows, outputs, inputs
2. **The Lua API design** -- how they expose compositor behavior to configuration
   without sacrificing safety
3. **Window management** -- tiling algorithm, focus management, workspace model
4. **Input handling** -- how keybinds are registered and dispatched
5. **Layer shell implementation** -- how faelight-bar would run as a layer surface
6. **VBlank and frame timing** -- how they handle the render loop
7. **Multi-output** -- how they handle multiple monitors

## What We Build (After Study)

faelight-compositor v3 gains:
- Stable window management (tiling + floating, forest-colored borders)
- Layer shell for faelight-bar and faelight-notify
- Keybind system configurable from state.db (not hardcoded)
- Multi-monitor awareness
- VT switch stability

The Lua configuration approach is interesting but not forest-aligned.
The forest configures through state.db and fsh commands, not Lua scripts.
However the API design pattern -- separating policy from mechanism -- is worth borrowing.

## Gates

✅ Pinnacle source cloned and studied -- findings documented in intent file 2026-05-26
✅ State { backend, pinnacle } split documented -- 40+ protocol handlers in main state 2026-05-26
✅ Tag system noted -- window.rs 971 lines, layout.rs 587 lines -- detailed in INT-343 2026-05-26
✅ Snowcap studied -- SCTK + Iced + wlr-layer-shell -- SnowcapLayer pattern documented 2026-05-26
✅ BindState with layer_stack (i3 modes), ModMask, bind metadata (group/desc) documented 2026-05-26
⏸ VBlank/frame timing -- deferred: INT-343 Phase 1 -- approved by: christian 2026-05-26
⏸ Multi-output -- deferred: INT-343 -- approved by: christian 2026-05-26
✅ 3 patterns identified: state split, BindState layer-stack, bind metadata→state.db 2026-05-26
⏸ faelight-compositor v3 scaffold -- deferred: INT-343 -- approved by: christian 2026-05-26
⏸ Layer shell for faelight-bar -- deferred: INT-344 -- approved by: christian 2026-05-26
⏸ Keybind→state.db -- deferred: INT-343 -- approved by: christian 2026-05-26

## Study Findings (2026-05-26)

### Pinnacle Overview
- v0.2.3 (Feb 2026), 576 stars, 1,778 commits, actively maintained
- 73.3% Rust, 25.7% Lua, 10,588 lines in src/ alone
- Built on Smithay -- same foundation as faelight-compositor
- Explicitly credits Niri for rendering patterns (validates our Niri→Pinnacle path)
- Has Nix flake -- NixOS migration aligned

### Pattern 1: Two-Level State Split
```rust
pub struct State {
    pub backend: Backend,  // udev (real hardware) OR winit (testing)
    pub pinnacle: Pinnacle,  // all compositor logic here
}
```
Forest adoption: `State { backend: Backend, faelight: Faelight }`
This separates backend concerns from compositor logic cleanly.
Testing becomes possible (winit backend in CI).

### Pattern 2: Protocol Handlers in Main State
All Smithay protocol handlers live directly in the Pinnacle struct:
```rust
pub layer_shell_state: WlrLayerShellState,
pub xdg_shell_state: XdgShellState,
pub seat_state: SeatState<State>,
// ... 40+ protocol states
```
The key ones for faelight-compositor v3:
- `WlrLayerShellState` -- required for faelight-bar as layer surface
- `XdgShellState` -- regular windows
- `SeatState` -- input routing
- `XWaylandShellState` -- if Xwayland support needed

### Pattern 3: BindState -- i3 Mode System
```rust
pub struct BindState {
    pub layer_stack: Vec<String>,  // i3-like modes
    pub keybinds: Keybinds,
    pub mousebinds: Mousebinds,
}
```
Mode switching:
```rust
bind_state.enter_layer("resize")    // enter resize mode
bind_state.enter_previous_layer()   // pop back
bind_state.current_layer()          // what mode are we in?
```
Each bind has metadata:
- `group: String` -- for cheatsheet grouping (feeds INT-260!)
- `desc: String` -- human description (feeds INT-260!)
- `is_quit_bind: bool`
- `allow_when_locked: bool`
- Edge::Press / Edge::Release -- trigger on press OR release
ModMask with Option<bool> per modifier -- None means "any state".

### Pattern 4: Bind ID System
```rust
static BIND_ID_COUNTER: AtomicU32 = AtomicU32::new(0);
```
Global atomic counter for unique bind IDs.
IndexMap for ordered storage (maintains insertion order for cheatsheet).
UnboundedSender/Receiver for async callback delivery.

### Snowcap -- Layer Shell Widget System
Snowcap is Pinnacle's faelight-bar equivalent. Stack:
- smithay-client-toolkit (SCTK) -- Wayland client protocols
- Iced -- GUI rendering (SAME as faelight-bar v2!)
- wlr-layer-shell -- compositor-native layer surfaces
- calloop -- event loop (same as Smithay server side)
- gRPC -- IPC between snowcap process and Pinnacle compositor

Key types:
```rust
pub struct SnowcapLayer {
    pub surface: SnowcapSurface,  // Iced surface
    pub layer: LayerSurface,       // SCTK layer surface
    output_size: iced::Size<u32>,
}
```
Snowcap runs as a SEPARATE PROCESS -- connects to compositor via Wayland.
Uses wlr-layer-shell to anchor to screen edges (top/bottom/left/right).
Forest: faelight-bar is already a separate process using Iced. This confirms the pattern.

### What faelight-bar needs for layer-shell (from Snowcap)
faelight-bar already has: Iced, Wayland client
faelight-bar needs to ADD:
1. `smithay-client-toolkit` with wlr-layer-shell feature
2. `LayerShell::bind()` to create layer surfaces
3. `Anchor::TOP` to pin bar to top of screen
4. `Layer::Top` for z-ordering (above windows, below overlay)
5. `exclusive_zone(height)` to push windows down

### 3 Patterns Directly Applied to faelight-compositor v3
1. State split: `State { backend: Backend, faelight: FaelightCompositor }`
2. BindState with layer_stack for i3-mode keybinds
3. Bind metadata (group, desc) → state.db → INT-260 Cheatsheet TUI

### New Intent Created: INT-343 -- faelight-compositor v3 Pinnacle-Informed
See INT-343 for the detailed build plan.

### New Intent Created: INT-344 -- faelight-bar layer-shell upgrade
See INT-344 for the faelight-bar layer-shell conversion plan.
