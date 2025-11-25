# 🔍 Meld Visual Diff Guide - Faelight Forest Edition

A practical guide to using Meld for visual file and directory comparison in your Faelight Forest workflow.

## 📖 What is Meld?

Meld is a visual diff and merge tool that helps you:
- Compare two or three files side-by-side
- See exactly what changed between versions
- Compare entire directories
- Merge changes visually
- Verify dotfiles sync

## 🚀 Quick Start

### Basic File Comparison
```bash
# Compare two files
meld file1.conf file2.conf

# Compare three files (3-way merge)
meld original.conf yours.conf theirs.conf
```

### Directory Comparison
```bash
# Compare two directories
meld ~/dotfiles/hypr ~/.config/hypr

# Compare with visual filtering
meld --diff ~/dotfiles ~/.config
```

## 🎯 Faelight Forest Workflow

### Verification Aliases

Your Fish config includes these powerful aliases:
```bash
# Verify individual configs
verify-hypr      # Compare Hypr configs
verify-waybar    # Compare Waybar configs
verify-kitty     # Compare Kitty configs
verify-fish      # Compare Fish configs
verify-nvim      # Compare Neovim configs

# Verify everything at once
verify-all       # Compare entire .config vs dotfiles
```

### Real-World Usage

**Scenario 1: Check what changed in Hypr config**
```bash
verify-hypr
```
- Left side: Your dotfiles (source of truth)
- Right side: Active config
- Colors show: Added (green), Removed (red), Changed (blue)

**Scenario 2: Sync changes to dotfiles**
1. Make changes in `~/.config/hypr/bindings.conf`
2. Run `verify-hypr` to see differences
3. If correct, copy to dotfiles:
```bash
   cp ~/.config/hypr/bindings.conf ~/dotfiles/hypr/
```
4. Commit changes:
```bash
   cd ~/dotfiles
   git add hypr/bindings.conf
   git commit -m "Update keybindings"
   git push
```

**Scenario 3: Compare before major changes**
```bash
# Backup current config
cp ~/.config/waybar/config.jsonc ~/.config/waybar/config.jsonc.backup

# Make changes...

# Compare with backup
meld ~/.config/waybar/config.jsonc.backup ~/.config/waybar/config.jsonc
```

## 🖱️ Thunar Integration

### Using Meld from Thunar

1. Open Thunar
2. Navigate to a directory
3. Select 2 files/folders
4. Right-click → **Compare with Meld**
5. Visual diff opens!

### Custom Action Setup

**Edit → Configure custom actions:**

**Name:** Compare with Meld  
**Command:** `meld %f %f`  
**Appearance Conditions:**
- ☑ Other Files
- File Pattern: `*`
- Select: 2 files minimum

## 🎨 Understanding Meld's Interface

### Color Coding

- **Green** = Added lines/files
- **Red** = Removed lines/files  
- **Blue** = Modified lines
- **Gray** = Unchanged

### Navigation

- **↑/↓** = Move between changes
- **Ctrl+D** = Next difference
- **Ctrl+E** = Previous difference
- **Ctrl+Shift+D** = Compare directories

### Action Buttons

- **←** = Copy change to left
- **→** = Copy change to right
- **X** = Delete change
- **Merge** = Merge changes (3-way)

## 💡 Pro Tips

### 1. Ignore Whitespace
```bash
meld --diff --ignore-blank-lines file1 file2
```

### 2. Filter Files

When comparing directories, use filters:
- Click **Filters** button
- Exclude: `*.pyc`, `node_modules`, `.git`
- Show only: `*.conf`, `*.json`, `*.md`

### 3. Text Filters

Ignore common differences:
- Timestamps
- Build numbers
- Auto-generated comments

**Settings → Text Filters:**
```regex
# Ignore dates
[0-9]{4}-[0-9]{2}-[0-9]{2}

# Ignore comments
^#.*$
```

### 4. Save Comparisons

File → **Save As** → Name your comparison
- Reopen later with same filters
- Great for recurring checks

### 5. Git Integration

Compare with previous commits:
```bash
# Compare current file with last commit
git show HEAD:hypr/bindings.conf > /tmp/old-bindings.conf
meld /tmp/old-bindings.conf ~/.config/hypr/bindings.conf
```

