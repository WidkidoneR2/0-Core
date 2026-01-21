# core-protect v1.0.0

🛡️ **Immutable Core Protection** - Filesystem-level protection for 0-Core using Linux immutability flags.

## Philosophy

> "The best way to prevent accidents is to make them impossible."

`core-protect` uses Linux's `chattr +i` (immutable flag) to make 0-core **physically unchangeable** at the filesystem level. Not by convention, not by scripts checking - by the kernel itself rejecting writes.

## How It Works

### Immutable Flag (`chattr +i`)
When a file/directory has the immutable flag:
- **Cannot be modified** - Even by root
- **Cannot be deleted** - Even by root
- **Cannot be renamed** - Even by root
- **Protection at kernel level** - Not bypassable

This is stronger than permissions, stronger than ACLs, stronger than SELinux contexts.

### The `edit` Workflow
When editing a protected package:

1. **Risk Assessment** - Reads `.dotmeta` blast radius
2. **Warning Display** - Shows failure modes for critical/high packages
3. **Confirmation** - Requires explicit acknowledgment
4. **Auto-Backup** - Creates git stash for critical/high packages
5. **Temporary Unlock** - Removes immutability
6. **Edit** - Opens your $EDITOR
7. **Auto Re-lock** - Restores protection after edit

You can't forget to re-lock - it's automatic.

## Usage

### Lock Core (Prevent Changes)
```bash
core-protect lock

# Output:
# 🔒 Locking 0-core (immutable protection)...
# ✅ Core protected! Cannot modify without unlocking.
```

Now try to edit a file in 0-core:
```bash
$ echo "test" > ~/0-core/shell-zsh/.zshrc
bash: ~/0-core/shell-zsh/.zshrc: Operation not permitted
```

Even `sudo` can't modify it!

### Unlock Core (Allow Changes)
```bash
core-protect unlock

# Output:
# 🔓 Unlocking 0-core for editing...
# ✅ Core unlocked! You can now edit.
```

### Check Status
```bash
core-protect status

# Output:
# 📊 Checking 0-core protection status...
# 🔒 Core is LOCKED (immutable)
```

### Safe Edit (Recommended)
```bash
# Edit with automatic backup + re-lock
core-protect edit shell-zsh

# For critical packages, shows warning:
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ⚠️  CRITICAL BLAST RADIUS COMPONENT
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#
# Package: hooks
# Risk: 🔴 Critical (system unusable if broken)
#
# Failure may cause:
#   • Git commits fail silently
#   • Pre-push checks bypassed
#   • Intent guard disabled
#
# ⚠️  Auto-backup will be created before editing
#
# Type 'CRITICAL' to proceed:
```

### Health Check
```bash
core-protect --health

# Output:
# 🏥 core-protect v1.0.0 - Health Check
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#   Checking chattr command... ✅
#   Checking lsattr command... ✅
#   Checking 0-core directory... ✅ /home/user/0-core
#   Checking sudo access... ✅
#   Checking protection status... 🔒 LOCKED
#
# ✅ All systems operational
```

## Risk-Based Editing

### Critical Packages (Type "CRITICAL" to proceed)
- **hooks** - Git integration
- **system** - Boot/service files
- Creates automatic backup before edit

### High Packages (Type "yes" to proceed)
- **wm-sway** - Window manager
- Creates automatic backup before edit

### Medium Packages (Type "y" to proceed)
- **shell-zsh** - Shell config
- No automatic backup

### Low Packages (No confirmation)
- **docs** - Documentation
- **theme-*** - Themes

## Use Cases

### 1. Production System Protection
```bash
# Lock after setup
core-protect lock

# System now immutable
# Prevents accidental edits during normal use
```

### 2. Safe Critical Edits
```bash
# Edit hooks (critical package)
core-protect edit hooks

# Shows warning, creates backup, auto re-locks
```

### 3. Quick Status Check
```bash
# Morning ritual
core-protect status
```

### 4. Temporary Unlock for Bulk Changes
```bash
# Unlock for migration
core-protect unlock

# Make multiple changes
git add .
git commit -m "migration"

# Re-lock
core-protect lock
```

## Integration with Workflow

### Daily Usage
```bash
# Keep locked by default
core-protect status  # Should show LOCKED

# Edit via core-protect only
core-protect edit shell-zsh
```

### Git Hooks
```bash
# In pre-commit hook
if core-protect status | grep -q "LOCKED"; then
  echo "⚠️  Core is locked - use 'core-protect edit' instead"
  exit 1
fi
```

### CI/CD
```bash
# Verify protection in CI
core-protect --health
```

## Why This Matters

Traditional approaches to preventing accidental edits:
1. **Convention** - "Just don't edit there" (fails with typos)
2. **Permissions** - `chmod 444` (easily bypassed with sudo)
3. **Scripts** - Check before commit (can be forgotten)

`core-protect` uses **kernel-level immutability**:
- Can't be bypassed
- Can't be forgotten
- Works even if you're not thinking about it

### Example: Accidental Edit
```bash
# Without core-protect
$ rm -rf ~/0-core/hooks
# 💥 Critical system destroyed

# With core-protect
$ rm -rf ~/0-core/hooks
rm: cannot remove '~/0-core/hooks': Operation not permitted
# ✅ Kernel blocked the operation
```

## Technical Details

### What Gets Protected
- All files in `~/0-core`
- All subdirectories
- The `0-core` directory itself

### Commands Used
```bash
# Lock
sudo chattr +i ~/0-core/*
sudo chattr +i ~/0-core

# Unlock
sudo chattr -i ~/0-core/*
sudo chattr -i ~/0-core

# Check
lsattr -d ~/0-core
```

### Requirements
- Linux kernel with `chattr` support (ext4, xfs, btrfs)
- `sudo` privileges
- `$EDITOR` environment variable set

### Limitations
- Requires sudo (needs to modify filesystem attributes)
- Linux-only (uses Linux-specific inode flags)
- Not effective against kernel tampering (but you have bigger problems)

## Design Principles

1. **Immutability by Default** - Lock after setup, unlock only for edits
2. **Kernel Enforcement** - Not bypassable by scripts or permissions
3. **Risk-Aware Editing** - Warnings scale with blast radius
4. **Automatic Safety** - Backups and re-locking are automatic
5. **Fail-Safe** - If you forget to lock, nothing breaks (just less protected)

## Demo for Linus
```bash
# Show health
core-protect --health

# Show status
core-protect status

# Try to edit (should fail)
echo "test" > ~/0-core/shell-zsh/.zshrc

# Safe edit with warnings
core-protect edit hooks

# Check re-locked
core-protect status
```

## Why This Exists

0-core is the **foundation** of Faelight Forest. If it breaks, everything breaks. `core-protect` makes accidental breakage **physically impossible** rather than merely discouraged.

It's the difference between a "Do Not Touch" sign and a locked safe.
