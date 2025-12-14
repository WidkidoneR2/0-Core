# v3.1.0 Hybrid Architecture - December 14, 2025

## 🏗️ Major Changes

### Directory Restructuring
- **dotfiles → 0-core**: Renamed for numbered priority system
- Created numbered top-level structure:
  - `~/0-core` - Immutable configuration (stow packages)
  - `~/1-src` - Source code and development
  - `~/2-work` - Active projects
  - `~/3-keep` - Important files
  - `~/9-temp` - Temporary/disposable
  - `~/secrets` - Never stowed, secure storage

### Semantic Package Naming
All packages renamed for clarity:
- `hypr` → `wm-hypr` (Window Manager)
- `fish` → `shell-fish` (Shell)
- `nvim` → `editor-nvim` (Editor)
- `waybar` → `bar-waybar` (Status Bar)
- `mako` → `notif-mako` (Notifications)
- `yazi` → `fm-yazi` (File Manager)
- `git` → `vcs-git` (Version Control)
- `starship` → `prompt-starship` (Prompt)
- `brave` → `browser-brave` (Browser)
- `gtk` → `theme-gtk` (GTK Theme)
- Theme packages: `theme-term-*`, `theme-launch-*`

### Yazi Navigation Enhancement
New instant teleports for muscle-memory navigation:

**Top-Level Directories:**
- `g+0` → Core configs (~/0-core)
- `g+1` → Source code (~/1-src)
- `g+2` → Active work (~/2-work)
- `g+3` → Important files (~/3-keep)
- `g+9` → Temporary (~/9-temp)
- `g+s` → Secrets (~/secrets)

**Config Quick Access:**
- `g+h` → Hyprland config (wm-hypr)
- `g+f` → Fish shell (shell-fish)
- `g+n` → Neovim (editor-nvim)
- `g+w` → Waybar (bar-waybar)
- `g+y` → Yazi config (fm-yazi)
- `g+c` → System config (~/.config)
- `g+D` → Documentation (docs)
- `g+S` → Scripts (scripts)

Total: 14 instant navigation bindings!

## 📊 Statistics
- Files changed: 105
- Commits: 6
- Development time: 4 hours
- Final health: 100%
- Teleports: 14 instant navigation bindings
- Documentation updated: 34 file references

## 🔧 Breaking Changes
- All stow package names changed (semantic naming)
- Main directory moved from `~/dotfiles` to `~/0-core`
- Update any external scripts/aliases referencing old paths
- Fish functions updated to new paths
- Yazi keymap completely redesigned

## ✅ What's Preserved
- All configurations unchanged
- Git history fully intact
- Snapshots available for rollback (#457, #462)
- Functionally 100% backwards compatible via symlinks
- No data loss, all work preserved

## 🎯 Migration Notes

### For New Users
Just clone to `~/0-core` and stow as normal. Everything works out of the box.

### For Existing Users
If you have an old `~/dotfiles`:
1. Pull latest changes
2. Run `git checkout main`
3. Directory is already named correctly on GitHub
4. No migration needed!

### Manual Migration (if needed)
```bash
# If you cloned before v3.1:
cd ~
mv dotfiles 0-core
cd 0-core
git pull
./deploy.sh
```

## 🚀 What's Next

### Planned for v3.2:
- Package metadata system (.dotmeta files)
- Enhanced dotctl wrapper
- Environment profiles
- Immutable core protection (chattr +i)

### Future Ideas:
- Auto-theme generation improvements
- More Yazi plugins integration
- Enhanced health checks
- Backup automation

## 🙏 Credits
- 12 hours debugging password issues (solved!)
- 4 hours building v3.1 architecture
- Legendary perseverance = legendary results

---

**v3.1.0 - The Hybrid Architecture is complete! 🏆**
