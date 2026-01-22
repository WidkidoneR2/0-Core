# workspace-view v1.0.0

🔍 **Sway Workspace Intelligence** - X-ray vision for your window manager.

## Philosophy

> "Understanding over convenience"  
> See your workspace state at a glance without switching between them.

## The Problem

- "What apps are on workspace 2?"
- "Where did I leave that terminal?"
- "Which workspace is active?"
- Switching workspaces just to check what's there

## The Solution

workspace-view gives you real-time intelligence about all your Sway workspaces.

## Features

- 🔍 Real-time workspace status
- 📱 Window detection with app_id
- 📂 Terminal working directories (cwd detection for foot)
- 🎯 Focused window highlighting
- 📊 Window counts per workspace
- 🎨 Multiple output modes (default, active, summary, json)
- 🌈 Color-coded active/visible workspaces
- 🌳 Tree-structured display
- 🔇 Empty workspace detection

## Usage

### Quick Reference
```bash
workspace-view              # Default detailed view
workspace-view --active     # Quick glance at current workspace
workspace-view --summary    # Compact one-line per workspace
workspace-view --json       # Machine-readable output
```

### Aliases (Recommended)
```bash
alias ws='workspace-view'
alias wsa='workspace-view --active'
alias wss='workspace-view --summary'
```

## Output Modes

### Default View

Shows all non-empty workspaces with full details:
```
🌲 Workspace Overview
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Workspace 1 [ACTIVE] 🟢 (2 windows)
  ├─ foot - ~/0-core/rust-tools/workspace-view ●
  └─ brave-browser - Claude - Brave

Workspace 2 (3 windows)
  ├─ foot - ~/projects/faelight
  ├─ foot - ~/Downloads
  └─ code - VSCode - myproject

Workspaces 3-10
  └─ (empty)
```

**Legend:**
- `[ACTIVE] 🟢` - Currently focused workspace
- `[VISIBLE]` - Visible on a monitor but not focused
- `●` - Currently focused window
- `~/path` - Terminal working directory (foot only)

### Active Workspace (`--active`)

Shows only the currently focused workspace:
```
🌲 Active Workspace
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Workspace 1 [ACTIVE] 🟢 (2 windows)
  ├─ foot - ~/0-core/rust-tools/workspace-view ●
  └─ brave-browser - Claude - Brave
```

**Use case:** Quick check of current workspace state

### Summary View (`--summary`)

Compact one-line per workspace:
```
🌲 Workspaces Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1 → 2 windows (foot, brave-browser) [ACTIVE]
2 → 3 windows (foot, foot, code)
3 → (empty)
```

**Use case:** Fast overview of all workspaces

### All Workspaces (`--all`)

Shows every workspace including empty ones with full details:
```
🌲 Workspace Overview
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Workspace 1 [ACTIVE] 🟢 (2 windows)
  ├─ foot - ~/0-core ●
  └─ brave-browser - Claude

Workspace 2
  └─ (empty)

Workspace 3
  └─ (empty)
...
```

**Use case:** Comprehensive workspace audit

### JSON Output (`--json`)

Machine-readable format for scripting:
```json
[
  {
    "num": 1,
    "visible": true,
    "focused": true,
    "window_count": 2,
    "windows": [
      {
        "app_id": "foot",
        "name": "foot",
        "cwd": "~/0-core/rust-tools/workspace-view",
        "pid": 12345,
        "focused": true
      },
      {
        "app_id": "brave-browser",
        "name": "Claude - Brave",
        "focused": false
      }
    ]
  }
]
```

**Use case:** Integration with scripts, status bars, automation

## Common Workflows

### Morning Workspace Check
```bash
# See what you left open yesterday
ws

# Or quick summary
wss
```

### Find That Terminal
```bash
# Where did I leave the terminal in ~/projects?
ws | grep projects

# Or use JSON + jq
ws --json | jq '.[] | select(.windows[].cwd | contains("projects"))'
```

### Monitor Active Workspace
```bash
# Quick check current workspace
wsa

# Or in a loop for continuous monitoring
watch -n 1 workspace-view --active
```

### Integrate with Status Bar
```bash
# Get focused workspace number for waybar/i3bar
ws --json | jq -r '.[] | select(.focused == true) | .num'

# Get window count for active workspace
ws --json | jq -r '.[] | select(.focused == true) | .window_count'
```

## Technical Details

### Terminal Working Directory Detection

workspace-view automatically detects working directories for `foot` terminals by:

