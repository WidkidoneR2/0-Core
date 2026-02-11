# Changelog - faelight-lock

All notable changes to this project will be documented in this file.

## [2.1.0] - 2026-02-11

### 🎉 Quick Wins - Enhanced Usability

**New Features:**
- ✅ Custom lock messages: `--message "Back at 3pm"`
- ✅ Grace period: `--grace 30` (30 second unlock window)
- ✅ Urgent lock: `--urgent` (skip grace period)
- ✅ Better error messages with installation hints

**Documentation:**
- 217+ line comprehensive README
- Installation instructions
- Sway integration examples
- Troubleshooting guide
- Comparison with other lockers
- CHANGELOG.md added

**Code Quality:**
- Zero clippy warnings
- Enhanced error handling
- Thread-safe grace period implementation

### Technical
- Lines of code: 146 (was 102)
- Dependencies: clap, faelight-core
- Features: 3 new command-line options

---

## [2.0.0] - 2026-02-10

Production-ready themed screen locker.

**Features:**
- Automatic Faelight Forest theme integration
- swaylock wrapper with optimized settings
- Health check support

---

**Version Format:** MAJOR.MINOR.PATCH
- MAJOR: Breaking changes
- MINOR: New features, non-breaking
- PATCH: Bug fixes only
