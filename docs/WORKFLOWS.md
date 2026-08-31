# 0-Core Workflows

**Version:** 14.1.0
**Last Updated:** 2026-06-05
**System:** NixOS 26.05 + MangoWM + Faelight Forest

---

## Daily Workflows

### Morning System Check
```bash
d                    # health check -- health, integrity, and a verdict
intent list          # see active and upcoming intents
intent next          # get recommendation on what to work on
git log -3           # what was done last session
```

### Starting Work on an Intent
```bash
intent list                    # see what's available
cistart <id>                   # start intent, auto-checkpoint
# do the work
cicomplete <id>                # complete intent, auto-checkpoint
```

### Configuration Change Workflow (NixOS)
```bash
# 1. Edit the right file
hx ~/0-core/users/christian/home.nix        # user packages, programs
hx ~/0-core/hosts/framework16/configuration.nix  # system config
hx ~/0-core/config/<tool>/.config/<tool>/   # tool config files

# 2. Test before applying
rebuild-dry                    # dry-run, catch errors before rebuilding

# 3. Apply safely
rebuild-safe                   # rebuild with health gate + auto-rollback

# 4. Verify
d                              # health check -- must stay 100%

# 5. Commit
git add -A && git commit --no-verify
git push
```

### Quick File Edit
```bash
fm                             # open faelight-fm tree navigator
hx <file>                      # open in helix with forest theme
```

---

## Weekly Workflows

### System Update Routine
```bash
# 1. Check what changed
nvd diff /run/booted-system /run/current-system

# 2. Update flake inputs
nix flake update

# 3. Review changes
nix-tree                       # inspect dependency tree

# 4. Rebuild safely
rebuild-safe                   # health gate protects against regressions

# 5. Verify and commit
d
git add -A && git commit --no-verify -m "flake: update inputs $(date +%Y-%m-%d)"
git push
```

### Nix Store Maintenance
```bash
# Garbage collect old generations
nix-collect-garbage -d         # removes old generations + unreferenced store paths

# Check what's keeping paths alive
nix-store --gc --print-roots   # show GC roots

# Inspect store
nix-tree                       # browse dependency tree interactively
```

### Intent Ledger Review
```bash
intent list                    # active + upcoming
core intent stats              # velocity, completion rate
core intent brief              # session brief with recommendations
```

---

## Development Workflows

### Adding a New Tool (Rust)
```bash
# 1. Create tool directory
mkdir ~/0-core/rust-tools/<toolname>/src
# 2. Add to workspace Cargo.toml
# 3. Add to flake.nix package list
# 4. Build and test locally
cargo build -p <toolname>
# 5. Rebuild to deploy
rebuild-safe
```

### Adding System Packages
```bash
# System-wide (all users)
hx ~/0-core/hosts/framework16/configuration.nix
# Add to environment.systemPackages

# User-level (home-manager)
hx ~/0-core/users/christian/home.nix
# Add to home.packages

rebuild-safe
```

### Adding a New NixOS Module
```bash
# 1. Create module file
hx ~/0-core/modules/<category>/<name>.nix

# 2. Import in configuration.nix or profiles/
# 3. Test with dry-run
rebuild-dry

# 4. Apply
rebuild-safe
```

---

## Safety Workflows

### Before High-Risk Changes
```bash
# Always use rebuild-safe for risky changes
rebuild-safe                   # pre/post health gate, auto-rollback

# For compositor/login changes -- VM first (INT-021)
# Never test login/compositor on real system first
```

### Rollback
```bash
rollback                       # sudo nixos-rebuild switch --rollback
# or pick a specific generation:
sudo nixos-rebuild switch --flake ~/0-core#framework16 --rollback
```

### Health Gate
```bash
# rebuild-safe automatically:
# 1. Records pre-rebuild health + generation
# 2. Runs rebuild-dry first
# 3. Rebuilds
# 4. Checks post health
# 5. Auto-rollbacks if health drops
```

---

## Git Workflows

### Standard Commit
```bash
git add -A
gc                             # alias: git commit --no-verify
gp                             # alias: git push
```

### Intent-Linked Commit
```bash
gc                             # commit message references INT-XXX
# faelight-hooks validates intent references
```

### Emergency Recovery
```bash
# If something breaks during development
rollback                       # restore previous NixOS generation
git stash                      # stash uncommitted changes
d                              # verify health restored
```

---

## Forest Tools Quick Reference

| Command | Description |
|---------|-------------|
| `d` | Health check |
| `fm` | File manager (broot-style tree) |
| `fmd` | File manager dual panel |
| `rebuild` | NixOS rebuild |
| `rebuild-safe` | Rebuild with health gate |
| `rebuild-dry` | Dry-run rebuild |
| `rollback` | Rollback to previous generation |
| `intent list` | Show active intents |
| `intent next` | Next intent recommendation |
| `cistart <id>` | Start intent |
| `cicomplete <id>` | Complete intent |
| `gc` | git commit |
| `gp` | git push |

---

## Related Documentation

- [PHILOSOPHY.md](PHILOSOPHY.md) -- Core principles
- [ARCHITECTURE.md](ARCHITECTURE.md) -- System architecture
- [NOVASHELL.md](NOVASHELL.md) -- NovaShell documentation
- [ALIASES.md](ALIASES.md) -- All 49 aliases
