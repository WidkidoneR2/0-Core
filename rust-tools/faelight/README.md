# faelight v1.0.0

🌲 **The Core Spine** - Unified CLI orchestrator for Faelight Forest.

## Philosophy

> "One command to rule them all, but each tool stands alone."

`faelight` is the **orchestration layer** that provides a unified interface to all Faelight Forest tools, while keeping each tool independent and composable. It's the difference between 30 separate commands and one coherent system.

## Architecture
```
faelight (orchestrator)
├── dot-doctor (health checks)
├── intent (intent ledger)
├── intent-guard (command safety)
├── profile (system profiles)
├── faelight-launcher (app launcher)
├── faelight-menu (power menu)
├── faelight-lock (screen lock)
├── faelight-bar (status bar)
├── faelight-git (git governance)
└── ... (29 total tools)
```

Each tool works independently, but `faelight` provides the unified interface.

## Usage

### System Health
```bash
# Quick health check
faelight health

# Detailed explanations
faelight health --explain

# System status
faelight status
```

### Profile Management
```bash
# List profiles
faelight profile list

# Switch profile
faelight profile switch work

# Current profile
faelight profile current
```

### Intent Ledger
```bash
# List all intents
faelight intent list

# Filter by category
faelight intent list future

# Show specific intent
faelight intent show 052

# Search intents
faelight intent search rust
```

### Core Protection
```bash
# Lock 0-core (prevent accidental changes)
faelight core lock

# Unlock 0-core
faelight core unlock

# Check lock status
faelight core status
```

### Sway Window Manager
```bash
# Reload sway config
faelight sway reload

# Sway status
faelight sway status
```

### Launch Applications
```bash
# Application launcher
faelight launch launcher

# Power menu
faelight launch menu

# Screen lock
faelight launch lock

# Test notification
faelight launch notify "Hello Linus!"
```

### Git Governance
```bash
# Install git hooks
faelight git install-hooks

# Remove git hooks
faelight git remove-hooks

# Verify commit readiness
faelight git verify

# Git status
faelight git status
```

### Configuration Management
```bash
# Validate all configs
faelight config validate

# Show current config
faelight config show

# Config file path
faelight config path

# Edit config
faelight config edit
```

### Self-Documentation
```bash
# Explain profiles
faelight explain profile

# Explain intents
faelight explain intent

# Explain health system
faelight explain health

# Explain core protection
faelight explain core
```

## Global Flags
```bash
# Dry run mode
faelight --dry-run core lock

# JSON output
faelight --json status
```

## Design Principles

### 1. **Orchestration, Not Duplication**
`faelight` doesn't reimplement logic - it delegates to specialized tools. Each tool maintains its own independence.

### 2. **Unified Interface**
One consistent command structure across all functionality:
```
faelight <domain> <action> [args]
```

### 3. **Self-Documenting**
The `explain` command provides built-in documentation:
```bash
faelight explain intent
faelight explain profile
```

### 4. **Fail-Safe Defaults**
- Dry-run mode prevents accidents
- Clear error messages
- Proper exit codes for CI/CD

### 5. **Composability**
Each subcommand works independently. You can use `faelight intent list` or `intent list` directly - both work.

## Integration with Philosophy

`faelight` embodies the Faelight Forest principles:

1. **Manual Control Over Automation** - You invoke actions explicitly, no background magic
2. **Intentionality** - Every command is documented and clear
3. **Composability** - Tools work together but stand alone
4. **Discoverability** - `faelight explain` teaches you the system

## Demo for Linus
```bash
# Show system status
faelight status

# Run health check
faelight health

# Show all intents
faelight intent list

# Explain the intent system
faelight explain intent

# Show profile options
faelight profile list

# Lock the core
faelight core lock

# Check lock status
faelight core status

# Unlock it
faelight core unlock
```

## Why This Matters

Traditional Linux systems give you hundreds of unrelated commands:
```bash
systemctl status foo
nmcli device wifi connect bar
pactl set-sink-volume @DEFAULT_SINK@ 50%
```

Faelight Forest gives you **one coherent interface**:
```bash
faelight health
faelight profile switch work
faelight launch launcher
```

But crucially: **Each tool still works independently**. `faelight` is the front door, but you can enter through any window.

This is **orchestration without coupling**.

## Technical Details

- **Language:** Rust
- **CLI Framework:** clap
- **Exit Codes:** Proper codes for CI/CD
- **Error Handling:** Clear messages, no panics
- **Performance:** Instant startup, delegates to tools

## Roadmap

Current features are stable. Future enhancements:

- Plugin system for custom commands
- Shell completion generation
- TUI dashboard mode
- Remote orchestration (SSH)

But these are v2.0+ features. v1.0 is production-ready.
