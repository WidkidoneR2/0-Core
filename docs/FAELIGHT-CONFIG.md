# ⚙️ Faelight Configuration

Typed TOML configuration for Faelight Forest. Configurations are **read-only data**, not procedural scripts.

**Version:** 8.4.0  
**Format:** TOML  
**Validation:** Type-checked on load

---

## Location
```
~/.config/faelight/
├── config.toml      # Global settings
├── profiles.toml    # Profile definitions
└── themes.toml      # Color themes
```

**Source:** `~/0-core/stow/config-faelight/`

---

## Commands
```bash
# Validation (via faelight or doctor)
doctor                      # Validates configs as part of health check
faelight config validate    # Direct validation

# Viewing
cat ~/.config/faelight/config.toml
cat ~/.config/faelight/profiles.toml
cat ~/.config/faelight/themes.toml

# Editing (unlock core first)
unlock-core
nvim ~/0-core/stow/config-faelight/.config/faelight/config.toml
lock-core
```

---

## config.toml

Global system settings:
```toml
[system]
version = "8.4.0"
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

---

## profiles.toml

Profile definitions for different use cases:
```toml
[default]
description = "Balanced daily driver"
icon = "DEF"
vpn = true
cpu_governor = "balanced"
notifications = "all"
bar_modules = ["workspaces", "window", "clock", "volume", "battery", "vpn"]

[work]
description = "Focus mode with VPN"
icon = "WRK"
vpn = true
cpu_governor = "balanced"
notifications = "focused"
auto_launch = []

[gaming]
description = "Maximum GPU performance"
icon = "GAM"
vpn = false
cpu_governor = "performance"
gpu_mode = "max"
notifications = "minimal"

[low-power]
description = "Battery optimization"
icon = "LOW"
vpn = false
cpu_governor = "powersave"
notifications = "minimal"
bar_refresh_ms = 2000
```

### Profile Fields

| Field | Type | Description |
|-------|------|-------------|
| `description` | string | Human-readable description |
| `icon` | string | Text icon (3-letter code) |
| `vpn` | bool | Enable Mullvad VPN |
| `cpu_governor` | string | `balanced`, `performance`, `powersave` |
| `notifications` | string | `all`, `focused`, `minimal`, `none` |
| `gpu_mode` | string | GPU power mode (optional) |
| `bar_modules` | array | Bar modules to show |
| `auto_launch` | array | Apps to launch on switch |
| `bar_refresh_ms` | int | Override bar refresh rate |

### Active Profile

Current profile is displayed in:
- `faelight-bar` status bar (e.g., "DEF", "GAM")
- `profile list` command output

---

## themes.toml

Color theme definitions:
```toml
[faelight-forest]
description = "The original Faelight Forest theme"

[faelight-forest.colors]
background = "#0f1411"      # Forest Night
foreground = "#d7e0da"      # Fog White
accent = "#6be3a3"          # Faelight Green
accent_secondary = "#5cc8ff" # Faelight Blue
highlight = "#5cc8ff"       # Info/links
warning = "#f5c177"         # Amber Leaf
error = "#e36b6b"           # Error Red
dim = "#778f7f"             # Muted Gray
border = "#6be3a3"          # Faelight Green
selected = "#2a3a25"        # Selection Background

[faelight-forest.bar]
bg = "#0f1411"
fg = "#d7e0da"
accent = "#6be3a3"
separator = "#778f7f"

[faelight-forest.notify]
bg = "#0f1411"
fg = "#d7e0da"
border = "#6be3a3"

[faelight-forest.menu]
bg = "#0f1411"
fg = "#d7e0da"
border = "#6be3a3"
selected_bg = "#2a3a25"
```

---

## Faelight Forest Color Palette

| Name | Hex | Usage |
|------|-----|-------|
| **Forest Night** | `#0f1411` | Main background |
| **Fog White** | `#d7e0da` | Main text |
| **Faelight Green** | `#6be3a3` | Primary accent |
| **Faelight Blue** | `#5cc8ff` | Secondary accent |
| **Amber Leaf** | `#f5c177` | Warnings |
| **Error Red** | `#e36b6b` | Errors |
| **Muted Gray** | `#778f7f` | Dimmed text |
| **Selection** | `#2a3a25` | Selection background |

This palette is consistently applied across:
- Sway window manager
- faelight-bar status bar
- Foot terminal
- Neovim (Faelight theme)
- tuigreet login screen

---

## Validation

Configs are validated on:
1. `doctor` health check (Faelight Config check)
2. Any `faelight` command that loads config
3. Tool startup (individual tools validate their sections)

**Invalid TOML** will show parse errors with line numbers.

---

## Stow Integration

Config files are stored in the 0-Core repository and symlinked via GNU Stow:
```bash
# Source location
~/0-core/stow/config-faelight/.config/faelight/

# Symlinked to
~/.config/faelight/

# Deploy
cd ~/0-core/stow
stow -t ~ config-faelight

# Verify
ls -la ~/.config/faelight/
readlink ~/.config/faelight/config.toml
```

---

## Editing Workflow

1. **Unlock core:**
```bash
   unlock-core
```

2. **Edit source files:**
```bash
   cd ~/0-core/stow/config-faelight/.config/faelight
   nvim config.toml
```

3. **Validate:**
```bash
   doctor
```

4. **Commit changes:**
```bash
   cd ~/0-core
   git add stow/config-faelight/
   git commit -m "config: Update faelight settings"
   git push
```

5. **Lock core:**
```bash
   lock-core
```

---

## Philosophy

**Configuration as data, not code.**

- Configs are **declarative**, not imperative
- TOML enforces structure and types
- Validation happens at load time
- No procedural logic in configs
- Single source of truth for system state

*Understanding over convenience.* 🌲
