# Changelog

## Architecture
- Created 00-meta/ (identity), 01-registry/ (canonical lists), 02-rules/ (enforcement), 03-interfaces/ (human-editable), 04-runtime/ (ephemeral)
- Renamed INTENT → intents (lowercase consistency)
- Deleted 1,579 lines of obsolete code
## [8.9.0] - 2026-02-02

### 🎯 Major Theme: Numbered Gravity Path Hardening

Systematic fixes for numbered gravity structure migration. Added paths modules to critical tools and fixed breaking path issues.

### 🔧 Critical Fixes

#### faelight-menu v2.0.0 (BREAKING)
- **CRITICAL**: Fixed system shutdown deadlock bug (4+ hour debug session!)
  - Root cause: Script exited Sway manually, killing itself before systemctl could run
  - Solution: Let systemd handle Sway termination
- Created `src/paths.rs` module for script paths
- Graceful shutdown now works reliably
- See: `intents/incidents/2026-02-03-shutdown-broken-numbered-gravity.md`

#### faelight-stow v1.0.0 (BREAKING)
- **BREAKING**: Updated for numbered gravity structure
- Created `src/paths.rs` module
- Fixed stow directory: `stow` → `03-interfaces/stow`
- All 13 packages now properly detected

#### faelight-bar v2.1.0
- **MAJOR**: Replaced simple 5-check health with doctor integration
- Created `src/paths.rs` module with `doctor_path()`
- Added lazy_static for 30-second health caching (prevents performance impact)
- Status bar now shows accurate system health matching doctor

#### keyscan v1.1.0
- Created `src/paths.rs` module
- Centralized sway config path management
- Better error handling with `unwrap_or_default()`

### 📦 Path Fixes (Previous Sessions)

Tools updated for numbered gravity structure:
- dot-doctor: stow, VERSION, themes paths
- bump-system-version: VERSION, stow, README, CHANGELOG paths
- get-version, latest-update, faelight-link: stow paths
- alias-audit: stow paths
- faelight-dashboard, entropy-check: VERSION paths
- profile, faelight, faelight-fetch: VERSION paths

**Path hardening progress: 15/40 tools (37.5%)**

### ✨ Features

#### Prompt 2.0
- Health indicators (● 100%, ● 93%, ● 53% with color coding)
- Git risk score (⚠️ risk=LOW/MED/HIGH)
- Entropy metrics in segment separators

### 🏥 Health

- **100%** when git is clean
- **93%** with uncommitted changes
- All 15 health checks passing
- Status bar accurately reflects system health

### 📝 Documentation

- Created Intent 077: Tool Hardening Sprint
- Incident report: 2026-02-03 shutdown deadlock
- Updated Intent 076: Path Resilience Audit (15/40 tools complete)

### 🎓 Lessons Learned

**Debugging Methodology:**
1. Comprehensive logging at all decision points
2. Manual testing to verify assumptions
3. Understanding session termination behavior
4. Don't overcomplicate (simple fixes often best)

**Architecture:**
- Individual `src/paths.rs` modules prepare for future `faelight-core/paths.rs`
- Incremental hardening is sustainable
- Critical tools first, then systematic audit

### Related

- Intent 076: Path Resilience Audit
- Intent 077: Tool Hardening Sprint
- Numbered gravity migration (v8.8.0)


## New Tools
- faelight-update v2.0.0: Better than topgrade with 9 package sources (Pacman, AUR, Cargo, Neovim, Yazi, Git, Firmware, Flatpak, Workspace)
- Lock/unlock workflow, cache cleanup, integrated health checks, git push reminder, .pacnew detection

## Registry System
- tools.toml: All 40 Rust tools documented with categories and descriptions
- aliases.toml: 50+ primary aliases (242 lines, 301 total in system)
- zones.toml: Zone definitions (0-core, 1-src, 2-projects, 3-archive, Downloads)
- profiles.toml: System profiles (default, work, gaming, low-power)

## Updates
- Updated all 40 Rust tools for new paths (stow → 03-interfaces/stow, VERSION → 00-meta/VERSION)
- Rewrote README.md (257 lines) showcasing numbered gravity architecture
- Fixed bump-system-version, doctor, and all path-dependent tools

## Health
- Achieved 100% system health (15/15 checks passing)
- All symlinks restored, all tools rebuilt, all paths updated

## v8.7.0 - 2026-01-29

**Milestone:** alias-audit v1.0.0, bump-tool-version v1.0.0, and Starship v2.0

### 🆕 New Tools (2)

**alias-audit v1.0.0** - Alias Health Checker
- Check for duplicate alias definitions
- Verify all 38 tools have proper aliases
- Detect excessive aliasing patterns
- Beautiful colored output with doctor integration
- Commands: `alias-audit`, `alias-audit duplicates`, `alias-audit missing`, `alias-audit tools`
- Result: 301 total aliases, 100% coverage (37/37 tools)

**bump-tool-version v1.0.0** - Individual Tool Version Management
- Auto-increment support (--major, --minor, --patch)
- Manual version specification
- Beautiful pre-flight dashboard (like bump-system-version)
- Handles workspace versions (converts to explicit)
- Updates Cargo.toml + README.md automatically
- Creates tool-specific git tags (e.g., faelight-link-v1.0.1)
- Git commit automation with confirmation prompts

### ✨ Enhanced Systems

**Starship Prompt v2.0**
- Smart path display (no duplication with zone names)
- Git diff stats (±files, insertions/deletions)
- Enhanced git status with counts (!2, +1, ?3, etc.)
- Profile icons: 💼 WORK, 🎮 GAMING, 🔋 LOW-POWER, 🛠️ DEV
- Conflict indicator: ⚔️ for merge conflicts
- Fixed: In 0-core root shows nothing, subdirs show relative path

**Alias System Overhaul**
- Hybrid pattern: short aliases (fm, fl, bar) + f-prefix (f-fm, f-link, f-bar)
- Fixed conflicts: fm (yazi→faelight-fm), fl (faelight→faelight-link)
- Added missing tool aliases (guard, zone, hooks, recent, ver, etc.)
- Profile icons support (bootstrap, t for teach)
- Updated version header: v8.1.0 → v8.6.0
- Total: 301 aliases covering 37/37 active tools (100%)

### 📊 Statistics

- Tools: 38 → 40 production tools (+2)
- Rust code: ~108,300 → ~109,000 lines (+~700)
- Aliases: 301 total (100% tool coverage)
- System health: 100%
- Commits: 4 major features

### 🔧 Version Management Evolution

- System versions: bump-system-version (whole forest)
- Tool versions: bump-tool-version (individual trees)
- Both support beautiful pre-flight dashboards
- bump-tool-version adds auto-increment flags

### 🌲 Philosophy

**Manual control over automation.** Two new tools exemplify this:
- alias-audit: Verify, don't assume
- bump-tool-version: Intentional, granular control

---

**Full system health maintained at 100% throughout development.**

## v8.6.0 - 2026-01-29

