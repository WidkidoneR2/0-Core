# Session Directives -- Faelight Forest

**Written:** 2026-05-30 (NixOS migration start)
**Updated:** 2026-06-05 (post-migration, pre-1.0.0)
**Author:** Christian + Claude

---

## Current State

- **OS:** NixOS 26.05 Yarara on Framework 16
- **Version:** Faelight Forest 14.1.0
- **Branch:** `nixos` (github.com/WidkidoneR2/0-Core)
- **Health:** 100% · Integrity: 100%
- **Complete intents:** 32
- **rust-tools:** 38 tools

## Full Forest Flow (working)
Power on → LUKS → tuigreet 🌲 forest green → niri → hyprlock 🌲 forest green

## Path to Faelight NixOS 1.0.0
INT-021  Pinnacle VM study          ← next priority
INT-005  Login sizing in VM         ← needs INT-021
INT-031  faelight-release v2        ← NixOS release manager
Pinnacle on real system             ← the final gate
──────────────────────────────────
Faelight NixOS 1.0.0

## Session Rules

1. **Health gate first** -- `d` at session start, `d` before closing
2. **rebuild-safe** for any risky change -- not plain `rebuild`
3. **VM first** for compositor/login changes (INT-021 sets this up)
4. **One intent at a time** -- Friday contradicts focus>speed
5. **After fsh rebuild** -- `exec faelight-shell` to pick up new binary
6. **Gate rule** -- never defer intent gates silently, ask first
7. **No sudo rm** -- fsh blocks it for safety

## Key File Paths

| File | Purpose |
|------|---------|
| `flake.nix` | System entry point |
| `hosts/framework16/configuration.nix` | System config |
| `users/christian/home.nix` | User packages + config |
| `config/niri/.config/niri/config.kdl` | Compositor config |
| `config/faelight-shell/.config/faelight-shell/config.fsh` | Shell aliases |
| `runtime/state.db` | Friday + integrity state |

## Key Aliases

| Alias | Command |
|-------|---------|
| `d` | health check |
| `rebuild-safe` | safe rebuild with health gate |
| `cistart <id>` | start intent |
| `cicomplete <id>` | complete intent |
| `fm` | file manager |
| `fmd` | dual panel file manager |
| `gc` | git commit |
| `gp` | git push |

## Architecture (Nix/Rust Boundary)

- **Rust:** intelligence, state, TUI, forest-aware behavior
- **Nix ecosystem:** infrastructure that already works correctly
- **Pattern:** Build to understand → Replace when better exists → Keep intelligence in Rust

---

*This document is the session briefing. Update it when the state changes significantly.*
