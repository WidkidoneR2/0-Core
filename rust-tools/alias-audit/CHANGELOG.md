# Changelog - Alias-Audit

## [9.1.0] - 2026-02-08

### ✨ Visual Improvements

**Beautiful Formatting:**
- Added box border headers (╭─────╮ style)
- Professional UI matching faelight-fetch/dotctl
- Summary box with statistics
- Better visual hierarchy

**Zone Integration:**
- Added zone detection for current directory
- Shows zone icon and name at top
- Zone-aware context display
- Integrated with faelight-zone v2.0.0

### 🔧 Technical Improvements

**Code Quality:**
- Fixed clippy error (unwrap_or_default)
- Changed `.or_insert_with(Vec::new)` → `.or_default()`
- Added faelight-zone dependency
- Proper imports (use faelight_zone)

**Better Output:**
- Box-bordered header and summary
- Zone context at top
- Cleaner command structure
- Professional appearance

### 📚 Documentation

**Comprehensive README (70 → 157 lines):**
- Complete feature documentation
- Zone integration section
- All commands documented
- Expected tools list
- Integration examples
- Use cases (CI/CD, pre-commit hooks)

### Production Quality

- ✅ Zero clippy warnings
- ✅ Comprehensive README (157 lines)
- ✅ CHANGELOG.md added
- ✅ Zone integration working
- ✅ Beautiful formatting
- ✅ Standalone ready

---

## [9.0.0] - Earlier

Previous version with basic alias checking.

See git history for full changelog.
