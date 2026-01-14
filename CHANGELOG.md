## [7.0.0] - 2026-01-14

### 🎉 Major Architectural Milestone

**Completed Intents:**
- ✅ Intent 047: faelight-core foundation library
- ✅ Intent 048: All 4 tools migrated (70-90% CPU reduction)
- ✅ Intent 036: Rust Hygiene across all tools
- ✅ Intent 050: Bar v2.0 cached model architecture

**New Features:**
- **faelight-core v0.1.0**: Shared GlyphCache, Theme, Canvas
- **faelight-bar v2.0**: Cached model, JSON parsing, per-module frequencies
- **Health checks**: All tools support `--health-check` flag
- **Performance**: 70-90% CPU reduction in text rendering
- **Safety**: Double-confirmation for dangerous menu actions

**Tool Versions:**
- faelight-lock v0.3
- faelight-menu v0.5
- faelight-notify v0.6
- faelight-bar v2.0

**Architecture:**
- Cached model pattern for system state
- JSON parsing instead of string matching
- Per-module update frequencies (500ms / 10s / 30s)
- Pure rendering loops (zero blocking I/O)

# Changelog
All notable changes to Faelight Forest / 0-Core.

## [6.9.1] - 2026-01-14

### 📦 Tool Updates
- **dot-doctor v0.3.0** - Keybind conflict detection integration
  - Added Sway keybind check using keyscan
  - 13 total health checks (was 12)
  - Validates 99 unique keybindings with no conflicts

### 🔒 Security
- Verified intent-guard v0.1.0 shell integration working
  - Pattern database detecting recursive deletes
  - Command safety guard active in zsh

### 📜 Documentation
- Intent 044: Keybind Conflict Detection - Complete ✅
- Intent 042: v6.9.0 milestone progress (4/5 must-have items complete)

> "The system tells you when you're about to break something." 🌲

---

---

## [6.9.0] - 2026-01-14

### 🚀 New Features
- **keyscan v0.1.0** - Sway keybind conflict detection
  - Parses Sway config for duplicate bindings
  - JSON and human-readable output
  - Validates 99 unique keybindings in current config
  
- **entropy-check v0.2.0** - Drift history tracking
  - `entropy-check trends` shows 30-day drift history
  - Tracks config, service, binary, symlink, and untracked file changes
  - Statistics: average drift, highest drift, clean check ratio
  
- **faelight-launcher v0.5.0** - Frecency tracking
  - Records app launch frequency and recency
  - Boosts frequently/recently used apps in ranking
  - Stores history in `~/.local/state/faelight/launcher-history.json`

### 🔧 Fixes
- Notesnook Wayland support via desktop file override
- Launcher now properly launches Notesnook with Wayland flags

### 📚 Documentation
- Intent 046: v7.0.0 Architectural Refactor (umbrella)
- Intent 047: faelight-core Shared Foundation Library
- Intent 048: Tool Migrations to faelight-core
- Intent 049: Launcher v2.0 Provider Architecture

> "The system should learn from you, not just respond to you." 🌲

---


---


## [6.8.1] - 2026-01-13

### 🐛 Bug Fixes
- Fixed "Untitled" entry in intent list (intent tool now skips index files)

### ✨ Features
- **entropy-check**: Added `--json` flag for machine-readable output
- **Shell aliases**: Added convenient shortcuts (fl, lock, theme, bump, update, prof)

### 📚 Documentation
- Created `docs/TOOLS.md` - Complete reference for all 25 Rust tools (274 lines)

### 🦀 Tool Updates
- intent v0.3 - Better filtering
- entropy-check v0.1.1 - JSON output support

---

## [6.8.0] - 2026-01-13

### 🚀 New Features
- **entropy-check v0.1** - Configuration drift detection tool
  - Config drift: Detects files modified outside stow
  - Service drift: Tracks service state changes
  - Binary drift: Monitors package version changes
  - Symlink drift: Identifies new broken symlinks
  - Untracked files: Finds new files in managed directories

### 📝 Incidents
- Incident 003: Stow symlink confusion (resolved in 5 minutes)
  - All safety nets worked (git, snapshots, manual recovery)
  - Zero data loss, learned valuable lessons

### 🦀 Rust Tools
- entropy-check v0.1 - Drift detection (Intent 015 complete)

> "The forest watches for silent changes. Systems drift—entropy-check observes." 🌲

---

## [6.7.2] - 2026-01-12

### 📐 Typography/UI
- faelight-notify v0.4.0: Typography hierarchy (16/20/17), increased dimensions (400x80)
- faelight-launcher v0.4.0: Typography polish (22/18/14), fuzzy ranking, selection padding
- faelight-menu v0.3.0: Typography clarity (20/18/14), danger divider, rust formatting

### 📦 Tool Updates
- faelight-notify v0.4.0 - Typography polish
- faelight-launcher v0.4.0 - Feel perfect
- faelight-menu v0.3.0 - Clarity & safety

> "Typography breathes. Polish speaks clearly. The forest refines its voice." 🌲

---

## [6.7.1] - 2026-01-11

