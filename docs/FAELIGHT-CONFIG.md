# ⚙️ Faelight Configuration

Typed TOML configuration for Faelight Forest. Configs are read-only inputs, not procedural scripts.

## Location
```
~/.config/faelight/
├── config.toml      # Global settings
├── profiles.toml    # Profile definitions
└── themes.toml      # Color themes
```

## Commands
```bash
faelight config validate   # Check all configs
faelight config show       # Display current settings
faelight config path       # Show config directory
faelight config edit       # Open in $EDITOR
```

## config.toml

Global system settings:
```toml
[system]
version = "6.3.0"
theme = "faelight-forest"
default_profile = "default"

[paths]
core_dir = "~/0-core"
scripts_dir = "~/0-core/scripts"
state_dir = "~/.local/state/faelight"

[health]
fail_on_warning = false
auto_check_on_unlock = true

[notifications]
enabled = true
timeout_ms = 5000
position = "top-right"

[bar]
refresh_ms = 500
show_vpn = true
show_battery = true
show_volume = true

[lock]
timeout_minutes = 10
show_clock = true
blur_background = false
```

## profiles.toml

Profile definitions:
```toml
[default]
description = "Balanced daily driver"
icon = "🏠"
vpn = true
cpu_governor = "balanced"
notifications = "all"
bar_modules = ["workspaces", "window", "clock", "volume", "battery", "vpn"]

[work]
description = "Focus mode with VPN"
icon = "💼"
vpn = true
cpu_governor = "balanced"
notifications = "focused"
auto_launch = ["tutanota-desktop", "slack"]

[gaming]
description = "Maximum performance"
icon = "🎮"
vpn = false
cpu_governor = "performance"
gpu_mode = "max"
notifications = "minimal"

[low-power]
description = "Battery optimization"
icon = "🔋"
vpn = false
cpu_governor = "powersave"
notifications = "minimal"
bar_refresh_ms = 2000
```

### Profile Fields

| Field | Type | Description |
|-------|------|-------------|
| `description` | string | Human-readable description |
| `icon` | string | Emoji icon |
| `vpn` | bool | Enable VPN |
| `cpu_governor` | string | `balanced`, `performance`, `powersave` |
| `notifications` | string | `all`, `focused`, `minimal`, `none` |
| `gpu_mode` | string | GPU power mode (optional) |
| `bar_modules` | array | Bar modules to show |
| `auto_launch` | array | Apps to launch on switch |
| `bar_refresh_ms` | int | Override bar refresh rate |

## themes.toml

Color theme definitions:
```toml
[faelight-forest]
description = "The original forest theme"

[faelight-forest.colors]
background = "#0f1411"
foreground = "#d7e0da"
accent = "#a3e36b"
accent_secondary = "#6be3a3"
highlight = "#5cc8ff"
warning = "#e3c76b"
error = "#e36b6b"
dim = "#778f7f"
border = "#a3e36b"
selected = "#2a3a25"

[faelight-forest.bar]
bg = "#141711"
fg = "#d7e0da"
accent = "#a3e36b"
separator = "#778f7f"

[faelight-forest.notify]
bg = "#141711"
fg = "#d7e0da"
border = "#a3e36b"

[faelight-forest.menu]
bg = "#141711"
fg = "#d7e0da"
border = "#a3e36b"
selected_bg = "#2a3a25"
```

### Faelight Forest Palette

| Name | Hex | Usage |
|------|-----|-------|
| Background | `#0f1411` | Main background |
| Foreground | `#d7e0da` | Main text |
| Accent | `#a3e36b` | Primary accent (green) |
| Secondary | `#6be3a3` | Secondary accent (teal) |
| Highlight | `#5cc8ff` | Info/links (blue) |
| Warning | `#e3c76b` | Warnings (yellow) |
| Error | `#e36b6b` | Errors (red) |
| Dim | `#778f7f` | Muted text |
| Selected | `#2a3a25` | Selection background |

## Validation

Configs are validated on:
- `faelight config validate`
- `dot-doctor` health check
- Any `faelight` command that loads config

Invalid TOML will show parse errors with line numbers.

## Stow Integration

Config files are stored in `~/0-core/config-faelight/` and symlinked via stow:
```bash
cd ~/0-core
stow config-faelight
```

---
_Configuration as data, not code._ 🌲
