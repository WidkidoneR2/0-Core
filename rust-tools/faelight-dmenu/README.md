# faelight-dmenu v2.0.0

🌲 **Intent-Aware Application Launcher** - Wayland-native dmenu with Intent Ledger integration.

## Philosophy

> "A launcher should understand your development workflow, not just your apps."

`faelight-dmenu` is the **only dmenu that integrates with your Intent Ledger**. It's not just an application launcher - it's a window into your entire development process.

## What Makes This Unique

### No Other dmenu Does This:

**1. Intent Ledger Integration**
Browse, search, and select from your documented intents directly from your launcher.
```bash
# Browse all intents
faelight-dmenu intents

# Filter by status
faelight-dmenu intents --status in-progress
faelight-dmenu intents --status complete

# Filter by category
faelight-dmenu intents --category philosophy
faelight-dmenu intents --category future

# Search by ID or title
faelight-dmenu intents --query rust
faelight-dmenu intents --query 066
```

**2. Triple-Mode Design**
- **stdin mode** - Classic dmenu (pipe anything)
- **apps mode** - Launch desktop applications
- **intents mode** - Browse Intent Ledger
- **commands mode** - Discover all faelight commands

**3. Wayland-Native**
Built from scratch for Wayland, not an X11 port.

**4. Faelight Aesthetic**
Matches your Faelight Forest theme perfectly with rounded corners and mint green accents.

## Usage

### Intent Mode (The Revolutionary Feature!)
```bash
# Browse all intents with visual status indicators
faelight-dmenu intents

# Output shows:
# ✓ 001 - Migrate to Arch Linux
# ● 064 - Implement faelight-dmenu v2.0
# ○ 065 - Add GPU profiling
# ✗ 023 - Try Hyprland (cancelled)

# Filter by status
faelight-dmenu intents --status complete      # Show only ✓
faelight-dmenu intents --status in-progress   # Show only ●
faelight-dmenu intents --status planned       # Show only ○

# Filter by category
faelight-dmenu intents --category decisions
faelight-dmenu intents --category experiments
faelight-dmenu intents --category philosophy

# Search for specific intents
faelight-dmenu intents --query dmenu          # Find dmenu-related intents
faelight-dmenu intents --query 066            # Find intent 066

# Use in scripts
SELECTED=$(faelight-dmenu intents --status in-progress)
echo "Selected intent: $SELECTED"
intent show "$SELECTED"
```

### Application Launcher Mode
```bash
# Launch applications
faelight-dmenu apps

# Shows all installed desktop applications
# Type to fuzzy search
# Enter to launch
# Escape to cancel
```

### Classic stdin Mode
```bash
# Pipe anything to it
ls | faelight-dmenu
find . -name "*.rs" | faelight-dmenu
git branch | faelight-dmenu

# Use with custom prompt
echo -e "Option 1\nOption 2\nOption 3" | faelight-dmenu -p "Choose:"
```

### Commands Mode
```bash
# Discover all faelight commands
faelight-dmenu commands

# Shows:
# faelight health
# faelight profile switch
# faelight intent list
# faelight core lock
# ... and all other faelight commands
```

## Keyboard Controls

- **Type** - Fuzzy search filter
- **↓ / ↑** - Navigate results
- **Enter** - Select
- **Escape** - Cancel
- **Backspace** - Delete character from search

## Integration Examples

### Keybinding in Sway
```
# Application launcher
bindsym $mod+d exec faelight-dmenu apps

# Intent browser
bindsym $mod+i exec faelight-dmenu intents

# Command palette
bindsym $mod+Shift+p exec faelight-dmenu commands
```

### Intent Workflow Script
```bash
#!/bin/bash
# Select an in-progress intent and show it

INTENT_ID=$(faelight-dmenu intents --status in-progress)

if [ -n "$INTENT_ID" ]; then
    intent show "$INTENT_ID"
fi
```

### Quick Intent Switcher
```bash
#!/bin/bash
# Switch between active work

INTENT=$(faelight-dmenu intents --status in-progress --category development)

if [ -n "$INTENT" ]; then
    echo "Now working on: $INTENT"
    # Open relevant files, etc.
fi
```

## Status Icons

The Intent mode uses visual indicators:

- ✓ **complete** - Intent successfully completed
- ● **in-progress** - Currently working on this
- ○ **planned** - Future work
- ✗ **cancelled** - Decided not to do
- ⊙ **deferred** - Postponed to later
- · **unknown** - No status set

## Design Features

### Visual Polish
- **Rounded corners** (12px radius)
- **Mint green border** (2px)
- **Dark forest green background** (semi-transparent)
- **Faelight color scheme** throughout

### Performance
- **Instant startup** (<50ms)
- **Fuzzy search** for fast filtering
- **Wayland-native** rendering

### Architecture
- **Rust** - Memory-safe, zero-cost abstractions
- **Wayland** - Modern compositor protocol
- **Intent integration** - Direct YAML parsing
- **XDG desktop** - Standard .desktop file support

## Why This Matters

Traditional launchers show you **what you can launch**.

`faelight-dmenu` shows you **what you're working on**.

### Example Workflow

**Morning ritual:**
```bash
# Check what you're working on
faelight-dmenu intents --status in-progress

# See what's planned next
faelight-dmenu intents --status planned

# Launch your tools
faelight-dmenu apps
```

**During development:**
```bash
# Quick reference to intent
faelight-dmenu intents --query <current-feature>

# Launch related tools
faelight-dmenu apps
```

**Planning session:**
```bash
# Review all future work
faelight-dmenu intents --status planned --category future

# Check philosophy/decisions
faelight-dmenu intents --category philosophy
```

## Demo for Linus
```bash
# Show basic app launcher
faelight-dmenu apps

# Show Intent integration (the killer feature!)
faelight-dmenu intents

# Filter in-progress intents
faelight-dmenu intents --status in-progress

# Search for specific intent
faelight-dmenu intents --query dmenu

# Show command discovery
faelight-dmenu commands

# Classic stdin mode
ls ~/0-core/rust-tools | faelight-dmenu
```

## Technical Details

- **Language:** Rust
- **Wayland Protocol:** layer-shell
- **Font Rendering:** fontdue
- **Intent Parsing:** Direct YAML frontmatter parsing
- **Desktop Files:** XDG .desktop specification
- **Search:** Fuzzy matching algorithm

## Comparison with Other Launchers

| Feature | faelight-dmenu | rofi | wofi | dmenu |
|---------|---------------|------|------|-------|
| Intent Integration | ✅ | ❌ | ❌ | ❌ |
| Wayland-Native | ✅ | ❌ | ✅ | ❌ |
| Fuzzy Search | ✅ | ✅ | ✅ | ❌ |
| Custom Theming | ✅ | ✅ | ✅ | ✅ |
| stdin Mode | ✅ | ✅ | ✅ | ✅ |
| Desktop Apps | ✅ | ✅ | ✅ | ❌ |
| Commands Discovery | ✅ | ❌ | ❌ | ❌ |

## Roadmap

Current features are stable. Future enhancements:

- **Intent actions** - Mark complete, edit, etc. from launcher
- **Git integration** - Show uncommitted changes
- **Project context** - Filter by current project
- **Custom icons** - Per-intent icons
- **Multi-select** - Select multiple items (--multi flag exists)

But these are v3.0+ features. v2.0 is production-ready.

## Why "Intent-Aware"?

Your Intent Ledger documents **why** you're building things. Your launcher should integrate with that. 

`faelight-dmenu` is the only launcher that understands your development philosophy, not just your installed applications.

**This is launcher-meets-workflow-tool.**
