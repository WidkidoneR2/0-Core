# faelight-update v0.4.0

🌲 **Intelligent update manager for Faelight Forest**

Like `topgrade`, but with **safety, awareness, and control** built in.

---

## Philosophy

**Topgrade problem:** Updates everything blindly without context or safeguards.

**faelight-update solution:**
- ✅ Health check BEFORE updating (don't update a broken system)
- ✅ Impact analysis for critical packages (kernel, display server, etc.)
- ✅ Snapshot support (rollback if things break)
- ✅ Interactive mode (choose what to update)
- ✅ Dry-run (see what WOULD update)
- ✅ Category filtering (update only cargo, only AUR, etc.)
- ✅ JSON output (scriptable, parseable)

**Manual control over automation.**

---

## Features

### Safety First
- **Pre-update health check** - won't update if system is unhealthy
- **Impact analysis** - warns about critical packages (kernel, sway, etc.)
- **Snapshot integration** - create restore point before updates
- **Dry-run mode** - see changes before applying

### Intelligent Detection
Checks multiple sources:
- **pacman** - Official Arch packages
- **AUR** - User repository (via paru)
- **cargo** - Rust crates (`cargo install-update`)
- **neovim** - LazyVim plugin updates
- **workspace** - Cargo workspace tools (0-core rust-tools)

### Flexible Control
- **Interactive** - select packages individually
- **Category filtering** - `--only cargo` or `--skip aur`
- **JSON output** - parse results programmatically
- **Verbose mode** - see version changes

---

## Usage

### Check what's available
```bash
faelight-update --dry-run
```

### Update everything safely
```bash
faelight-update
```
**Runs health check first** - won't proceed if system unhealthy.

### Interactive selection
```bash
faelight-update --interactive
```
Choose which packages to update.

### Update only Rust tools
```bash
faelight-update --only cargo,workspace
```

### Skip AUR packages
```bash
faelight-update --skip aur
```

### Create snapshot before updating
```bash
faelight-update --snapshot
```
Requires `faelight-snapshot` installed.

### See detailed version changes
```bash
faelight-update --verbose
```

### Get JSON output (for scripts)
```bash
faelight-update --json | jq .
```

---

## Update Categories

| Category | Source | Command Used |
|----------|--------|--------------|
| **pacman** | Official repos | `checkupdates` |
| **aur** | AUR packages | `paru -Qua` |
| **cargo** | Installed crates | `cargo install-update -l` |
| **neovim** | LazyVim plugins | `:Lazy sync` |
| **workspace** | 0-core rust-tools | Cargo workspace check |

---

## How It's Better Than Topgrade

| Feature | topgrade | faelight-update |
|---------|----------|-----------------|
| Pre-update health check | ❌ | ✅ |
| Impact analysis | ❌ | ✅ |
| Interactive selection | Partial | ✅ Full |
| Category filtering | ❌ | ✅ |
| Snapshot integration | ❌ | ✅ |
| JSON output | ❌ | ✅ |
| Workspace awareness | ❌ | ✅ |
| Dry-run mode | ✅ | ✅ |
| Stops if unhealthy | ❌ | ✅ |

**Key difference:** topgrade updates blindly. faelight-update **thinks first**.

---

## Examples

### Daily update routine
```bash
# Check what's available
faelight-update --dry-run

# Update everything safely (with health check)
faelight-update

# Or update interactively
faelight-update --interactive
```

### Before important work
```bash
# Create snapshot, then update
faelight-update --snapshot

# Or skip updates if system is healthy
doctor && echo "System good, skip updates"
```

### Update only development tools
```bash
# Rust tools only
faelight-update --only cargo,workspace

# Skip system packages, update dev tools
faelight-update --skip pacman,aur
```

### CI/CD health check
```bash
# Check for critical updates
faelight-update --dry-run --json | jq '.critical_count'
```

---

## Impact Analysis

**Critical packages** get special warnings:
- `linux` (kernel) - requires reboot
- `sway` - requires session restart
- `systemd` - core system component
- Graphics drivers - may break display

**Example output:**
```
⚠️  Critical updates available:
  - linux: 6.11.5 → 6.11.6 (REQUIRES REBOOT)
  - sway: 1.9 → 1.10 (REQUIRES SESSION RESTART)
  
Continue? (y/N)
```

---

## Integration

### With dot-doctor
```bash
doctor && faelight-update
```
Only update if system is healthy.

### With faelight-snapshot
```bash
faelight-update --snapshot
```
Auto-creates restore point.

### In scripts
```bash
#!/bin/bash
# Weekly update script

# Check health first
if ! doctor --quiet; then
  echo "System unhealthy, skipping updates"
  exit 1
fi

# Update with snapshot
faelight-update --snapshot --json | tee update-log.json
```

---

## Configuration

No config file needed - intelligent defaults:
- Always runs health check (skip with `--skip-health`)
- Auto-detects installed package managers
- Respects `$CARGO_HOME`, `$XDG_CONFIG_HOME`
- Uses system timezone for timestamps

---

## Exit Codes

- `0` - Updates successful (or no updates available)
- `1` - Updates failed or health check failed
- `2` - User cancelled interactive selection

---

## Design Philosophy

**Three principles:**

1. **Safety over speed** - health check before updates
2. **Transparency over convenience** - show what will change
3. **Control over automation** - interactive when needed

**The goal:** Make updates boring and safe, not exciting and risky.

---

## Part of 0-Core

One of 30+ Rust tools in the Faelight Forest ecosystem.

Complements `dot-doctor` (health checks) and `faelight-snapshot` (rollback).

See: https://github.com/WidkidoneR2/0-Core
