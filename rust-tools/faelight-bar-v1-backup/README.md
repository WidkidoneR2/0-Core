# 🌲 faelight-bar

Hand-coded Wayland status bar for Faelight Forest with Sway integration.

## Features

- **Manual Wayland Rendering** - Direct layer-shell implementation for maximum performance
- **Sway Integration** - Real-time workspace information via IPC
- **Manual Glyph Rendering** - Custom font rasterization for efficiency
- **Visual Indicators** - Color-coded status with circle indicators
- **Click Actions** - Interactive elements (profile, volume, VPN)

## Display Elements

**Left Side:**
- **Profile** (🌲) - Current system profile (gaming, work, low-power)
- **Workspaces** - Sway workspace indicators with active highlighting
- **Health** (● 80%) - System health percentage with color coding:
  - Green (●) - 80%+ healthy
  - Amber (●) - 50-79% needs attention
  - Red (●) - <50% critical
- **Lock Status**:
  - ● LCK (green) - Core is locked (immutable)
  - ○ UNL (amber) - Core is unlocked (editable)

**Center:**
- **Window Title** - Active window name

**Right Side:**
- **Time** - Current date and time (MMM DD HH:MM)
- **Battery** - Percentage and charging status
- **WiFi** - Network connection status
- **VPN** - Mullvad VPN connection (green=connected, red=disconnected)
- **Volume** - Audio level and mute status

## Click Actions

| Element | Action |
|---------|--------|
| Profile | Cycle through system profiles |
| Volume | Toggle mute/unmute |
| VPN | Connect/disconnect Mullvad |

## Usage
```bash
# Start the bar (usually auto-started by Sway)
faelight-bar

# Show version
faelight-bar --version

# Show help
faelight-bar --help

# Health check
faelight-bar --health-check
```

## Architecture

**Why Hand-Coded?**
- No framework overhead
- Direct control over rendering
- Optimized for performance
- Educational value - understanding Wayland from first principles

**Technical Stack:**
- `wayland-client` - Direct Wayland protocol bindings
- `smithay-client-toolkit` - Layer shell support
- `rusttype` - Manual glyph rasterization
- Sway IPC - Workspace and window information

**Rendering Pipeline:**
1. Create layer surface (overlay layer)
2. Allocate shared memory buffer
3. Rasterize glyphs to ARGB8888 buffer
4. Attach buffer to surface
5. Commit surface updates

## Configuration

Uses `faelight-core` for consistent theming across the Faelight Forest ecosystem.

## Dependencies

**Runtime:**
- Wayland compositor (Sway)
- HackNerdFont-Regular.ttf
- `wpctl` (volume control)
- `acpi` (battery info)
- `mullvad` (VPN status, optional)
- `doctor` (health monitoring)

## Version

v0.9.0 - Visual enhancements and CLI standardization

## Improvements in v0.9.0

- Added circle indicators for health and lock status
- Improved spacing and visual clarity
- Added `--version`, `--help`, `--health-check` flags
- Enhanced lock status display (● LCK / ○ UNL)
- Color-coded health indicators

## Future Enhancements (v2.0)

See Intent 066 for planned architectural improvements:
- Model/View separation for 70-90% CPU reduction
- Glyph caching
- Animated state transitions
- Intelligent redraw triggers
- Nerd Font icon rendering
