# Changelog - bin-doctor

## [2.0.0] - 2026-02-11

### 🎉 PRODUCTION READY - BULLETPROOF

**Security Improvements:**
- ✅ Fixed all critical .unwrap() calls
- ✅ Proper error handling for HOME variable
- ✅ Safe TOML parsing with fallbacks
- ✅ File I/O error checking
- ✅ Git availability verification

**New Features:**
- `--json` flag for JSON output
- `--quiet` flag for minimal output
- Git availability check on register
- Comprehensive error hints

**Commands:**
- `register <name>` - Register binary after cargo install
- `check` - Check all binaries for drift
- `list` - List all tracked binaries
- `drift` - Show only drifted binaries

**Error Handling:**
- HOME not set → Clear error message
- Git not found → Installation hint
- Manifest parse error → Use empty manifest
- File write error → Permission hint
- All errors have helpful hints

**Code Quality:**
- Zero clippy warnings
- Proper error handling
- 310+ lines of clean code
- Helpful error messages

**Dependencies:**
- Git (checked at runtime)

---

## [1.0.0] - Earlier

Binary manifest tracking system.

---

**Version Format:** MAJOR.MINOR.PATCH
