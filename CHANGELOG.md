# Changelog
All notable changes to Faelight Forest / 0-Core.

---

## [6.1.0] - 2026-01-09
### 🎨 Polish & Cleanup

**Login Manager**
- Added tuigreet with Faelight Forest green theme
- Auto-launches Sway on login

**Sway Enhancements**
- Enhanced keybindings (resize, scratchpad, screenshots, clipboard, notifications)
- Window rules for floating apps
- wm-sway package created and stowed

**GitHub Cleanup**
- Removed wm-hypr/ package entirely
- Removed Ghostty themes (not using)
- Removed old theme packages
- Removed COMPLETE_GUIDE.md, MELD_GUIDE.md
- Updated all docs for Sway references
- Updated zshrc aliases (swayconf, nsway, cdsway)

**Package Consolidation**
- New: term-foot (Faelight colors)
- New: launcher-fuzzel (Faelight colors)
- Removed: theme-term-foot-dark, theme-launch-fuzzel-dark, theme-gtk, tools-topgrade, foot-theme-light

**System**
- Mullvad VPN auto-connect enabled

### Technical
- All Hyprland references removed from codebase
- Clean stow structure
- Health: 100%

> "The forest shed its old growth." 🌲

---

All notable changes to Faelight Forest / 0-Core.

---

## [6.0.0] - 2026-01-09
### 🌲 Sway Edition - Complete Platform Migration

**Migrated from Hyprland to Sway after catastrophic update (2.3M errors)**

#### Window Manager
- **Sway WM** replaces Hyprland as compositor
- Full keybinding configuration matching Hyprland setup
- Auto-start on login via ~/.bash_profile
- Gaps, borders, and Faelight theming

#### faelight-bar Updates
- Sway IPC integration (replaces Hyprland socket)
- Active window detection via `swaymsg -t get_tree`
- Workspace tracking via `swaymsg -t get_workspaces`
- v0.7 - Sway Edition

#### Neovim Theme
- **faelight.lua** custom colorscheme
- Forest Night (#0f1411) background
- Faelight Green (#6be3a3) primary accent
- Faelight Blue (#5cc8ff) secondary accent
- Treesitter, LSP, Telescope, Neo-tree support

#### System Updates
- dot-doctor updated for Sway (swaymsg, faelight-bar)
- Security hardening applied (99-hardening.conf)
- fail2ban enabled with sshd jail
- Gitleaks pre-commit hook installed
- Topgrade configured for Faelight workflow

#### Terminal
- Foot terminal (replaces Kitty)
- Fuzzel launcher with Faelight theming
- Mako notifications

#### Configuration
- `.envrc` updated to v6.0.0
- Full shell restoration (188+ aliases)
- All Rust tools verified working

### Technical
- Platform: Arch Linux + Sway 1.11
- Health: 100% (12/12 checks passing)
- All 15 binary dependencies present

> "The forest found a new home." 🌲🦀

---


All notable changes to Faelight Forest / 0-Core.

---

## [5.1.0] - 2026-01-06

### Added

- **faelight-bar** — Custom Rust Wayland status bar replacing Waybar
  - Wayland layer-shell integration
  - Font rendering with fontdue
  - Hyprland IPC (workspaces, active window)
  - Profile-aware theming (DEF/GAM/WRK/LOW)
  - System modules: VPN, Battery, WiFi, Volume
  - Health diagnostics (quick 5-check subset)
  - Lock status indicator (LCK/UNL)
  - Click handling (VPN toggle, volume mute, profile cycle)
  - Date/time display
- `start-faelight-bar` launcher script
- Local Omarchy autostart override

### Changed

- Replaced Waybar with faelight-bar as daily driver
- Updated Hyprland config to use local autostart

### Technical

- ~500 lines of Rust
- Zero external bar dependencies
- Direct Hyprland socket IPC
- Wayland pointer events for click handling

> "The forest grew its own bar." 🦀🌲

## [5.0.0] - 2026-01-06

### Complete Rust Transition 🦀

**ALL 12 CORE SCRIPTS REWRITTEN IN RUST**

#### Rust Tools (12/12)

- `bump-system-version` - System version management
- `get-version` - Package version reader
- `latest-update` - Recently updated finder
- `bump-version` - Package version bumper
- `dotctl` - Central control utility
- `intent` - Intent Ledger management
- `profile` - System profile switching
- `teach` - Interactive teaching mode
- `core-diff` - Package-aware diff tool
- `dot-doctor` - 12-check health monitor
- `core-protect` - Immutable protection
- `safe-update` - Safe system updates

#### Benefits

- ⚡ Faster execution (compiled vs interpreted)
- 🔒 Memory safety (no buffer overflows)
- ✅ Type safety (errors at compile time)
- 🛠️ Better error handling

#### Structure

- Added `rust-tools/` with 12 Cargo projects
- Each tool is standalone binary
- `target/` directories gitignored

> "The forest speaks Rust now." 🦀🌲

## [4.2.0] - 2026-01-06

### Profile Sharing

- Added `profile export <name>` - export profiles for sharing
- Added `profile import <file>` - import community profiles
- Export includes metadata (date, system, version)
- Import strips export header automatically
- Overwrite protection with confirmation prompt

## [4.1.0] - 2026-01-06

### Teaching Mode

- Added `teach` command - interactive learning guide
- 8 lessons: Philosophy, Structure, Tools, Workflow, Profiles, Intent, Aliases, Customization
- Uses gum for beautiful prompts
- Menu-based or sequential walkthrough

## [4.0.1] - 2026-01-06

### Profile Tab Completions

- Added file-based zsh completions for profile command
- Tab completion shows all commands and profiles

## [4.0.0] - 2026-01-06

### System Profiles - The Omega Legacy

- Added `profile` command - switch system state with one command
- 4 default profiles: default, gaming, work, low-power
- Profile state tracking and history logging
- Starship prompt shows active profile
- dot-doctor Check 12: Profile health monitoring
- Philosophy: One command switches everything

## [3.6.0] - 2026-01-05

### Intent Ledger Integration

- **dot-doctor Check 11**: Intent Ledger health monitoring
  - Count intents by status (complete, planned, in-progress, decided, abandoned)
  - Validate `packages:` references exist
  - Detect version/tag status mismatch
  - Flag stale planned intents (> 30 days)
- **New frontmatter fields**: `packages:` and `version:` for intents
- **Fixed**: INTENT and archive directories no longer flagged as non-semantic
- **Fixed**: Health percentage now shows whole number

---

## [3.5.2] - 2026-01-04

### Shell Safety & Polish

- **Dangerous command highlighting**: `rm -rf`, `chmod 777`, `dd`, `mkfs` show RED (includes sudo variants)
- **Dangerous command warning**: `preexec` hook shows warning before destructive commands
- **Tab completions**: Completions for `core-diff`, `dotctl`, `intent`
- **Starship lock status**: 🔒/🔓 inline in prompt (replaces verbose `core_guard`)
- **Removed**: `core_guard` function (replaced by Starship module)

---

## [3.5.1] - 2026-01-01

### Git Guardrails

- Block commits when core is locked (immutability protection)
- Warn on push to main (requires typed confirmation: 'push-main')
- `git!` escape hatch for experts
- Only active in ~/0-core directory

---

---

_The forest remembers. 🌲_