### 🚀 faelight-launcher v0.4
- Typography hierarchy (title 24, item 18, hint 16)
- Fuzzy scoring with ranking (exact > starts > contains)
- Escape: first clears search, second closes
- Fixed icons (tree, vim, terminal)
- ROW_HEIGHT: 46 for better spacing

### 🔒 faelight-menu v0.3
- Typography hierarchy (title 22, item 20, hint 15)
- Divider line before dangerous actions
- Danger always visible (not hidden until selected)
- Width: 400, ROW_HEIGHT: 52

### 🔔 faelight-notify v0.4
- Typography hierarchy (app 14, title 18, body 16)
- Compact size: 200x60 (70% smaller)
- Less intrusive notifications

### 🔧 Fixes
- Super+D → faelight-launcher (was fuzzel)
- Super+Escape → faelight-menu

> "The forest refines itself." 🌲

---

## [6.7.0] - 2026-01-11

### 🔗 faelight-stow v0.1 (Intent 034)
- Verify stow symlinks on demand
- --quiet --notify for systemd integration
- --fix option for manual repair
- Philosophy: verify-only, you stay in control

### 📸 faelight-snapshot v0.1 (Intent 028)
- Snapper wrapper for btrfs snapshots
- list/create/delete/diff/rollback commands
- Adjusted timeline: 0 hourly, 3 daily, 2 weekly

### 🖥️ faelight-dashboard v0.1 (Intent 032)
- Beautiful TUI system overview
- Health, Git, Security, Stats panels
- Keyboard shortcuts: h/g/i/r/q
- Built with ratatui

### 🔧 New Aliases
- `dashboard` / `dash` - System overview
- `snap` / `snapshot` - Btrfs snapshots
- `stow-check` - Verify symlinks

### Technical
- Intents 028, 032, 034 complete
- 12 Rust tools now
- Health: 100%

> "The forest watches over itself." 🌲

---

## [6.6.0] - 2026-01-11

### 🔍 faelight-launcher v0.3 (Intent 027)
- Fuzzy search - type to filter apps in real-time
- Substring + character sequence matching
- Search box with placeholder text
- Backspace clears filter
- Selection resets on filter change

### 🦀 Nushell Integration
- shell-nushell package added
- Starship prompt works natively
- Core aliases converted
- Run `nu` to switch anytime
- Zsh remains default

### 🛠️ LazyVim Rust Support
- rust-analyzer LSP configured
- rustaceanvim for enhanced experience
- crates.nvim for Cargo.toml

### Technical
- Intent 027 complete
- 10 Rust tools
- Health: 100%

> "The forest speaks many languages." 🌲

---

## [6.5.0] - 2026-01-10

### 🛡️ Security Hardening
- UFW firewall enabled (deny incoming, allow outgoing)
- Mullvad VPN relocated to Atlanta
- DNS over TLS with Quad9 + DNSSEC
- gocryptfs encrypted vault for ~/secrets

### 🔧 core-diff --policy shell (Intent 025)
- `core-diff --policy shell`: Scan changed shell scripts
- `core-diff --policy shell --all`: Full scan of all scripts
- Detects forbidden patterns: sudo, systemctl, pacman, rm -rf, curl|sh, eval

### 🎨 UI Polish
**faelight-launcher v0.2**
- Hack Nerd Font Bold for better readability
- 12 apps in alphabetical order
- Proper Nerd Font icons for all apps

**faelight-menu v0.2**
- Hack Nerd Font Bold
- Increased footer font size (11→14)

**faelight-notify v0.3**
- Hack Nerd Font Regular
- Dynamic text truncation with "..."
- Wider notifications (600px)

**faelight-bar v0.8**
- Hack Nerd Font Regular

### Technical
- Intent 025 complete
- 6/6 security layers active
- Health: 100%

> "The forest hardens its defenses." 🌲

---

## [6.4.0] - 2026-01-10

### 🏗️ Architectural Milestone

**faelight v0.1 - Unified CLI**
- Single entry point for all Faelight commands
- Subcommands: health, profile, intent, core, sway, git, config, launch, explain
- Global flags: --json, --dry-run
- Shell becomes thin aliases

**faelight-config - Typed TOML Layer**
- ~/.config/faelight/config.toml - global settings
- ~/.config/faelight/profiles.toml - profile definitions
- ~/.config/faelight/themes.toml - color themes
- Schema validation via serde

**dot-doctor v0.2 - Health Engine Upgrade**
- --explain: detailed explanations for each check
- --json: machine-readable output
- --graph: dependency visualization
- --check <id>: run specific check only
- --fail-on-warning: CI mode
- Dependency graph (blocked checks when parent fails)
- Severity levels (Critical/High/Medium/Low)

**faelight-git v0.1 - Git Governance**
- install-hooks: pre-commit, commit-msg, pre-push
- verify: check commit readiness
- status: lock status
- Blocks commits when core locked
- Suggests intent references for significant changes

