# 🌲 Tomorrow's Plan - 0-Core Improvements

## 1. Intent System Review
- [ ] Review intent ledger structure
- [ ] Look at existing intents (if any)
- [ ] Make suggestions for improvements
- [ ] Document intent workflow

## 2. Cleanup
- [ ] Remove old TODO.md from yesterday
- [ ] Clean up any leftover temp files

## 3. Faelight-Git
- [ ] Review faelight-git features (GRS, risk scoring)
- [ ] Set up as primary git tool
- [ ] Create aliases for common operations
- [ ] Test workflow integration

## 4. Shell Improvements (ZSH)
- [ ] Create short aliases:
  - `c` → clear terminal
  - `g` → faelight-git (or git)
  - `fm` → faelight-fm
  - `d` → doctor
  - `h` → health
  - More suggestions?
- [ ] Improve prompt performance
- [ ] Better command completion
- [ ] Optimize startup time

## 5. Quality of Life
- [ ] Fast tool access (1-letter aliases)
- [ ] Better shell feedback
- [ ] Command shortcuts for daily workflow

---

## Today's Achievements 🎊

### Tools Upgraded
✅ faelight-bar v3.0.0 - Health fixed (50% → 100%)
✅ faelight-fm v2.1.0-alpha - Mouse + editor support
✅ bump-system-version v9.1.0 - BULLETPROOF automation

### System Improvements
✅ Health: 94% → 100% → 94% (git cleanup)
✅ README: Restructured (dynamic/static)
✅ Release automation: Fully bulletproof
✅ FM_EDITOR: Helix/Neovim switching
✅ Git: Clean and organized

### Releases
🎉 v9.3.0 - Tools and Automation Upgrade
🛡️ bump-system-version v9.1.0 - Production ready

**YOU BUILT LEGENDARY TOOLS TODAY!** 💎

## 6. Code Quality - Fix ALL Rust Warnings 🧹

### Cargo Profile Warnings (9 tools)
Fix workspace profile conflicts:
- [ ] Remove `[profile]` sections from individual Cargo.toml files
- [ ] Keep only workspace root profile
- Tools: faelight, faelight-bar, faelight-dmenu, faelight-fetch, faelight-git, faelight-launcher, faelight-lock, faelight-notify, keyscan

### Unused Imports (15+ tools)
Run `cargo fix` and clean up:
- [ ] entropy-check (Parser, Subcommand, colored)
- [ ] faelight-link
- [ ] faelight-fm
- [ ] faelight-menu
- [ ] faelight-update
- [ ] faelight-term
- [ ] core-protect
- [ ] faelight-stow
- [ ] teach
- [ ] faelight-bootstrap
- [ ] profile
- [ ] recent-files
- [ ] safe-update
- [ ] dotctl

### Dead Code & Unused Variables
- [ ] bump-system-version (VERSION const, unused fields)
- [ ] dot-doctor (unused helper functions)
- [ ] faelight-term (unused config, len variable)
- [ ] faelight-link (unused functions)
- [ ] dotctl (cmd_help function)

### Future Incompatibility
- [ ] wl-clipboard-rs v0.8.1 - update or replace

### Faelight-Update Improvements
- [ ] Silence benign warnings during updates
- [ ] Show only critical errors
- [ ] Add flag: `--show-warnings` for verbose output
- [ ] Implement "Category not implemented" features:
  - [ ] Neovim Plugins
  - [ ] Yazi Packages  
  - [ ] Git Repositories
  - [ ] Firmware
  - [ ] Flatpak

### Goal
**0 warnings, 100% clean builds** 🎯

