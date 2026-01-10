# 🛠️ Faelight Forest Tools Reference

Complete reference for all custom Rust tools.

## Core Tools

| Tool | Version | Description | Location |
|------|---------|-------------|----------|
| `faelight` | 0.1.0 | Unified CLI | `~/0-core/scripts/faelight` |
| `dot-doctor` | 0.2.0 | Health Engine | `~/0-core/scripts/dot-doctor` |
| `faelight-git` | 0.1.0 | Git Governance | `~/0-core/scripts/faelight-git` |

## UI Tools

| Tool | Version | Description | Keybinding |
|------|---------|-------------|------------|
| `faelight-bar` | 0.7.0 | Status bar | Auto-start |
| `faelight-notify` | 0.2.0 | Notification daemon | Auto-start |
| `faelight-launcher` | 0.1.0 | App launcher | `Super+Space` |
| `faelight-menu` | 0.1.0 | Power menu | `Super+Shift+Escape` |
| `faelight-lock` | 0.1.0 | Screen locker | `Super+Ctrl+Escape` |

## Utility Tools

| Tool | Description |
|------|-------------|
| `dotctl` | Package management |
| `core-diff` | Configuration diff |
| `bump-system-version` | Version management |
| `profile` | Profile switching |
| `intent` | Intent ledger CLI |

## Quick Commands
```bash
# Status & Health
faelight status              # System overview
faelight health              # Health check
faelight health --explain    # Detailed health

# Profiles
faelight profile list        # Show profiles
faelight profile switch work # Switch profile

# Git
faelight git verify          # Check commit readiness
faelight git status          # Lock status

# Config
faelight config validate     # Validate configs
faelight config show         # Show config

# Core
faelight core lock           # Lock core
faelight core unlock         # Unlock core

# Intents
faelight intent list         # List all
faelight intent show 021     # Show specific
```

## Keybindings Summary

| Key | Action |
|-----|--------|
| `Super+Space` | App launcher |
| `Super+Shift+Escape` | Power menu |
| `Super+Ctrl+Escape` | Lock screen |
| `Super+Alt+k` | KeePassXC |
| `Super+Alt+e` | Tutanota |
| `Super+Alt+n` | Notesnook |
| `Super+Alt+d` | Discord |
| `Super+Alt+c` | Filen |

## Architecture
```
faelight (unified CLI)
├── health     → dot-doctor
├── profile    → profile script
├── intent     → intent script
├── core       → lock-core/unlock-core
├── sway       → swaymsg
├── git        → faelight-git
├── config     → TOML validation
├── launch     → faelight-* tools
└── status     → VERSION + state files
```

## Documentation

| Document | Description |
|----------|-------------|
| [HEALTH-ENGINE.md](HEALTH-ENGINE.md) | dot-doctor reference |
| [FAELIGHT-CLI.md](FAELIGHT-CLI.md) | Unified CLI reference |
| [FAELIGHT-GIT.md](FAELIGHT-GIT.md) | Git governance |
| [FAELIGHT-CONFIG.md](FAELIGHT-CONFIG.md) | TOML configuration |

---
_The forest's toolkit._ 🌲
