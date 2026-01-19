# Intent #064: Fix bump-system-version for Stow Structure

**Status:** planned  
**Priority:** HIGH  
**Created:** 2026-01-19

## Problem
`bump-system-version` is completely broken after stow migration:
- Looks for files in old locations
- Doesn't validate `--help` vs version numbers  
- Manual bumps take 45+ minutes with multiple error fixes
- High risk of forgetting to update all version references

## Goal
Restore automated version bumping with proper stow structure support

## Success Criteria
- [ ] Updates `VERSION` file (without 'v' prefix)
- [ ] Updates `stow/shell-zsh/.config/zsh/.zshrc` welcome message
- [ ] Updates README.md badges (shields.io URLs)
- [ ] Updates README.md header
- [ ] Updates README.md version history table (adds new row)
- [ ] Updates `Cargo.toml` workspace version
- [ ] Validates version format (X.Y.Z)
- [ ] Rejects `--help` and invalid arguments
- [ ] Creates git commit + tag automatically
- [ ] Runs in < 5 seconds

## Implementation
1. Update file paths in `rust-tools/bump-system-version/src/main.rs`
2. Add proper argument validation
3. Test with dry-run mode
4. Document in RELEASE.md

## Estimated Effort
2-3 hours (worth it to save 45 min per release!)

## Additional Files to Update
- [ ] README.md milestone line: `**vX.Y.Z Milestone: Description here. 🌲🦀**`
- [ ] CHANGELOG.md (template generation)
- [ ] Update relevant intents to "complete" status if specified
