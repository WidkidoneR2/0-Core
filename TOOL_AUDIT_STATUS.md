# Faelight Forest Tool Audit - Linus Presentation Prep
## Goal: Audit all 29 tools by end of week

### ✅ COMPLETED (6/29 = 21%)
1. faelight-launcher v3.3.0 - PNG icons, health check ✅
2. get-version v2.0.0 - Stow support, --help/--version/--health ✅
3. latest-update v2.0.0 - Stow support, --help/--version/--health ✅
4. faelight-lock v1.0.0 - Production ready, --version ✅
5. faelight-stow v0.3.0 - Auto-discover packages ✅
6. dot-doctor v0.5.0 - Auto-discover packages ✅

### 🔄 IN PROGRESS (0/29)
(None currently)

### 📋 REMAINING (23/29 = 79%)

#### TIER 1: Core System Tools (CRITICAL - Do First)
7. [ ] faelight-bar - Status bar (your flagship)
8. [ ] faelight-menu - App menu
9. [ ] faelight-notify - Notification system
10. [ ] faelight-bootstrap - System installer

#### TIER 2: Version/Release Tools (HIGH PRIORITY)
11. [ ] bump-system-version - Already works well, audit for polish
12. [ ] bump-version - Tool versioning
13. [ ] faelight-snapshot - Snapshot manager
14. [ ] faelight-git - Git wrapper

#### TIER 3: Development Tools (MEDIUM PRIORITY)
15. [ ] faelight - Main CLI
16. [ ] intent - Intent manager
17. [ ] intent-guard - Intent validator
18. [ ] archaeology-0-core - History tool

#### TIER 4: Config Management (MEDIUM PRIORITY)
19. [ ] dotctl - Dotfile controller
20. [ ] core-diff - Config differ
21. [ ] core-protect - Protection tool
22. [ ] profile - Profile manager

#### TIER 5: Utilities (LOWER PRIORITY - Quick Wins)
23. [ ] entropy-check - Entropy checker
24. [ ] faelight-dashboard - Dashboard
25. [ ] keyscan - Keybind scanner
26. [ ] safe-update - Safe updater
27. [ ] teach - Tutorial tool
28. [ ] workspace-view - Workspace viewer

#### TIER 6: Library (NOT A TOOL - Skip Audit)
29. faelight-core - Shared library (already used by other tools)

---

## Audit Checklist (Apply to Each Tool)

### Compilation & Warnings
- [ ] Compiles cleanly (0 warnings)
- [ ] No unused imports/variables
- [ ] No clippy warnings

### CLI Standards
- [ ] Has `--help` / `-h`
- [ ] Has `--version` / `-v`
- [ ] Has `--health-check` (if applicable)
- [ ] Clear error messages
- [ ] Proper exit codes

### Code Quality
- [ ] No hardcoded paths
- [ ] Auto-discovery where applicable
- [ ] Proper error handling
- [ ] Uses faelight-core Theme (if applicable)

### Documentation
- [ ] Has README.md
- [ ] Clear usage examples
- [ ] Version documented

### Testing
- [ ] Binary exists and works
- [ ] Copied to ~/0-core/scripts/
- [ ] Tested basic functionality

---

## Quick Win Patterns (Look for These)

### Pattern 1: Missing Flags
- Add `--version`, `--help`, `--health-check`
- Time: 10-15 min per tool

### Pattern 2: Hardcoded Values
- Replace with auto-discovery (like faelight-stow)
- Time: 20-30 min per tool

### Pattern 3: No README
- Create basic documentation
- Time: 5-10 min per tool

### Pattern 4: Compilation Warnings
- Fix unused variables/imports
- Time: 5 min per tool

---

## Time Estimates

**Quick Wins (Already Good):** 5-15 min/tool
- Just add --version, README, clean warnings

**Medium Work (Needs Fixes):** 20-45 min/tool
- Auto-discovery, proper flags, testing

**Heavy Work (Major Refactor):** 1-3 hours/tool
- Architecture changes, new features

---

## Realistic Schedule (7 Days Until Linus)

### Day 1 (Today): Tier 1 Core Tools (4 tools)
- Morning: faelight-bar audit (not full v2.0, just polish)
- Afternoon: faelight-menu, faelight-notify
- Evening: faelight-bootstrap

### Day 2: Tier 2 Version Tools (4 tools)
- bump-system-version (quick audit)
- bump-version
- faelight-snapshot
- faelight-git

### Day 3: Tier 3 Dev Tools (4 tools)
- faelight (main CLI)
- intent
- intent-guard
- archaeology-0-core

### Day 4: Tier 4 Config Tools (4 tools)
- dotctl
- core-diff
- core-protect
- profile

### Day 5: Tier 5 Utilities (6 tools)
- All remaining utilities (quick wins)

### Day 6: Polish & Testing
- Final health check
- README updates
- System version bump
- Create tool summary document

### Day 7: Presentation Prep
- Test all tools
- Prepare demos
- Final commits

---

## Success Criteria

By presentation day:
- [ ] All 28 tools audited (skip faelight-core library)
- [ ] All have --version, --help
- [ ] All compile cleanly
- [ ] All have README.md
- [ ] System health: 100%
- [ ] Version: v7.7.0 (or higher)
- [ ] TOOL_SUMMARY.md created for Linus
