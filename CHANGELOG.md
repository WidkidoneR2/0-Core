# Changelog

All notable changes to Faelight Forest / 0-Core.

---

## [5.0.0] - 2026-01-06

### Complete Rust Transition 🦀

**ALL 12 CORE SCRIPTS REWRITTEN IN RUST**

#### Rust Tools (12/12)

- `bump-system-version` - System version management
- `get-version` - Package version reader
- `latest-update` - Recently updated finder
- `bump-version` - Package version bumper
- `dotctl` - Central control utility
- `intent` - Intent Ledger management
- `profile` - System profile switching
- `teach` - Interactive teaching mode
- `core-diff` - Package-aware diff tool
- `dot-doctor` - 12-check health monitor
- `core-protect` - Immutable protection
- `safe-update` - Safe system updates

#### Benefits

- ⚡ Faster execution (compiled vs interpreted)
- 🔒 Memory safety (no buffer overflows)
- ✅ Type safety (errors at compile time)
- 🛠️ Better error handling

#### Structure

- Added `rust-tools/` with 12 Cargo projects
- Each tool is standalone binary
- `target/` directories gitignored

> "The forest speaks Rust now." 🦀🌲

## [4.2.0] - 2026-01-06

### Profile Sharing

- Added `profile export <name>` - export profiles for sharing
- Added `profile import <file>` - import community profiles
- Export includes metadata (date, system, version)
- Import strips export header automatically
- Overwrite protection with confirmation prompt

## [4.1.0] - 2026-01-06

### Teaching Mode

- Added `teach` command - interactive learning guide
- 8 lessons: Philosophy, Structure, Tools, Workflow, Profiles, Intent, Aliases, Customization
- Uses gum for beautiful prompts
- Menu-based or sequential walkthrough

## [4.0.1] - 2026-01-06

### Profile Tab Completions

- Added file-based zsh completions for profile command
- Tab completion shows all commands and profiles

## [4.0.0] - 2026-01-06

### System Profiles - The Omega Legacy

- Added `profile` command - switch system state with one command
- 4 default profiles: default, gaming, work, low-power
- Profile state tracking and history logging
- Starship prompt shows active profile
- dot-doctor Check 12: Profile health monitoring
- Philosophy: One command switches everything

## [3.6.0] - 2026-01-05

### Intent Ledger Integration

- **dot-doctor Check 11**: Intent Ledger health monitoring
  - Count intents by status (complete, planned, in-progress, decided, abandoned)
  - Validate `packages:` references exist
  - Detect version/tag status mismatch
  - Flag stale planned intents (> 30 days)
- **New frontmatter fields**: `packages:` and `version:` for intents
- **Fixed**: INTENT and archive directories no longer flagged as non-semantic
- **Fixed**: Health percentage now shows whole number

---

## [3.5.2] - 2026-01-04

### Shell Safety & Polish

- **Dangerous command highlighting**: `rm -rf`, `chmod 777`, `dd`, `mkfs` show RED (includes sudo variants)
- **Dangerous command warning**: `preexec` hook shows warning before destructive commands
- **Tab completions**: Completions for `core-diff`, `dotctl`, `intent`
- **Starship lock status**: 🔒/🔓 inline in prompt (replaces verbose `core_guard`)
- **Removed**: `core_guard` function (replaced by Starship module)

---

## [3.5.1] - 2026-01-01

### Git Guardrails

- Block commits when core is locked (immutability protection)
- Warn on push to main (requires typed confirmation: 'push-main')
- `git!` escape hatch for experts
- Only active in ~/0-core directory

---

---

_The forest remembers. 🌲_
