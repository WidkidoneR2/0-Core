# Changelog

All notable changes to Faelight Forest / 0-Core.

---

## [3.6.0] - 2026-01-05

### Intent Ledger Integration

- **dot-doctor Check 11**: Intent Ledger health monitoring
  - Count intents by status (complete, planned, in-progress, decided, abandoned)
  - Validate `packages:` references exist
  - Detect version/tag status mismatch
  - Flag stale planned intents (> 30 days)
- **New frontmatter fields**: `packages:` and `version:` for intents
- **Fixed**: INTENT and archive directories no longer flagged as non-semantic
- **Fixed**: Health percentage now shows whole number

---

## [3.5.2] - 2026-01-04

### Shell Safety & Polish

- **Dangerous command highlighting**: `rm -rf`, `chmod 777`, `dd`, `mkfs` show RED (includes sudo variants)
- **Dangerous command warning**: `preexec` hook shows warning before destructive commands
- **Tab completions**: Completions for `core-diff`, `dotctl`, `intent`
- **Starship lock status**: 🔒/🔓 inline in prompt (replaces verbose `core_guard`)
- **Removed**: `core_guard` function (replaced by Starship module)

---

## [3.5.1] - 2026-01-01

### Git Guardrails

- Block commits when core is locked (immutability protection)
- Warn on push to main (requires typed confirmation: 'push-main')
- `git!` escape hatch for experts
- Only active in ~/0-core directory

---

## [3.5.0] - 2025-12-31

### Intent Ledger Foundation

- `INTENT/` directory structure (decisions, experiments, philosophy, future)
- `intent` script (add, list, show, search)
- YAML frontmatter with status tracking
- 7 initial intents captured

_"Code shows HOW. Intent shows WHY."_

---

## [3.4.4] - 2025-12-27

### Shell Improvements

- Removed `alias cat='bat'` (breaks pipes/scripts)
- Fixed `pacman -Sy` → `checkupdates` (prevents partial upgrades)
- Directory history (Fish-like): `cd ~1`, `cd ~2`, `dirs -v`
- Core guard protection on prompt
- Created `docs/ALIASES.md` (188+ aliases documented)

---

## [3.4.3] - 2025-12-26

### Manual Installation

- Archived `install.sh` (automated installation)
- Created `docs/MANUAL_INSTALLATION.md`
- Philosophy: Understanding over automation

---

## [3.4.2] - 2025-12-26

### Topgrade Fix

- Fixed invalid config format
- AMD Framework optimized
- Manual control preserved

---

## [3.4.1] - 2025-12-26

### core-diff Polish

- Better error messages with suggestions
- Exit codes for scripting (0, 1, 2)
- UX polish throughout

---

## [3.4.0] - 2025-12-23

### core-diff + Philosophy

- **core-diff**: Package-aware diff tool with risk-based grouping
- **PHILOSOPHY.md**: Complete 0-Core manifesto
- Delta and Meld integration
- Aliases: `cdiff`, `cds`, `cdh`, `cdm`, `cdd`

---

## [3.3.5] - 2025-12-22

### Cleanup

- Added missing `.dotmeta` files
- System cleanup (removed obsolete configs)
- 100% health achieved

---

## [3.3.4] - 2025-12-22

### Quick Wins

- Lock reminders on version bump and shell startup
- Config aging report (Check 9)
- Intentional defaults checker (Check 10)

---

## [3.3.3] - 2025-12-21

### GitHub Polish

- Archived Fish to `archive/shell-fish` branch
- Professional screenshots added
- Zsh history expansion disabled (fixes git commits)

---

## [3.3.1] - 2025-12-17

### Blast Radius Warnings

- `edit-core` shows risk warnings based on `.dotmeta`
- Color-coded: 🔴 Critical, 🟠 High, 🔵 Medium, 🟢 Low
- Auto-backup for critical/high packages

---

## [3.2.0] - 2025-12-16

### Smart Update System

- `safe-update` script with auto-recovery
- Pre/post Btrfs snapshots
- `.pacnew` detection
- **NO systemd timers** — manual control only

---

## [3.1.0] - 2025-12-14

### Hybrid Architecture (The Great Transformation)

- **dotfiles → 0-core**: Numbered priority system
- **Semantic naming**: `hypr` → `wm-hypr`, `fish` → `shell-fish`
- **Yazi teleports**: 14 instant navigation bindings
- **Immutable protection**: `chattr +i` via `core-protect`
- 18+ hours of focused work
- 12-hour password debugging → Manual control philosophy born

---

## Pre-3.0 (Dotfiles Era)

See `archive/CHANGELOG-v2.8.md` for v2.8.x history.

---

_The forest remembers. 🌲_
