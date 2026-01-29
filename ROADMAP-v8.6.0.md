# 🚀 Roadmap to v8.6.0 - Production Ready
**Target: Post-Friday (Feb 2026)**

> Making Faelight Forest so polished and easy that a 5-year-old could use it.

---

## 🎯 **CORE MISSION:**

Three tools achieving v1.0.0 production status:
1. **faelight-link** - Zone-aware symlink manager
2. **faelight-fm** - File manager with daemon integration  
3. **bump-system-version** - Effortless release automation

Plus: New **bump-tool-version** for individual tool versioning!

---

## 📋 **HIGH PRIORITY IMPROVEMENTS**

### 🔧 **1. bump-system-version → v6.0.0** ⭐⭐⭐⭐⭐
**Goal:** "So easy a 5-year-old could do it"

**Current State:** v5.1.0 - Manual version entry only

**New Features:**
- ✅ `--minor` flag: Auto-increment minor (8.5.0 → 8.6.0)
- ✅ `--patch` flag: Auto-increment patch (8.5.0 → 8.5.1)
- ✅ `--major` flag: Auto-increment major (8.5.0 → 9.0.0)
- ✅ Pre-release health audit (unused files, dependency mismatches)
- ✅ Conventional commit changelog generation
- ✅ Better error messages (friendly, not scary)
- ✅ `--test` mode (preview changes without executing)

**Usage:**
```bash
# Current
bump-system-version 8.6.0

# New (v6.0.0)
bump-system-version --minor   # Simple!
bump-system-version --patch   # Simple!
bump-system-version --major   # Simple!
bump-system-version --test 8.6.0  # Preview first
```

**Time Estimate:** 4-6 hours
**Priority:** CRITICAL - Makes releases painless

---

### 🔗 **2. faelight-link → v1.0.0** ⭐⭐⭐⭐⭐
**Goal:** Complete stow replacement, production ready

**Current State:** v0.2.0 - Conflict detection working

**Missing for v1.0.0:**
- ✅ `unstow` command (remove links cleanly)
- ✅ `audit` command (find broken/orphaned links)
- ✅ `clean` command (remove orphans)
- ✅ Zone awareness (prevent linking to locked zones)
- ✅ Intent tracking (why was this linked?)
- ✅ Post-change validation suggestions
- ✅ Better documentation with examples

**Usage:**
```bash
# Stow/unstow
faelight-link stow editor-nvim
faelight-link unstow editor-nvim

# Maintenance
faelight-link audit        # Check health
faelight-link clean        # Remove orphans

# Status
faelight-link status       # Overview
```

**Time Estimate:** 4-6 hours
**Priority:** CRITICAL - Daily use tool

---

### 📁 **3. faelight-fm → v1.0.0** ⭐⭐⭐⭐⭐
**Goal:** Production-ready file manager with full operations

**Current State:** v0.2.0-beta - Edit works perfectly!

**Missing for v1.0.0:**
- ✅ File operations (copy, move, rename, delete with confirmation)
- ✅ Bulk operations (mark multiple files)
- ✅ Better search/filter (regex support)
- ✅ Improved keybindings documentation
- ✅ Error handling polish
- ✅ Performance optimization for large directories

**Current Features (Already Great!):**
- ✅ Zone-aware navigation
- ✅ Edit in nvim (perfect redraw!)
- ✅ Git status markers
- ✅ Preview overlay
- ✅ Info overlay
- ✅ Help overlay
- ✅ Daemon integration

**Time Estimate:** 4-6 hours
**Priority:** CRITICAL - Daily driver

---

### 🆕 **4. bump-tool-version (NEW!)** ⭐⭐⭐⭐
**Goal:** Easy individual tool version management

**The Problem:**
- 38 tools, each with its own version
- Hard to track which tools were updated
- Manual version bumping in Cargo.toml
- No automation for tool releases

**The Solution:**
```bash
# Bump individual tool versions
bump-tool-version faelight-link --minor   # 0.2.0 → 0.3.0
bump-tool-version faelight-fm --patch     # 0.2.0 → 0.2.1
bump-tool-version dot-doctor --major      # 8.5.0 → 9.0.0

# Update multiple tools
bump-tool-version --list                  # Show all tool versions
bump-tool-version --check                 # Find tools needing updates

# Auto-detection
cd rust-tools/faelight-link
bump-tool-version --minor                 # Detects current tool
```

**Features:**
- Auto-updates Cargo.toml version
- Updates version strings in source
- Creates git tag: `tool-name-vX.Y.Z`
- Updates README if exists
- Tracks in system changelog
- Integration with bump-system-version

**Time Estimate:** 3-4 hours
**Priority:** HIGH - Quality of life improvement

---

## 🎯 **MEDIUM PRIORITY (If Time Permits)**

