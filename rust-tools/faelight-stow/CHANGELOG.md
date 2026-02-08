# Changelog - Faelight Stow

## [2.2.0] - 2026-02-08

### 🎯 Collision Detection Feature

#### Added
- **🔍 Collision Preflight** - New `check` subcommand!
  - Detects pre-existing non-symlink files
  - Finds duplicate targets between packages
  - Safe dry-run before stowing
  - Example: `faelight-stow check shell-zsh wm-sway`

#### Improved
- Prevents accidental file overwrites
- Shows exactly what conflicts exist
- Recommends backup strategy before stowing

---

## [2.1.0] - 2026-02-08

Production-ready standalone release with comprehensive docs.

## [2.0.0] - Earlier

Initial release with auto-discovery and verification.

See git history for full changelog.
