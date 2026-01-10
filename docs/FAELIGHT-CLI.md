# 🌲 Faelight CLI

The unified command-line interface for Faelight Forest. One binary to rule them all.

## Quick Start
```bash
faelight status          # System overview
faelight health          # Run health check
faelight profile list    # Show profiles
faelight intent list     # Browse intents
```

## Commands

### System

| Command | Description |
|---------|-------------|
| `faelight status` | Show version, profile, lock status |
| `faelight status --json` | JSON output |

### Health

| Command | Description |
|---------|-------------|
| `faelight health` | Run health check (delegates to dot-doctor) |
| `faelight health --explain` | Detailed explanations |
| `faelight health --json` | JSON output |

### Profiles

| Command | Description |
|---------|-------------|
| `faelight profile list` | Show available profiles |
| `faelight profile current` | Show active profile |
| `faelight profile switch <name>` | Switch to profile |

Available profiles: `default`, `work`, `gaming`, `low-power`

### Intents

| Command | Description |
|---------|-------------|
| `faelight intent list` | List all intents |
| `faelight intent list future` | List by category |
| `faelight intent show 021` | Show specific intent |
| `faelight intent search rust` | Search intents |

### Core Protection

| Command | Description |
|---------|-------------|
| `faelight core status` | Check if core is locked |
| `faelight core lock` | Lock core (block commits) |
| `faelight core unlock` | Unlock core |

### Git Governance

| Command | Description |
|---------|-------------|
| `faelight git status` | Check lock status |
| `faelight git verify` | Verify commit readiness |
| `faelight git install-hooks` | Install git hooks |
| `faelight git remove-hooks` | Remove git hooks |

### Sway

| Command | Description |
|---------|-------------|
| `faelight sway reload` | Reload Sway config |
| `faelight sway status` | Show Sway version |

### Launch

| Command | Description |
|---------|-------------|
| `faelight launch launcher` | Open app launcher |
| `faelight launch menu` | Open power menu |
| `faelight launch lock` | Lock screen |
| `faelight launch notify "msg"` | Send notification |

### Config

| Command | Description |
|---------|-------------|
| `faelight config validate` | Validate all config files |
| `faelight config show` | Show current config |
| `faelight config path` | Show config directory |
| `faelight config edit` | Edit config in $EDITOR |

### Explain

| Command | Description |
|---------|-------------|
| `faelight explain profile` | Explain profile system |
| `faelight explain intent` | Explain intent ledger |
| `faelight explain health` | Explain health system |
| `faelight explain core` | Explain core protection |

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON (where supported) |
| `--dry-run` | Show what would happen |
| `--help` | Show help |
| `--version` | Show version |

## Shell Aliases

Add to your shell config:
```bash
alias fl="faelight"
alias fls="faelight status"
alias flh="faelight health"
alias flp="faelight profile"
```

---
_The spine of the forest._ 🌲
