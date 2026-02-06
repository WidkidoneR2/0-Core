# TODO - 2026-02-06

## 1. SYSTEM VERSION BUMP
- [ ] Run bump-system-version to v9.3.0
- [ ] Verify .zshrc version
- [ ] Git commit system bump

## 2. FIX FAELIGHT-BAR HEALTH
- [ ] Debug why bar shows 50% when dot-doctor shows 100%
- [ ] Check health calculation in bar
- [ ] Verify dot-doctor health.json integration
- [ ] Test fix

## 3. FM VERSION CLEANUP
- [ ] Remove all "beta" / "alpha" tags
- [ ] Consolidate version to v2.1.0 everywhere:
  - Cargo.toml
  - README.md
  - ROADMAP.md
  - Any other docs
- [ ] Update feature status (production-ready)
- [ ] Git commit cleanup

## 4. FOCUS TARGETS
- Term (faelight-term)
- Bar (faelight-bar) 
- FM (faelight-fm)

## WINS FROM TONIGHT
🖱️ Full mouse support in FM
🎯 Zone clicking working
✨ Clean UX with helpful messages

## 5. FM EDITOR ENHANCEMENT
- [ ] Add FM_EDITOR environment variable support
- [ ] Default to Helix (`hx`) for FM quick edits
- [ ] Fallback chain: FM_EDITOR → EDITOR → hx
- [ ] Test with both Helix and Neovim
- [ ] Document in FM README

RATIONALE:
- Helix = fast, clean, Rust-aligned (FM context)
- Neovim = heavy dev work (terminal context)
- Best of both worlds approach
