# faelight-snapshot v2.0.0

Btrfs snapshot manager - wrapper around `snapper` for safe system snapshots.

## Features

- ✅ List root and home snapshots
- ✅ Create named snapshots
- ✅ Delete snapshots by number
- ✅ Compare changes (diff)
- ✅ Rollback to previous state
- ✅ System status
- ✅ Health check integration

## Usage
```bash
# List all snapshots
faelight-snapshot list

# Create snapshot before update
faelight-snapshot create "before kernel update"

# Show what changed since snapshot #22
faelight-snapshot diff 22

# Delete snapshot
faelight-snapshot delete 22

# Rollback (requires reboot)
faelight-snapshot rollback 22

# Check status
faelight-snapshot status

# Health check
faelight-snapshot --health
```

## Commands

- `list [root|home]` - List snapshots (default: both)
- `create <description>` - Create pre-update snapshot
- `delete <number>` - Delete snapshot by number
- `diff <number>` - Show changes since snapshot
- `rollback <number>` - Rollback to snapshot (requires reboot)
- `status` - Show snapshot system status

## Security

Uses `sudo` for snapper operations. Requires:
- Btrfs filesystem
- snapper installed
- Proper sudo configuration

## Integration

Part of Faelight Forest:
- Health monitoring
- Pre-update workflow
- System recovery

## Notes

- Automatic hourly snapshots enabled via snapper
- Snapshots are stored on Btrfs subvolumes
- Rollback requires reboot to take effect
- Only works on Btrfs filesystems

## Version: 2.0.0
