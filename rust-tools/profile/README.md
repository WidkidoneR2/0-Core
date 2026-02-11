# profile v2.1.0

🎮 **System Profile Manager** - Context-aware system configuration switching.

## Philosophy

> "Your system should adapt to what you're doing, not the other way around."

`profile` lets you instantly switch between different system configurations (work, gaming, low-power) with a single command. Each profile can control power settings, VPN, notifications, and run custom activation/deactivation scripts.

## Core Concept

A profile is a configuration context that defines how your system should behave:

- **gaming** - GPU performance mode, notifications off, power unlocked
- **work** - VPN on, focus mode, balanced power
- **low-power** - Battery optimization, minimal background activity
- **default** - Balanced daily driver configuration

## Usage

### Switch Profiles
```bash
# Switch to gaming mode
profile gaming

# Output:
# 🔄 Switching Profile
#   From: 🏠 default
#   To:   🎮 gaming
#
# ℹ️  Deactivating default...
# ℹ️  Activating gaming...
# ✅ Switched to 🎮 gaming
```

### List Available Profiles
```bash
profile list

# Output:
# 📋 Available Profiles
#
#   ▶ 🏠 default (active)
#     Balanced daily driver
#   🎮 gaming
#     GPU performance, notifications off
#   💼 work
#     VPN on, focus mode
#   🔋 low-power
#     Battery optimization
```

### Check Current Status
```bash
profile status

# Output:
# 📊 Profile Status
#
#   Current: 🏠 default
#
#   System State:
#     Power: balanced
#     VPN: Disconnected
#     Notifications: Normal
```

### View Switch History
```bash
profile history

# Output:
# 📜 Profile History
#
#   2025-01-21 08:30:15 | default -> work
#   2025-01-21 12:45:22 | work -> default
#   2025-01-21 18:20:33 | default -> gaming
#   2025-01-21 23:15:44 | gaming -> default
```

### Edit Profiles
```bash
# Edit a profile
profile edit gaming

# Opens in $EDITOR
```

### Export/Import Profiles
```bash
# Export for sharing
profile export gaming
# ✅ Exported to: ~/0-core/profiles/exports/gaming.profile

# Import from file
profile import ~/Downloads/streaming.profile
# ✅ Imported profile: streaming
```

### Health Check
```bash
profile health

# Output:
# 🏥 profile v1.0.0 - Health Check
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#   Checking profile directory... ✅
#   Checking state tracking... ✅
#   Checking profiles... ✅ 4 profiles found
#   Checking current profile... ✅ default (valid)
#   Checking powerprofilesctl... ✅
#   Checking mullvad... ✅
#   Checking makoctl... ✅
#
# ✅ All systems operational
```

## Profile File Format

Profiles are stored in `~/0-core/profiles/*.profile`:
```ini
# Description: GPU performance, notifications off

[activate]
# Commands run when switching TO this profile
powerprofilesctl set performance
makoctl mode -a dnd
notify-send "🎮 Gaming Mode" "Performance unlocked"

[deactivate]
# Commands run when switching FROM this profile
powerprofilesctl set balanced
makoctl mode -r dnd
notify-send "🏠 Default Mode" "Back to normal"
```

## Built-in Profiles

### 🏠 default
Balanced daily driver configuration.

**Activates:**
- Balanced power profile
- Normal notifications
- Standard system settings

**Use for:** General computing, web browsing, development

### 🎮 gaming
Maximum performance for gaming.

**Activates:**
- Performance power profile
- Do Not Disturb mode
- GPU performance unlocked
- Background services minimized

**Use for:** Gaming, GPU-intensive tasks

### 💼 work
Focus mode with VPN protection.

**Activates:**
- VPN connection (Mullvad)
- Do Not Disturb mode
- Balanced power profile
- Work applications auto-launch

**Use for:** Professional work, video calls, focus sessions

### 🔋 low-power
Battery life optimization.

**Activates:**
- Power saver mode
- Screen brightness reduced
- Background sync disabled
- CPU governor set to powersave

**Use for:** Mobile computing, extending battery life

## Integration Examples

