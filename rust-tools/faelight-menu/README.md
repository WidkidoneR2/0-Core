# ⚡ faelight-menu

Power menu for Faelight Forest with Wayland layer-shell integration.

## Features

- **Text-only selection** - Clean UI without background bars
- **Arrow indicator** (▶) - Shows current selection
- **Red text warnings** - Dangerous actions highlighted in red
- **Confirmation required** - Shutdown/Reboot need double-confirmation
- **Keyboard shortcuts** - Fast access via single keys
- **Wayland native** - No X11 dependencies

## Usage
```bash
# Open power menu
faelight-menu

# Show version
faelight-menu --version

# Show help
faelight-menu --help

# Health check
faelight-menu --health-check
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑↓` | Navigate menu |
| `L` | Lock screen |
| `O` | Logout (exit Sway) |
| `S` | Suspend system |
| `R` | Reboot (requires confirmation) |
| `P` | Shutdown (requires confirmation) |
| `Enter` | Select/Confirm |
| `Esc` | Close menu |

## Menu Actions

1. **🔌 Lock** - Lock screen with swaylock
2. **🚪 Logout** - Exit Sway session
3. **💤 Suspend** - Suspend to RAM
4. **🔄 Reboot** - Reboot system (dangerous - requires confirmation)
5. **⏻ Shutdown** - Power off system (dangerous - requires confirmation)

## Visual Design
```
⚡ Power Menu
─────────────────
▶ 🔌  Lock        <- Selected (arrow indicator)
  🚪  Logout
  💤  Suspend
─────────────────
  🔄  Reboot
  ⏻   Shutdown

↑↓ or L/O/S/R/P  Enter Select  Esc Close
```

## Configuration

Uses `faelight-core` theme for consistent colors across the Faelight Forest ecosystem.

## Version

v0.6.0 - Production-ready power menu

## Dependencies

- Wayland compositor (Sway)
- swaylock (for lock action)
- systemctl (for power actions)
