# Changelog - archaeology-0-core

## [3.0.0] - 2026-02-11

### 🎉 PRODUCTION READY - BULLETPROOF

**Security Improvements:**
- ✅ Replaced all .unwrap() with proper error handling
- ✅ Git availability check on startup
- ✅ Path encoding validation
- ✅ Helpful error hints for all failures

**New Features:**
- `--quiet` flag for minimal output
- Git dependency checking
- Better error messages

**Commands:**
- `timeline` - Chronological timeline
- `recent <days>` - Recent activity
- `intent <id>` - Intent-specific commits
- `since <tag>` - Changes since version
- `package <name>` - Package history
- `stats` - Evolution statistics

**Flags:**
- `--json` - JSON output
- `--quiet` - Minimal output

**Error Handling:**
- Git not found → Installation hint
- Invalid paths → Encoding error
- Failed git commands → Clear messages
- All errors have helpful hints

**Code Quality:**
- Zero clippy warnings
- Zero .unwrap() calls
- 584+ lines of clean code
- Comprehensive error handling

**Dependencies:**
- Git (checked at runtime)
- All errors provide installation hints

---

## [2.0.0] - Earlier

System history explorer for 0-Core.

---

**Version Format:** MAJOR.MINOR.PATCH
