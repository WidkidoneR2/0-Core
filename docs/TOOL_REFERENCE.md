# 🛠️ Faelight Forest Tools Reference

Complete reference for all 34 custom Rust tools in 0-Core v8.4.0.

**Total Tools:** 34 production-ready Rust binaries + 1 library  
**Test Status:** 34/34 passing (run `~/0-core/scripts/test-all-tools`)  
**Language:** 100% Rust (105,879 lines)

---

## Tool Categories

1. [Core Infrastructure](#core-infrastructure-11-tools) - System foundation
2. [Desktop Environment](#desktop-environment-9-tools) - User interface
3. [Development & Workflow](#development--workflow-11-tools) - Productivity
4. [Version Management](#version-management-4-tools) - Release automation

---

## Core Infrastructure (11 tools)

### dot-doctor v0.5.0
**Health monitoring engine with 14 comprehensive checks**
```bash
doctor                    # Run all health checks (100% = healthy)
doctor --explain          # Detailed explanations
doctor --json             # JSON output
doctor --fix              # Auto-apply safe fixes
doctor --history          # Show health history
doctor --check stow       # Run specific check only
```

**Location:** `~/0-core/scripts/dot-doctor`  
**Alias:** `doctor`

---

### faelight-update v0.5.0
**Smart update manager for pacman, AUR, cargo, neovim, and 0-Core workspace**
```bash
faelight-update                    # Interactive update
faelight-update --dry-run          # Check without applying
faelight-update -v                 # Verbose with version changes
faelight-update --interactive      # Select packages
faelight-update --only pacman      # Update specific category
faelight-update --skip aur         # Skip category
faelight-update --snapshot         # Create pre-update snapshot
```

**Features:**
- Impact analysis (kernel, critical packages, major versions)
- Smart detection (no false positives)
- Health check before updates
- Workspace rebuild after updates

**Location:** `~/0-core/scripts/faelight-update`

---

### core-protect v1.0.1
**Filesystem immutability protection**
```bash
core-protect status       # Check lock status
core-protect lock         # Make immutable (chattr +i)
core-protect unlock       # Allow editing
```

**Aliases:** `lock-core`, `unlock-core`  
**Location:** `~/0-core/scripts/core-protect`

---

### safe-update v1.0.0
**System update with BTRFS snapshots**
```bash
safe-update              # Update with pre/post snapshots
safe-update --no-snapshot # Skip snapshot creation
```

**Location:** `~/0-core/scripts/safe-update`

---

### core-diff v2.0.0
**Package-aware diff with risk levels**
```bash
core-diff summary        # Show package changes
core-diff stow           # Diff stow packages
core-diff rust-tools     # Diff Rust workspace
```

**Location:** `~/0-core/scripts/core-diff`

---

### dotctl v2.0.0
**Central control utility**
```bash
dotctl status            # Show version info
dotctl packages          # List all packages
dotctl health            # Quick health check
```

**Location:** `~/0-core/scripts/dotctl`

---

### entropy-check v1.0.0
**Configuration drift detection**
```bash
entropy-check            # Detect drift from canonical state
entropy-check --fix      # Auto-fix drift
```

**Location:** `~/0-core/scripts/entropy-check`

---

### intent-guard v1.0.0
**Command safety validation**
```bash
intent-guard <command>   # Validate before execution
```

**Location:** `~/0-core/scripts/intent-guard`

---

### faelight-stow v0.3.0
**GNU Stow package management**
```bash
faelight-stow            # Check stow status
faelight-stow deploy     # Deploy package
faelight-stow remove     # Remove package
```

**Location:** `~/0-core/scripts/faelight-stow`

---

### faelight-snapshot v1.0.0
**BTRFS snapshot manager**
```bash
faelight-snapshot list           # List snapshots
faelight-snapshot create         # Create snapshot
faelight-snapshot create --tag pre-update
faelight-snapshot restore <name> # Restore snapshot
```

**Location:** `~/0-core/scripts/faelight-snapshot`

---

### faelight-core (Library)
**Shared library for config, health checks, and IPC**

**Location:** `~/0-core/rust-tools/faelight-core/`  
**Note:** Library only, no binary

---

## Desktop Environment (9 tools)

### faelight-fetch v1.0.0
**System information display (neofetch alternative)**
```bash
faelight-fetch           # Show system info with ASCII art
faelight-fetch --json    # JSON output
```

**Location:** `~/0-core/scripts/faelight-fetch`

---

### faelight-bar v1.0.0
**Wayland status bar with Sway IPC integration**
```bash
# Starts automatically via Sway config
pgrep -f faelight-bar    # Check if running
```

**Features:**
- Workspace display
- Window title
- Lock status (🔒/🔓)
- Profile indicator (DEF/WRK/GAM/LOW)
- VPN status (Mullvad)
- Battery
- Volume
- Clock
- Beautiful gradient separators

**Location:** `~/0-core/scripts/faelight-bar`  
**Config:** `~/.config/faelight/config.toml`

---

### faelight-launcher v3.3.0
**XDG application launcher with fuzzy search**
```bash
faelight-launcher        # Open launcher
```

**Keybinding:** `Super+D`  
**Location:** `~/0-core/scripts/faelight-launcher`

---

### faelight-dmenu v2.0.0
**Wayland dmenu replacement**
```bash
faelight-dmenu           # Open dmenu
echo -e "option1\noption2" | faelight-dmenu
```

**Location:** `~/0-core/scripts/faelight-dmenu`

---

### faelight-menu v0.7.0
**Power menu (lock/logout/reboot/shutdown)**
```bash
faelight-menu            # Open power menu
```

**Keybinding:** `Super+Shift+E`  
**Location:** `~/0-core/scripts/faelight-menu`

---

### faelight-notify v0.9.0
**Wayland notification daemon**
```bash
# Starts automatically via Sway config
notify-send "Test" "Message"  # Send notification
```

**Location:** `~/0-core/scripts/faelight-notify`

---

### faelight-lock v1.0.0
**Screen locker**
```bash
faelight-lock            # Lock screen
```

**Keybinding:** `Super+L`  
**Location:** `~/0-core/scripts/faelight-lock`

---

### faelight-dashboard v1.0.0
**System dashboard TUI**
```bash
faelight-dashboard       # Open dashboard
```

**Location:** `~/0-core/scripts/faelight-dashboard`

---

### faelight-term v10.0.0
**Terminal emulator with color emoji support**
```bash
faelight-term            # Open terminal
faelight-term --version  # Show version
```

**Features:**
- Color emoji rendering
- Copy/paste support
- Mouse selection
- Wayland native

**Status:** Beta (APIs may change)  
**Location:** `~/0-core/scripts/faelight-term`

---

## Development & Workflow (11 tools)

### intent v2.0.0
**Intent Ledger management**
```bash
intent list                    # List all intents
intent list decisions          # List by category
intent show 067                # Show specific intent
intent search rust             # Search intents
intent add future "description" # Add new intent
```

**Location:** `~/0-core/scripts/intent`  
**Ledger:** `~/0-core/INTENT/`

---

### archaeology-0-core v1.0.0
**System history explorer**
```bash
archaeology-0-core --timeline  # Show commit history
archaeology-0-core --stats     # Show statistics
```

**Location:** `~/0-core/scripts/archaeology-0-core`

---

### workspace-view v1.0.0
**Sway workspace intelligence**
```bash
workspace-view           # Show workspace info
workspace-view --json    # JSON output
```

**Location:** `~/0-core/scripts/workspace-view`

---

### faelight-git v3.0.0
**Git workflow automation and governance**
```bash
fg status                # Check git status
fg sync                  # Pull + push workflow
fg verify                # Verify commit readiness
fg install-hooks         # Install git hooks
```

**Alias:** `fg`  
**Location:** `~/0-core/scripts/faelight-git`

---

### faelight-hooks v1.0.0
**Git hooks manager with pre-commit/pre-push validation**
```bash
faelight-hooks install   # Install hooks
faelight-hooks remove    # Remove hooks
faelight-hooks test      # Test hooks
```

**Features:**
- Secret scanning (gitleaks)
- Merge conflict detection
- Conventional commit validation
- Branch protection warnings
- Pre-push uncommitted changes check

**Location:** `~/0-core/scripts/faelight-hooks`  
**Hooks:** `~/0-core/hooks/`

---

### recent-files v0.2.0
**Time-based file discovery dashboard**
```bash
recent-files today       # Files modified today
recent-files week        # Files modified this week
recent-files --path ~/   # Search specific path
```

**Location:** `~/0-core/scripts/recent-files`

---

### profile v1.0.0
**System profile switching**
```bash
profile list             # Show available profiles
profile switch gaming    # Switch to gaming profile
profile current          # Show current profile
```

**Profiles:** default, work, gaming, low-power  
**Location:** `~/0-core/scripts/profile`  
**Config:** `~/.config/faelight/profiles.toml`

---

### teach v1.0.0
**Interactive learning guide**
```bash
teach                    # Start interactive tutorial
teach --begin            # Start from beginning
```

**Location:** `~/0-core/scripts/teach`

---

### faelight v1.0.0
**Unified binary interface (optional convenience wrapper)**
```bash
faelight --help          # Show all commands
faelight health          # Run health check
faelight status          # System status
```

**Note:** Delegates to individual tools  
**Location:** `~/0-core/scripts/faelight`

---

### keyscan v1.0.0
**Keybind conflict detection**
```bash
keyscan                  # Scan for conflicts
keyscan --verbose        # Show all bindings
```

**Location:** `~/0-core/scripts/keyscan`

---

### faelight-zone v1.1.0
**Filesystem spatial awareness**
```bash
faelight-zone            # Show current zone info
faelight-zone --all      # Show all zones
```

**Zones:** 0-core, 1-src, 2-projects, 3-archive, 4-media  
**Location:** `~/0-core/scripts/faelight-zone`

---

## Version Management (4 tools)

### bump-system-version v5.0.0
**Stress-free release automation with pre-flight dashboard**
```bash
bump-system-version 8.5.0       # Bump to new version
bump-system-version --dry-run   # Preview changes
```

**Features:**
- Pre-flight safety dashboard
- Semantic version validation
- Cargo.toml updates across workspace
- Git tag creation
- Automatic commit and push

**Location:** `~/0-core/scripts/bump-system-version`

---

### faelight-bootstrap v1.0.0
**One-command system setup**
```bash
faelight-bootstrap              # Interactive setup
faelight-bootstrap --help       # Show options
```

**Location:** `~/0-core/scripts/faelight-bootstrap`

---

### get-version v2.0.0
**Package version reader**
```bash
get-version faelight-term       # Get tool version
get-version --all               # Show all versions
```

**Location:** `~/0-core/scripts/get-version`

---

### latest-update v2.0.0
**Recently updated package finder**
```bash
latest-update                   # Show recently updated
latest-update --count 10        # Show top 10
```

**Location:** `~/0-core/scripts/latest-update`

---

## Sway Keybindings

### Core Actions
| Key | Action |
|-----|--------|
| `Super+Return` | Terminal (Foot) |
| `Super+D` | Application launcher |
| `Super+Shift+E` | Power menu |
| `Super+L` | Lock screen |
| `Super+Q` | Close window |

### Window Management
| Key | Action |
|-----|--------|
| `Super+H/J/K/L` | Focus window (vim keys) |
| `Super+Shift+H/J/K/L` | Move window |
| `Super+V` | Toggle floating |
| `Super+F` | Fullscreen |

### Workspaces
| Key | Action |
|-----|--------|
| `Super+1-5` | Switch to workspace |
| `Super+Shift+1-5` | Move to workspace |

### Applications
| Key | Action |
|-----|--------|
| `Super+B` | Browser (Brave) |
| `Super+Alt+K` | KeePassXC |
| `Super+Alt+E` | Tutanota |

**Full list:** See `~/.config/sway/config`  
**Conflict check:** Run `keyscan`

---

## Testing

Run comprehensive test suite:
```bash
~/0-core/scripts/test-all-tools
```

**Expected:** 34/34 passing (1 library skipped)

---

## Architecture

All tools are Rust binaries compiled from the workspace:
```
~/0-core/
├── Cargo.toml              # Workspace manifest
├── rust-tools/             # Source code (34 packages)
│   ├── faelight/
│   ├── dot-doctor/
│   ├── faelight-update/
│   └── ... (31 more)
└── scripts/                # Compiled binaries (gitignored)
    ├── faelight
    ├── doctor
    └── ... (34 binaries)
```

**Build:** `cargo build --release` (from ~/0-core)  
**Deploy:** Binaries copied from `target/release/` to `scripts/`

---

## Documentation

| Document | Description |
|----------|-------------|
| `docs/ARCHITECTURE.md` | System structure |
| `docs/BUILD.md` | Build workflow |
| `docs/TESTING.md` | Test suite documentation |
| `docs/FAELIGHT-CLI.md` | Unified CLI reference |
| `docs/FAELIGHT-CONFIG.md` | TOML configuration |

---

## Quick Reference
```bash
# System Health
doctor                    # 14 comprehensive checks

# Updates
faelight-update --dry-run # Check for updates
faelight-update -v        # Update with version info

# Git Workflow
fg status                 # Git status
fg sync                   # Pull + push

# Core Protection
lock-core                 # Protect filesystem
unlock-core               # Allow editing

# Profiles
profile switch gaming     # Switch profile

# Intent Ledger
intent list               # View decisions
```

---

*The forest's toolkit.* 🌲  
*34 production-ready Rust tools. 105,879 lines of code. 100% intentional.*
