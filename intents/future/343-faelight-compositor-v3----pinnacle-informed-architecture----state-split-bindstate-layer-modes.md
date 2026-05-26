---
id: 343
title: "faelight-compositor v3 -- Pinnacle-Informed Architecture -- State Split, BindState, Layer Modes"
status: planned
date: 2026-05-26
tags: [compositor, wayland, smithay, pinnacle, state, keybinds, layer-shell, i3]
depends_on: [337, 323]
---

## Why This Intent Exists

INT-337 studied Pinnacle's architecture. Three patterns are ready to apply
directly to faelight-compositor:
1. State split (backend vs compositor logic)
2. BindState with i3-style layer modes
3. Bind metadata → state.db → INT-260 Cheatsheet TUI

faelight-compositor v2 runs on real hardware. v3 makes it architecturally sound
and replaces Niri permanently.

## The Three Pinnacle Patterns

### Pattern 1: State Split
```rust
// Pinnacle pattern
pub struct State {
    pub backend: Backend,   // udev OR winit
    pub pinnacle: Pinnacle,
}

// Forest adoption
pub struct State {
    pub backend: Backend,         // udev (real) OR winit (testing)
    pub faelight: FaelightState,  // all compositor logic
}
```
Benefits:
- Testing with winit backend (no real GPU needed)
- Clean separation: backend bugs vs compositor bugs
- Future: add headless backend for Friday automation

### Pattern 2: BindState with Layer Modes (i3 modes)
```rust
pub struct BindState {
    pub layer_stack: Vec<String>,  // current mode stack
    pub keybinds: Keybinds,        // IndexMap for ordered storage
    pub mousebinds: Mousebinds,
}

// Mode switching
bind_state.enter_layer("resize")
bind_state.enter_previous_layer()
bind_state.current_layer()  // None = normal mode
```
This is how i3 modes work. Enter "resize" mode, all keys temporarily
remap to resize actions. ESC pops back to previous mode.

Forest keybind modes:
- Normal (default)
- Resize -- Super+R activates, arrows resize, ESC exits
- Launcher -- Super+D activates, letters filter menu, ESC exits
- Session -- Power/logout/lock options

### Pattern 3: Bind Metadata → state.db
```rust
pub struct BindData {
    pub group: String,    // "Window", "Layout", "App", "Forest"
    pub desc: String,     // "Focus next window"
    pub id: u32,          // unique bind ID
    pub allow_when_locked: bool,
}
```
Every keybind registered → write to state.db keybinds table.
INT-260 (Cheatsheet TUI) reads from this table.
No hardcoded cheatsheet. Dynamic, always current.

## Architecture for faelight-compositor v3

### File Structure
faelight-compositor/src/
main.rs          -- entry point, backend selection
state.rs         -- State { backend, faelight } + FaelightState
handlers.rs      -- Smithay protocol handlers
input/
mod.rs          -- InputState { bind_state, libinput_state }
bind.rs         -- BindState, ModMask, keybind registration
window.rs        -- WindowElement, focus management
layout.rs        -- tiling algorithms
output.rs        -- multi-monitor
backend/
udev.rs         -- real GPU (DRM/KMS)
winit.rs        -- testing
protocol.rs      -- wlr-layer-shell, ext-workspace, etc.

### Protocol Requirements
Must implement:
- `WlrLayerShellState` -- for faelight-bar (layer surfaces)
- `XdgShellState` -- regular windows
- `SeatState` -- input routing
- `OutputManagerState` -- multi-monitor
- `XdgDecorationState` -- CSD/SSD negotiation
- `WlrDataControlState` -- clipboard (wl-clipboard)

Should implement (for Niri compatibility during migration):
- `ForeignToplevelManagerState` -- taskbar integration
- `ExtWorkspaceManagerState` -- workspace protocol

### Keybind → state.db Flow
User runs: core keybind add "Super+Return" "faelight-term" "App" "Open terminal"
→ Writes to state.db: keybinds table
→ faelight-compositor reads state.db on startup
→ Registers keybind with BindState
→ On keypress: executes action
→ INT-260 Cheatsheet reads keybinds table: always current

### Layer Modes Implementation
```rust
// Forest mode system (from Pinnacle BindState)
impl FaelightState {
    pub fn enter_mode(&mut self, mode: &str) {
        self.input.bind_state.enter_layer(mode.to_string());
        // Show mode indicator in faelight-bar
        self.notify_bar_mode(mode);
    }
    pub fn exit_mode(&mut self) {
        self.input.bind_state.enter_previous_layer();
        self.notify_bar_mode("");
    }
}
```

## Gates
- [ ] Phase 1: State { backend, faelight } split implemented, builds without regression
- [ ] Phase 2: BindState with layer_stack implemented -- i3 modes work
- [ ] Phase 3: Bind metadata written to state.db on registration
- [ ] Phase 4: INT-260 Cheatsheet reads from state.db -- always current
- [ ] Phase 5: WlrLayerShellState -- faelight-bar anchors as layer surface
- [ ] Phase 6: Resize mode working -- Super+R → resize, ESC → normal
- [ ] Phase 7: Replace Niri permanently -- all forest workflows work
- [ ] Final: faelight-compositor v3 is daily driver, Niri removed

## Note
Do this in R&D VM (INT-328) before touching daily system.
Pinnacle is GPL-3.0. faelight-compositor is clean-room -- we study patterns, not copy code.

---
"The compositor owns the screen.
The state owns the compositor.
Every keybind has a name.
Every mode has a purpose.
The forest configures itself." 🌲
