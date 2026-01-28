# 🌲 Faelight FM v0.2.0-beta

**Semantic File Manager for Faelight Forest** - Intent-aware, zone-conscious, daemon-powered navigation built in Rust.

> 🚀 **NEW in v0.2.0** - Daemon integration, file previews, git markers, and nvim editing!

---

## 🎯 Philosophy

Faelight FM is not a generic file manager. It's a **system state inspector** that understands:

- **Zones** - Spatial awareness (0-core, 1-src, 2-projects, etc.)
- **Intent** - Why files exist (tracked via Intent Ledger)
- **Git Status** - Live repository state with visual markers
- **Daemon Architecture** - Universal backend for multiple frontends
- **Safety** - Immutability, snapshots, recovery

Traditional file managers show you files. Faelight FM shows you **meaning**.

---

## ✨ Current Features (v0.2.0-beta)

### 🔌 Daemon Integration (NEW!)
- **Hybrid Architecture** - Connects to faelight-daemon when available, falls back to direct filesystem
- **Universal Backend** - Same data layer powers TUI, Neovim plugins, and future integrations
- **Real-time Updates** - Directory listings served via RPC for consistency

### 🗂️ Navigation
- **hjkl** or arrow keys - Navigate files
- **Enter** or **l** - Enter directory
- **h** - Go to parent directory
- **0-5** - Jump to zone roots instantly
  - `0` → 0-core
  - `1` → 1-src  
  - `2` → 2-projects
  - `3` → 3-archive
  - `4` → 4-media
  - `5` → secrets
- **/** - Search/filter files in current directory
- **Mouse scroll** - Navigate up/down
- **q** or **ESC** - Quit

### 📝 File Operations (NEW!)
- **e** - Edit selected file in nvim (preserves TUI state on return)
- **p** - Toggle preview overlay (shows file contents)
- **i** - Toggle info overlay (file metadata, git status, intent)
- **?** - Toggle help overlay (keybindings reference)

### 🎨 Visual Features
- **Git status markers** - Live indicators (M=modified, A=added, ??=untracked)
- **Zone-aware colors** - Each zone has its own color identity
- **File previews** - Inline content preview with syntax awareness
- **Current zone highlighting** - Always know where you are
- **Directory vs file distinction** - Different colors for clarity
- **Selection highlighting** - Clear visual feedback

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

### Module Structure
```
faelight-fm/
├── src/
│   ├── app.rs              # AppState + main loop + daemon client
│   ├── daemon/             # RPC client for faelight-daemon
│   ├── model/              # Semantic data model (Entry, Zone, Intent)
│   ├── fs/                 # Filesystem operations (hybrid mode)
│   ├── zones/              # Zone detection + navigation
│   ├── ui/                 # TUI rendering (ratatui)
│   │   ├── colors.rs       # Faelight Forest color system
│   │   ├── filelist.rs     # Main file list with git markers
│   │   ├── preview.rs      # File preview overlay
│   │   ├── info.rs         # File info overlay
│   │   └── help.rs         # Keybindings overlay
│   ├── input/              # Keyboard + mouse handlers
│   └── error.rs            # Error types
└── Cargo.toml
```

---

## 🚀 Usage
```bash
# Start faelight-daemon (auto-starts via systemd, or manually)
systemctl --user start faelight-daemon

# Run faelight-fm (connects to daemon automatically)
~/0-core/target/release/faelight-fm

# Or start in specific directory
~/0-core/target/release/faelight-fm ~/0-core

# Works without daemon too (falls back to direct filesystem)
```

### Keybindings Reference

**Navigation:**
- `j/k` or `↓/↑` - Move selection
- `h/l` or `←/→` - Parent/Enter directory  
- `0-5` - Jump to zone roots
- `/` - Search/filter

**Actions:**
- `e` - Edit file in nvim
- `p` - Preview file
- `i` - File info
- `?` - Help overlay

**Exit:**
- `q` or `ESC` - Quit (or close overlay)

---

## 🎨 Git Integration

**Status Markers:**
- `M` - Modified (orange)
- `A` - Added/staged (green)
- `??` - Untracked (yellow)
- Clean files - No marker

Git status is live-updated via the daemon for performance.

---

## 🔮 Roadmap

### ✅ v0.2.0 - Daemon & Preview (COMPLETED!)
- [x] Daemon integration with hybrid fallback
- [x] Git status markers
- [x] File preview overlay
- [x] Edit in nvim
- [x] Search/filter
- [x] Info & help overlays

### v0.3.0 - Intent & Health
- [ ] Real intent detection via Intent Ledger
- [ ] Intent display in file list
- [ ] Health checks via dot-doctor
- [ ] File-level health badges

### v0.4.0 - Safe Mutations
- [ ] Copy with intent validation
- [ ] Move with zone awareness
- [ ] Delete with snapshot requirement
- [ ] Rename with intent preservation

### v1.0.0 - Production Ready
- [ ] Full feature parity with workflow
- [ ] Comprehensive testing
- [ ] Performance optimization
- [ ] Documentation complete

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

---

## ⚠️ Known Limitations (Beta)

- **Edit redraw issue** - Returning from nvim requires extra 'q' press (cosmetic, functional)
- **No write operations** - Copy/move/delete coming in v0.4.0
- **No command mode** - Planned for later
- **Basic preview** - Text files only, no images/PDFs yet

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

## 🔧 Development
```bash
# Build
cargo build --release

# Run tests
cargo test

# Check for issues
cargo clippy

# Format code
cargo fmt
```

---

## 📚 Related Projects

Part of the **Faelight Forest** ecosystem:

- **faelight-daemon** - Universal RPC backend for file operations
- **faelight-bar** - Hybrid Wayland status bar
- **faelight-zone** - Spatial awareness library
- **dot-doctor** - System health monitoring
- **intent** - Intent Ledger management

---

## 📜 License

Part of the 0-Core personal computing environment.

---

**Built with intention. Part of the forest.** 🌲
