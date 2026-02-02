# Intent 077: Tool Hardening Sprint - dot-doctor, core-diff, faelight-bar

## Goal
Harden three critical tools to use faelight-core paths and demonstrate ecosystem integration.

## Tomorrow's Session Plan

### 1. dot-doctor (finish up)
**Status:** Partially updated tonight (stow, VERSION, themes paths fixed)

**Remaining Work:**
- [ ] Audit all hardcoded paths in `rust-tools/dot-doctor/src/main.rs`
- [ ] Check for INTENT references (should be intents now)
- [ ] Check for any other old structure assumptions
- [ ] Test all 15 health checks work correctly
- [ ] Verify it reads from 00-meta/, 01-registry/, etc.
- [ ] Update README.md with new paths
- [ ] Rebuild and test

**Files to check:**
- `src/main.rs` - main logic
- `src/main.rs.backup` - old version (can we delete?)

### 2. core-diff
**Status:** Unknown - needs audit

**What it does:**
- Shows changes in 0-core with policy awareness
- Likely has paths to git repo, configs, rules

**Work needed:**
- [ ] Audit for hardcoded paths
- [ ] Check if it references old structure
- [ ] Update to use numbered gravity awareness
- [ ] Test it shows changes correctly
- [ ] Can it show changes in 00-meta/, 01-registry/, etc?
- [ ] Update README.md

### 3. faelight-bar
**Status:** Partially updated (VERSION path fixed)

**What it does:**
- Custom Wayland status bar
- Shows health, version, lock status, etc.
- Runs continuously

**Work needed:**
- [ ] Audit for all path references
- [ ] Check health percentage display (uses doctor?)
- [ ] Verify version reading from 00-meta/VERSION
- [ ] Check if it reads any configs from old locations
- [ ] Test bar restart shows correct info
- [ ] Update README.md

**Critical:** Bar is always visible - it MUST work flawlessly

## Success Criteria
- [ ] All 3 tools have zero hardcoded paths
- [ ] All 3 tools work perfectly with new structure  
- [ ] doctor shows 100% health
- [ ] core-diff shows changes in numbered dirs correctly
- [ ] faelight-bar displays correct version/health
- [ ] All READMEs updated

## Stretch Goal
If time permits, start creating `faelight-core/src/paths.rs` so future tools can use it.

## Timeline
- **Morning:** dot-doctor completion
- **Afternoon:** core-diff audit
- **Evening:** faelight-bar hardening
- **End of day:** Commit, test, verify

## Notes
These are visible, critical tools:
- doctor = system health (runs constantly)
- core-diff = development workflow
- bar = always on screen

Getting these perfect shows attention to detail.
