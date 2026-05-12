---
id: 295
title: "faelight-bar v3 -- COSMIC panel study, ironbar, eww, i3status-rust, quickshell, libcosmic"
status: planned
date: 2026-05-12
tags: [faelight-bar, bar, cosmic, libcosmic, ironbar, eww, i3status-rust, quickshell, wayland, layer-shell]
---

faelight-bar v2 exists (INT-239) but was built with iced.
faelight-bar v3 is the forest-first status bar built with the full v3 stack.
It is Friday's face on the desktop.
It speaks the forest state at a glance.

---

WHY A NEW BAR

The current bar drops on shell reloads (race condition, no supervisor).
The current bar uses iced (not libcosmic -- different widget system).
The current bar polls state.db (not event-driven).
The current bar has no Friday integration.

v3 fixes all of this:
  libcosmic for the widget layer (same as FM)
  wlr-layer-shell protocol for proper desktop anchoring
  zbus D-Bus subscription (INT-294) for event-driven updates
  Friday signals appear in the bar center
  No polling -- pure event-driven architecture
  Supervisor/watchdog built in (INT-239 original goal)

---

STUDY SOURCES

IRONBAR (highest priority):
  Pure Rust Wayland bar
  Written for wlroots-based compositors (Sway, Hyprland, Niri)
  Modular widget system -- add/remove modules in config
  Source: github.com/JakeStanger/ironbar
  Study:
    src/bar.rs -- layer-shell surface setup
    src/modules/ -- how each widget is implemented
    src/config.rs -- configuration system
  Key patterns:
    wlr-layer-shell anchoring
    Multi-monitor support
    Module hot-reload
    GTK-based (we will use libcosmic instead)

EWW (reference):
  ElKowar's Wacky Widgets
  XML/Yuck configuration language
  Extremely flexible, script-driven
  Source: github.com/elkowar/eww
  Study:
    How it handles dynamic data (polling vs events)
    Widget composition model
    Wayland layer-shell integration
  Key insight:
    eww proves that a bar can be fully declarative
    Forest bar could be declarative via kdl config

I3STATUS-RUST (module reference):
  Status bar for i3/Sway written in Rust
  Rich module ecosystem (time, weather, git, battery, etc.)
  Source: github.com/greshake/i3status-rust
  Study:
    src/blocks/ -- each status block implementation
    How blocks refresh independently
    Error handling per module
  Key patterns:
    Block trait -- every widget implements the same interface
    Async refresh with configurable intervals
    Click handler per block

QUICKSHELL (future reference):
  QML-based shell toolkit for Wayland
  Most flexible of all -- full scripting
  Source: github.com/quickshell-mirror/quickshell
  Note: QML not Rust -- study patterns only, not implementation
  Key insight: reactive data binding model

COSMIC PANEL (primary implementation reference):
  The actual COSMIC Desktop panel
  Pure Rust, libcosmic, Wayland-native
  Applet system -- each widget is a separate process
  Source: github.com/pop-os/cosmic-panel
  Study:
    Panel surface setup (layer-shell)
    Applet IPC model
    How COSMIC panel subscribes to system state
  Key pattern: applet = separate process + D-Bus IPC

---

FAELIGHT-BAR v3 DESIGN

Three zones (from INT-239 original design, upgraded):

LEFT ZONE -- Core protection + system:
  Lock icon (red=unlocked, green=locked)
  Health % with trend arrow
  Git status (clean/dirty/ahead)
  Active workspace indicator

CENTER ZONE -- Friday's face:
  Active intent title (scrolling if long)
  Friday signal (appears when Friday has high-confidence signal)
  Signal fades after 5 seconds, center returns to intent
  Simulation accuracy indicator (shown when running)
  Deploy progress (when deploy is running)

RIGHT ZONE -- Time and system:
  Clock (time + date)
  Battery (Framework laptop -- important)
  Network status
  Audio volume

---

TECHNICAL ARCHITECTURE

Layer shell setup (wlr-layer-shell):
  Anchor: Top of screen
  Exclusive zone: bar height (prevents windows going under bar)
  Layer: Overlay (above all windows)
  Height: 28px

Widget system:
  Each zone is a libcosmic widget
  Zones communicate via internal channel (not D-Bus)
  Friday signals arrive via D-Bus subscription (INT-294)
  State stored in Arc<Mutex<BarState>>

Supervisor/watchdog:
  Bar runs as a systemd user service OR
  faelight-daemon supervises bar process
  If bar crashes: restart within 500ms
  No more bar dropping on shell reload

Friday integration:
  Subscribe to org.faelight.Forest.Friday.FridaySuggested
  Center zone animates briefly when Friday speaks
  Confidence shown as opacity (higher confidence = more opaque)
  Dismissed after 5 seconds or on click

---

IMPLEMENTATION PHASES

Phase 0 -- Study:
  Read ironbar source (layer-shell setup, module system)
  Read i3status-rust (block trait, async refresh)
  Read cosmic-panel (applet model, D-Bus)
  Gate: architecture document written

Phase 1 -- Layer shell surface:
  wlr-layer-shell anchored to top of screen
  libcosmic rendering pipeline
  Three zones visible
  Gate: bar appears on screen, anchored correctly

Phase 2 -- Static widgets:
  Clock (right zone)
  Health % (left zone)
  Active intent (center zone)
  Gate: all three zones show correct static data

Phase 3 -- Dynamic updates via D-Bus:
  Subscribe to forest health signals (INT-294)
  Subscribe to intent change signals
  Subscribe to deploy signals
  Gate: bar updates within 100ms of forest state change

Phase 4 -- Friday face:
  Subscribe to Friday suggestion signals
  Center zone animation when Friday speaks
  Confidence-gated display (only SUGGEST+ level shown)
  Gate: Friday signal appears in bar within 100ms

Phase 5 -- Supervisor:
  Systemd user service or faelight-daemon watchdog
  Bar restarts automatically if it crashes
  Gate: shell reload does not drop the bar

Phase 6 -- Polish:
  Battery widget (Framework laptop)
  Network status
  Audio volume
  Fractional scaling for HiDPI Niri
  Gate: bar looks identical to design at all resolutions

---

DEPENDS ON

INT-294 (Forest Event Bus v2) -- D-Bus signals
INT-292 (faelight-term v3 stable) -- proves the libcosmic stack
INT-239 (faelight-bar v2) -- reference implementation to supersede

---

GATES

[ ] Phase 0: ironbar, i3status-rust, cosmic-panel studied
[ ] Phase 1: layer-shell bar surface anchored to top of screen
[ ] Phase 2: clock, health, active intent all showing correctly
[ ] Phase 3: D-Bus subscription -- bar updates on forest state change
[ ] Phase 4: Friday signal appears in center zone
[ ] Phase 5: supervisor -- bar never drops on shell reload
[ ] Phase 6: battery, network, audio, HiDPI all working
[ ] Bar replaces faelight-bar v2 completely
[ ] Friday's face is always visible on the desktop

---

"The bar is the forest heartbeat.
At a glance: health, intent, Friday.
Always present. Never wrong.
v3 is the heartbeat the forest deserves." 🌲
