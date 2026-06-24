# Alias Reference -- Faelight Forest

**Version:** 14.1.0
**Total Aliases:** 50
**Philosophy:** Every alias has a purpose. If it doesn't earn its place, it goes.
**Updated:** 2026-06-05
**System:** NixOS 26.05 + fsh v2.5.0

---

## All Aliases

| Alias | Command | Purpose |
|-------|---------|----------|
| `b` | `bat --paging=never` | bat -- syntax-highlighted cat |
| `bar` | `faelight-bar` | Launch faelight-bar |
| `bump` | `echo \"faelight-release disabled -- needs NixOS re…` | Disabled -- needs INT-031 |
| `c` | `clear` | Clear terminal |
| `cicomplete` | `core intent complete` | Complete active intent |
| `cistart` | `core intent start` | Start an intent |
| `clip` | `faelight-clipboard` | Clipboard manager |
| `core` | `/run/current-system/sw/bin/core` | Core engine binary |
| `d` | `/run/current-system/sw/bin/core doctor run` | Health check (doctor run) |
| `deploy` | `~/0-core/pkgs/faelight/scripts/deploy` | Deploy script |
| `fg` | `faelight-git` |  |
| `fm` | `faelight-fm` | faelight-fm tree navigator |
| `fm` | `faelight-fm` | faelight-fm tree navigator |
| `fmd` | `faelight-fm --dual` | faelight-fm dual panel |
| `forecast` | `core friday health-forecast` |  |
| `forest-status` | `~/0-core/pkgs/faelight/scripts/forest-status` |  |
| `fu` | `faelight-update` |  |
| `g` | `git` | git |
| `gaa` | `git add -A` |  |
| `gc5` | `gc | first 5` |  |
| `gcm` | `git commit -m` |  |
| `gd` | `GIT_EXTERNAL_DIFF=difft git diff` |  |
| `gp` | `git push` | git push |
| `gs` | `git status` | git status |
| `gst` | `git status` |  |
| `intent` | `/run/current-system/sw/bin/intent` | Intent ledger binary |
| `l` | `eza -lh --icons --group-directories-first` | eza list with icons |
| `la` | `eza -la --icons` | eza list all including hidden |
| `ll` | `eza -la --icons` |  |
| `lock` | `faelight-lock` |  |
| `ls` | `eza --icons` |  |
| `menu` | `faelight-menu` |  |
| `notify` | `faelight-notify` |  |
| `pulse` | `faelight-pulse` |  |
| `rebuild` | `sudo nixos-rebuild switch --flake ~/0-core#framewo…` | NixOS rebuild switch |
| `rebuild-check` | `sudo nixos-rebuild dry-run --flake ~/0-core#framew…` | Dry-run + quick doctor |
| `rebuild-dry` | `sudo nixos-rebuild dry-run --flake ~/0-core#framew…` | Dry-run rebuild |
| `rebuild-home` | `sudo nixos-rebuild switch --flake ~/0-core#framewo…` |  |
| `rebuild-safe` | `bash ~/0-core/pkgs/faelight/scripts/rebuild-safe` | Rebuild with health gate |
| `rollback` | `~/0-core/pkgs/faelight/scripts/rollback` | Rollback to previous generation |
| `sec` | `security-audit` |  |
| `term` | `faelight-term` |  |
| `update-flake` | `cd ~/0-core && nix flake update && sudo nixos-rebu…` |  |
| `v` | `nvim` | nvim |
| `vault` | `faelight-vault` |  |
| `y` | `yazi` | yazi file manager |
| `ya` | `yazi` |  |

---

## Philosophy

Aliases in fsh are intentional -- each one represents a repeated action that earned permanent residence.
The old 368-alias system was Arch-era complexity. 50 aliases is the NixOS-era discipline.

**Rules:**
- Every alias must be used regularly or it gets removed
- Aliases document intent, not just shortcuts
- `rebuild-safe` over `rebuild` for any risky change
- Forest tools get short aliases (`d`, `fm`, `fmd`)

---

*Source of truth: `~/0-core/config/faelight-shell/.config/faelight-shell/config.fsh`*
