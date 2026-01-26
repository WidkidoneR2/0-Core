# recent-files v0.2.0

🕒 **Interactive file discovery dashboard**

Find and open recently modified files with beautiful categorization and interactive selection.

---

## Why This Exists

**Problem:** Finding that file you just edited requires mental archaeology.

**Solution:** Time-based file discovery with intelligent categorization and instant access.

---

## Features

### Time-Based Discovery
```bash
recent-files hour    # Last 60 minutes
recent-files today   # Last 24 hours (default)
recent-files week    # Last 7 days  
recent-files month   # Last 30 days
```

### Smart Categorization

Files automatically grouped by type:
- 🦀 **Code** - .rs, .py, .js, .c, .go, etc.
- 📄 **Documents** - .md, .txt, .pdf, .doc
- ⚙️  **Config** - .toml, .yaml, .json, .conf
- 🖼️  **Images** - .png, .jpg, .svg, .gif
- 🎬 **Videos** - .mp4, .mkv, .avi
- 📦 **Archives** - .zip, .tar, .gz
- 📁 **Other** - Everything else

### Interactive Features

**Instant open most recent:**
```bash
recent-files today --open-first
# Opens your most recently modified file immediately
```

**Interactive picker with fzf:**
```bash
recent-files today --open
# Browse files with fzf, preview with bat, select to edit
```

---

## Usage

### Basic Discovery
```bash
recent-files today
```

**Output:**
```
🕒 Recent Files Dashboard
════════════════════════════════════════════════════════════
📂 Searching: /home/christian
⏰ Range: Last 24 Hours

🦀 Code (20 files)
  0-core/rust-tools/faelight-hooks/src/main.rs 15KB 5m ago
  0-core/rust-tools/recent-files/src/main.rs 12KB 1h ago
  ...
  
📄 Documents (17 files)
  0-core/docs/ARCHITECTURE.md 8KB 2h ago
  ...

────────────────────────────────────────────────────────────
📊 Total: 61 files modified in Last 24 Hours
```

### Workflow Examples

**Resume work after break:**
```bash
recent-files today --open-first
# Jump right back to what you were working on
```

**Review yesterday's work:**
```bash
recent-files today --limit 20
# Show more files per category
```

**Find that config you edited:**
```bash
recent-files week --open
# Browse interactively, filter in fzf
```

**Search specific directory:**
```bash
recent-files today --dir ~/0-core/rust-tools
# Only search rust-tools/
```

**Get full paths:**
```bash
recent-files today --full-paths
# Show absolute paths instead of relative
```

---

## Options
```
Usage: recent-files [RANGE] [OPTIONS]

Arguments:
  [RANGE]  Time range [default: today] [possible: hour, today, week, month]

Options:
  -d, --dir <DIR>      Directory to search (default: $HOME)
  -f, --full-paths     Show full paths instead of relative
  -l, --limit <N>      Max results per category [default: 10]
  -o, --open           Interactive file picker with fzf
      --open-first     Open most recent file immediately
  -h, --help           Print help
  -V, --version        Print version
```

---

## Requirements

**Core:**
- Rust toolchain (for building)
- Standard Unix filesystem

**Optional (for --open):**
- `fzf` - Interactive selection
- `bat` - File preview in fzf

---

## Philosophy

**"Recent" is relative to your workflow.**

Some days you touch 5 files. Some days 500. `recent-files` adapts - showing you what YOU modified in YOUR chosen timeframe.

**Manual discovery over automated suggestions.**

The tool shows you what changed. YOU decide what matters. No ML, no predictions, no guessing.

**Speed matters.**

Average scan time: <100ms for 10,000 files. Built in Rust for performance.

---

## Use Cases

### Daily Standup Prep
```bash
recent-files today
# "What did I actually work on yesterday?"
```

### Context Switching
```bash
recent-files week --open
# Jump between projects via recently touched files
```

### Code Review
```bash
recent-files today --dir ~/0-core/rust-tools
# "What changed in this codebase today?"
```

### Debugging
```bash
recent-files hour
# "Which files did that script modify?"
```

---

## Design Decisions

**Why categorize by file type?**
- Mental model: "I edited a config" not "I edited .toml"
- Reduces cognitive load when scanning output

**Why default to $HOME?**
- Most tools default to current directory
- File work is scattered across projects
- $HOME gives complete picture

**Why skip hidden files?**
- `.git/`, `.cache/`, `node_modules/` are noise
- Focus on YOUR files, not tooling artifacts

**Why fzf integration?**
- Best interactive filter available
- Muscle memory from other tools
- Respects user's existing fzf config

---

## Performance

**Typical scan (~/home with 50k files):**
- Cold: ~200ms
- Warm: ~80ms

**Memory usage:**
- ~5MB for 10k files
- Scales linearly

**File limit skipping:**
- No hard caps
- Shows top N per category
- Instant summary regardless of total count

---

## Part of 0-Core

One of 34 Rust tools in the Faelight Forest ecosystem.

Built during a snow day. Solves real workflow friction.

See: https://github.com/WidkidoneR2/0-Core
