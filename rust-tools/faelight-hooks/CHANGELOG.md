# Changelog - Faelight Git Hooks

## [10.1.0] - 2026-02-08

### 🎊 Production-Ready Standalone Release

#### Added
- **Comprehensive README** with standalone installation instructions
- **Examples section** covering all hook behaviors
- **Configuration documentation** for conventional commits
- **Hook details** explaining what each hook checks

#### Fixed
- Clippy warnings: Changed `&PathBuf` to `&Path` (3 instances)
- Import cleanup: Removed unused `PathBuf` import

#### Improved
- Documentation for standalone use
- Clear examples for commit messages
- Secret detection examples
- Pre-push protection documentation

---

## [10.0.0] - Earlier

Production release with all core hooks:
- Pre-commit: Secret detection, conflict detection
- Commit-msg: Conventional commit validation
- Pre-push: Main branch protection

See git history for full changelog.
