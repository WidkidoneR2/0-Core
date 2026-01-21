# faelight-snapshot v1.0.0

🌲 **Btrfs Snapshot Manager** - Elegant wrapper around `snapper` for managing system snapshots.

## Features

- **Dual-config management** - Manages both `root` and `home` snapshots simultaneously
- **Safety confirmations** - 3-second delay before destructive operations
- **Rollback protection** - Type-to-confirm before system rollback
- **Health monitoring** - Verify snapper installation and btrfs functionality
- **Production-tested** - Used daily on Faelight Forest systems

## Usage
```bash
# List all snapshots (root + home)
faelight-snapshot list

# Create pre-update snapshot
faelight-snapshot create "before kernel update"

# Show changes since snapshot #42
faelight-snapshot diff 42

# Delete snapshot #42 (3-second confirmation)
faelight-snapshot delete 42

# Rollback to snapshot #42 (requires confirmation + reboot)
faelight-snapshot rollback 42

# System status (configs, counts, disk usage)
faelight-snapshot status

# Health check
faelight-snapshot --health
```

## Architecture

Wraps `snapper` with:
- Automatic dual-config operations (root + home)
- UX improvements (emojis, confirmations, truncated output)
- Safety guardrails (delays, type-to-confirm)
- Health monitoring

## System Requirements

- `snapper` installed and configured
- Btrfs filesystem on `/` and `/home`
- Snapper configs: `root` and `home`
- `sudo` privileges for snapshot operations

## Demo for Linus
```bash
# Show health status
faelight-snapshot --health

# Show current snapshots
faelight-snapshot list

# Create a test snapshot
faelight-snapshot create "demo for Linus"

# Show system status
faelight-snapshot status
```
