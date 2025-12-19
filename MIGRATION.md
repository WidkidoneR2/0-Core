# 🐚 Shell Migration: Fish → Zsh

## Overview

In **v3.3.2**, Faelight Forest migrated from Fish to Zsh as the primary shell.

## Why We Switched

### Problems with Fish
- ❌ Not POSIX/bash compatible (can't run bash scripts)
- ❌ Development friction (heredocs, script sourcing issues)
- ❌ Smaller ecosystem than Zsh
- ❌ Barrier for open source contributors (most devs use bash/zsh)

### Benefits of Zsh
- ✅ Full bash compatibility (no more script friction)
- ✅ Same Fish-like features via plugins
- ✅ Industry standard (professional development)
- ✅ Better for open source (more accessible)
- ✅ Massive ecosystem (oh-my-zsh, zinit, etc.)

## What Changed

### Migrated to Zsh
- Complete `.zshrc` (539 lines)
- All 188 aliases converted
- All functions converted (ya, weekly-check, update-check, etc.)
- Plugins: zsh-autosuggestions + zsh-syntax-highlighting
- Faelight Forest colors configured

### What Stayed the Same
- ✅ Faelight Forest theme (100% identical)
- ✅ Starship prompt (shell-independent)
- ✅ All keybindings (unchanged)
- ✅ Neovim/LazyVim (unchanged)
- ✅ Hyprland/Waybar (unchanged)
- ✅ Visual experience (identical!)

## Fish Configuration Archive

The complete Fish configuration is preserved in the `archive/shell-fish` branch:
```bash
# View Fish config:
git checkout archive/shell-fish
# Return to Zsh (main):
git checkout main
```

## Migration Date

**Version:** v3.3.2  
**Date:** December 18, 2024  
**Reason:** Better bash compatibility and open source accessibility

## For New Users

Just use Zsh! It's the primary shell with all features you need.

## For Fish Users

If you prefer Fish, you can:
1. Check out the `archive/shell-fish` branch
2. Use that configuration
3. Note: It won't receive updates

---

**Current Shell:** Zsh (recommended)  
**Archived Shell:** Fish (reference only)
