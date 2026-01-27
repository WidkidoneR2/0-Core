# 🌲 Faelight FM v0.1.0-beta

**Semantic File Manager for Faelight Forest** - Intent-aware, zone-conscious navigation built in Rust.

> ⚠️ **BETA SOFTWARE** - This is an early prototype demonstrating the architecture described in the design document. Not production-ready yet.

---

## 🎯 Philosophy

Faelight FM is not a generic file manager. It's a **system state inspector** that understands:
- **Zones** - Spatial awareness (0-core, 1-src, 2-projects, etc.)
- **Intent** - Why files exist (tracked via Intent Ledger)
- **Health** - System state (via dot-doctor integration)
- **Safety** - Immutability, snapshots, recovery

Traditional file managers show you files. Faelight FM shows you **meaning**.

---

## ✨ Current Features (v0.1.0-beta)

### Navigation
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
- **Mouse scroll** - Navigate up/down
- **q** or **ESC** - Quit

### Visual Features
- **Zone-aware colors** - Each zone has its own color identity
- **Current zone highlighting** - Always know where you are
- **Directory vs file distinction** - Different colors for clarity
- **Selection highlighting** - Clear visual feedback
- **Health badges** - ✔ ⚠ ✘ status indicators

---

## 🏗️ Architecture
```
faelight-fm/
├── src/
│   ├── app.rs              # AppState + main loop
│   ├── model/              # Semantic data model (Entry, Zone, Health)
│   ├── fs/                 # Filesystem operations (read-only for now)
│   ├── zones/              # Zone detection + navigation
│   ├── ui/                 # TUI rendering (ratatui)
│   │   ├── colors.rs       # Faelight Forest color system
│   │   ├── topbar.rs       # System status bar
│   │   ├── zones.rs        # Zone panel
│   │   ├── filelist.rs     # Main file list
│   │   └── status.rs       # Selection status
│   ├── input/              # Keyboard + mouse handlers
│   └── error.rs            # Error types
└── Cargo.toml
```

**What's NOT here (by design):**
- ❌ Plugins - No plugin system
- ❌ Previews - No inline file previews
- ❌ Themes - Colors are part of the design
- ❌ Scripting - No user scripts

These absences are **intentional guardrails** that prevent feature creep.

---

## 🚀 Usage
```bash
# Navigate to faelight-fm directory
cd ~/1-src/faelight-fm

# Build
cargo build --release

# Run (starts in current directory)
./target/release/faelight-fm

# Or start in specific directory
./target/release/faelight-fm ~/0-core
```

---

## 🎨 Color System

**Faelight Forest Palette:**
- **0-core** - Red (locked/immutable)
- **1-src** - Green (active development)
- **2-projects** - Blue (structured work)
- **3-archive** - Dim gray (dormant)
- **4-media** - Purple (creative content)
- **secrets** - Red (protected)

**UI Hierarchy:**
- Bright text = directories
- Normal text = files
- Dim text = unselected zones
- Bold + background = current selection/zone

---

## 🔮 Roadmap (Future Versions)

### v0.2.0 - Intent Integration
- [ ] Real intent detection via Intent Ledger
- [ ] Intent display in file list
- [ ] Intent-based warnings before destructive actions

### v0.3.0 - Health Integration
- [ ] Real health checks via dot-doctor
- [ ] File-level health badges
- [ ] Directory health aggregation

### v0.4.0 - Safe Actions
- [ ] Copy with intent validation
- [ ] Move with zone awareness
- [ ] Delete with snapshot requirement
- [ ] Rename with intent preservation

### v0.5.0 - Git Awareness
- [ ] Git status badges (modified, untracked)
- [ ] Read-only git context
- [ ] Branch awareness

### v1.0.0 - Production Ready
- [ ] Full feature parity with core workflow
- [ ] Comprehensive testing
- [ ] Performance optimization
- [ ] Documentation complete

---

## 🎓 Design Philosophy

From the design document:

> **"If a feature doesn't have a place to live, it doesn't get added."**

Every module has a clear responsibility. If you can't decide where code belongs, it probably doesn't belong in Faelight FM.

**Separation of Concerns:**
- `model/` - What things ARE
- `fs/` - How to ACCESS them (thin layer)
- `zones/` - Where they EXIST spatially
- `intent/` - Why they EXIST semantically
- `actions/` - The ONLY place that mutates
- `ui/` - How to SHOW them (no logic)

---

## ⚠️ Known Limitations (Beta)

- **No file operations yet** - Read-only navigation only
- **Simplified zone detection** - Looks for path patterns
- **No command mode** - Planned for later
- **No intent display** - Integration pending
- **No health checks** - Integration pending
- **Mouse clicking disabled** - Scroll wheel only (safety)

---

## 🤝 Comparison to Yazi

**Yazi excels at:**
- Blazing fast navigation
- Rich previews (images, archives, PDFs)
- Generic file management
- Mature plugin ecosystem

**Faelight FM excels at:**
- System awareness (zones, intent, health)
- Safety enforcement (immutability, snapshots)
- Meaning over content
- 0-Core integration

**Can it replace Yazi?**  
Not yet - but it's getting there. For system work in 0-Core, Faelight FM already provides better context than Yazi.

---

## 🔧 Development
```bash
# Build
cargo build --release

# Run tests (when we add them)
cargo test

# Check for issues
cargo clippy

# Format code
cargo fmt
```

---

## 📚 Related Projects

Part of the **Faelight Forest** ecosystem:
- **faelight-bar** - Hybrid Wayland status bar
- **faelight-zone** - Spatial awareness library
- **dot-doctor** - System health monitoring
- **intent** - Intent Ledger management
- **bump-system-version** - Release automation

---

## 📜 License

Part of the 0-Core personal computing environment.

---

**Built with intention. Part of the forest.** 🌲