## 🔄 Common Workflows

### Workflow 1: Daily Config Check
```bash
# Morning routine: check if configs drifted
verify-all

# If differences found, investigate:
verify-hypr    # Check Hyprland
verify-waybar  # Check Waybar
# etc.
```

### Workflow 2: Pre-Commit Verification
```bash
# Before committing to dotfiles
cd ~/dotfiles

# Check what changed
meld . ~/.config

# Review each difference
# Copy confirmed changes
# Commit to git
```

### Workflow 3: Testing New Configs
```bash
# 1. Backup current config
cp ~/.config/hypr/hyprland.conf ~/.config/hypr/hyprland.conf.working

# 2. Test experimental changes in hyprland.conf
nvim ~/.config/hypr/hyprland.conf
hyprctl reload

# 3. If broken, compare:
meld ~/.config/hypr/hyprland.conf.working ~/.config/hypr/hyprland.conf

# 4. Revert or fix as needed
```

### Workflow 4: Learning from Examples
```bash
# Compare your config with examples
meld ~/dotfiles/waybar/config.jsonc /path/to/example/config.jsonc

# See what features they use
# Adapt to your setup
```

## 🐛 Troubleshooting

### Meld won't open from Thunar

**Problem:** Thunar custom action doesn't work

**Solution:**
```bash
# Test command directly
cd ~/Downloads
meld file1.txt file2.txt

# If works, recreate Thunar custom action
# Use exact command: meld %f %f
```

### "Permission Denied" errors

**Problem:** Can't compare system files

**Solution:**
```bash
# Copy to temp location first
sudo cp /etc/config.conf /tmp/
meld /tmp/config.conf ~/dotfiles/config.conf
```

### Large directory comparisons slow

**Problem:** Comparing ~/.config with dotfiles takes forever

**Solution:**
```bash
# Be specific with subdirectories
meld ~/.config/hypr ~/dotfiles/hypr
meld ~/.config/waybar ~/dotfiles/waybar

# Use filters to exclude large dirs
# Settings → Filters → Add: node_modules, .git, cache
```

## 📚 Advanced Features

### 3-Way Merge

Perfect for resolving conflicts:
```bash
meld base.conf yours.conf theirs.conf
```
- **Base** = Common ancestor
- **Yours** = Your changes  
- **Theirs** = Their changes
- Middle pane = Result

### Directory Merge
```bash
# Merge two directory structures
meld --auto-merge ~/dotfiles ~/.config ~/dotfiles-merged
```

### Blank Comparison

Start with empty comparison:
```bash
meld --diff
# Then: File → Load file into left/right panes
```

## 🎯 Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Next difference | `Ctrl+D` |
| Previous difference | `Ctrl+E` |
| Copy to right | `Alt+Right` |
| Copy to left | `Alt+Left` |
| Refresh comparison | `Ctrl+R` |
| Find | `Ctrl+F` |
| Save | `Ctrl+S` |
| Quit | `Ctrl+Q` |

## 🔗 Integration with Other Tools

### With LazyVim

Set Meld as git diff tool:
```bash
git config --global diff.tool meld
git config --global difftool.prompt false

# Then use:
git difftool file.conf
```

### With Yazi

Open Meld from Yazi:
1. Select file in Yazi
2. Press `!` (shell command)
3. Type: `meld %f ~/dotfiles/path/to/file`

### With Scripts

Create verification script:
```bash
#!/usr/bin/env bash
# ~/dotfiles/scripts/verify-all-meld

echo "🔍 Verifying all dotfiles..."

meld ~/.config/hypr ~/dotfiles/hypr &
sleep 1
meld ~/.config/waybar ~/dotfiles/waybar &
sleep 1
meld ~/.config/kitty ~/dotfiles/kitty &

echo "✅ Opened all comparisons"
```

## 📖 Additional Resources

- **Official Docs:** https://meldmerge.org/
- **Man page:** `man meld`
- **Help in app:** Help → Contents

---

**💡 Pro Tip:** Add Meld to your daily workflow! Run `verify-all` before starting work to catch any config drift. Your future self will thank you! 🌲

*Part of the Faelight Forest dotfiles collection*
