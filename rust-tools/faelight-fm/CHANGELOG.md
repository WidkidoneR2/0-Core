# Changelog - faelight-fm

All notable changes to this project will be documented in this file.

## [2.2.0] - 2026-02-11

### 🎉 PHASE 1 COMPLETE - Multi-Select & Bulk Operations

**New Features - Surpassing Yazi:**

1. **Multi-Select System**
   - Press `Space` to toggle file selection
   - Visual checkboxes: `[✓]` for selected, `[ ]` for unselected
   - Automatic advancement to next file for rapid selection
   - Status bar shows selection count

2. **Bulk Operations**
   - `Shift+Y` - Bulk yank (copy) selected files
   - `Shift+C` - Clear all selections
   - `Shift+D` - Bulk delete (confirmation pending)
   - Operations work on all selected files at once

3. **Visual Feedback**
   - Green checkboxes for selected files
   - Status messages with file counts
   - Integrated with existing zone/git/intent display

**Technical:**
- Added `selected_files: HashSet<PathBuf>` to AppState
- Added `multi_select_mode` flag
- Enhanced filelist rendering with checkbox spans
- Zero performance impact on existing features

**Preserved:**
- ✅ Zone jumping (0-5)
- ✅ Git status markers
- ✅ Intent integration
- ✅ Health badges
- ✅ All existing keybindings
- ✅ Beautiful UI layout

**Code Quality:**
- Clean build, zero warnings
- 2,090+ lines of careful enhancements
- Backward compatible

### Coming in Phase 2
- Enhanced preview with syntax highlighting
- Fuzzy search with content search
- Full bulk delete with confirmation

---

## [2.1.0-alpha] - Earlier

Production file manager with zones, git, and intent integration.

---

**Version Format:** MAJOR.MINOR.PATCH
