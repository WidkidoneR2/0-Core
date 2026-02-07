# 🧹 CODE QUALITY SPRINT - v9.4.0
**Goal: 0 warnings, 100% clean builds across all 40 tools**

## GAME PLAN

### Phase 1: Quick Wins (30 mins)
**Cargo Profile Warnings** - 9 tools affected
Remove `[profile.*]` sections from individual Cargo.toml files:
```bash
# These files have duplicate profiles:
rust-tools/faelight/Cargo.toml
rust-tools/faelight-bar/Cargo.toml
rust-tools/faelight-dmenu/Cargo.toml
rust-tools/faelight-fetch/Cargo.toml
rust-tools/faelight-git/Cargo.toml
rust-tools/faelight-launcher/Cargo.toml
rust-tools/faelight-lock/Cargo.toml
rust-tools/faelight-notify/Cargo.toml
rust-tools/keyscan/Cargo.toml
```

**Action:**
1. Remove `[profile.release]` and `[profile.dev]` sections
2. Keep ONLY workspace root profile (root Cargo.toml)
3. Test: `cargo build --release` - should show 0 profile warnings

### Phase 2: Auto-fix Unused Imports (45 mins)
**15+ tools with unused imports**

**Strategy:**
```bash
# Let Rust fix what it can automatically:
cargo fix --allow-dirty --allow-staged

# Then manually verify:
cargo clippy --all-targets
```

**Tools to fix:**
- entropy-check (Parser, Subcommand, colored)
- faelight-link, faelight-fm, faelight-menu
- faelight-update, faelight-term
- core-protect, faelight-stow, teach
- faelight-bootstrap, profile, recent-files
- safe-update, dotctl

### Phase 3: Dead Code Cleanup (60 mins)
**Remove unused functions and variables**

**Tools with dead code:**
1. **intent** (3 functions)
   - cmd_status_change
   - find_intent_file  
   - update_frontmatter

2. **bump-system-version**
   - VERSION const
   - Unused struct fields

3. **dot-doctor**
   - Unused helper functions

4. **faelight-term**
   - Unused config variables
   - Unused len variable

5. **faelight-link**
   - Unused functions

6. **dotctl**
   - cmd_help function

**Action:** Delete or comment out with TODO if might be useful later

### Phase 4: Verification (15 mins)
```bash
# Clean build from scratch
cargo clean
cargo build --release --all

# Check for any warnings
cargo clippy --all-targets

# Verify all tools work
doctor
intent validate
```

## EXPECTED RESULTS

**Before:**
- ~40 warnings across tools
- Profile conflicts
- Unused imports cluttering code
- Dead code confusing maintenance

**After:**
- ✅ 0 warnings
- ✅ Clean builds
- ✅ Lean, maintained code
- ✅ Ready for v9.4.0 release

## COMMIT STRATEGY

1. `fix: Remove duplicate Cargo profiles (9 tools)`
2. `fix: Remove unused imports (cargo fix + manual)`
3. `refactor: Remove dead code (intent, bump, doctor, etc)`
4. `chore: Verify clean builds - 0 warnings! 🎯`

## TIME ESTIMATE
**Total: ~2.5 hours**
- Phase 1: 30 mins
- Phase 2: 45 mins
- Phase 3: 60 mins
- Phase 4: 15 mins

Perfect for a focused evening session! 🔥

## TESTING CHECKLIST
After each phase:
- [ ] cargo build --release succeeds
- [ ] doctor shows 100% health
- [ ] All scripts work
- [ ] No new errors introduced

---

**READY TO MAKE IT PRISTINE! 🌲💎**

When you come back tonight, just run:
```bash
cat CODE_QUALITY_SPRINT.md
```

And we'll crush through it systematically!
