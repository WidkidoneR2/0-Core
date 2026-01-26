# 🌲 0-Core Quick Reference Card

**Version:** 8.4.0 | **Tools:** 34 | **Health:** Run `doctor`

---

## Essential Commands
```bash
# System Health
doctor                    # 14 comprehensive checks (100% = healthy)
doctor --explain          # Detailed health info

# Updates
faelight-update           # Smart update manager
faelight-update --dry-run # Check without applying

# Git Workflow  
fg status                 # Git status with lock info
fg sync                   # Pull + commit + push

# Core Protection
lock-core                 # Protect filesystem (chattr +i)
unlock-core               # Allow editing

# Profiles
profile list              # Show available profiles
profile switch gaming     # Switch to profile

# Intent Ledger
intent list               # View all intents
intent show 067           # View specific intent
```

---

## Key Keybindings

| Key | Action |
|-----|--------|
| `Super+Return` | Terminal (Foot) |
| `Super+D` | App launcher |
| `Super+L` | Lock screen |
| `Super+Shift+E` | Power menu |
| `Super+B` | Browser (Brave) |

---

## Tool Categories

**Core (11):** doctor, faelight-update, core-protect, safe-update, dotctl, entropy-check, intent-guard, faelight-stow, faelight-snapshot, core-diff, faelight-core

**Desktop (9):** faelight-fetch, faelight-bar, faelight-launcher, faelight-dmenu, faelight-menu, faelight-notify, faelight-lock, faelight-dashboard, faelight-term

**Development (11):** intent, archaeology-0-core, workspace-view, faelight-git, faelight-hooks, recent-files, profile, teach, faelight, keyscan, faelight-zone

**Version (4):** bump-system-version, faelight-bootstrap, get-version, latest-update

---

## Daily Workflow
```bash
# Morning check
doctor && fg status && faelight-update --dry-run

# Edit configs
unlock-core
nvim ~/0-core/stow/wm-sway/.config/sway/config
lock-core

# Update system
faelight-update
```

---

## Emergency Commands
```bash
# System broke? Check health
doctor --explain

# Rollback last commit
git revert HEAD && git push

# Check recent changes
git log --oneline -10

# Restore snapshot (if BTRFS)
faelight-snapshot list
faelight-snapshot restore <name>
```

---

## Documentation

- **Full Tool Reference:** `docs/TOOL_REFERENCE.md`
- **Architecture:** `docs/ARCHITECTURE.md`
- **Workflows:** `docs/WORKFLOWS.md`
- **Build Guide:** `docs/BUILD.md`

---

**Test All Tools:** `~/0-core/scripts/test-all-tools` (34/34 passing)

*The forest's essential commands.* 🌲
