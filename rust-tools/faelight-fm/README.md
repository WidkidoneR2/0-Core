# 🌲 Faelight FM v1.0.0
**Semantic File Manager for Faelight Forest**

![Production Ready](https://img.shields.io/badge/status-production-brightgreen)
![Version](https://img.shields.io/badge/version-1.0.0-blue)

Intent-aware, zone-conscious, daemon-powered file manager built in Rust. **Better than yazi.**

---

## 🎯 What Makes It Different

Faelight FM is not a generic file manager. It's a **system state inspector** that understands:

- **🗺️ Zones** - Spatial awareness (0-core, 1-src, 2-projects, etc.)
- **🎯 Intent** - Why files exist (tracked via Intent Ledger)
- **🌿 Git Status** - Live repository state with visual markers
- **🔌 Daemon Architecture** - Universal backend for multiple frontends
- **🛡️ Zone Protection** - Cannot modify locked Core zone
- **💬 Real-time Feedback** - Status messages for every action
- **📸 Safety** - Immutability, snapshots, recovery

Traditional file managers show you files. **Faelight FM shows you meaning.**

---

## ✨ Features

### 📁 File Operations (v1.0.0!)
- **y** - Yank (copy) file with visual feedback
- **d** - Cut file (move mode)
- **v** - Paste yanked/cut file
- **Status messages** - "Yanked: filename.txt (Copy)" in real-time
- **Zone protection** - Cannot paste into locked Core zone
- **Error handling** - Clear messages for conflicts

### 🗂️ Navigation
- **hjkl** or arrow keys - Navigate files
- **Enter** or **l** - Enter directory
- **h** - Go to parent directory
- **0-5** - Jump to zone roots instantly
  - `0` → 0-core
  - `1` → 1-workspace
  - `2` → 2-src
  - `3` → 3-projects
  - `4` → 4-archive
  - `5` → 5-scratch
- **/** - Search/filter files in current directory
- **Mouse scroll** - Navigate up/down
- **q** or **ESC** - Quit

### 📝 File Viewing & Editing
- **e** - Edit selected file in nvim (preserves TUI state on return)
- **p** - Toggle preview overlay (shows file contents)
- **i** - Toggle info overlay (file metadata, git status, intent)
- **?** - Toggle help overlay (keybindings reference)

### 🔌 Daemon Integration
- **Hybrid Architecture** - Connects to faelight-daemon when available
- **Universal Backend** - Same data layer powers TUI, Neovim, and future tools
- **Real-time Updates** - Directory listings served via RPC for consistency
- **Fallback Mode** - Works without daemon (direct filesystem access)

### 🎨 Visual Excellence
- **Git status markers** - Live indicators (M=modified, A=added, ??=untracked)
- **Zone-aware colors** - Each zone has its own color identity
- **File previews** - Inline content preview with syntax awareness
- **Status bar** - Shows messages, intent info, file details
- **Selection highlighting** - Clear visual feedback
- **Clean UI** - Minimalist, information-dense design

---

## 🚀 Usage
```bash
# Run faelight-fm (connects to daemon automatically)
~/0-core/target/release/faelight-fm

# Or start in specific directory
~/0-core/target/release/faelight-fm ~/projects

# Works without daemon too (falls back to direct filesystem)
```

### Complete Keybindings

**Navigation:**
- `j/k` or `↓/↑` - Move selection
- `h/l` or `←/→` - Parent/Enter directory
- `0-5` - Jump to zone roots
- `/` - Search/filter

**File Operations:**
- `y` - Yank (copy) file
- `d` - Cut file (move mode)
- `v` - Paste file
- `e` - Edit in nvim

**Information:**
- `p` - Preview file overlay
- `i` - File info overlay
- `?` - Help overlay

**Exit:**
- `q` or `ESC` - Quit (or close overlay)

### File Operation Examples
```bash
# Copy a file
1. Navigate to file
2. Press 'y' → Status: "Yanked: file.txt (Copy)"
3. Navigate to destination
4. Press 'v' → Status: "Copied file.txt"

# Move a file
1. Navigate to file
2. Press 'd' → Status: "Yanked: file.txt (Cut)"
3. Navigate to destination
4. Press 'v' → Status: "Moved file.txt"

# Zone protection
1. Navigate to ~/0-core file
2. Press 'y' to yank
3. Try to paste → Status: "Cannot modify locked Core zone"
```

---

## 🏗️ Architecture

### Daemon-Powered Design
```
┌──────────────┐          ┌──────────────┐
│ Faelight FM  │          │   Neovim     │
│    (TUI)     │          │  (Plugins)   │
└──────┬───────┘          └──────┬───────┘
       │                         │
       └─────────┬───────────────┘
                 │
           ┌─────▼─────┐
           │  DAEMON   │
           │   (RPC)   │
           └─────┬─────┘
                 │
           ┌─────▼─────┐
           │Filesystem │
           └───────────┘
```

**Benefits:**
- Single source of truth for file metadata
- Consistent git status across all frontends
- Intent awareness shared between tools
- Future-ready for additional integrations

---

## 🎨 Git Integration

**Status Markers:**
- `M` - Modified (orange)
- `A` - Added/staged (green)
- `??` - Untracked (yellow)
- Clean files - No marker

Git status is live-updated via the daemon for performance.

---

## 📋 Version History

### ✅ v1.0.0 - Production Ready! (2026-01-29)
- ✨ **File operations** (y/d/v keybindings)
- ✨ **Status message system** (real-time feedback)
- ✨ **Zone protection** (locked Core awareness)
- 🎨 Enhanced status bar
- 🛡️ Safe operations with error handling
- 🎊 **OUT OF BETA!**

### ✅ v0.2.0-beta - Daemon & Preview (2026-01-28)
- 🔌 Daemon integration with hybrid fallback
- 🌿 Git status markers
- 📝 File preview overlay
- ✏️ Edit in nvim
- 🔍 Search/filter
- ℹ️ Info & help overlays

### ✅ v0.1.0-beta - Foundation
- 🗂️ Basic navigation
- 🗺️ Zone awareness
- 🎨 TUI rendering

---

## 🌐 Universal File Management

**ONE daemon serves MULTIPLE frontends:**

### Current Integrations:
- ✅ **Faelight FM** (TUI) - Full-featured file browser
- ✅ **LazyVim** - Telescope file picker, commands
- ✅ **AstroVim** - Same faelight.lua config
- ✅ **NvChad** - Same faelight.lua config

### Neovim Commands:
```vim
:FaelightPing     " Check daemon
:FaelightOpen     " List files
:FaelightPicker   " Telescope picker
<leader>ff        " File picker keybind
```

**Philosophy:** Write the integration ONCE, use it EVERYWHERE.

---

## 🏆 Better Than Yazi

What makes Faelight FM superior:

| Feature | Yazi | Faelight FM |
|---------|------|-------------|
| Zone Awareness | ❌ | ✅ Spatial context |
| Intent Tracking | ❌ | ✅ Why files exist |
| Daemon Backend | ❌ | ✅ Universal RPC |
| Zone Protection | ❌ | ✅ Core lock enforcement |
| Status Messages | ✅ Basic | ✅ Contextual feedback |
| Git Integration | ✅ | ✅ Live markers |
| Multiple Frontends | ❌ | ✅ TUI + Neovim |

---

## 🎓 Design Philosophy

> **"If a feature doesn't have a place to live, it doesn't get added."**

Every module has a clear responsibility. If you can't decide where code belongs, it probably doesn't belong in Faelight FM.

**Separation of Concerns:**
- `model/` - What things ARE
- `daemon/` - How to COMMUNICATE
- `fs/` - How to ACCESS (thin layer)
- `zones/` - Where they EXIST spatially
- `ui/` - How to SHOW them (no logic)
- `input/` - How to INTERACT

**Core Principles:**
- *Understanding over convenience* - Every operation is transparent
- *Manual control over automation* - User decides, system executes
- *Documentation over memorization* - Help always available
- *Safety over speed* - Zone protection, confirmations, backups

---

## 🏗️ Development

- **v0.1.0:** ~4 hours (foundation + zone awareness)
- **v0.2.0:** ~3 hours (daemon + preview + git)
- **v1.0.0:** ~2 hours (file operations + status)

**Total:** Production file manager in 9 hours! 🚀

---

## 📚 Related Projects

Part of the **Faelight Forest** ecosystem:

- **faelight-daemon** - Universal RPC backend
- **faelight-bar** - Hybrid Wayland status bar
- **faelight-zone** - Spatial awareness library
- **faelight-link** - Zone-aware symlink manager
- **dot-doctor** - System health monitoring
- **intent** - Intent Ledger management

---

**Built with intention. Part of the forest.** 🌲 **Production ready.**
