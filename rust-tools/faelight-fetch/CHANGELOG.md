# Changelog - Faelight-Fetch

## [2.1.0] - 2026-02-11

### 🎉 Production Ready

**Enhanced Error Handling:**
- 12 error messages with helpful hints
- Health check validates all dependencies
- Graceful fallbacks for missing tools
- User-friendly error output with 💡 solutions

**CLI Improvements:**
- Added --health-check flag
- Added --help and --version
- Proper clap-based argument parsing

**Documentation:**
- 170-line comprehensive README
- Installation instructions
- Examples and usage
- CHANGELOG.md maintained

**Code Quality:**
- Zero clippy warnings
- Safe error handling (unwrap_or_else with fallbacks)
- Health checks for: directory access, HOME, dot-doctor, profile, faelight-zone

### ✨ Major Visual Improvements (from earlier v2.1.0)

**Better Formatting:**
- Box border header (╭─────╮ style)
- Right-aligned labels with consistent spacing
- Clean, professional output

**Zone Integration:**
- Zone detection with icons (🦀 WORK, 🌲 CORE)
- Integrated with faelight-zone v2.0.0

### Technical
- Lines of code: 277
- Dependencies: faelight-zone, sysinfo, clap
- Error messages: 12 helpful hints

---

## [2.0.0] - Earlier

Initial production version with zone awareness.

---

**Version Format:** MAJOR.MINOR.PATCH
