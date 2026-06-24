# Faelight Forest -- Command Reference

**Version:** 14.1.0
**Last Updated:** 2026-06-05
**System:** NixOS 26.05 + MangoWM + Faelight Forest

> The forest understands your intent. These are the commands you reach for daily.

---

## Daily Commands

### Health & Status
```bash
d                          # health check -- 22 checks, health + integrity
core doctor run            # same as d
core status                # forest state narrative
core doctor run --json     # machine-readable health output
```

### Intent Ledger
```bash
intent list                # show active + upcoming intents
intent show <id>           # show specific intent details
intent next                # recommendation on what to work on
intent stats               # velocity and completion metrics
core intent start <id>     # start an intent (cistart alias)
core intent complete <id>  # complete an intent (cicomplete alias)
core intent defer <id> "reason"    # defer a gate with reason
core intent override <id> "reason" # override a gate with audit log
core intent brief          # session brief
core intent velocity       # completion velocity metrics
core intent next           # next intent recommendation
```

### Friday Intelligence
```bash
friday                     # show Friday patterns
friday chat                # interactive Friday session
core friday status         # Friday system status
core knowledge show <key>  # show a specific knowledge entry
```

### Forest Tools
```bash
fm                         # faelight-fm tree navigator
fmd                        # faelight-fm dual panel
core git status            # git status with forest context
core security scan         # security audit
core security report       # security report
```

---

## NixOS Commands

### Rebuild & Deploy
```bash
rebuild                    # sudo nixos-rebuild switch --flake ~/0-core#framework16
rebuild-safe               # rebuild with pre/post health gate + auto-rollback
rebuild-dry                # dry-run -- catch errors before rebuilding
rebuild-check              # dry-run + quick doctor
rollback                   # rollback to previous generation
```

### Nix Inspection
```bash
nix-tree                   # browse dependency tree interactively
nvd diff /run/booted-system /run/current-system  # what changed
nix flake update           # update flake inputs
nix-collect-garbage -d     # garbage collect old generations
nix-store --gc --print-roots  # show what's keeping store paths alive
```

---

## Git Commands

```bash
g                          # git
gc                         # git commit --no-verify
gp                         # git push
gs                         # git status
lg                         # lazygit -- interactive git TUI
core git status            # forest-aware git status
```

---

## Core Engine Commands

### Doctor
```bash
core doctor run            # full health check
core doctor run --json     # JSON output
core doctor check <name>   # run specific check
```

### Security
```bash
core security scan         # full security audit
core security report       # formatted report
core security debt         # how long findings have been present
core security trend        # finding count over time
```

### Sandbox
```bash
core sandbox create <name> # create isolated experiment
core sandbox list          # list active sandboxes
core sandbox graduate <n>  # promote to labs/graduated/
```

### Profile
```bash
core profile status        # current profile
core profile list          # all profiles
core profile switch <name> # switch profile
```

### Events & History
```bash
core events                # event ledger
core journal               # forest journal
core why                   # causality engine -- why is system in this state
core trace                 # trace event history
```

### Prediction & Intelligence
```bash
core predict               # prediction engine
core react                 # reaction engine
core goals                 # goal engine
core friday status         # Friday intelligence status
```

---

## Forest File Manager (faelight-fm)

### Single Panel (`fm`)
| Key | Action |
|-----|--------|
| `j/k` | Navigate up/down |
| `Enter` | Expand directory / open file in helix |
| `h` | Go up to parent |
| `g/G` | Top / bottom |
| `/` | Filter (fuzzy search) |
| `:` | Command mode (`:cd`, `:cp`, `:mv`) |
| `s` | Stage/unstage git file |
| `n` | Nix store info |
| `r` | Show GC roots |
| `.` | Toggle hidden files |
| `y` | Yank path to clipboard |
| `d` | Delete (moves to trash) |
| `q` | Quit |

### Dual Panel (`fmd`)
| Key | Action |
|-----|--------|
| `Tab` | Switch active panel |
| `:cp` | Copy selected to other panel |
| `:mv` | Move selected to other panel |
| `:cd <path>` | Navigate to path |

---

## Key Bindings (MangoWM)

| Binding | Action |
|---------|--------|
| `Mod+Return` | Alacritty terminal |
| `Mod+Alt+Return` | faelight-ade |
| `Mod+Escape` | faelight-menu |
| `Mod+Ctrl+Escape` | hyprlock (lock screen) |
| `Mod+D` | App launcher |
| `Mod+H/J/K/L` | Focus window |
| `Mod+1-9` | Switch workspace |

---

## Related Documentation

- [PHILOSOPHY.md](PHILOSOPHY.md) -- Core principles
- [WORKFLOWS.md](WORKFLOWS.md) -- Day-to-day workflows
- [ALIASES.md](ALIASES.md) -- All 50 aliases
- [FAELIGHT-SHELL.md](FAELIGHT-SHELL.md) -- fsh documentation
- [ARCHITECTURE.md](ARCHITECTURE.md) -- System architecture
