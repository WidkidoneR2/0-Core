---
id: 078
title: "Implement i3bar-style Modular Status Bar Architecture"
status: cancelled
category: future
created: 2026-02-07
tags:
  - faelight-bar
  - wayland
  - architecture
  - modular
dependencies:
  - "current faelight-bar rendering working"
priority: high
effort: 2-3 weeks
---

## Problem Statement

Current faelight-bar is monolithic:
- All status indicators compiled into the binary
- Adding features requires recompilation
- Icons don't render properly in Wayland
- No separation between bar daemon and status sources
- Difficult to debug individual components
- Community can't easily extend

This violates 0-Core principles of modularity and manual control.

## Vision

A modular status bar system inspired by i3bar but native to Wayland:
- **faelight-bar**: Core daemon handling rendering and Wayland layer-shell
- **status-blocks/**: External scripts providing status data via JSON
- **bar-config.toml**: Declarative configuration for blocks
- **Click protocol**: Bidirectional communication for interactivity

## Architecture

### Component Breakdown

1. **faelight-bar (Core Daemon)**
   - Responsibilities:
     - Wayland layer-shell window management
     - JSON protocol parsing
     - Script execution and management
     - Text rendering (no complex icons initially)
     - Click region detection and event routing
     - Config file parsing
   - Does NOT:
     - Fetch status data directly
     - Contain business logic for indicators
     - Handle specific system queries

2. **status-blocks/ (External Scripts)**
   - Location: `~/0-core/status-blocks/`
   - Language: Bash for simplicity, can be any language
   - Output: JSON to stdout
   - Input: JSON click events on stdin
   - Each script is independent and cacheable

3. **bar-config.toml (Configuration)**
   - Location: `~/.config/faelight-bar/config.toml`
   - Defines: Block order, update intervals, click actions
   - Example blocks: health, intents, updates, lock, time, workspaces

### JSON Protocol Specification

#### Block Output Format (Script → Bar)
```json
{
  "name": "block-name",           // Required: unique identifier
  "instance": "optional-instance", // Optional: for multiple instances
  "full_text": "Display Text",    // Required: what to show
  "short_text": "Short",          // Optional: abbreviated version
  "color": "#RRGGBB",             // Optional: text color (hex)
  "background": "#RRGGBB",        // Optional: background color
  "border": "#RRGGBB",            // Optional: border color
  "border_top": 1,                // Optional: border width (pixels)
  "border_right": 1,
  "border_bottom": 1,
  "border_left": 1,
  "min_width": 100,               // Optional: minimum width (pixels)
  "align": "left",                // Optional: left|center|right
  "urgent": false,                // Optional: attention flag
  "separator": true,              // Optional: show separator after
  "separator_block_width": 9,     // Optional: separator spacing
  "markup": "none"                // Optional: none|pango
}
```

#### Click Event Format (Bar → Script)
```json
{
  "name": "block-name",
  "instance": "optional-instance",
  "button": 1,                    // 1=left, 2=middle, 3=right, 4=scroll-up, 5=scroll-down
  "x": 1234,                      // Click coordinates relative to bar
  "y": 56,
  "modifiers": ["Shift", "Ctrl"]  // Keyboard modifiers held
}
```

### Configuration File Format
```toml
# ~/0-core/bar-config.toml

[bar]
position = "top"                  # top|bottom
height = 30                       # pixels
background = "#1a1a1a"
foreground = "#ffffff"
separator = " | "
font = "HackNerdFont 12"

# Left-aligned blocks
[[blocks]]
name = "health"
script = "~/0-core/status-blocks/health-indicator"
interval = 30                     # seconds
signal = 10                       # optional: update on signal USR1+10
align = "left"

[[blocks]]
name = "intents"
script = "~/0-core/status-blocks/intent-counter"
interval = 5
align = "left"

[[blocks]]
name = "updates"
script = "~/0-core/status-blocks/update-counter"
interval = 300
cache = "~/.cache/faelight/update-count"  # optional: use cached value
align = "left"

# Center-aligned blocks
[[blocks]]
name = "workspaces"
script = "~/0-core/status-blocks/workspace-indicator"
signal = 20                       # update on workspace change
align = "center"

# Right-aligned blocks
[[blocks]]
name = "lock"
script = "~/0-core/status-blocks/lock-status"
interval = 1
align = "right"

[[blocks]]
name = "time"
script = "~/0-core/status-blocks/time-display"
interval = 60
format = "%H:%M"                  # passed as --format argument
align = "right"
```

## Implementation Plan

### Phase 1: Core JSON Protocol (Week 1)

**Files to Create:**
- `rust-tools/faelight-bar/src/protocol.rs` - JSON protocol types
- `rust-tools/faelight-bar/src/config.rs` - Config file parsing
- `rust-tools/faelight-bar/src/block.rs` - Block data structure
- `rust-tools/faelight-bar/src/executor.rs` - Script execution engine

**Tasks:**
1. Define Rust structs for JSON protocol
   - `BlockOutput` struct matching JSON spec
   - `ClickEvent` struct matching click spec
   - Serde serialization/deserialization
   
2. Create config parser
   - Use `toml` crate
   - `BarConfig` struct
   - `BlockConfig` struct with validation
   
3. Build script executor
   - Spawn scripts as child processes
   - Capture stdout as JSON
   - Handle script failures gracefully
   - Implement interval-based updates
   - Implement signal-based updates

4. Testing
   - Unit tests for JSON parsing
   - Unit tests for config parsing
   - Integration test with mock script

**Deliverable:** Core types and execution engine working, tested with dummy scripts

### Phase 2: Status Block Scripts (Week 1-2)

**Directory Structure:**
```
~/0-core/status-blocks/
├── health-indicator      # Health dot (🟢🟡🔴)
├── intent-counter       # WIP count
├── update-counter       # Package updates
├── lock-status         # Lock state
├── time-display        # Current time
├── workspace-indicator # Active workspace
└── README.md           # Script development guide
```

**Script Template:**
```bash
#!/bin/bash
# status-blocks/template

# Read config/arguments
INTERVAL=${1:-60}

# Main loop (or run once)
while true; do
    # Gather data
    DATA=$(get_status)
    
    # Output JSON
    jq -n \
        --arg text "$DATA" \
        --arg color "#00ff00" \
        '{
            name: "block-name",
            full_text: $text,
            color: $color,
            separator: true
        }'
    
    # Wait for interval
    sleep "$INTERVAL"
done

# Click handler (read from stdin if running persistently)
while read -r event; do
    BUTTON=$(echo "$event" | jq -r '.button')
    case "$BUTTON" in
        1) handle_left_click ;;
        2) handle_middle_click ;;
        3) handle_right_click ;;
    esac
done
```

**Scripts to Create:**

1. **health-indicator**
   - Read: `~/.cache/faelight/health-status`
   - Output: 🟢 (100%), 🟡 (90-99%), 🔴 (<90%)
   - Click: Launch `doctor`

2. **intent-counter**
   - Command: `intent list --active | grep -c in-progress`
   - Output: `WIP N` or hidden if 0
   - Click: Launch `intent list`

3. **update-counter**
   - Read: `~/.cache/faelight/update-count`
   - Output: ` N` (red) or ` ✓` (white)
   - Click: Launch `faelight-update`

4. **lock-status**
   - Read: `~/.cache/faelight/core.lock`
   - Output: 🔒 (locked) or 🔓 (unlocked)
   - Click: Toggle lock state

5. **time-display**
   - Command: `date +"%H:%M"`
   - Output: Current time
   - Click: Open calendar

6. **workspace-indicator**
   - Command: `swaymsg -t get_workspaces`
   - Output: Active workspace name
   - Click: Workspace switcher

**Deliverable:** All status blocks working as standalone scripts

### Phase 3: Rendering Engine (Week 2)

**Files to Modify:**
- `rust-tools/faelight-bar/src/render/bar.rs` - Main rendering
- `rust-tools/faelight-bar/src/render/text.rs` - Text rendering
- `rust-tools/faelight-bar/src/render/layout.rs` - Block layout
- `rust-tools/faelight-bar/src/input.rs` - Click handling

**Tasks:**

1. Text Rendering from JSON
   - Parse `BlockOutput` JSON
   - Render `full_text` with `color`
   - Apply `background` and `border`
   - Handle `align` (left/center/right)
   - Respect `min_width`
   - Draw separators if `separator: true`

2. Layout Engine
   - Three regions: left, center, right
   - Dynamic sizing based on content
   - Separator spacing
   - Handle overflow gracefully

3. Click Region Mapping
   - Track bounding box for each block
   - Map click coordinates to block
   - Generate `ClickEvent` JSON
   - Send to script stdin
   - Handle script response

4. Update Management
   - Interval-based updates (spawn on timer)
   - Signal-based updates (listen for signals)
   - Efficient re-rendering (only changed blocks)
   - Debouncing for rapid updates

**Deliverable:** Full rendering pipeline working with real scripts

### Phase 4: Polish & Documentation (Week 2-3)

**Tasks:**

1. Pango Markup Support
   - Add `pango` feature flag
   - Parse Pango markup in `full_text`
   - Bold, italic, colors via markup
   - Fallback to plain text if parsing fails

2. Urgent/Attention States
   - Visual indicator for `urgent: true`
   - Blink or color change
   - System notification integration

3. Smooth Animations
   - Fade in/out for appearing/disappearing blocks
   - Smooth color transitions
   - Workspace switch animations (optional)

4. Error Handling
   - Script timeout handling (kill after 5s)
   - Script crash recovery (restart)
   - Invalid JSON handling (log and skip)
   - Visual indicator for broken blocks

5. Documentation
   - `status-blocks/README.md` - How to write blocks
   - `bar-config.toml.example` - Example config
   - Intent ledger update with final architecture
   - User guide in main README

**Deliverable:** Production-ready modular bar

## Technical Specifications

### Dependencies

**Rust Crates:**
- `serde_json` - JSON parsing
- `toml` - Config parsing
- `tokio` - Async runtime for script management
- Existing: `wayland-client`, `smithay-client-toolkit`

**External:**
- `jq` - Recommended for scripts (optional)
- No new system dependencies

### Performance Targets

- Bar startup: <100ms
- Script execution: <50ms each
- Re-render time: <16ms (60fps)
- Memory footprint: <10MB
- Click latency: <10ms

### Backwards Compatibility

- Keep current bar working during migration
- Feature flag: `--modular` to enable new mode
- Gradual migration of status blocks
- Old code removed after verification

## Testing Strategy

### Unit Tests
- JSON protocol serialization/deserialization
- Config file parsing with various inputs
- Block layout calculations
- Click region detection

### Integration Tests
- Mock scripts outputting test JSON
- Config file loading and validation
- Script execution and output capture
- Click event generation and routing

### Manual Testing
- All status blocks rendering correctly
- Click handlers working for each block
- Config changes applying without restart
- Script crashes handled gracefully
- Multiple monitor support

## Success Criteria

✅ All current bar features working as external scripts
✅ Config file controls all block behavior
✅ Click handlers work for all blocks
✅ Scripts can be added without recompiling
✅ Performance meets targets
✅ Documentation complete
✅ Zero regressions from current bar
✅ Community can create custom blocks

## Risks & Mitigation

**Risk:** Script performance issues
**Mitigation:** Cache files, efficient scripts, timeout handling

**Risk:** JSON parsing overhead
**Mitigation:** Benchmark, optimize, consider msgpack if needed

**Risk:** Wayland rendering complexity
**Mitigation:** Start with text-only, add features incrementally

**Risk:** Breaking existing bar users
**Mitigation:** Feature flag, parallel development, thorough testing

## Future Enhancements

- Pango markup support for rich formatting
- Icon font support (once Wayland rendering is solid)
- Custom click actions in config
- IPC for external bar control
- Tray icon protocol
- System tray support
- Plugin system for compiled blocks
- Community block repository

## References

- i3bar protocol: https://i3wm.org/docs/i3bar-protocol.html
- Wayland layer-shell: https://wayland.app/protocols/wlr-layer-shell-unstable-v1
- Current faelight-bar: `~/0-core/rust-tools/faelight-bar/`
- Starship prompt: Similar modular philosophy

## Notes

This represents a fundamental architectural shift from monolithic to modular design. The bar becomes a rendering engine, with all intelligence in external scripts. This aligns with 0-Core philosophy: manual control, understanding over convenience, and modularity over monoliths.

Status blocks will initially be bash scripts for rapid development, but can be replaced with compiled Rust binaries for performance-critical blocks later.
