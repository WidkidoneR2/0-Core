# 🦀 Faelight Forest Tools Reference

Complete reference for all custom Rust tools in 0-Core.

## Quick Reference

| Tool | Purpose | Common Usage |
|------|---------|--------------|
| `entropy-check` | Drift detection | `drift` or `entropy` |
| `faelight-launcher` | App launcher | `launcher` or `Super+D` |
| `faelight-menu` | Power menu | `powermenu` or `Super+Shift+E` |
| `faelight-notify` | Notifications | Runs as service |
| `faelight-bar` | Status bar | Runs as service |
| `dot-doctor` | Health checks | `doctor` or `health` |
| `intent` | Intent ledger | `intent list` |
| `profile` | Profile system | `prof` or `profile` |
| `core-diff` | Policy scanner | `cdiff` |
| `theme-switch` | Theme manager | `theme` |

---

## Core System Tools

### entropy-check v0.1

**Drift Detection - Prevents Silent System Changes**

**Purpose:** Detects configuration drift before it becomes catastrophic.

**Commands:**
```bash
entropy-check              # Check for drift
entropy-check --baseline   # Create/update baseline
entropy-check --json       # JSON output for automation
drift                      # Alias
entropy                    # Alias
```

**What it Checks:**
- **Config Drift:** Files modified outside stow (checksum mismatches)
- **Service Drift:** Services changed state unexpectedly
- **Binary Drift:** Package versions changed since last check
- **Symlink Drift:** New broken symlinks
- **Untracked Files:** New files in managed directories

**Example Output:**
```
📊 Drift Report - 0-Core v6.8.0
✅ Config Drift (0 files)
✅ Service Drift (0 changes)
✅ Binary Drift (0 changes)
✅ Symlink Drift (0 new breaks)
✅ Untracked Files (0 new)
✨ No drift detected. System stable.
```

**Baseline Location:** `~/.config/faelight/entropy-baseline.json`

---

### dot-doctor v0.8

**System Health Engine**

**Purpose:** Comprehensive health checks with dependency graph model.

**Commands:**
```bash
dot-doctor          # Run all checks
doctor              # Alias
health              # Alias
```

**Checks:**
- Stow symlinks
- System services
- Broken symlinks
- Binary dependencies
- Git repository status
- Theme packages
- Scripts
- Package metadata
- Intent ledger
- Profile system
- Faelight config

**Exit Codes:**
- `0` - All checks passed
- `1` - Warnings present
- `2` - Failures present

---

### intent v0.3

**Intent Ledger Management**

**Purpose:** Track decisions, experiments, and system evolution.

**Commands:**
```bash
intent list              # List all intents
intent show <id>         # Show specific intent
intent add               # Add new intent
intent search <term>     # Search intents
```

**Categories:**
- `decisions/` - Architectural decisions
- `experiments/` - Active experiments
- `philosophy/` - Core principles
- `future/` - Planned work
- `incidents/` - System issues

---

## UI Tools

### faelight-launcher v0.4

**Rust App Launcher**

**Purpose:** Fast, fuzzy application launcher.

**Keybind:** `Super+D`

**Features:**
- Fuzzy search with ranking
- Desktop file parsing
- Icon support
- Keyboard-only navigation

**Aliases:**
```bash
launcher
```

---

### faelight-menu v0.3

**Power Menu**

**Purpose:** System power operations with safety checks.

**Keybind:** `Super+Shift+E`

**Options:**
- Lock (faelight-lock)
- Logout (exit Sway)
- Reboot
- Shutdown

**Safety:** Confirmation required for destructive actions.

**Aliases:**
```bash
powermenu
```

---

### faelight-notify v0.4

**Notification Daemon**

**Purpose:** Wayland-native notification system.

**Service:** `faelight-notify.service` (user systemd)

**Features:**
- freedesktop notification spec
- Clean typography
- Timeout handling
- Close actions

---

### faelight-bar v1.0

**Status Bar**

**Purpose:** Custom Wayland status bar.

**Service:** `faelight-bar.service` (user systemd)

**Modules:**
- Workspaces
- Window title
- VPN status
- Battery
- Time

---

## Configuration Tools

### profile v0.4

**Profile System**

**Purpose:** Manage system profiles (default, work, gaming, etc).

**Commands:**
```bash
profile list          # List profiles
profile switch <name> # Switch profile
profile current       # Show current
prof                  # Alias
```

---

### theme-switch v0.2

**Theme Manager**

**Purpose:** System-wide theme switching.

**Commands:**
```bash
theme-switch list           # List themes
theme-switch apply <name>   # Apply theme
theme                       # Alias
```

---

## Development Tools

### core-diff v0.3

**Shell Policy Scanner**

**Purpose:** Detect risky shell patterns before execution.

**Commands:**
```bash
core-diff                    # Show all changes
core-diff --high-risk        # High-risk only
core-diff summary            # Statistics
cdiff                        # Alias
```

---

### bump-system-version v0.1

**Version Management**

**Purpose:** Bump system version with validation.

**Commands:**
```bash
bump-system-version <version>
bump <version>               # Alias
```

---

## Philosophy

All tools follow these principles:

1. **Manual Control Over Automation**
2. **Understanding Over Convenience**  
3. **Explicit Over Implicit**
4. **Recovery Over Perfection**

**Written in Rust.** Total: 8,369 lines across 25 tools.

---

_"The forest provides the tools. You decide how to use them."_ 🌲
