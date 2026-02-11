# 🔒 faelight-lock v2.0.0

**Themed screen locker for Wayland - swaylock with Faelight Forest colors**

A simple wrapper around swaylock that automatically applies your Faelight Forest theme colors for visual consistency across your desktop.

## Features

✅ **Auto-themed** - Pulls colors from faelight-core Theme  
✅ **One Command** - Just run `faelight-lock`, colors are automatic  
✅ **Swaylock Integration** - All swaylock features work  
✅ **Health Check** - Verifies swaylock installation  
✅ **Custom Messages** - Display personalized lock screen text  
✅ **Grace Period** - Unlock window for quick return  
✅ **Urgent Lock** - Skip grace for instant lock
✅ **Consistent UX** - Matches your bar, menus, and apps  
✅ **Zero Config** - Works out of the box

## Usage
```bash
# Lock screen

# Quick Wins (v2.1.0)
faelight-lock --message "Back at 3pm"
faelight-lock --grace 30  # 30 second grace period
faelight-lock --urgent    # Skip grace, lock immediately
faelight-lock

# Lock screen (from keybind)
bindsym $mod+Escape exec faelight-lock

# Health check
faelight-lock --health-check

# Show version
faelight-lock --version
```

## Installation

### From 0-Core Workspace
```bash
cd ~/0-core
cargo build --release -p faelight-lock
cargo install --path rust-tools/faelight-lock
```

Binary installs to: `~/.cargo/bin/faelight-lock`

### Dependencies
```bash
# Install swaylock (Arch Linux)
sudo pacman -S swaylock

# Verify installation
which swaylock
faelight-lock --health-check
```

### Sway Integration
```bash
# Add to ~/.config/sway/config

# Lock screen keybind
bindsym $mod+l exec faelight-lock

# Auto-lock after 5 minutes idle
exec swayidle -w \
    timeout 300 'faelight-lock' \
    timeout 600 'swaymsg "output * dpms off"' \
    resume 'swaymsg "output * dpms on"'

# Lock before suspend
exec swayidle -w \
    before-sleep 'faelight-lock'
```

## How It Works

### Theme Integration
1. Loads Faelight Forest theme from `faelight-core`
2. Extracts colors: background, accent, text, danger
3. Converts RGB to hex format for swaylock
4. Passes themed arguments to swaylock command

### Color Mapping
```rust
bg_primary     → Background, inside colors
accent         → Ring color (normal state)
accent_hover   → Ring color (clear state)
text_primary   → Text color
danger         → Ring/text color (wrong password)
```

### Swaylock Arguments
```bash
swaylock -f \
  --color <bg> \
  --inside-color <bg> \
  --ring-color <accent> \
  --key-hl-color <accent> \
  --text-color <text> \
  --ring-wrong-color <danger> \
  --text-wrong-color <danger> \
  --indicator-radius 100 \
  --indicator-thickness 10
```

## Examples

### Manual Lock
```bash
# Lock immediately
faelight-lock

# Lock with notification
notify-send "Locking screen..." && sleep 1 && faelight-lock
```

### Auto-lock on Idle
```bash
# Install swayidle
sudo pacman -S swayidle

# Add to sway config
exec swayidle -w \
    timeout 300 'faelight-lock' \
    timeout 310 'swaymsg "output * dpms off"' \
    resume 'swaymsg "output * dpms on"'
```

### Lock Before Suspend
```bash
# Ensure screen locks before system suspends
exec swayidle -w before-sleep 'faelight-lock'

# System suspend (will auto-lock first)
systemctl suspend
```

## Technical Details

- **Lines of Code:** 102
- **Dependencies:** clap, faelight-core
- **Wrapper Around:** swaylock
- **Platform:** Wayland (Sway)

### Why a Wrapper?

**Problem:** swaylock requires ~20 color arguments for theming

**Solution:** faelight-lock pulls colors from theme automatically

**Result:**
- Before: `swaylock --color 141711 --ring-color a3e36b ...` (long)
- After: `faelight-lock` (automatic)

## Troubleshooting

### "swaylock not found"
```bash
# Install swaylock
sudo pacman -S swaylock

# Verify
which swaylock
```

### "swaylock exited with error"
```bash
# Check swaylock works directly
swaylock --help

# Run with verbose output
swaylock -f --debug
```

### Colors Don't Match
```bash
# Verify theme loads
faelight-lock --health-check

# Check if faelight-core theme changed
# Rebuild if theme was updated
cd ~/0-core
cargo build --release -p faelight-lock
cargo install --path rust-tools/faelight-lock --force
```

## Comparison

| Feature | faelight-lock | Plain swaylock | i3lock |
|---------|---------------|----------------|--------|
| Wayland | ✅ Yes | ✅ Yes | ❌ X11 only |
| Auto-themed | ✅ Yes | ❌ Manual | ❌ Manual |
| Config needed | ❌ No | ✅ Yes | ✅ Yes |
| Consistency | ✅ Perfect | ⚠️ Manual | ⚠️ Manual |

## Configuration

Currently uses Faelight Forest theme colors. To customize:

**Option 1:** Modify faelight-core theme  
**Option 2:** Fork and hardcode your own colors  
**Option 3:** Wait for v3.0 config file support (planned)

## Known Issues

None - tool is simple and stable.

See [GitHub Issues](https://github.com/WidkidoneR2/0-Core/issues) if you find bugs.

## Roadmap

**v2.1:** Config file for custom colors  
**v2.2:** Multiple theme support  
**v3.0:** Custom lock screen images

## Part of 0-Core

One of 40 Rust tools in the Faelight Forest ecosystem.

**Philosophy:** Theme consistency through automation.

See: https://github.com/WidkidoneR2/0-Core
