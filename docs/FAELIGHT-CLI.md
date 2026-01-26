# 🌲 Faelight CLI

The unified command-line interface for Faelight Forest.

> **Note:** This is an **optional convenience wrapper**. All commands can be run directly (e.g., `doctor`, `intent`, `profile`). The `faelight` binary delegates to individual tools.

**Version:** 1.0.0  
**Status:** Production (Optional)

---

## Quick Start
```bash
faelight --help       # Show all commands
faelight health       # Run health check
faelight status       # System overview
```

---

## Commands

### System Status
```bash
faelight status              # Show version, profile, lock status
faelight status --json       # JSON output
faelight --version           # Show faelight version
```

**Direct equivalent:** `dotctl status`

---

### Health Check
```bash
faelight health              # Run health check
faelight health --explain    # Detailed explanations
faelight health --json       # JSON output
```

**Direct equivalent:** `doctor`, `dot-doctor`

---

### Profiles
```bash
faelight profile list        # Show available profiles
faelight profile current     # Show active profile
faelight profile switch <name>  # Switch to profile
```

**Direct equivalent:** `profile list`, `profile switch <name>`

**Available profiles:** default, work, gaming, low-power

---

### Intent Ledger
```bash
faelight intent list         # List all intents
faelight intent list future  # List by category
faelight intent show 067     # Show specific intent
faelight intent search rust  # Search intents
```

**Direct equivalent:** `intent list`, `intent show 067`

---

### Core Protection
```bash
faelight core status         # Check if core is locked
faelight core lock           # Lock core
faelight core unlock         # Unlock core
```

**Direct equivalent:** `lock-core`, `unlock-core`, `core-protect status`

---

### Git Governance
```bash
faelight git status          # Check lock status
faelight git verify          # Verify commit readiness
faelight git install-hooks   # Install git hooks
```

**Direct equivalent:** `faelight-git status`, `fg status`

---

### Sway Integration
```bash
faelight sway reload         # Reload Sway config
faelight sway status         # Show Sway status
```

**Direct equivalent:** `swaymsg reload`

---

### Launch Applications
```bash
faelight launch launcher     # Open app launcher
faelight launch menu         # Open power menu
faelight launch lock         # Lock screen
faelight launch notify "msg" # Send notification
```

**Direct equivalent:** `faelight-launcher`, `faelight-menu`, `faelight-lock`, etc.

---

### Configuration
```bash
faelight config validate     # Validate all config files
faelight config show         # Show current config
faelight config path         # Show config directory
faelight config edit         # Edit config in $EDITOR
```

**Direct equivalent:** Edit `~/.config/faelight/*.toml`

---

### Explain System
```bash
faelight explain profile     # Explain profile system
faelight explain intent      # Explain intent ledger
faelight explain health      # Explain health system
faelight explain core        # Explain core protection
```

---

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON (where supported) |
| `--dry-run` | Show what would happen |
| `--help` | Show help |
| `--version` | Show version |

---

## Shell Aliases (Optional)

Add to `~/.zshrc`:
```bash
alias fl="faelight"
alias fls="faelight status"
alias flh="faelight health"
alias flp="faelight profile"
```

---

## Direct vs Unified Interface

You have **two options** for running commands:

### Option 1: Direct Tools (Recommended)
```bash
doctor                  # Direct health check
intent list             # Direct intent access
profile switch gaming   # Direct profile switch
lock-core               # Direct core lock
```

### Option 2: Unified Faelight CLI
```bash
faelight health         # Via unified interface
faelight intent list    # Via unified interface
faelight profile switch gaming
faelight core lock
```

**Both work identically.** Use whichever you prefer.

---

## Implementation Status

The `faelight` binary exists as a convenience wrapper that delegates to individual tools. Current implementation status:

- ✅ `faelight --help` - Working
- ✅ `faelight --version` - Working
- ⚠️ Other commands - May be partially implemented

**Recommendation:** Use individual tools directly (`doctor`, `intent`, `profile`, etc.) for guaranteed functionality.

---

## Philosophy

The `faelight` unified CLI follows the principle of **"convenience without magic"**:
- Transparent delegation to individual tools
- No hidden behavior
- Optional, not required
- Individual tools remain fully functional

*The spine of the forest.* 🌲