**Milestone:** faelight-link v1.0.0 and faelight-fm v1.0.0 - Production Ready

### 🎉 Major Achievements

Two flagship tools graduate from beta to production in a single release!

### ✨ New Features

**bump-system-version v6.0.0** - The Confidence Release
- Added auto-increment flags (--minor, --patch, --major)
- Automatic version calculation removes mental math
- Clear explanations for each increment type
- Version calculation shown before confirmation
- Both manual and auto-increment modes supported
- Enhanced help with comprehensive examples

**faelight-link v1.0.0** - Production Ready!
- Implemented unstow command (full recursive symlink removal)
- Added audit command (comprehensive health checks)
- Added clean command (broken link cleanup)
- Health percentage tracking (currently 100%)
- Safe removal with confirmation prompts
- Complete stow replacement functionality

**faelight-fm v1.0.0** - Production Ready!
- File operations: copy (y), cut (d), paste (v)
- Real-time status message system
- Zone protection (locked Core enforcement)
- Enhanced status bar with contextual feedback
- Error handling with clear user messages
- Better than yazi in every way

### 📚 Documentation

- Updated faelight-link README with production status
- Updated faelight-fm README with production status
- Added production badges to both tools
- Comprehensive usage examples and keybinding references
- "Better than yazi" comparison table

### 🏗️ Code Improvements

- Extracted run_release() function in bump-system-version
- Added file operation functions to faelight-fm fs/ops.rs
- Implemented YankMode and MessageColor enums
- Enhanced UI status bar rendering
- Zone lock checking for safe operations

### 🎯 Philosophy

"It's like aging from 17 to 18 - your whole world changes."

This release removes anxiety from progression through:
- Auto-increment removes decision fatigue
- Status messages provide real-time feedback
- Zone protection prevents accidents
- Confirmation prompts maintain control

### 📊 Statistics

- 3 major tool upgrades
- 2 tools from beta to production (v1.0.0)
- 1 major version bump (v5.1.0 → v6.0.0)
- ~650 lines of new code
- 13 commits
- 100% system health maintained
- 9 hours of focused development

### 🌲 The Forest Grows Stronger

**Production Tools:** 36 → 38 (faelight-link, faelight-fm now production-ready)
**System Health:** 100%
**Confidence Level:** Maximum

## [8.5.0] - 2026-01-26

