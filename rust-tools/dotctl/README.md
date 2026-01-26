# dotctl v2.0.0

🎮 **Control utility for 0-Core package management**

Central command interface for managing dotfile packages, versions, and system health.

---

## Purpose

**dotctl** is the control panel for 0-Core's package-based dotfile system.

Each package in `~/0-core/stow/` is independently versioned and tracked. dotctl provides a unified interface to manage them.

---

## Commands

### Show System Status
```bash
dotctl status
```
Shows:
- System version (0-Core v8.3.0)
- All stow packages and their versions
- Package metadata

### Bump Package Version
```bash
dotctl bump shell-zsh 3.3.1 "Added custom aliases"
```
Updates package version and records change reason.

### Show Package History
```bash
dotctl history wm-sway
```
Displays changelog for a specific package.

### Run Health Check
```bash
dotctl health
```
Alias for `dot-doctor` - runs full system health check.

### Show Version
```bash
dotctl version
# or
dotctl -v
```

---

## Package System

**Each stow package** has:
- Independent version number
- Changelog history
- Metadata file (.dotmeta)
- Stow-managed symlinks

**Example package:**
```
stow/shell-zsh/
├── .dotmeta          # Version: 3.3.1
├── CHANGELOG.md      # Version history
└── .config/zsh/      # Actual config files
```

---

## Usage Examples

### Daily workflow
```bash
# Check system status
dotctl status

# Make changes to wm-sway config
vim ~/0-core/stow/wm-sway/.config/sway/config

# Bump version after changes
dotctl bump wm-sway 2.1.0 "Added workspace keybinds"

# Verify health
dotctl health
```

### Before system update
```bash
# Record current state
dotctl status > ~/pre-update-status.txt

# Update system
faelight-update

# Verify everything still works
dotctl health
```

### Check package history
```bash
# See what changed in a package
dotctl history shell-zsh

# Review all changes
dotctl status
```

---

## Integration

### With dot-doctor
```bash
dotctl health
# Runs: dot-doctor with full health checks
```

### With faelight-stow
Package versions tracked via stow metadata:
```bash
dotctl bump shell-zsh 3.3.1
# Creates/updates: stow/shell-zsh/.dotmeta
```

### With bump-system-version
System releases reference package versions:
```bash
bump-system-version
# Includes all package versions in release notes
```

---

## Philosophy

**Package-level granularity**

Instead of one monolithic dotfile version, each logical component (shell, window manager, terminal, etc.) has its own version.

**Benefits:**
- Track what actually changed
- Independent package evolution
- Clear changelog per component
- Easier rollback of specific components

---

## Design

**Simple, focused commands:**
- `status` - What's installed?
- `bump` - Record a change
- `history` - What changed over time?
- `health` - Is everything working?

**No magic** - just version tracking and metadata management.

---

## Exit Codes

- `0` - Success
- `1` - Command failed
- `2` - Invalid arguments

---

## Part of 0-Core

One of 30+ Rust tools in the Faelight Forest ecosystem.

Central control interface for package-based dotfile management.

See: https://github.com/WidkidoneR2/0-Core
