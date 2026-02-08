# Changelog - Faelight Stow

## [2.1.0] - 2026-02-08

### 🎊 Production-Ready Standalone Release

#### Added
- **Comprehensive README** (189 lines) with standalone installation
- **Installation Guide** for any system with GNU Stow
- **Examples Section** covering daily use, automation, CI/CD
- **Use Cases** demonstrating personal dotfiles, multi-machine sync
- **CHANGELOG.md** documenting improvements

#### Fixed
- All clippy warnings (2 errors fixed)
  - Removed needless borrow in path handling
  - Changed `&PathBuf` to `&Path` for better API
- Version consistency across Cargo.toml, code, and README

#### Improved
- **Error messages** with helpful recovery steps:
  - Stow failures: Show manual stow command
  - Restow failures: Suggest checking for conflicts
- README highlights universal usefulness
- Documentation for automation and scripting

---

## [2.0.0] - Earlier

Initial production release:
- Auto-discovery of stow packages
- Symlink verification
- Auto-repair with --fix
- Silent mode for scripting

See git history for full changelog.
