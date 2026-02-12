# Changelog - safe-update

All notable changes to safe-update will be documented in this file.

## [2.1.0] - 2026-02-12

### 🎯 Production-Ready Upgrade

**Major Improvements:**
- ✅ **Proper clap integration** - Now uses clap::Parser instead of manual arg parsing
- ✅ **colored crate integration** - Replaced raw ANSI codes with colored crate
- ✅ **Better error handling** - Replaced all `.unwrap()` calls with proper error messages
- ✅ **Zero unsafe operations** - All I/O operations properly handle errors
- ✅ **paru support** - Changed from yay to paru for AUR helper

### Changed
- **Argument parsing** - Switched from manual `env::args()` to proper clap::Parser
- **Color output** - Now using `colored` crate for cleaner code
- **Error messages** - All failures now show helpful context instead of panicking
- **AUR helper** - Changed from yay to paru (rebuild_paru instead of rebuild_yay)
- **Log saving** - Shows warning if log can't be saved instead of silent failure
- **Version handling** - clap now handles `--version` and `--help` automatically

### Fixed
- **Header comment** - Updated to v2.1.0
- **String allocations** - Removed unnecessary `.to_string()` calls in arg comparisons
- **Silent failures** - Directory creation and log saving now report errors
- **I/O operations** - stdout flush and stdin read now handle errors gracefully

### Technical
- Removed unused manual arg parsing code
- Added `impl From<Cli> for Config` for clean conversion
- Better separation of concerns with dedicated conversion logic
- Maintained 100% backward compatibility - all existing flags work identically

---

## [2.0.0] - 2025-01-21

### Initial Production Release

**Features:**
- ✅ Pre-flight health checks (snapper, internet, disk space, doctor)
- ✅ Btrfs snapshot creation (before/after updates)
- ✅ Automatic AUR helper recovery on library mismatch
- ✅ .pacnew file detection and warnings
- ✅ Post-update system health verification
- ✅ Rollback instructions with snapshot numbers
- ✅ Drift baseline updates via entropy-check
- ✅ Update logging to `~/.local/share/faelight/update-logs/`

**Command Options:**
- `--dry-run` - Preview updates without applying
- `--yes, -y` - Skip confirmation prompt
- `--skip-snapshot` - Fast updates without snapshots
- `--health` - Run pre-flight checks only
- `--version, -V` - Show version
- `--help, -h` - Show help

**Philosophy:**
- Manual control over automation
- Reliability over convenience
- Updates never automatic - user controls when
