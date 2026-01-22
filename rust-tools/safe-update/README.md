# safe-update v1.0.0

🛡️ **Safe System Updates** - Update your system with snapshots, health checks, and intelligent recovery.

## Philosophy

> "Manual control over automation. Reliability over convenience."

safe-update arose from [Incident 001](../../INTENT/incidents/001-password-sudo-failure.md) - the lesson that automation at boot can fail catastrophically. Updates are **never** automatic. You trigger them manually, with full visibility and control.

## The Problem

System updates can:
- Break packages (library mismatches)
- Leave orphaned configs (.pacnew files)
- Introduce regressions
- Fail without clear recovery paths

**The Arch way:** You're responsible for your system.  
**The 0-Core way:** We give you the tools to update safely.

## The Solution

safe-update provides:
- 📸 Pre/post update snapshots (automatic rollback points)
- 🏥 Health checks before and after
- 🔧 Automatic yay rebuild on library mismatch
- 📋 .pacnew file detection
- 🔄 Intelligent recovery from common failures
- 📊 Update logging and drift tracking
- 🎯 Dry-run mode (see before doing)

## Features

- ✅ Snapper snapshots (before/after)
- ✅ Pre-flight health checks
- ✅ Confirmation prompts
- ✅ Dry-run mode
- ✅ Automatic yay recovery
- ✅ .pacnew detection
- ✅ Post-update health verification
- ✅ Rollback instructions
- ✅ Entropy baseline updates
- ✅ Update logging

## Usage

### Standard Update
```bash
safe-update
```

**What happens:**
1. Pre-flight health checks
2. Confirmation prompt
3. Pre-update snapshot created
4. System update (topgrade)
5. Automatic recovery if yay breaks
6. .pacnew file detection
7. Post-update snapshot created
8. Health verification
9. Entropy baseline updated
10. Update log saved

### Dry Run (See Before Doing)
```bash
safe-update --dry-run
```

Shows what WOULD be updated without making changes:
- Package updates
- AUR updates
- Cargo updates
- Everything topgrade would do

**Use this first** to preview updates!

### Skip Confirmation
```bash
safe-update --yes
```

Skips the confirmation prompt (still shows preview).

### Skip Snapshots
```bash
safe-update --skip-snapshot
```

For systems without snapper or when you want faster updates.

### Combination Flags
```bash
safe-update --dry-run --skip-snapshot    # Quick preview
safe-update --yes --skip-snapshot        # Fast update (use carefully!)
```

## Flags
```
--dry-run            Preview updates without applying
--yes, -y            Skip confirmation prompt
--skip-snapshot      Don't create snapshots
--version, -v        Show version
--health             Run pre-flight checks only
--help, -h           Show help
```

## Output Example

### Standard Update
```
═══════════════════════════════════════════════════════════
🛡️  Safe System Update v1.0.0
═══════════════════════════════════════════════════════════

🏥 Pre-flight Checks
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✅ Snapper available
  ✅ Internet connection working
  ✅ Disk space sufficient (15.3 GB free)
  ✅ System health: 100%

📋 Update Preview
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Running dry-run to preview updates...

[topgrade dry-run output showing what would update]

Packages to update: 23
AUR packages: 5
Cargo crates: 12

⚠️  Proceed with update? (yes/no): yes

📸 Creating Snapshots
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✅ Pre-update snapshot created (#127)

🔄 System Update
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Running topgrade...

[Update proceeds interactively]

✅ Update completed successfully!

📋 Post-Update Checks
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✅ No .pacnew files found
  ✅ Post-update snapshot created (#128)

🏥 System Health Check
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[doctor runs]
✅ System health: 100%

📊 Drift Tracking
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✅ Entropy baseline updated

💡 Rollback Available
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Before: Snapshot #127
  After:  Snapshot #128
  
  To rollback: sudo snapper -c root rollback 127

═══════════════════════════════════════════════════════════
✅ Safe update complete! System is healthy! 🌲
═══════════════════════════════════════════════════════════

Update log: ~/.local/share/faelight/update-logs/2026-01-22-2045.log
```

## Automatic Recovery

### Yay Library Mismatch

**Problem:** After system library updates, yay may break with:
```
error while loading shared libraries: libalpm.so.14
```

**Solution:** safe-update automatically:
1. Detects the library mismatch
2. Rebuilds yay from source
3. Retries the update
4. Reports success or failure

**You don't have to do anything!**

### Network Failures

If network fails during update:
1. Update stops cleanly
2. Snapshot still created
3. System remains in working state
4. Re-run `safe-update` when network returns

## Common Workflows

### Weekly Update Routine
```bash
# 1. Preview what would update
safe-update --dry-run

# 2. If comfortable, proceed
safe-update

# 3. Check drift
entropy-check

# 4. Review any .pacnew files if found
sudo pacdiff
```

### Quick Update (No Snapshots)
```bash
# When you trust the update and want speed
safe-update --skip-snapshot --yes

# Still gets health checks and recovery!
```

### Pre-Flight Check Only
```bash
# Check if system is ready for update
safe-update --health
```

### After Failed Update
```bash
# If update failed and system is broken:

# 1. Rollback to pre-update snapshot
sudo snapper -c root rollback <snapshot-number>
sudo reboot

# 2. After reboot, check health
doctor

# 3. Investigate what failed
journalctl -b -1  # Previous boot logs
```

## Safety Mechanisms

### Pre-Flight Checks

Before updating, safe-update verifies:
- ✅ **Snapper available** (if not using --skip-snapshot)
- ✅ **Internet connection** (can reach archlinux.org)
- ✅ **Disk space** (at least 2GB free in /)
- ✅ **System health** (doctor reports 100%)

