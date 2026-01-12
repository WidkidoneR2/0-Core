---
id: 015
date: 2026-01-07
type: future
title: "entropy-check - Configuration Drift Detection"
status: in-progress
tags: [rust, safety, drift, v6.8]
---

## The Vision
Detect configuration drift before it becomes catastrophic. Observe changes, report anomalies, preserve stewardship.

## Why
The Hyprland lesson: systems change silently. 2.3M errors from one update.

entropy-check catches drift early:
- Files modified outside stow
- Services that changed state
- Binaries updated unexpectedly
- New untracked files appearing
- Symlink integrity over time

## Philosophy
> "Systems drift. Humans change. Contexts evolve."
> entropy-check watches for the silent changes.

**Surfaces drift. Doesn't fix it.** You decide how to respond.

---

## v0.1 - Drift Detection (v6.8.0)

### Core Checks
- [x] **Config Drift**: Files modified outside stow (checksum mismatches)
- [x] **Service Drift**: Services changed state unexpectedly
- [x] **Binary Drift**: Package versions changed since last check
- [x] **Symlink Drift**: New broken symlinks since baseline
- [ ] **Untracked Files**: New files in managed directories

### Output Format
```bash
entropy-check

📊 Drift Report - 0-Core v6.7.2
Last check: 2 days ago
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️  Config Drift (2 files)
   ~/.config/sway/config (modified 4 hours ago)
   ~/.zshrc (modified yesterday)
   
⚠️  Service Drift (1 service)
   faelight-notify.service (stopped, expected running)

✅ Binary Drift (0 changes)
✅ Symlink Drift (0 new breaks)
✅ Untracked Files (0 new)

💡 Recommendation:
   Review modified configs, consider restowing
   Restart faelight-notify service
```

### Technical Design
- Store baseline state: `~/.config/faelight/entropy-baseline.json`
- Compare current vs baseline on each run
- Track: checksums, service states, package versions, symlinks
- Update baseline: `entropy-check --update-baseline`

### Commands
```bash
entropy-check              # Check for drift
entropy-check --baseline   # Create/update baseline
entropy-check --reset      # Reset all tracking
entropy-check --json       # Machine-readable output
```

---

## v0.2 - Usage Entropy (v6.9.0)

### Usage Tracking (Future)
- [ ] Alias execution frequency (shell integration)
- [ ] Config file access times (atime tracking)
- [ ] Profile switch history
- [ ] Script execution counts
- [ ] Suggest pruning candidates

**Deferred to v6.9.0** - drift detection is critical, usage tracking is quality-of-life.

---

## Success Criteria (v0.1)
- [x] Detects config modifications outside stow
- [x] Reports service state changes
- [x] Tracks binary version drift
- [x] Identifies new broken symlinks
- [x] Baseline creation and updates
- [ ] Integration with dot-doctor workflow

---

_The forest watches for silent changes._ 🌲