### 📊 **faelight-fetch: Profile Display** ⭐⭐⭐
**Already implemented!** Just verify it works.

**Time:** 15 minutes (verification only)

---

### 🎨 **faelight (wrapper): Shell Completion** ⭐⭐⭐
**Goal:** Tab completion for faelight commands

**Status:** Attempted, needs clean implementation

**Features:**
- Zsh completion support
- Nushell completion support
- Generated via clap

**Time Estimate:** 1 hour (with fresh approach)
**Priority:** MEDIUM - Nice to have

---

### 📝 **core-diff: Post-Change Validation** ⭐⭐⭐
**Goal:** Suggest actions after detecting changes

**Features:**
```bash
After running core-diff:
⚠️  Kernel updated - reboot recommended
⚠️  Service files changed - run systemctl daemon-reload
💡 Sway config changed - reload with Mod+Shift+C
```

**Time Estimate:** 1-2 hours
**Priority:** MEDIUM - Helpful but not critical

---

### 📚 **Enhanced Documentation** ⭐⭐⭐
**Goal:** Comprehensive guides for all tools

**What to Document:**
- Each tool's README with examples
- Use cases and workflows
- Best practices
- Troubleshooting guides

**Time Estimate:** 2-3 hours total
**Priority:** MEDIUM - Important for long-term

---

## 📅 **IMPLEMENTATION TIMELINE**

### **Week 1: Core Features (Feb 1-7)**
```
Day 1-2: bump-system-version v6.0.0
  - --minor/--patch/--major flags
  - Health audit integration
  - Better UX
  
Day 3-4: faelight-link v1.0.0
  - unstow command
  - audit/clean commands
  - Zone awareness
  
Day 5-6: faelight-fm v1.0.0
  - File operations
  - Bulk operations
  - Polish UX

Day 7: bump-tool-version v1.0.0
  - Build from scratch
  - Test with all tools
```

### **Week 2: Polish & Release (Feb 8-14)**
```
Day 8-9: Testing & Bug Fixes
  - Test all new features
  - Fix any issues
  - Performance checks

Day 10-11: Documentation
  - Update all READMEs
  - Write usage guides
  - Add examples

Day 12: Shell Completion (if time)
  - faelight completion
  - Test in zsh/nu

Day 13: Final Testing
  - Run full system health
  - Test all workflows
  - Verify 100% health

Day 14: RELEASE v8.6.0! 🎉
```

---

## ✅ **SUCCESS CRITERIA FOR v8.6.0:**

**Must Have:**
- ✅ bump-system-version v6.0.0 (--minor/--patch/--major)
- ✅ faelight-link v1.0.0 (production ready)
- ✅ faelight-fm v1.0.0 (production ready)
- ✅ bump-tool-version v1.0.0 (tool versioning)
- ✅ All tools tested and working
- ✅ 100% system health
- ✅ All commits pushed
- ✅ Documentation updated

**Nice to Have:**
- ✅ Shell completion for faelight
- ✅ core-diff post-change validation
- ✅ Enhanced documentation for all tools

**Celebration Criteria:**
- ✅ 4 new v1.0.0 tools
- ✅ System feels polished and professional
- ✅ "5-year-old could use it" achieved
- ✅ No beta tools in daily workflow

---

## 🎊 **WHAT v8.6.0 REPRESENTS:**

**Not Just Version Numbers - A Philosophy Realized:**

1. **Simplicity:** Tools that "just work"
2. **Quality:** Production-ready, not prototypes
3. **Polish:** Attention to every detail
4. **Joy:** Building is fun, using is effortless

**From Vision to Reality:**
- Intent 077: Ecosystem Philosophy ✅
- Working implementations ✅
- Daily driver quality ✅
- Linus-ready presentation ✅

---

## 💭 **NOTES:**

**Why These Features Matter:**

**bump-system-version improvements:**
- Current: "What version number should I use?" (decision fatigue)
- Future: "I made a small fix" → `bump --patch` (effortless)

**faelight-link v1.0.0:**
- Current: "Hope stow doesn't break" (anxiety)
- Future: "Link knows zones, tracks intent" (confidence)

**faelight-fm v1.0.0:**
- Current: "Beta tool, might be rough" (hesitation)
- Future: "Production quality, rock solid" (trust)

**bump-tool-version:**
- Current: Manually edit 38 Cargo.toml files (tedious)
- Future: One command per tool (automated)

---

## 🌲 **THE FOREST GROWS STRONGER**

Every improvement makes the ecosystem more coherent.
Every v1.0.0 represents mastery achieved.
Every release brings the vision closer to reality.

**This is what v8.6.0 means: The forest is production-ready.** 🌲

---

**Created:** January 28, 2026
**Target Release:** February 14, 2026 (2-week sprint)
**Current Version:** v8.5.0
**Current Status:** Ready to build! 🚀
