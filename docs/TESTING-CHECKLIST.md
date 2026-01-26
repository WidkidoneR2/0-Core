# System Testing Checklist - v8.4.0
Date: 2026-01-27
Tester: Christian

## Core Infrastructure (11 tools)

- [ ] **dot-doctor** - `doctor` → Shows 14 checks, 100% health
- [ ] **faelight-update** - `faelight-update --dry-run` → Lists updates
- [ ] **core-protect** - `core-protect status` → Shows lock status
- [ ] **safe-update** - `safe-update --help` → Shows help
- [ ] **core-diff** - `core-diff summary` → Shows package changes
- [ ] **dotctl** - `dotctl status` → Shows version info
- [ ] **entropy-check** - `entropy-check` → Shows drift status
- [ ] **intent-guard** - `intent-guard --help` → Shows help
- [ ] **faelight-stow** - `faelight-stow` → Checks stow status
- [ ] **faelight-snapshot** - `faelight-snapshot list` → Lists snapshots
- [ ] **faelight-core** - Library only (no binary to test)

## Desktop Environment (9 tools)

- [ ] **faelight-fetch** - `faelight-fetch` → Shows system info
- [ ] **faelight-bar** - Already running in Sway
- [ ] **faelight-launcher** - Launch with keybind → Opens launcher
- [ ] **faelight-dmenu** - Launch with keybind → Opens dmenu
- [ ] **faelight-menu** - Launch with keybind → Shows power menu
- [ ] **faelight-notify** - `notify-send "test"` → Shows notification
- [ ] **faelight-lock** - `faelight-lock` → Locks screen (test carefully!)
- [ ] **faelight-dashboard** - `faelight-dashboard` → Shows dashboard
- [ ] **faelight-term** - `faelight-term` → Opens terminal with emoji

## Development & Workflow (11 tools)

- [ ] **intent** - `intent list` → Shows intents
- [ ] **archaeology-0-core** - `archaeology-0-core --timeline` → Shows history
- [ ] **workspace-view** - `workspace-view` → Shows workspaces
- [ ] **faelight-git** - `fg status` → Shows git status
- [ ] **faelight-hooks** - Pre-commit/push work automatically
- [ ] **recent-files** - `recent-files today` → Shows recent files
- [ ] **profile** - `profile list` → Shows profiles
- [ ] **teach** - `teach` → Shows learning guide
- [ ] **faelight** - `faelight --help` → Shows help
- [ ] **keyscan** - `keyscan` → Scans keybindings
- [ ] **faelight-zone** - `faelight-zone` → Shows current zone

## Version Management (4 tools)

- [ ] **bump-system-version** - `bump-system-version --help` → Shows help
- [ ] **faelight-bootstrap** - `faelight-bootstrap --help` → Shows help
- [ ] **get-version** - `get-version faelight-term` → Shows version
- [ ] **latest-update** - `latest-update` → Shows recently updated

---

## Issues Found:

(Document any crashes, errors, or unexpected behavior here)

---

## Test Summary:

- Total: 35 tools
- Passed: __/35
- Failed: __/35
- Skipped: __/35

Status: [ ] All Pass [ ] Issues Found
