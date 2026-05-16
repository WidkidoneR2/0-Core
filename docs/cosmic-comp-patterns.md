# cosmic-comp Architecture Patterns
# Studied: 2026-05-16 -- INT-287 Phase 1
# Source: github.com/pop-os/cosmic-comp (depth=1 clone)

---
## Pattern 1: State Ownership Model

cosmic-comp owns ALL state in one `State` struct (state.rs, 1408 lines).
Substates: Common, BackendData (KmsState | WinitState | X11State)

Key insight: no global mutable state anywhere.
Everything flows through `State` passed as &mut to event handlers.

Forest application:
  faelight-compositor: adopt single State struct pattern
  fsh: core + Friday state already follows this -- reinforce it
  faelight-fm v2: single AppState owns all file manager state

Protocols implemented in State (all relevant to forest):
  OverlapNotifyState    → faelight-notify v5 (overlap-aware notifications)
  ToplevelInfoState     → fsh can query all open windows
  ToplevelManagementState → fsh can manage windows programmatically
  WorkspaceState        → faelight-compositor workspace model
  OutputConfigurationState → multi-monitor support
  CursorShapeManagerState → GPU cursor rendering

---
## Pattern 2: Backend Auto-Detection

cosmic-comp selects backend via COSMIC_BACKEND env var:
  x11   → X11 backend (testing inside X)
  winit → winit backend (testing inside Wayland)
  kms   → KMS/DRM backend (real hardware)

Auto-detection logic (backend/mod.rs):
  If DISPLAY or WAYLAND_DISPLAY set → try x11, fallback to winit
  Otherwise → kms (bare metal)

Forest application:
  faelight-compositor: adopt FAELIGHT_BACKEND env var
  FAELIGHT_BACKEND=winit → test inside Niri (current)
  FAELIGHT_BACKEND=kms   → bare metal (INT-308 Phase 4)
  Auto-detect: if WAYLAND_DISPLAY set → winit, else kms

KMS backend structure (backend/kms/):
  device.rs   → DRM device management
  surface/    → per-CRTC rendering surfaces
  render/     → GPU render pipeline
  socket.rs   → Wayland socket creation

---
## Pattern 3: IPC via zbus + calloop bridge

cosmic-comp uses zbus (Rust D-Bus) for system integration:
  dbus/power.rs    → power management (suspend/resume)
  dbus/logind.rs   → session management
  dbus/mod.rs      → calloop::channel bridges async zbus to event loop

The bridge pattern:
  let (tx, rx) = calloop::channel::channel();
  block_on(async_zbus_call()) → tx.send(result)
  evlh.insert_source(rx, |event, _, state| { handle(event, state) })

Forest application:
  INT-294 (Forest Event Bus v2): adopt this exact bridge pattern
  faelight-notify v5: zbus for D-Bus notification protocol
  faelight-compositor: logind session management via zbus
  fsh: system signals (suspend/resume/power) via zbus → friday_knowledge

---
## Pattern 4: Workspace Model

workspace.rs (2175 lines) uses:
  TilingLayout + FloatingLayout -- two layout modes, runtime switchable
  id_tree::Tree -- workspace tree structure
  IndexSet -- ordered unique window set per workspace
  Animation via keyframe crate (EaseInOutCubic)

Forest application:
  faelight-compositor: adopt TilingLayout pattern for INT-308 Phase 3
  F-DWL (INT-290): full workspace model based on this
  faelight-fm v2: panel layout uses similar tiling concepts

---
## Pattern 5: OverlapNotifyState (critical for notify)

cosmic-comp implements overlap detection for layer-shell surfaces.
When a panel overlaps a window, surfaces are notified.

Forest application:
  faelight-notify v5 (INT-301): use OverlapNotifyState so notifications
  auto-dodge windows. Compositor-native, not a hack.
  faelight-bar v3 (INT-295): layer-shell surface with overlap awareness.

---
## Impact on Existing Intents

INT-308 (faelight-compositor v2):
  Phase 1: implement ToplevelInfoState, OverlapNotifyState, CursorShapeManagerState
  Phase 4: adopt FAELIGHT_BACKEND auto-detection for safe KMS testing

INT-293 (faelight-fm v2):
  Use single AppState pattern from cosmic-comp state.rs
  libcosmic widget toolkit for the visual layer

INT-295 (faelight-bar v3):
  layer_map_for_output pattern from cosmic-comp
  OverlapNotifyState for window-aware positioning

INT-301 (faelight-notify v5):
  OverlapNotifyState -- compositor-native overlap detection
  zbus bridge for D-Bus notification protocol

INT-294 (Forest Event Bus v2):
  calloop::channel + zbus bridge -- adopt exactly

faelight-menu v2 (new intent needed):
  grabs/ pattern from cosmic-comp for pointer grab on menu open
  ToplevelManagementState for window focus after selection

---
## eww Assessment

eww is a widget system for X11/Wayland desktops.
It renders widgets as layer-shell surfaces using GTK.

After studying cosmic-comp:
  eww is NOT needed for the forest.
  libcosmic does everything eww does, but in Rust, with our stack.
  layer_map_for_output + our own widget rendering = no GTK dependency.
  eww would be a foreign dependency in a forest that owns its stack.

Decision: eww is NOT adopted. libcosmic is the widget layer.
This is consistent with the 0-Core philosophy: own the stack.