### 📚 Documentation
- docs/HEALTH-ENGINE.md - dot-doctor reference
- docs/FAELIGHT-CLI.md - unified CLI commands
- docs/FAELIGHT-GIT.md - git governance
- docs/FAELIGHT-CONFIG.md - TOML configuration
- docs/TOOLS-REFERENCE.md - master tools list

### Technical
- 9 custom Rust tools in daily use
- Intents 021-024 complete
- Health: 100%

> "The forest evolves." 🌲

---

## [6.4.0] - 2026-01-10

### 🏗️ Architectural Milestone

**faelight v0.1 - Unified CLI**
- Single entry point for all Faelight commands
- Subcommands: health, profile, intent, core, sway, git, config, launch, explain
- Global flags: --json, --dry-run
- Shell becomes thin aliases

**faelight-config - Typed TOML Layer**
- ~/.config/faelight/config.toml - global settings
- ~/.config/faelight/profiles.toml - profile definitions
- ~/.config/faelight/themes.toml - color themes
- Schema validation via serde

**dot-doctor v0.2 - Health Engine Upgrade**
- --explain: detailed explanations for each check
- --json: machine-readable output
- --graph: dependency visualization
- --check <id>: run specific check only
- --fail-on-warning: CI mode
- Dependency graph (blocked checks when parent fails)
- Severity levels (Critical/High/Medium/Low)

**faelight-git v0.1 - Git Governance**
- install-hooks: pre-commit, commit-msg, pre-push
- verify: check commit readiness
- status: lock status
- Blocks commits when core locked
- Suggests intent references for significant changes

### 📚 Documentation
- docs/HEALTH-ENGINE.md - dot-doctor reference
- docs/FAELIGHT-CLI.md - unified CLI commands
- docs/FAELIGHT-GIT.md - git governance
- docs/FAELIGHT-CONFIG.md - TOML configuration
- docs/TOOLS-REFERENCE.md - master tools list

### Technical
- 9 custom Rust tools in daily use
- Intents 021-024 complete
- Health: 100%

> "The forest evolves." 🌲

---


## [6.3.0] - 2026-01-10
### 🦀 New Rust Tools

**faelight-menu v0.1**
- Power menu with lock/logout/suspend/reboot/shutdown
- Keyboard navigation (j/k, arrows, quick keys)
- Super+Shift+Escape keybinding

**faelight-lock v0.1**
- Screen locker with Faelight Forest theming
- Wraps swaylock with custom colors
- Super+Ctrl+Escape keybinding

### Improvements
- faelight-bar: 500ms frame throttle to reduce CPU usage
- dot-doctor: Fixed stow symlink checking
- Intent 012, 018, 020 marked complete

### Technical
- 7 custom Rust tools now in daily use
- Health: 100%

> "The forest protects its own." 🌲

---


All notable changes to Faelight Forest / 0-Core.

---

## [6.2.0] - 2026-01-10
### 🦀 Custom Rust Tools

**faelight-notify v0.2**
- Custom Rust notification daemon
- D-Bus org.freedesktop.Notifications implementation
- Wayland layer-shell rendering
- Click to dismiss, auto-expire
- Replaces mako in autostart

**faelight-launcher v0.1**
- Custom Rust app launcher
- Keyboard navigation (j/k, arrows)
- Static app list with icons
- Launch with Enter, close with Escape
- Bound to Super+Space

**faelight-bootstrap v0.1**
- One-command 0-core setup
- Validates Arch Linux, installs deps
- Clones repo, stows packages, builds tools
- curl | bash installation ready

**dot-doctor**
- Fixed: ignore target/ directories in semantic check

### Technical
- All tools use Wayland layer-shell
- Shared font rendering approach
- Health: 100%

> "The forest builds its own tools." 🌲

---

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

## [6.9.0] - 2026-01-14

### Added
- **keyscan v0.1.0** - Sway keybind conflict detection
  - Parses Sway config for duplicate bindings
  - JSON and human-readable output
  - Validates 99 unique keybindings in current config
  
- **entropy-check v0.2.0** - Drift history tracking
  - `entropy-check trends` shows 30-day drift history
  - Tracks config, service, binary, symlink, and untracked file changes
  - Statistics: average drift, highest drift, clean check ratio
  
- **faelight-launcher v0.5.0** - Frecency tracking
  - Records app launch frequency and recency
  - Boosts frequently/recently used apps in ranking
  - Stores history in `~/.local/state/faelight/launcher-history.json`

### Changed
- intent-guard v0.2.1 - Refined false positive detection
- Notesnook Wayland support via desktop file override

### Fixed
- Launcher now launches Notesnook with proper Wayland flags

### Documentation
- Intent 046: v7.0.0 Architectural Refactor (umbrella)
- Intent 047: faelight-core Shared Foundation Library
- Intent 048: Tool Migrations to faelight-core
- Intent 049: Launcher v2.0 Provider Architecture

### Philosophy
Three new tools that extend 0-Core's intelligence:
- keyscan prevents configuration conflicts
- entropy trends reveal system drift patterns
- launcher frecency learns user behavior

**Core Principle:** The system should learn from you, not just respond to you.

