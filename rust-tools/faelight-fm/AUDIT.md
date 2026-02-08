# 🔍 Faelight-FM Comprehensive Audit
**Date:** 2026-02-08
**Version:** v2.1.0-alpha
**Lines:** 2,081
**Goal:** Make better than yazi in EVERY way

## A) CURRENT STATE AUDIT - VERIFIED BY USE

### ✅ Working Features (Tested)
- ✅ Navigate up/down with keyboard
- ✅ Enter directories
- ✅ Mouse clickable navigation
- ✅ Opens files in editor (neovim/lazyvim/astrovim/chadnv/helix)
- ✅ **Zone awareness** (unique to faelight-fm!)
- ✅ Shows current path
- ✅ Shows current profile
- ✅ Shows health status
- ✅ Shows version number
- ✅ ESC to quit

### ❌ Missing Features
- ❌ **No preview pane** (critical!)
- ❌ No search (?)
- ❌ No bulk operations
- ❌ No bookmarks
- ❌ No tabs
- ❌ No image preview
- ❌ No archive preview
- ❌ No syntax highlighting in preview

### 🎯 Unique Features (vs Yazi)
- **Zone System** - Classifies directories by purpose
- **Profile Integration** - Shows current profile
- **Health Status** - System health indicator
- **Intent System** - Intent-aware navigation
- **0-Core Integration** - Native to your ecosystem

## B) YAZI FEATURE COMPARISON

### What Yazi Has (Must Match/Beat)
```
Core Navigation:
  [✅] File listing
  [✅] Directory navigation
  [✅] Keyboard shortcuts
  [✅] Mouse support
  [❌] Preview pane ⚠️ CRITICAL GAP
  [❌] Dual-pane mode
  
Advanced Features:
  [❌] Image preview (ueberzug)
  [❌] Archive preview (.tar.gz, .zip)
  [❌] Syntax highlighted preview
  [❌] PDF preview
  [❌] Video thumbnails
  [❌] Bulk rename
  [❌] Bulk operations (copy/move/delete)
  [❌] Bookmarks (jump to paths)
  [❌] Tabs (multiple directories)
  [❌] Search/filter
  [❌] Trash support
  [❌] Shell integration
  
Yazi Plugins:
  [❌] Git status (you have git module!)
  [❌] Archive extraction
  [❌] Custom openers
```

## C) ROADMAP TO YAZI KILLER

### Phase 1: Critical Gaps (Beta-blocking)
**Priority: 🔴 CRITICAL**
1. **Preview Pane** - The biggest gap!
   - Text file preview
   - Syntax highlighting
   - Binary file detection
   - Large file handling (first/last 500 lines)
   
2. **Search/Filter** - Essential for file managers
   - Fuzzy search
   - Regex support
   - Filter by type
   
3. **Bulk Operations** - Core functionality
   - Select multiple files
   - Bulk delete
   - Bulk move/copy

### Phase 2: Feature Parity (v1.0.0)
**Priority: 🟡 HIGH**
4. **Image Preview** - Nice to have
5. **Archive Preview** - List contents
6. **Bookmarks** - Quick navigation
7. **Tabs** - Multiple directories
8. **Dual-pane** - Side-by-side browsing

### Phase 3: Beyond Yazi (v2.0.0)
**Priority: 🟢 ENHANCEMENT**
9. **Zone-Aware Actions** - Different actions per zone
10. **Intent Integration** - Create intents from FM
11. **Health Checks** - Verify files as you browse
12. **Profile Switching** - Switch profiles from FM
13. **Git Operations** - Stage/commit from FM
14. **Smart Previews** - Render markdown, show images
15. **AI Integration** - File summaries, categorization

### What Makes Faelight-FM Better Than Yazi
```
YAZI:                   FAELIGHT-FM:
- Generic FM            - 0-Core integrated
- No context            - Zone-aware
- No health             - Health monitoring
- No profiles           - Profile system
- No intents            - Intent system
- Plugin system         - Native Rust tools
- Lua config            - TOML config
- Standalone            - Ecosystem member
```

## D) IMMEDIATE NEXT STEPS

### Step 1: Preview Pane (Blocks Beta)
**Goal:** Match yazi's preview pane
**Tasks:**
- [ ] Add preview pane to UI layout
- [ ] Text file preview (UTF-8)
- [ ] Syntax highlighting (using bat or syntect)
- [ ] Binary file detection
- [ ] Large file handling
- [ ] Image preview (later phase)

### Step 2: Search (Blocks Beta)  
**Goal:** Fast file filtering
**Tasks:**
- [ ] Search mode toggle (/)
- [ ] Fuzzy matching
- [ ] Filter file list
- [ ] Clear search (ESC)

### Step 3: Bulk Operations (Blocks Beta)
**Goal:** Select and operate on multiple files
**Tasks:**
- [ ] Visual selection mode
- [ ] Multi-select with space
- [ ] Bulk delete confirmation
- [ ] Bulk move/copy

## E) ALPHA → BETA → V1.0.0 PATH
```
CURRENT: v2.1.0-alpha (~40% complete)
  ✅ Basic navigation works
  ✅ Zone system unique
  ❌ No preview pane
  ❌ No search
  ❌ No bulk ops

TARGET: v2.2.0-beta (60% complete)
  ✅ Preview pane with syntax highlighting
  ✅ Search/filter
  ✅ Bulk operations
  ✅ All core navigation
  
TARGET: v3.0.0 (100% feature parity)
  ✅ Everything yazi has
  ✅ Image/archive preview
  ✅ Bookmarks + tabs
  ✅ Dual-pane mode

TARGET: v4.0.0 (Beyond Yazi)
  ✅ Zone-aware actions
  ✅ Intent integration
  ✅ Profile switching
  ✅ Smart previews
  ✅ Git operations from FM
```

## F) ESTIMATED EFFORT

**Preview Pane:** 2-3 sessions (400-600 lines)
**Search:** 1 session (150-200 lines)
**Bulk Ops:** 1-2 sessions (200-300 lines)
**Image Preview:** 1 session (100-150 lines)
**Full Parity:** 8-10 sessions
**Beyond Yazi:** 5-8 sessions

**Total to v4.0.0:** ~15-20 dedicated sessions

---

## Conclusion

Faelight-FM is **solid alpha** with unique features yazi doesn't have. The path to replacing yazi is clear:

1. **Phase 1** (Beta): Preview + Search + Bulk Ops
2. **Phase 2** (v3.0.0): Full feature parity
3. **Phase 3** (v4.0.0): Unique 0-Core integration

**It's not just about matching yazi - it's about being the PERFECT file manager for 0-Core.**