**If any check fails, update is aborted.**

### Snapshots

Btrfs snapshots via snapper provide instant rollback:
- **Pre-update:** Captures system state before changes
- **Post-update:** Captures result for comparison
- **Rollback:** `sudo snapper -c root rollback <number>`

**Note:** Requires btrfs filesystem and snapper configured.

### Confirmation

Unless `--yes` flag is used, you must confirm:
```
⚠️  Proceed with update? (yes/no): 
```

Type exactly `yes` to proceed. Anything else cancels.

### Atomic Updates

Updates run in stages:
1. Snapshot (safe)
2. Update (can be rolled back)
3. Snapshot (safe)
4. Health check (informational)

**Each stage can fail independently without breaking the system.**

### Logging

All updates logged to:
```
~/.local/share/faelight/update-logs/YYYY-MM-DD-HHMM.log
```

Contains:
- Timestamp
- Packages updated
- Errors encountered
- Snapshot numbers
- Health check results

## Integration

### With entropy-check

After successful update:
```bash
entropy-check --baseline  # Update drift baseline
```

**Built-in:** safe-update does this automatically!

### With doctor

Health verification:
```bash
doctor  # Before update
safe-update
doctor  # After update
```

### With topgrade

safe-update wraps topgrade with safety:
- Pre/post snapshots
- Automatic yay recovery
- Health verification

**You never run `topgrade` directly.**

## Files Created
```
~/.local/share/faelight/update-logs/           Update logs
/var/lib/snapper/snapshots/                   Btrfs snapshots
~/.config/faelight/entropy-baseline.json      Updated baseline
```

## Requirements

### Required
- Arch Linux (uses pacman, topgrade, yay)
- topgrade installed
- yay (AUR helper)
- Internet connection

### Optional
- snapper + btrfs (for snapshots, can skip with --skip-snapshot)
- dot-doctor (for health checks, skips if missing)
- entropy-check (for drift tracking, skips if missing)

## Troubleshooting

### "Snapper not available"

**Solution 1:** Install snapper
```bash
yay -S snapper
sudo snapper -c root create-config /
```

**Solution 2:** Skip snapshots
```bash
safe-update --skip-snapshot
```

### "Not enough disk space"

**Need:** At least 2GB free in `/`

**Solution:**
```bash
# Clean package cache
sudo pacman -Sc

# Remove orphaned packages
sudo pacman -Rns $(pacman -Qdtq)

# Check disk usage
df -h /
```

### "Update failed for unknown reason"

**Investigate:**
```bash
# Check update log
tail -100 ~/.local/share/faelight/update-logs/$(ls -t ~/.local/share/faelight/update-logs/ | head -1)

# Check system journal
journalctl -b -n 100

# Try manual topgrade
topgrade --dry-run  # See what failed
```

### ".pacnew files found"

**Meaning:** pacman created new config files instead of overwriting yours.

**Action Required:**
```bash
# Review and merge configs
sudo pacdiff

# Or manually for each file
vimdiff /etc/config.pacnew /etc/config
```

### "Yay rebuild failed"

**Manual recovery:**
```bash
cd /tmp
rm -rf yay
git clone https://aur.archlinux.org/yay.git
cd yay
makepkg -si
```

## Design Principles

### 1. Manual Control
Updates are **never** automatic. You trigger them when ready.

### 2. Maximum Visibility
You see everything: preview, progress, results, recovery paths.

### 3. Fail-Safe
Every stage can fail without breaking the system. Snapshots provide escape hatch.

### 4. Intelligence
Automatic recovery from common failures (yay rebuild, etc).

### 5. Integration
Works with entropy-check, doctor, topgrade - leverages the ecosystem.

## Comparison

### vs `sudo pacman -Syu`
- ❌ No snapshots
- ❌ No recovery
- ❌ No health checks
- ❌ Manual only (pacman packages)

### vs `topgrade`
- ✅ Updates everything (pacman + AUR + cargo + etc)
- ❌ No snapshots
- ❌ No recovery
- ❌ No health verification

### vs `safe-update`
- ✅ Updates everything (via topgrade)
- ✅ Automatic snapshots
- ✅ Intelligent recovery
- ✅ Health verification
- ✅ Drift tracking
- ✅ Rollback instructions

## Why This Exists

From [Incident 001](../../INTENT/incidents/001-password-sudo-failure.md):

**What went wrong:** Automated updates at boot failed due to credential issues, locked account.

**Lesson learned:**
> "Boot environment ≠ User environment. Never automate privileged operations."

**Solution:** Manual updates with maximum safety.

safe-update embodies that lesson - you control when updates happen, but the tool ensures they happen safely.

## Exit Codes

- `0` - Update succeeded
- `1` - Pre-flight checks failed
- `2` - User cancelled
- `3` - Update failed
- `4` - Post-update health failed

## Philosophy in Practice

**Scenario:** Major kernel update available.

**Without safe-update:**
```bash
sudo pacman -Syu  # YOLO
# Update fails, system broken, no recovery path
```

**With safe-update:**
```bash
safe-update --dry-run  # Preview
# See kernel update, feel nervous
safe-update  # Proceed
# Snapshot created
# Update happens
# Snapshot created
# Health verified
# Rollback path shown

# If it breaks:
sudo snapper -c root rollback <pre-update-number>
sudo reboot
# System restored
```

**This is intentional stewardship.**

---

**Update safely. Trust, then verify. 🛡️🌲**
