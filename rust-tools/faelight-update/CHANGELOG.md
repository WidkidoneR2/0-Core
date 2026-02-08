# Changelog - Faelight Update Manager

## [3.1.0] - 2026-02-08

### 🎊 Production-Ready Standalone Release

#### Added
- **Comprehensive README** with standalone installation instructions
- **Examples section** covering basic usage, scripting, and selective updates  
- **Helpful error messages** with actionable solutions (💡)
- **Version flag** (`--version`) support
- **Dependencies documentation** for standalone users

#### Fixed
- All 12 clippy warnings - zero-warning production build
- `--count-only` help text clarity
- Error messages now include recovery steps:
  - Cargo: Shows install commands for cargo-update
  - Snapshot: Suggests installation or skip flag
  - Health check: Points to 'doctor' command
  - npm/pip: Provides Arch-specific guidance

#### Improved
- Error handling with context and solutions
- Code quality (clippy clean)
- Documentation for standalone use
- User experience with helpful error messages

---

## [3.0.0] - 2026-02-07

### Major Release - Skip pip/yazi on Arch

#### Added
- `--count-only` flag for status bar integration
- PEP 668 compliance (skip pip on Arch Linux)
- Config file support for auto-yes categories
- Smart .pacnew handling with pacdiff integration

#### Changed
- Skip yazi updates (no stable command available)
- Improved health check integration
- Better workspace update detection

#### Fixed
- Double-run bug in pacman updates
- AUR rebuild warnings
- Prompt cache auto-updates

---

## Earlier Versions

See git history for full changelog of v1.x and v2.x releases.
