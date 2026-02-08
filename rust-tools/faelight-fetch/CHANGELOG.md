# Changelog - Faelight-Fetch

## [2.1.0] - 2026-02-08

### ✨ Major Visual Improvements

**Better Formatting:**
- Added box border header (╭─────╮ style)
- Right-aligned labels with consistent spacing
- Better visual hierarchy with grouping
- Cleaner, more professional output

**Zone Integration:**
- Added zone detection with icons
- Shows current directory zone (🦀 WORK, 🌲 CORE, etc.)
- Integrated with faelight-zone v2.0.0
- Zone-aware context at a glance

### 🔧 Technical Improvements

**Code Quality:**
- Fixed clippy error (double_ended_iterator_last)
- Changed `.last()` to `.next_back()` for efficiency
- Added faelight-zone dependency
- Improved SystemState struct

**New Fields:**
- `zone: String` - Current directory zone
- `zone_icon: String` - Zone visual indicator

### 📚 Documentation

**Comprehensive README (170 lines):**
- Complete feature documentation
- Zone detection table
- Health icon meanings
- Installation instructions
- Use cases and examples
- Philosophy section

### Production Quality

- ✅ Zero clippy warnings
- ✅ Comprehensive README
- ✅ CHANGELOG.md added
- ✅ Zone integration
- ✅ Standalone ready

---

## [2.0.0] - Earlier

Initial release with basic system information display.

See git history for full changelog.