1. Getting the terminal's PID from Sway tree
2. Finding child process (usually shell)
3. Reading `/proc/<pid>/cwd` symlink
4. Shortening `$HOME` to `~` for display

**Supported:** foot terminal  
**Not supported:** Other terminals (PRs welcome!)

### Window Detection

Uses `swaymsg -t get_tree` to:
- Parse Sway's tree structure
- Extract window information (app_id, name, pid)
- Detect focused state
- Group by workspace number

### Performance

- **Execution time:** ~50-100ms (depends on workspace/window count)
- **Memory:** Minimal (< 5MB)
- **Dependencies:** Only requires `swaymsg` (included with Sway)

## Flags
```
--active, -a     Show only the active workspace
--all            Show all workspaces including empty ones
--summary, -s    Compact summary view
--json           Machine-readable JSON output
--version, -v    Show version
--health         Run health check
--help, -h       Show this help message
```

## Integration Examples

### Waybar Custom Module
```json
"custom/workspaces": {
  "exec": "workspace-view --json | jq -r '.[] | select(.focused) | \"\\(.num): \\(.window_count) windows\"'",
  "interval": 2,
  "format": "🌲 {}"
}
```

### dmenu/rofi Workspace Switcher
```bash
# List workspace numbers with window counts
ws --json | jq -r '.[] | "\(.num): \(.window_count) windows"' | dmenu
```

### Find Window Script
```bash
#!/bin/bash
# find-window.sh - Find which workspace has a specific app
APP=$1
ws --json | jq -r ".[] | select(.windows[].app_id == \"$APP\") | .num"
```

Usage:
```bash
./find-window.sh brave-browser  # → 1
swaymsg workspace 1              # Switch to it
```

## Design Principles

### 1. Zero Configuration
Works immediately on any Sway setup. No config files needed.

### 2. Fast & Lightweight
Single binary, minimal dependencies, instant execution.

### 3. Multiple Interfaces
Terminal UI for humans, JSON for scripts. Choose your interface.

### 4. Non-Invasive
Read-only. Never modifies workspace state. Observe, don't change.

### 5. Terminal-Aware
Special handling for terminals (cwd detection) because they're special.

## Limitations

- Only works with Sway window manager
- Terminal cwd detection only for `foot` (other terminals need adding)
- Requires `swaymsg` in PATH
- No historical tracking (shows current state only)
- No window movement suggestions (may add in future)

## Exit Codes

- `0` - Success
- `1` - Error (swaymsg failed, invalid args, etc.)

## Troubleshooting

### "Failed to run swaymsg"
**Cause:** Not running under Sway, or swaymsg not in PATH  
**Fix:** Only works in Sway sessions

### Empty workspaces not showing
**Cause:** Using default view (hides empty workspaces)  
**Fix:** Use `--all` flag to show all workspaces

### Terminal cwd not showing
**Cause:** Not using foot terminal, or insufficient permissions  
**Fix:** Currently only foot is supported. Other terminals show window name instead.

### Colors not showing
**Cause:** Terminal doesn't support ANSI colors  
**Fix:** Use `--json` mode for color-free output

## Comparison with Alternatives

### vs `swaymsg -t get_workspaces`
- ✅ workspace-view: Human-readable, formatted output
- ❌ swaymsg: Raw JSON only

### vs `swaymsg -t get_tree`
- ✅ workspace-view: Organized by workspace, filtered, colored
- ❌ get_tree: Massive unstructured JSON dump

### vs Manual Switching
- ✅ workspace-view: See all at once, no switching
- ❌ Manual: Must visit each workspace individually

## Future Possibilities

- 🎯 Watch mode (`--watch`) for continuous updates
- 🎯 Window layout visualization (tiling patterns)
- 🎯 Workspace usage statistics
- 🎯 Move suggestions (recommend where to move windows)
- 🎯 Support for more terminals (kitty, alacritty, etc.)
- 🎯 Floating vs tiling window indication
- 🎯 Scratchpad window detection

## Philosophy in Practice

**Scenario:** You have 10 workspaces but don't remember where you left your terminal.

**Without workspace-view:**
- $mod+1 (check workspace 1)
- $mod+2 (check workspace 2)
- $mod+3 (check workspace 3)
- ...

**With workspace-view:**
```bash
ws | grep foot
# → Workspace 3 (1 window)
#     └─ foot - ~/projects/important-project ●
```

One command. Instant answer.

**This is intentional stewardship.**

---

**See your workspaces. Know your system. 🌲**