### Keybinding in Sway
```
# Quick profile switcher
bindsym $mod+p exec profile-selector  # Use with faelight-dmenu

# Direct profile switches
bindsym $mod+Shift+g exec profile gaming
bindsym $mod+Shift+w exec profile work
bindsym $mod+Shift+d exec profile default
```

### Auto-Switch Based on Power
```bash
#!/bin/bash
# Auto-switch to low-power when on battery

if acpi -a | grep -q "off-line"; then
    profile low-power
else
    profile default
fi
```

### Time-Based Switching
```bash
# Crontab: Switch to work mode at 9 AM
0 9 * * 1-5 profile work

# Switch to default at 6 PM
0 18 * * 1-5 profile default
```

## Creating Custom Profiles
```bash
# 1. Create new profile file
nvim ~/0-core/profiles/streaming.profile

# 2. Add configuration
cat > ~/0-core/profiles/streaming.profile << 'EOF'
# Description: Optimized for streaming and recording

[activate]
powerprofilesctl set performance
pactl set-sink-volume @DEFAULT_SINK@ 80%
obs --startstreaming
notify-send "📹 Streaming Mode" "Ready to stream"

[deactivate]
obs --stopstreaming
powerprofilesctl set balanced
notify-send "🏠 Default Mode" "Stream ended"
EOF

# 3. Test it
profile streaming
```

## Profile Composition

Profiles can call other commands for modular configuration:
```ini
[activate]
# Use other 0-core tools
faelight core unlock
faelight profile apply streaming-modules
systemctl --user start recording-suite

# Run custom scripts
~/scripts/setup-streaming-layout.sh
```

## System State Monitoring

`profile status` shows real-time system state:

- **Power profile** - Via `powerprofilesctl`
- **VPN status** - Via `mullvad status`
- **Notifications** - Via `makoctl mode`

Add custom monitoring by editing the status command!

## Export Format

Exported profiles include metadata:
```
# ═══════════════════════════════════════════════════════════
# 0-Core Profile Export
# Name: gaming
# Exported: 2025-01-21 10:30:00
# System: Arch Linux
# Version: v7.6.5
# ═══════════════════════════════════════════════════════════

[activate]
...
```

## Demo for Linus
```bash
# Show available profiles
profile list

# Show current status
profile status

# Switch to gaming mode
profile gaming

# Check system adapted
profile status

# View history
profile history

# Show health
profile health

# Edit a profile (show customization)
profile edit gaming
```

## Design Principles

1. **Instant Switching** - No delays, immediate context change
2. **Bidirectional** - Every profile can activate AND deactivate cleanly
3. **Composable** - Profiles can use other 0-core tools
4. **Auditable** - History tracking for debugging
5. **Portable** - Export/import for sharing configurations

## Why This Matters

Traditional approaches to system configuration:
- **Manual changes** - Error-prone, easy to forget to revert
- **Multiple tools** - `powerprofilesctl`, `mullvad`, `makoctl` separately
- **No history** - Can't track what changed when
- **No sharing** - Can't share configurations

`profile` provides:
- **One command** - `profile gaming` changes everything
- **Automatic cleanup** - Deactivation reverts changes
- **Full history** - Track every switch
- **Community sharing** - Export/import profiles

**This is context-aware computing.**

## Technical Details

- **Language:** Rust
- **Storage:** `~/.local/state/0-core/`
- **Profiles:** `~/0-core/profiles/*.profile`
- **Format:** INI-style with `[activate]` and `[deactivate]` sections
- **History:** Timestamped log file

## Roadmap

Current features are stable. Future enhancements:

- **Auto-detection** - Suggest profile based on running apps
- **Partial activation** - Activate specific sections only
- **Profile dependencies** - Parent/child profile inheritance
- **GUI selector** - Visual profile switcher
- **Cloud sync** - Sync profiles across machines

But these are v2.0+ features. v1.0 is production-ready.

## Why "Profile"?

Your system has different modes of operation. Gaming requires performance. Work requires VPN and focus. Battery life requires conservation.

`profile` makes switching between these contexts instant and automatic.

**This is adaptive computing.**
