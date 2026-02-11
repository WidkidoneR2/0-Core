# Changelog - workspace-view

## [2.0.0] - 2026-02-11

### 🎉 PRODUCTION READY - SWAY WORKSPACE INTELLIGENCE

**Features:**
- Real-time workspace monitoring
- Multiple output modes
- Live watch mode
- JSON output for scripting
- Tree-based window layout
- Color-coded output

**View Modes:**
- Default: Detailed workspace view
- `--active`: Current workspace only
- `--summary`: Compact one-line view
- `--all`: Include empty workspaces
- `--watch [sec]`: Live updates (default 2s)
- `--json`: Machine-readable output

**Display:**
- Workspace numbers and names
- Window titles and app_ids
- Tree hierarchy
- Active workspace highlighting
- Empty workspace detection

**Philosophy:**
- "Understanding over convenience"
- Visual workspace awareness
- Real-time state monitoring

**Usage:**
```bash
workspace-view              # Detailed view
workspace-view --active     # Current workspace
workspace-view --summary    # Compact view
workspace-view --watch      # Live 2s updates
workspace-view --watch 5    # Live 5s updates
workspace-view --json       # JSON output
```

**Aliases:**
```bash
alias ws='workspace-view'
alias wsa='workspace-view --active'
alias wss='workspace-view --summary'
alias wsw='workspace-view --watch'
```

**Code Quality:**
- Zero clippy warnings
- 540 lines of clean code
- Comprehensive README (360 lines)
- Sway IPC integration

**Integration:**
- Part of Faelight Forest
- Sway window manager
- Real-time workspace intelligence

---

## [1.0.0] - Earlier

Sway workspace viewer.

---

**Version Format:** MAJOR.MINOR.PATCH
