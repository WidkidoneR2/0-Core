# Changelog - Dot-Doctor

## [3.2.0] - 2026-02-08

### 🐛 Critical Bug Fixes

#### Intent Ledger (Previously Broken!)
- **Fixed directory path**: Was looking for `INTENT/` but actual directory is `intents/` (case sensitivity bug)
- **Added missing categories**: Now scans `cancelled` and `deferred` in addition to decisions, experiments, philosophy, future, incidents
- **Result**: Now correctly shows "42 intents" instead of broken "0 intents"

#### Broken Symlinks (Previously Inaccurate!)
- **Made comprehensive**: Now scans ALL of ~/.config and stow directory
- **Increased depth**: 4 → 6 levels to catch deeply nested symlinks
- **Result**: Now finds ALL 7 broken symlinks instead of showing "0 found"
- **Scope**: Previously only checked 5 specific directories (sway, foot, fuzzel, yazi, zsh)

#### Improved
- **All Clippy Errors Fixed** (12 errors → 0):
  - Fixed empty line after doc comment
  - Removed redundant imports
  - Inlined color codes in format strings
  - Simplified identical if blocks
  - Changed useless format! to .to_string()
  - Fixed needless borrows

#### Documentation
- **Comprehensive README** (44 → 202 lines):
  - Standalone installation instructions
  - All 19 health checks explained
  - Examples: daily use, CI/CD, cron jobs, pre-commit hooks
  - Troubleshooting guide
  - Design philosophy
- **Added CHANGELOG.md** documenting all improvements

#### Production Quality
- ✅ Zero clippy warnings
- ✅ Accurate health checks (verified against reality)
- ✅ Complete documentation
- ✅ Standalone ready

---

## [3.1.0] - Earlier

Initial production release with 19 health checks.

See git history for full changelog.
