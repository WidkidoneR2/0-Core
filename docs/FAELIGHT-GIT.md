# 🔧 Faelight Git Governance

Git becomes a policy boundary, not just a storage tool.

## Quick Start
```bash
# Install hooks
faelight git install-hooks

# Check status before committing
faelight git verify

# Check if commits are allowed
faelight git status
```

## Commands

| Command | Description |
|---------|-------------|
| `faelight git install-hooks` | Install pre-commit, commit-msg, pre-push hooks |
| `faelight git remove-hooks` | Remove hooks (restores backups) |
| `faelight git verify` | Check commit readiness |
| `faelight git status` | Check lock status |

## Hooks

### pre-commit
**Purpose:** Block commits when core is locked

When you run `git commit` with core locked:
```
═══════════════════════════════════════════
🔒 COMMIT BLOCKED - Core is locked!
═══════════════════════════════════════════

Run unlock-core to unlock before committing.
```

### commit-msg
**Purpose:** Suggest intent references for significant changes

For commits touching `rust-tools/`, `INTENT/`, `main.rs`, `VERSION`, or `CHANGELOG.md`:
```
💡 This looks like a significant change.
   Consider adding an intent reference:
   Intent: 0XX
```

This is a suggestion, not a block.

### pre-push
**Purpose:** Final verification before push

Runs a quick health check on git status.

## Workflow

### Normal Development
```bash
unlock-core                    # Unlock for changes
# ... make changes ...
faelight git verify            # Check readiness
git add -A
git commit -m "feat: Add feature

Intent: 021"
git push
lock-core                      # Lock when done
```

### When Locked
```bash
git commit -m "test"
# 🔒 COMMIT BLOCKED - Core is locked!

unlock-core
git commit -m "test"           # Now works
```

## Hook Installation

Hooks are installed to `~/0-core/.git/hooks/`:
```
.git/hooks/
├── pre-commit      # Calls faelight-git hook-pre-commit
├── commit-msg      # Calls faelight-git hook-commit-msg
└── pre-push        # Calls faelight-git hook-pre-push
```

Existing hooks are backed up with `.backup` suffix.

## Integration with Core Protection

| Core State | Commits | Pushes |
|------------|---------|--------|
| 🔓 Unlocked | ✅ Allowed | ✅ Allowed |
| 🔒 Locked | ❌ Blocked | ❌ Blocked (no commits to push) |

---
_The forest guards its history._ 🌲