### Added
- **Hybrid Wayland bar architecture** with integrated application launcher
  - Wayland layer-shell keyboard mode switching (first implementation of its kind)
  - Compact 400px dropdown overlay (doesn't disrupt window positions)
  - Real-time fuzzy search with nucleo (500+ applications)
  - Single-process design using compositor-mediated input modes

### Changed
- **faelight-bar**: Complete modular rewrite from v1.0.0 to v2.0.0
  - State machine architecture (bar/menu mode transitions)
  - Separate render pipeline (bar.rs, menu.rs with transparent overlays)
  - Input handling subsystem (keyboard navigation, pointer events)
  - Menu subsystem (fuzzy filtering, desktop app discovery)
  - Reduced codebase by 589 lines while adding functionality

### Documentation
- **FAELIGHT-CLI.md**: Refreshed for current CLI tool ecosystem
- **FAELIGHT-CONFIG.md**: Updated configuration patterns and examples
- **HEALTH-ENGINE.md**: Synchronized health check documentation
- **MANUAL_INSTALLATION.md**: Revised installation workflows
- **PHILOSOPHY.md**: Refined 0-Core principles and design philosophy
- **POLICIES.md**: Updated governance and contribution guidelines
- **QUICK_REFERENCE.md**: Enhanced quick reference for daily operations
- **TESTING.md**: Expanded testing strategies and tooling
- **TOOL_REFERENCE.md**: Comprehensive tool catalog refresh
- **WORKFLOWS.md**: Modernized common workflow documentation

### Fixed
- Click regions now properly tracked during rendering (profile, VPN, volume)
- Profile cycling preserved from v1.0.0
- Exclusive zone handling prevents window jumping during menu activation

## [8.4.0] - 2026-01-26

### 🎣 Added - Git Hooks Management
- **faelight-hooks v1.0.0** - Comprehensive Rust-based git hooks manager
  - Pre-commit hook: Secret scanning (gitleaks), merge conflict detection
  - Pre-push hook: Main branch protection, uncommitted changes warning  
  - Commit-msg hook: Conventional commit format validation
  - CLI commands: `install`, `check`, `config`
  - Skip functionality for flexible workflows (`--skip secrets,conflicts`)
  - Replaces bash pre-commit hook with production-ready Rust tool

### 🏗️ Changed - Architecture
- **Source-first build strategy** - Git tracks only source code now
  - Added `scripts/` and `backups/` to .gitignore
  - Binaries built locally, not committed to git
  - Repository size reduced: 60MB → 10MB (~83% smaller)
  - Aligns with "Understanding over convenience" philosophy
  - See Intent 065 for rationale

### 📚 Documentation
- Added `docs/ARCHITECTURE.md` - Complete directory structure documentation
- Added `docs/BUILD.md` - Build and deployment workflow guide
- Added `hooks/README.md` - Migration from bash to faelight-hooks
- Both intents documented: 065 (source-first), 066 (faelight-hooks)

### 🔧 Fixed
- Removed duplicate push confirmation prompts (shell function + hook)
- Fixed git hook stdin reading using /dev/tty for proper user input
- Simplified git wrapper function (removed redundant push case)

### 🎯 Philosophy Alignment
This release deepens 0-Core's commitment to:
- **Manual control over automation** - Explicit builds, intentional workflows
- **Understanding over convenience** - Source-first enforces building from code
- **Intent over convention** - Complete documentation of WHY decisions were made

### 📊 Statistics
- New Rust tool: faelight-hooks (34th in ecosystem)
- Documentation files: +3 (ARCHITECTURE.md, BUILD.md, hooks/README.md)
- Intents documented: +2 (Total: 26 intents, 11 complete)
- System health: 100% maintained throughout development

# v8.3.0 - Tool Upgrades & Terminal Perfection

**Released:** 2026-01-25

## 🚀 Major Features

### faelight-term v9.0.0 - Terminal Emulator Excellence
- ✅ **Color emoji support** (🌲🦀🔓🟢) - better rendering than foot/alacritty/kitty
- ✅ Copy/paste support (Ctrl+Shift+C/V)
- ✅ Mouse text selection (click and drag)
- ✅ Fixed text baseline alignment
- ✅ Default font size increased to 17.0
- ✅ Fixed top line clipping with increased padding

### dot-doctor v0.5.0 - Health System Enhancements
- ✅ Auto-fix mode (`--fix`) - automatically apply safe fixes
- ✅ Health history tracking (`--history`) - track system health over time
- ✅ Trending indicators (↑/↓) between health checks
- ✅ Snapshots saved to `~/.local/state/0-core/health-history.jsonl`

### bump-system-version v5.0.0 - Stress-Free Releases
- ✅ Pre-flight dashboard showing all operations before execution
- ✅ Confirmation checkpoint - no accidental releases
- ✅ Git status warning for uncommitted changes
- ✅ System health check before proceeding
- ✅ Fixed double-v bug in version formatting

## ✨ Tool Improvements

- **faelight-update v0.4.0** - Impact analysis for critical packages
- **faelight-bar v1.0.0** - Gradient separators, production-ready
- **faelight-dmenu v2.0.0** - Intent Ledger integration, stable
- **faelight-menu v0.7.0** - Color-coded danger actions (red/amber)

## 📊 Statistics

- 7 tools upgraded
- ~3,000 lines of code written/modified
- 25+ features added
- System health: 100%

# v8.2.0 Release - The Observant Garden
> "The forest knows where it is, what rules apply, and what needs attention." 🌲

**Release Date:** 2026-01-24

## 🚀 New Features

### faelight-zone v1.1.0 - Spatial Awareness System
- Complete filesystem zone detection (Core, Workspace, Src, Project, Archive, Scratch)
- UPPERCASE for critical zones (0-CORE, RUST-TOOLS)
- Lowercase for safe zones (1-src, /tmp)
- Reusable library + CLI binary
- ~150 lines of pristine Rust

### faelight-term v0.1.0 - Terminal Emulator Foundation
- Full ANSI 16-color support
- Cursor rendering and movement
- Zoom controls (Ctrl+Plus/Minus/0)
- Backspace handling
- Font rendering with JetBrains Mono
- 460+ lines of production Rust
- Tool #31 in Faelight Forest

## ⚡ Enhancements

### Starship Prompt - Complete Operational Dashboard
- Zone indicator with real-time updates (🔒 0-CORE, 🦀 RUST-TOOLS)
- Cargo root detection (📦 root vs ↳ subdir)
- Intent Ledger awareness (⚠ X open)
- Rust toolchain detection
- Enhanced git status with counts

### faelight-bar v0.10.1
- Clean, focused system status
- Removed static zone (kept in prompt where it updates)
- Profile | Workspaces | Health | Lock

### faelight-update v0.3.0
- Fixed pacman update detection (checkupdates)
- Fixed AUR sync (paru -Qua)
- Now matches topgrade perfectly

## 🔧 Bug Fixes

### bump-system-version v4.1.0
- Dynamic date calculation (was hardcoded)
- Dynamic CHANGELOG filename (was hardcoded to v8.0.0)
- Uses git log for last release date

## 📊 Impact

**Spatial Awareness:**
- Terminal prompt shows: WHERE, BUILD SAFETY, INCIDENTS, SECURITY, TOOLCHAIN
- Never build from wrong directory again
- System-wide operational intelligence

**Tools Created/Enhanced:** 5
- faelight-zone (NEW)
- faelight-term (NEW)
- faelight-update (FIXED)
- faelight-bar (SIMPLIFIED)
- bump-system-version (FIXED)

**Lines of Code:** ~700+
**System Health:** 100%

> "Built from source, protected by hooks, the forest tends its own garden." 🌲
> "The impossible is just undiscovered architecture." 🌲
> "From beta to production - the forest matures with intention." 🌲
> "From aliases to versions, from paths to profiles - clarity through tooling." 🌲
> "The numbers guide the way. Gravity guides growth." 🌲
---

# v8.1.0 Release - The Garden
> "A garden requires attention, not automation. Each update chosen, each change understood, each tool grown with care." 🌲

**Release Date:** 2026-01-23

## 📊 Release Statistics
- **System Health:** 100%
- **Total Tools:** 31 (up from 30)
- **Health Checks:** 14 (up from 13)

## 🚀 New Features

### faelight-update v0.2.0 - Interactive Update Manager 🌟
- Multi-source detection (pacman, paru, cargo, workspace, neovim)
- Interactive TUI with checkboxes for selective updates
- Health-check-first approach (runs doctor before updating)
- Confirmation dialogs and dry-run mode
- Better than topgrade: manual control, intentional updates

### dot-doctor v0.6.0 - Enhanced Security Monitoring 🔒
- Added 14th health check: Security Hardening
- UFW firewall status verification
- fail2ban service monitoring
- Mullvad VPN connection check
- SSH hardening validation (PermitRootLogin, PasswordAuthentication)

## 📦 Tool Updates
- **faelight-update**: Created v0.2.0 - Interactive update manager
- **dot-doctor**: v0.5.0 → v0.6.0 - Added security checks
- **Total tools**: 30 → 31

## 🛠️ Improvements
- Added aliases: `fu`, `fui`, `fuup`, `cdcore`, `dotgit`
- Enhanced workspace organization
- Improved ecosystem integration

---


# v8.0.0 Release - Complete Tool Audit

> "The audit is complete. Every tool documented, tested, and production-ready." 🌲

**Release Date:** 2026-01-23

## 📊 Audit Statistics

- **Total Commits:** 163
- **Tools Audited:** 29/29 (100%)
- **Intent Success Rate:** 73%
- **System Health:** 100%

## 🚀 Revolutionary Features

- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")

## 📦 Tool Updates

- Add changelog compiler and v8.0.0 draft
- Add faelight-dmenu v0.1.0 - Generic selector tool
- Add faelight-dmenu v0.2.0 with --apps mode
- Audit keyscan v1.0.0 - Production ready, no changes needed
- 🔖 Bump faelight-stow to v0.2.0
- bump-system-version v3.0.0 - Complete release automation
- bump-system-version v4.0.0 - Linus Edition release automation
- Clean up Intent Ledger for v8.0.0 focus
- core-diff v2.0.0 - Risk-aware diff tool for Linus presentation
- core-protect v1.0.0 - Immutable protection for Linus presentation
- core-protect v1.0.1 - Fix chattr error messages
- docs: Add intent entry for v8.0.0 milestone completion
- docs: Clean up audit files, update THEORY_OF_OPERATION for v8.0.0
- docs: Complete intent 044 with comprehensive v8.0.0 milestone documentation
- docs: Complete README polish for v8.0.0 presentation
- dot-doctor v0.6.0: Security hardening checks 🔒
- entropy-check v1.0.0 - Production ready drift detection
- faelight-bootstrap v1.0.0 - Linus Edition one-command installer
- faelight-dashboard v1.0.0 - Production-ready TUI system overview
- faelight-dmenu v2.0.0 - Code cleanup
- faelight-dmenu v2.0.0 - Intent-aware launcher for Linus presentation
- faelight-git v2.0.0 - Git governance with risk scoring
- faelight-git v2.1.0 - Production hardening
- ✨ faelight-launcher v3.1.0 - Refined UI & fixes
- 🎨 faelight-launcher v3.3.0 - PNG Icon System
- 🔒 faelight-lock v1.0.0 - Production ready
- faelight-snapshot v1.0.0 - Production audit complete
- 🔧 faelight-stow v0.3.0 - Auto-discover packages
- faelight-update v0.1.0: Foundation for ultimate update manager 🚀
- faelight-update v0.2.0: Interactive update manager 🚀
- faelight v1.0.0 - System orchestrator for Linus presentation
- feat: Add faelight-fetch v1.0.0 - canonical system info (Tool #32)
- feat(faelight-bar): Upgrade to v0.9.0 - Visual polish for Linus presentation
- feat(faelight-launcher): v3.2.0 - selection rounded corners + README
- feat(faelight-menu): Upgrade to v0.6.0 with UI polish and CLI standards
- feat(faelight-menu): upgrade to v0.7.0 with graceful shutdown
- 🔧 Fix fastfetch logo version - Update to v7.6.0
- fix: update all v7.6.2 references to v7.6.3 in README
- fix: update badges and version history table to v7.6.3
- fix: update version history table to show v7.6.3
- fix: update welcome message to v7.6.3 (in root .zshrc)
- 🔧 Fix v7.5.0 documentation inconsistencies
- 🔧 get-version v2.0.0 - Stow path support
- gitignore: Add CHANGELOG-v8.0.0-DRAFT.md (generated file)
- intent: add milestone updating to #064; fix v7.6.3 milestone line
- intent-guard v1.0.0 - Safety guardrails for Linus presentation
- intent v2.0.0 - Mind-blowing upgrade for Linus presentation
- keyscan v1.0.0 - Full keybind analysis tool
- 📅 latest-update v2.0.0 - Stow path support
- profile v1.0.0 - System profile manager for Linus presentation
- refactor(zsh): Major reorganization and upgrade to v8.0.0
- 🌲 Release v7.6.0 - Visual Identity & Philosophy
- 🌲 Release v7.6.1 - Foundation Fixes
- 🌲 Release v7.6.2 - UI Refinements
- 🌲 Release v7.6.3 - Stow Migration Complete
- 🌲 Release v7.6.4 - Release Automation Complete
- 🌲 Release v7.6.5 - Tool audit quick wins
- 🌲 Release v8.0.0 - Complete tool audit - 30 production-ready Rust tools, 100% system health, philosophy-driven architecture
- safe-update v1.0.0 - Production-ready safe system updates
- teach v1.0.0 - Ultimate interactive learning system
- Tool audit: faelight-git v2.1.0 complete - 34% done
- Update Cargo.lock and faelight-git binary for v2.0.0
- Update Cargo.lock for entropy-check v1.0.0
- Update Cargo.lock for intent v2.0.0 and intent-guard v1.0.0
- Update Cargo.lock for safe-update v1.0.0
- Update Cargo.lock for workspace-view v1.0.0
- Upgrade archaeology-0-core to v1.0.0 - Production ready
- Upgrade dotctl to v2.0.0 - Major rewrite
- workspace-view v1.0.0 - Production ready workspace intelligence

## 🔧 Changes by Category

### 🚀 New Features
- feat(sway): Add 18 new keybindings for 0-Core tools
- testing new feature
- 🧹 Remove leftover CHANGELOG_NEW_ENTRY.md temp file
- fix(dot-doctor): update stow path check for new directory structure
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")

### 🔧 Fixes & Improvements
- Fix: Restore main README, add dot-doctor README
- Intent cleanup: Fix cancelled intents frontmatter
- intent: Fix and complete Intent 064 - Rust Tools Audit
- fix(foot): Add term=foot to enable proper color support
- feat(yazi): Enhanced theme with neon highlights for stow packages and critical files
- fix(dot-doctor): Update .dotmeta check to reflect intentional removal
- fix: Resolve stow symlink conflicts for automatic version propagation
- core-protect v1.0.1 - Fix chattr error messages
- Fix fastfetch Nerd Font icons in foot terminal
- Fix PATH duplication with typeset -U path
- fix(faelight-bar): Improve spacing and color scheme
- 🔧 fix(bump-system-version): CHANGELOG insertion and README handling
- fix: update welcome message to v7.6.3 (in root .zshrc)
- intent: add milestone updating to #064; fix v7.6.3 milestone line
- intent: add #064 - fix bump-system-version for stow structure
- fix: update version history table to show v7.6.3
- fix: update badges and version history table to v7.6.3
- fix: update all v7.6.2 references to v7.6.3 in README
- fix(dot-doctor): update stow path check for new directory structure
- ✨ faelight-launcher v3.1.0 - Refined UI & fixes

### 📜 Documentation
- Fix: Restore main README, add dot-doctor README
- dot-doctor v0.6.0: Security hardening checks 🔒
- Document investigation status
- docs: Add Foot config reference for future matching
- docs: Complete intent 044 with comprehensive v8.0.0 milestone documentation
- docs: Add intent entry for v8.0.0 milestone completion
- fix(dot-doctor): Update .dotmeta check to reflect intentional removal
- docs: Clean up audit files, update THEORY_OF_OPERATION for v8.0.0
- docs: Complete README polish for v8.0.0 presentation
- feat(faelight-launcher): v3.2.0 - selection rounded corners + README
- 🔧 fix(bump-system-version): CHANGELOG insertion and README handling
- docs: add complete v7.6.x version history (7.6.1-7.6.3)
- fix: update all v7.6.2 references to v7.6.3 in README
- docs: reorganize README structure and update for stow migration
- docs: update README for stow-based structure
- fix(dot-doctor): update stow path check for new directory structure
- ✅ Fix dot-doctor: Detect all 11 stowable packages
- 🐛 dot-doctor: Change count from 7 to 11 packages
- 🐛 Fix dot-doctor: Dynamic package detection
- 🔧 Fix v7.5.0 documentation inconsistencies

### 🦀 Rust Improvements
- intent: Fix and complete Intent 064 - Rust Tools Audit
- intent: Complete Intent 064 - Rust Tools Audit finished
- refactor(zsh): Major reorganization and upgrade to v8.0.0
- 🌲 Release v8.0.0 - Complete tool audit - 30 production-ready Rust tools, 100% system health, philosophy-driven architecture
- Update Cargo.lock for safe-update v1.0.0
- Update Cargo.lock for workspace-view v1.0.0
- Update Cargo.lock for entropy-check v1.0.0
- Update Cargo.lock for intent v2.0.0 and intent-guard v1.0.0
- Update Cargo.lock and faelight-git binary for v2.0.0
- refactor: migrate dotfile packages to stow/ directory structure

## 📅 Complete Audit Timeline


### 2026-01-23
- aliases: Add 0-core git helpers
- aliases: Add faelight-update shortcuts
- faelight-update v0.2.0: Interactive update manager 🚀
- faelight-update v0.1.0: Foundation for ultimate update manager 🚀
- Fix: Restore main README, add dot-doctor README
- dot-doctor v0.6.0: Security hardening checks 🔒
- faelight-term: INTERACTIVE TERMINAL WORKING! 🚀
- faelight-term: WORKING TERMINAL! 🎉
- Intent cleanup: Fix cancelled intents frontmatter
- Intent 070: Infrastructure complete, status → active
- Document investigation status
- faelight-term WIP: PTY + rendering working, Wayland window visibility issue under investigation

### 2026-01-22
- docs: Add Foot config reference for future matching
- feat: Add font rendering to faelight-term (Phase 3)
- feat: faelight-term proof-of-concept working! (Intent 070)
- feat: Create faelight-term proof-of-concept (Intent 070 Phase 2)
- research: Complete Phase 1 faelight-term research (Intent 070)
- chore: Intent cleanup and reorganization
- chore: Complete fastfetch → faelight-fetch migration
- chore: Replace fastfetch with faelight-fetch in shell greeter
- feat: Add faelight-fetch v1.0.0 - canonical system info (Tool #32)
- intent: Fix and complete Intent 064 - Rust Tools Audit
- intent: Complete Intent 064 - Rust Tools Audit finished
- intent: Add Intent 070 - Build faelight-term (Tool #31)
- chore: Add TERM environment variables for proper color support
- fix(foot): Add term=foot to enable proper color support
- feat(yazi): Enhanced theme with neon highlights for stow packages and critical files
- docs: Complete intent 044 with comprehensive v8.0.0 milestone documentation
- docs: Add intent entry for v8.0.0 milestone completion
- chore: Remove temporary backup files after successful reorganization
- refactor(zsh): Major reorganization and upgrade to v8.0.0
- feat(zsh): Add 38 comprehensive tool aliases and workflows
- feat(sway): Add 18 new keybindings for 0-Core tools
- fix(dot-doctor): Update .dotmeta check to reflect intentional removal
- fix: Resolve stow symlink conflicts for automatic version propagation
- docs: Clean up audit files, update THEORY_OF_OPERATION for v8.0.0
- docs: Complete README polish for v8.0.0 presentation
- 🌲 Release v8.0.0 - Complete tool audit - 30 production-ready Rust tools, 100% system health, philosophy-driven architecture
- chore: Update Neovim plugin lockfile

### 2026-01-21
- faelight-bootstrap v1.0.0 - Linus Edition one-command installer
- bump-system-version v4.0.0 - Linus Edition release automation
- gitignore: Add CHANGELOG-v8.0.0-DRAFT.md (generated file)
- faelight-dashboard v1.0.0 - Production-ready TUI system overview
- Clean up structure: Remove infrastructure/, gitignore logs/
- Remove empty INCIDENTS/ directory
- teach v1.0.0 - Ultimate interactive learning system
- teach v1.0.0 - Ultimate interactive learning system
- Update Cargo.lock for safe-update v1.0.0
- safe-update v1.0.0 - Production-ready safe system updates
- Update Cargo.lock for workspace-view v1.0.0
- workspace-view v1.0.0 - Production ready workspace intelligence
- Update Cargo.lock for entropy-check v1.0.0
- entropy-check v1.0.0 - Production ready drift detection
- faelight-dmenu v2.0.0 - Code cleanup
- Update faelight-dmenu binary - restore border rendering
- Update Neovim plugin lockfile
- core-protect v1.0.1 - Fix chattr error messages
- Intent 067: Post-Linus evolution plan
- Add changelog compiler and v8.0.0 draft
- Add comprehensive audit tracker
- bump-system-version v3.0.0 - Complete release automation
- profile v1.0.0 - System profile manager for Linus presentation
- faelight-dmenu v2.0.0 - Intent-aware launcher for Linus presentation
- core-protect v1.0.0 - Immutable protection for Linus presentation
- core-diff v2.0.0 - Risk-aware diff tool for Linus presentation
- faelight v1.0.0 - System orchestrator for Linus presentation
- Update Cargo.lock for intent v2.0.0 and intent-guard v1.0.0
- intent-guard v1.0.0 - Safety guardrails for Linus presentation
- intent v2.0.0 - Mind-blowing upgrade for Linus presentation
- faelight-snapshot v1.0.0 - Production audit complete
- Audit keyscan v1.0.0 - Production ready, no changes needed
- Upgrade archaeology-0-core to v1.0.0 - Production ready
- Upgrade dotctl to v2.0.0 - Major rewrite
- Fix fastfetch Nerd Font icons in foot terminal
- Clean up Intent Ledger for v8.0.0 focus
- Add audit checklist and sync neovim plugins

### 2026-01-20
- Add faelight-dmenu v2.0 with apps and intents modes
- testing new feature
- Add faelight-dmenu v0.2.0 with --apps mode
- Fix PATH duplication with typeset -U path
- Auto-start faelight-notify daemon in sway
- Add faelight-dmenu v0.1.0 - Generic selector tool
- Add faelight-dmenu v0.1 - Generic selector with Faelight theme
- keyscan v1.0.0 - Full keybind analysis tool
- Tool audit: faelight-git v2.1.0 complete - 34% done
- faelight-git v2.1.0 - Production hardening
- Update Cargo.lock and faelight-git binary for v2.0.0
- faelight-git v2.0.0 - Git governance with risk scoring
- Backup before faelight-git v2.0 upgrade
- feat(faelight-menu): upgrade to v0.7.0 with graceful shutdown
- chore: Update package list (84 → 93 packages)
- fix(faelight-bar): Improve spacing and color scheme
- Update lazy-lock.json
- feat(faelight-bar): Upgrade to v0.9.0 - Visual polish for Linus presentation
- feat(faelight-menu): Upgrade to v0.6.0 with UI polish and CLI standards

### 2026-01-19
- 📦 Update Neovim plugin lockfile
- 🔧 Auto-discover stow packages - Quick Wins #5 & #6
- 🔧 faelight-stow v0.3.0 - Auto-discover packages
- 📝 Add Session 1.5 to faelight-bar PROGRESS.md
- 📝 Intent 066: Add faelight-icons ecosystem integration
- 📝 Setup faelight-bar v2.0 tracking infrastructure
- WIP: [current task] - End of session 2026-01-19
- 🧹 Remove leftover CHANGELOG_NEW_ENTRY.md temp file
- 🌲 Release v7.6.5 - Tool audit quick wins
- 🔒 faelight-lock v1.0.0 - Production ready
- 🧹 Remove obsolete theme-switch tool
- 📅 latest-update v2.0.0 - Stow path support
- 🔧 get-version v2.0.0 - Stow path support
- 🎨 faelight-launcher v3.3.0 - PNG Icon System
- feat(faelight-launcher): v3.2.0 - selection rounded corners + README
- feat(intent): Intent 052 Phase 1 - Auto-move workflow commands
- feat(intent): reorganize intent system with cancelled/deferred directories
- 🌲 Release v7.6.4 - Release Automation Complete
- 🔧 fix(bump-system-version): CHANGELOG insertion and README handling
- feat(bump-system-version): v3.0 - stow-aware with proper validation
- fix: update welcome message to v7.6.3 (in root .zshrc)
- intent: add milestone updating to #064; fix v7.6.3 milestone line
- intent: add #064 - fix bump-system-version for stow structure
- docs: add complete v7.6.x version history (7.6.1-7.6.3)
- fix: update version history table to show v7.6.3
- fix: update badges and version history table to v7.6.3
- fix: update all v7.6.2 references to v7.6.3 in README
- 🌲 Release v7.6.3 - Stow Migration Complete
- docs: reorganize README structure and update for stow migration
- docs: update README for stow-based structure
- feat: complete stow migration and infrastructure cleanup
- fix(dot-doctor): update stow path check for new directory structure
- chore: complete stow migration housekeeping
- refactor: migrate dotfile packages to stow/ directory structure
- 🌲 Release v7.6.2 - UI Refinements
- ✨ faelight-launcher v3.1.0 - Refined UI & fixes
- 🧹 Add .install-files/ to .gitignore
- 🦀 Add bump-system-version v2.0 and faelight-snapshot v1.0 - complete release automation tools
- fix: Remove broken command substitution from fastfetch logo
- chore(nvim): update lazy-lock plugin versions
- 🔧 Fix: Restore corrupted .zshrc + Incident Report
- 🔧 Fix: Restore corrupted .zshrc + Incident Report
- 🌲 Release v7.6.1 - Foundation Fixes
- 🔖 Bump faelight-stow to v0.2.0
- 🐛 Fix intent-guard: Only block moves FROM 0-core
- ✅ Fix dot-doctor: Detect all 11 stowable packages
- 🐛 dot-doctor: Change count from 7 to 11 packages
- 🐛 Fix dot-doctor: Dynamic package detection
- 🐛 Fix bump-system-version: Add validation, preserve history
- 🔧 Fix fastfetch logo version - Update to v7.6.0
- 🌲 Release v7.6.0 - Visual Identity & Philosophy
- 🔧 Fix v7.5.0 documentation inconsistencies
- 🔧 Fix v7.5.0 documentation inconsistencies

### 2026-01-18
- intent: mark 059 complete, document 060 phase 1 progress
- intent(063): formalize trust levels (OPEN / LOCKED / SEALED)
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🔧 dot-doctor: Fix stow check with canonicalize
- 🔧 dot-doctor: Remove launcher-fuzzel from stow checks
- 🗑️ Remove launcher-fuzzel (replaced by faelight-launcher v3.0)
- 🔍 faelight-launcher v3.0 - Universal Search
- 🦀 Monorepo foundation (Intent 059 - Phase 1)
- 🔒 Rewrite Intent 058: Manual fixing with explicit permission
- 📦 Update nvim plugin lockfile
- 🔧 CRITICAL: Redesign v7.7-v8.0 to honor philosophy

### 2026-01-16
- 🧹 Remove obsolete fuzzel power menu script
- 📝 Update README: 30 Rust tools + 13 health checks

---

**The forest is fully documented. 🌲🦀**

# v8.0.0 Release - Complete Tool Audit

> "The audit is complete. Every tool documented, tested, and production-ready." 🌲

**Release Date:** 2026-01-22

## 📊 Audit Statistics

- **Total Commits:** 131
- **Tools Audited:** 29/29 (100%)
- **Intent Success Rate:** 73%
- **System Health:** 100%

## 🚀 Revolutionary Features

- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")

## 📦 Tool Updates

- Add changelog compiler and v8.0.0 draft
- Add faelight-dmenu v0.1.0 - Generic selector tool
- Add faelight-dmenu v0.2.0 with --apps mode
- Audit keyscan v1.0.0 - Production ready, no changes needed
- 🔖 Bump faelight-stow to v0.2.0
- bump-system-version v3.0.0 - Complete release automation
- bump-system-version v4.0.0 - Linus Edition release automation
- Clean up Intent Ledger for v8.0.0 focus
- core-diff v2.0.0 - Risk-aware diff tool for Linus presentation
- core-protect v1.0.0 - Immutable protection for Linus presentation
- core-protect v1.0.1 - Fix chattr error messages
- entropy-check v1.0.0 - Production ready drift detection
- faelight-bootstrap v1.0.0 - Linus Edition one-command installer
- faelight-dashboard v1.0.0 - Production-ready TUI system overview
- faelight-dmenu v2.0.0 - Code cleanup
- faelight-dmenu v2.0.0 - Intent-aware launcher for Linus presentation
- faelight-git v2.0.0 - Git governance with risk scoring
- faelight-git v2.1.0 - Production hardening
- ✨ faelight-launcher v3.1.0 - Refined UI & fixes
- 🎨 faelight-launcher v3.3.0 - PNG Icon System
- 🔒 faelight-lock v1.0.0 - Production ready
- faelight-snapshot v1.0.0 - Production audit complete
- 🔧 faelight-stow v0.3.0 - Auto-discover packages
- faelight v1.0.0 - System orchestrator for Linus presentation
- feat(faelight-bar): Upgrade to v0.9.0 - Visual polish for Linus presentation
- feat(faelight-launcher): v3.2.0 - selection rounded corners + README
- feat(faelight-menu): Upgrade to v0.6.0 with UI polish and CLI standards
- feat(faelight-menu): upgrade to v0.7.0 with graceful shutdown
- 🔧 Fix fastfetch logo version - Update to v7.6.0
- fix: update all v7.6.2 references to v7.6.3 in README
- fix: update badges and version history table to v7.6.3
- fix: update version history table to show v7.6.3
- fix: update welcome message to v7.6.3 (in root .zshrc)
- 🔧 Fix v7.5.0 documentation inconsistencies
- 🔧 get-version v2.0.0 - Stow path support
- gitignore: Add CHANGELOG-v8.0.0-DRAFT.md (generated file)
- intent: add milestone updating to #064; fix v7.6.3 milestone line
- intent-guard v1.0.0 - Safety guardrails for Linus presentation
- intent v2.0.0 - Mind-blowing upgrade for Linus presentation
- keyscan v1.0.0 - Full keybind analysis tool
- 📅 latest-update v2.0.0 - Stow path support
- profile v1.0.0 - System profile manager for Linus presentation
- 🌲 Release v7.3.0 - Workspace Intelligence
- 🚀 Release v7.4.0 - Faelight Launcher XDG + Intent System v1.0
- 🌲 Release v7.4.0 - Version bump and CHANGELOG
- 🌲 Release v7.6.0 - Visual Identity & Philosophy
- 🌲 Release v7.6.1 - Foundation Fixes
- 🌲 Release v7.6.2 - UI Refinements
- 🌲 Release v7.6.3 - Stow Migration Complete
- 🌲 Release v7.6.4 - Release Automation Complete
- 🌲 Release v7.6.5 - Tool audit quick wins
- safe-update v1.0.0 - Production-ready safe system updates
- teach v1.0.0 - Ultimate interactive learning system
- Tool audit: faelight-git v2.1.0 complete - 34% done
- Update Cargo.lock and faelight-git binary for v2.0.0
- Update Cargo.lock for entropy-check v1.0.0
- Update Cargo.lock for intent v2.0.0 and intent-guard v1.0.0
- Update Cargo.lock for safe-update v1.0.0
- Update Cargo.lock for workspace-view v1.0.0
- Upgrade archaeology-0-core to v1.0.0 - Production ready
- Upgrade dotctl to v2.0.0 - Major rewrite
- workspace-view v1.0.0 - Production ready workspace intelligence

## 🔧 Changes by Category

### 🚀 New Features
- testing new feature
- 🧹 Remove leftover CHANGELOG_NEW_ENTRY.md temp file
- fix(dot-doctor): update stow path check for new directory structure
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")

### 🔧 Fixes & Improvements
- core-protect v1.0.1 - Fix chattr error messages
- Fix fastfetch Nerd Font icons in foot terminal
- Fix PATH duplication with typeset -U path
- fix(faelight-bar): Improve spacing and color scheme
- 🔧 fix(bump-system-version): CHANGELOG insertion and README handling
- fix: update welcome message to v7.6.3 (in root .zshrc)
- intent: add milestone updating to #064; fix v7.6.3 milestone line
- intent: add #064 - fix bump-system-version for stow structure
- fix: update version history table to show v7.6.3
- fix: update badges and version history table to v7.6.3
- fix: update all v7.6.2 references to v7.6.3 in README
- fix(dot-doctor): update stow path check for new directory structure
- ✨ faelight-launcher v3.1.0 - Refined UI & fixes
- fix: Remove broken command substitution from fastfetch logo
- 🔧 Fix: Restore corrupted .zshrc + Incident Report
- 🔧 Fix: Restore corrupted .zshrc + Incident Report
- 🌲 Release v7.6.1 - Foundation Fixes
- 🐛 Fix intent-guard: Only block moves FROM 0-core
- ✅ Fix dot-doctor: Detect all 11 stowable packages
- 🐛 Fix dot-doctor: Dynamic package detection

### 📜 Documentation
- feat(faelight-launcher): v3.2.0 - selection rounded corners + README
- 🔧 fix(bump-system-version): CHANGELOG insertion and README handling
- docs: add complete v7.6.x version history (7.6.1-7.6.3)
- fix: update all v7.6.2 references to v7.6.3 in README
- docs: reorganize README structure and update for stow migration
- docs: update README for stow-based structure
- fix(dot-doctor): update stow path check for new directory structure
- ✅ Fix dot-doctor: Detect all 11 stowable packages
- 🐛 dot-doctor: Change count from 7 to 11 packages
- 🐛 Fix dot-doctor: Dynamic package detection
- 🔧 Fix v7.5.0 documentation inconsistencies
- 🔧 Fix v7.5.0 documentation inconsistencies
- intent: mark 059 complete, document 060 phase 1 progress
- 🔧 dot-doctor: Fix stow check with canonicalize
- 🔧 dot-doctor: Remove launcher-fuzzel from stow checks
- 📝 Update README: 30 Rust tools + 13 health checks

### 🦀 Rust Improvements
- Update Cargo.lock for safe-update v1.0.0
- Update Cargo.lock for workspace-view v1.0.0
- Update Cargo.lock for entropy-check v1.0.0
- Update Cargo.lock for intent v2.0.0 and intent-guard v1.0.0
- Update Cargo.lock and faelight-git binary for v2.0.0
- refactor: migrate dotfile packages to stow/ directory structure
- intent(063): formalize trust levels (OPEN / LOCKED / SEALED)
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 📝 Update README: 30 Rust tools + 13 health checks

## 📅 Complete Audit Timeline


### 2026-01-22
- chore: Update Neovim plugin lockfile

### 2026-01-21
- faelight-bootstrap v1.0.0 - Linus Edition one-command installer
- bump-system-version v4.0.0 - Linus Edition release automation
- gitignore: Add CHANGELOG-v8.0.0-DRAFT.md (generated file)
- faelight-dashboard v1.0.0 - Production-ready TUI system overview
- Clean up structure: Remove infrastructure/, gitignore logs/
- Remove empty INCIDENTS/ directory
- teach v1.0.0 - Ultimate interactive learning system
- teach v1.0.0 - Ultimate interactive learning system
- Update Cargo.lock for safe-update v1.0.0
- safe-update v1.0.0 - Production-ready safe system updates
- Update Cargo.lock for workspace-view v1.0.0
- workspace-view v1.0.0 - Production ready workspace intelligence
- Update Cargo.lock for entropy-check v1.0.0
- entropy-check v1.0.0 - Production ready drift detection
- faelight-dmenu v2.0.0 - Code cleanup
- Update faelight-dmenu binary - restore border rendering
- Update Neovim plugin lockfile
- core-protect v1.0.1 - Fix chattr error messages
- Intent 067: Post-Linus evolution plan
- Add changelog compiler and v8.0.0 draft
- Add comprehensive audit tracker
- bump-system-version v3.0.0 - Complete release automation
- profile v1.0.0 - System profile manager for Linus presentation
- faelight-dmenu v2.0.0 - Intent-aware launcher for Linus presentation
- core-protect v1.0.0 - Immutable protection for Linus presentation
- core-diff v2.0.0 - Risk-aware diff tool for Linus presentation
- faelight v1.0.0 - System orchestrator for Linus presentation
- Update Cargo.lock for intent v2.0.0 and intent-guard v1.0.0
- intent-guard v1.0.0 - Safety guardrails for Linus presentation
- intent v2.0.0 - Mind-blowing upgrade for Linus presentation
- faelight-snapshot v1.0.0 - Production audit complete
- Audit keyscan v1.0.0 - Production ready, no changes needed
- Upgrade archaeology-0-core to v1.0.0 - Production ready
- Upgrade dotctl to v2.0.0 - Major rewrite
- Fix fastfetch Nerd Font icons in foot terminal
- Clean up Intent Ledger for v8.0.0 focus
- Add audit checklist and sync neovim plugins

### 2026-01-20
- Add faelight-dmenu v2.0 with apps and intents modes
- testing new feature
- Add faelight-dmenu v0.2.0 with --apps mode
- Fix PATH duplication with typeset -U path
- Auto-start faelight-notify daemon in sway
- Add faelight-dmenu v0.1.0 - Generic selector tool
- Add faelight-dmenu v0.1 - Generic selector with Faelight theme
- keyscan v1.0.0 - Full keybind analysis tool
- Tool audit: faelight-git v2.1.0 complete - 34% done
- faelight-git v2.1.0 - Production hardening
- Update Cargo.lock and faelight-git binary for v2.0.0
- faelight-git v2.0.0 - Git governance with risk scoring
- Backup before faelight-git v2.0 upgrade
- feat(faelight-menu): upgrade to v0.7.0 with graceful shutdown
- chore: Update package list (84 → 93 packages)
- fix(faelight-bar): Improve spacing and color scheme
- Update lazy-lock.json
- feat(faelight-bar): Upgrade to v0.9.0 - Visual polish for Linus presentation
- feat(faelight-menu): Upgrade to v0.6.0 with UI polish and CLI standards

### 2026-01-19
- 📦 Update Neovim plugin lockfile
- 🔧 Auto-discover stow packages - Quick Wins #5 & #6
- 🔧 faelight-stow v0.3.0 - Auto-discover packages
- 📝 Add Session 1.5 to faelight-bar PROGRESS.md
- 📝 Intent 066: Add faelight-icons ecosystem integration
- 📝 Setup faelight-bar v2.0 tracking infrastructure
- WIP: [current task] - End of session 2026-01-19
- 🧹 Remove leftover CHANGELOG_NEW_ENTRY.md temp file
- 🌲 Release v7.6.5 - Tool audit quick wins
- 🔒 faelight-lock v1.0.0 - Production ready
- 🧹 Remove obsolete theme-switch tool
- 📅 latest-update v2.0.0 - Stow path support
- 🔧 get-version v2.0.0 - Stow path support
- 🎨 faelight-launcher v3.3.0 - PNG Icon System
- feat(faelight-launcher): v3.2.0 - selection rounded corners + README
- feat(intent): Intent 052 Phase 1 - Auto-move workflow commands
- feat(intent): reorganize intent system with cancelled/deferred directories
- 🌲 Release v7.6.4 - Release Automation Complete
- 🔧 fix(bump-system-version): CHANGELOG insertion and README handling
- feat(bump-system-version): v3.0 - stow-aware with proper validation
- fix: update welcome message to v7.6.3 (in root .zshrc)
- intent: add milestone updating to #064; fix v7.6.3 milestone line
- intent: add #064 - fix bump-system-version for stow structure
- docs: add complete v7.6.x version history (7.6.1-7.6.3)
- fix: update version history table to show v7.6.3
- fix: update badges and version history table to v7.6.3
- fix: update all v7.6.2 references to v7.6.3 in README
- 🌲 Release v7.6.3 - Stow Migration Complete
- docs: reorganize README structure and update for stow migration
- docs: update README for stow-based structure
- feat: complete stow migration and infrastructure cleanup
- fix(dot-doctor): update stow path check for new directory structure
- chore: complete stow migration housekeeping
- refactor: migrate dotfile packages to stow/ directory structure
- 🌲 Release v7.6.2 - UI Refinements
- ✨ faelight-launcher v3.1.0 - Refined UI & fixes
- 🧹 Add .install-files/ to .gitignore
- 🦀 Add bump-system-version v2.0 and faelight-snapshot v1.0 - complete release automation tools
- fix: Remove broken command substitution from fastfetch logo
- chore(nvim): update lazy-lock plugin versions
- 🔧 Fix: Restore corrupted .zshrc + Incident Report
- 🔧 Fix: Restore corrupted .zshrc + Incident Report
- 🌲 Release v7.6.1 - Foundation Fixes
- 🔖 Bump faelight-stow to v0.2.0
- 🐛 Fix intent-guard: Only block moves FROM 0-core
- ✅ Fix dot-doctor: Detect all 11 stowable packages
- 🐛 dot-doctor: Change count from 7 to 11 packages
- 🐛 Fix dot-doctor: Dynamic package detection
- 🐛 Fix bump-system-version: Add validation, preserve history
- 🔧 Fix fastfetch logo version - Update to v7.6.0
- 🌲 Release v7.6.0 - Visual Identity & Philosophy
- 🔧 Fix v7.5.0 documentation inconsistencies
- 🔧 Fix v7.5.0 documentation inconsistencies

### 2026-01-18
- intent: mark 059 complete, document 060 phase 1 progress
- intent(063): formalize trust levels (OPEN / LOCKED / SEALED)
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🔧 dot-doctor: Fix stow check with canonicalize
- 🔧 dot-doctor: Remove launcher-fuzzel from stow checks
- 🗑️ Remove launcher-fuzzel (replaced by faelight-launcher v3.0)
- 🔍 faelight-launcher v3.0 - Universal Search
- 🦀 Monorepo foundation (Intent 059 - Phase 1)
- 🔒 Rewrite Intent 058: Manual fixing with explicit permission
- 📦 Update nvim plugin lockfile
- 🔧 CRITICAL: Redesign v7.7-v8.0 to honor philosophy

### 2026-01-16
- 🧹 Remove obsolete fuzzel power menu script
- 📝 Update README: 30 Rust tools + 13 health checks
- Update nvim lazy-lock.json

### 2026-01-15
- 🕐 Fix dates to 2026 + Intent 062: Forest ASCII Art
- 🎨 Intent 062: Faelight Forest ASCII Branding
- 🎯 Create v7.5-v8.0 Roadmap Intents (2026 Edition)
- 🌲 Release v7.4.0 - Version bump and CHANGELOG
- 🚀 Release v7.4.0 - Faelight Launcher XDG + Intent System v1.0

---

**The forest is fully documented. 🌲🦀**
## [7.6.5] - 2026-01-20

### 📦 Tool Updates
- **faelight-launcher v3.3.0** - PNG icon system with 8+ app icons, graceful fallback for missing icons
- **get-version v2.0.0** - Fixed stow path support, added --help/--version/--health-check
- **latest-update v2.0.0** - Fixed stow paths, human-readable time formatting, --all flag
- **faelight-lock v1.0.0** - Production ready, added --version flag and README

### 🧹 Cleanup
- Removed theme-switch (282 lines of obsolete Hyprland/Omarchy code)
- Cleaned up faelight-bootstrap tool list

### 🎯 Linus Presentation Progress
- 4/30 tools audited and polished (13%)
- Quick wins strategy in progress
- Intent 065 tracking

> "Three tools fixed, one deleted, zero regrets. The forest grows cleaner." 🌲

> "The audit is complete. Every tool documented, tested, and production-ready." 🌲
> "A garden requires attention, not automation. Each update chosen, each change understood, each tool grown with care" 🌲
> "Every tool knows its place. Every path knows its purpose. The garden observes itself" 🌲
> "Excellence emerges through intentional iteration" 🌲
---


## [7.6.4] - 2026-01-19

### 🔧 Fixes
- **bump-system-version v3.1.0** - Fixed CHANGELOG template insertion
  - No longer requires blank line after "# Changelog" header
  - Removed automatic version history table insertion (manual edit required)
  - Cleaner error messages and validation

### 📦 Tool Updates
- bump-system-version v3.1.0 - Complete release automation (Intent 060)

> "The tools that build the forest must also grow." 🌲

---

## [7.6.2] - 2026-01-19

## [7.6.3] - 2026-01-19

### 🚀 New Features
- Complete GNU Stow-based package management (Intent #063)
- All 11 dotfile packages migrated to stow/ directory
- Automated deployment: `stow -t ~ package-name`

### 🔧 Fixes  
- Updated dot-doctor to recognize new stow/ structure
- Fixed theme package detection for stow layout
- Eliminated duplicate documentation/ directory

### 📦 Tool Updates
- dot-doctor v0.4 - stow-aware health checks

> "From scattered chaos to organized intention - the forest found its structure." 🌲

---

### 📐 Typography/UI
- **faelight-launcher v3.1.0** - Refined UI with improved spacing and text rendering
