# entropy-check v1.0.0

🔍 **Configuration Drift Detection** - Detect silent system changes before they become catastrophic.

## Philosophy

> "Systems drift. Humans change. Contexts evolve."  
> entropy-check watches for the silent changes.

**Surfaces drift. Doesn't fix it.** You decide how to respond.

## What It Does

entropy-check monitors your system for unexpected changes:

- **Config Drift** - Files modified outside stow management
- **Service Drift** - Services that changed state unexpectedly
- **Binary Drift** - Package versions changed since last check
- **Symlink Drift** - New broken symlinks since baseline
- **Untracked Files** - New files in managed directories

## The Hyprland Lesson

**2.3 million configuration errors from one update.**

That's what happens when systems change silently. entropy-check catches drift early.

## Usage

### Create Baseline (First Time)
```bash
# Create initial snapshot of your system
entropy-check --baseline
```

Creates: `~/.config/faelight/entropy-baseline.json`

### Check for Drift
```bash
# Check current state against baseline
entropy-check
```

Output:
```
🔍 entropy-check v1.0.0 - Configuration Drift Detection

📊 Drift Report - 0-Core v7.6.5
Baseline created: 2026-01-22
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Config Drift (0 files)
✅ Service Drift (0 changes)
✅ Binary Drift (0 changes)
✅ Symlink Drift (0 new breaks)
⚠️  Untracked Files (3 new)
   /home/user/.config/app/config.json (new file)
   ...

💡 Recommendation:
   New untracked files detected, review and add to baseline if intended
```

### View Drift Trends
```bash
# Show drift history over time
entropy-check --trends
```

Output:
```
📊 Drift Trends - Last 30 days
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Total checks: 15

Recent Drift History:
  ✅ 2026-01-22 - 0 total drifts (v7.6.5)
  ⚠️  2026-01-21 - 3 total drifts (v7.6.5)
      config:1 untracked:2
  🔴 2026-01-20 - 8 total drifts (v7.6.4)
      config:3 services:2 untracked:3

📈 Statistics:
   Average drift per check: 2.1
   Highest drift: 8
   Clean checks: 10/15
```

### Machine-Readable Output
```bash
# Get JSON output for scripting
entropy-check --json
```

## Common Workflows

### Daily Check
```bash
# Add to your morning routine
entropy-check
```

### After System Update
```bash
# Check what changed
entropy-check

# If changes are intentional, update baseline
entropy-check --baseline
```

### Before Major Changes
```bash
# Snapshot current state
entropy-check --baseline

# Make changes...

# See what drifted
entropy-check
```

### Monitoring Over Time
```bash
# Check trends
entropy-check --trends

# Export history for analysis
entropy-check --json > drift-$(date +%Y-%m-%d).json
```

## What Gets Tracked

### Config Files
- All files in `~/.config`
- Checksummed with SHA256
- Detects modifications, not just timestamps

### Services
- systemd user services
- Current state (active/inactive)
- Detects unexpected state changes

### Binaries
- Installed package versions
- Detects package updates

### Symlinks
- All symlinks in `~/.config`
- Target paths
- Detects broken links

### Untracked Files
- New files appearing in managed directories
- Helps catch config file creep

## Files Created
```
~/.config/faelight/
├── entropy-baseline.json    # System baseline snapshot
└── entropy-history.json     # Last 30 days of drift checks
```

## Integration with Workflow

### With dot-doctor
```bash
# System health includes drift check
doctor
# Calls entropy-check internally
```

### With safe-update
```bash
# Safe updates check drift before/after
safe-update
# Creates baseline before update
# Checks drift after update
```

### With Intent Ledger

When you make intentional changes:
```bash
# Record the intent
intent add decision "Update sway config for ultrawide monitor"

# Make changes...

# Update baseline to match intent
entropy-check --baseline
```

## Example Scenarios

### Scenario 1: Unexpected Config Change
```bash
$ entropy-check

⚠️  Config Drift (1 file)
   ~/.config/sway/config (modified 2 hours ago)
```

**Action:** Investigate who/what changed it. If intentional, update baseline.

### Scenario 2: Service State Changed
```bash
$ entropy-check

⚠️  Service Drift (1 change)
   faelight-bar.service (inactive → active)
```

**Action:** Normal after manual start. Update baseline if this is the new normal.

### Scenario 3: New Files Appeared
```bash
$ entropy-check

⚠️  Untracked Files (5 new)
   ~/.config/app/cache/... (new files)
```

**Action:** Normal app behavior. These files aren't dangerous, but you're aware.

### Scenario 4: Clean System
```bash
$ entropy-check

✅ Config Drift (0 files)
✅ Service Drift (0 changes)
✅ Binary Drift (0 changes)
✅ Symlink Drift (0 new breaks)
✅ Untracked Files (0 new)

🎉 System stable! No drift detected.
```

**Action:** Nothing. System is clean.

## Flags
```
--baseline          Create/update baseline snapshot
--trends            Show drift history (last 30 days)
--json              Output in JSON format
--version, -v       Show version
--health            Run health check
--help, -h          Show help
```

## Design Principles

### 1. Observe, Don't Fix
entropy-check **never** modifies your system. It only reports.

### 2. Baseline is Truth
You define what "normal" looks like via baseline.

### 3. Drift is Neutral
Drift isn't bad - it's information. You decide if it's a problem.

### 4. History Matters
Trends reveal patterns. One-off drift vs. systematic drift are different.

### 5. Transparency
All data stored in plain JSON. You own it.

## Technical Details

- **Language:** Rust
- **Hashing:** SHA256 for file integrity
- **Storage:** JSON (human-readable)
- **History:** Last 30 days auto-pruned
- **Depth:** Max 3 levels in ~/.config

## Why This Exists

Manual config management has blind spots:
- Files change when you're not looking
- Services restart automatically
- Packages update in the background
- Symlinks break silently

entropy-check makes the invisible visible.

## Performance

- **Baseline creation:** ~1-2 seconds
- **Drift check:** ~0.5-1 second
- **Minimal overhead:** Only scans ~/.config
- **Efficient:** Checksums cached in baseline

## Limitations

- Only monitors `~/.config` directory
- Can't detect all types of drift (kernel params, etc.)
- Baseline must be updated manually after intentional changes
- 30-day history limit

These are intentional - entropy-check does one thing well.

## Recovery from Drift

entropy-check doesn't fix drift, but guides recovery:
```bash
# 1. Identify drift
entropy-check

# 2. Investigate cause
git log ~/.config/file  # If in git
intent list             # Check recent intents

# 3. Decide action
# Option A: Change was good → update baseline
entropy-check --baseline

# Option B: Change was bad → revert
git checkout ~/.config/file
stow --restow package-name

# Option C: Partial drift → manual fix
# Edit file, then update baseline
```

## Integration Example
```bash
# Morning routine
alias morning='clear && fortune && echo && doctor && entropy-check --trends'

# After updates
alias check-drift='entropy-check && echo && entropy-check --trends'

# Before intent work
alias intent-snapshot='entropy-check --baseline'
```

## Exit Codes

- `0` - Success (may or may not have drift)
- `1` - Error (baseline missing, invalid data, etc.)

Note: Drift itself is not an error - it's information.

## Philosophy in Practice

**Scenario:** You update a config file.

**Without entropy-check:**
- Change made
- Forgotten why
- Breaks something later
- No record of when it changed

**With entropy-check:**
- Change detected immediately
- Review via `entropy-check`
- Record intent via Intent Ledger
- Update baseline
- Clear history of what/when/why

**This is intentional stewardship.**

---

**The forest watches itself. 🌲**
