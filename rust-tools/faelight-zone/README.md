# faelight-zone v2.1.0

🗺️ **Spatial awareness for the Faelight Forest filesystem**

Know WHERE you are. Know WHAT rules apply. Never build from the wrong directory again.

---

## Concept

The filesystem has **semantic meaning**. Different directories have different purposes, risk levels, and governance rules.

**faelight-zone** makes this explicit by detecting which "zone" you're in:

| Zone | Icon | Label | Meaning | Case |
|------|------|-------|---------|------|
| **Core** | 🔒 | CORE | System configuration (0-core) | UPPERCASE |
| **Workspace** | 🦀 | WORK | Active development (rust-tools) | UPPERCASE |
| **Src** | 🛠 | SRC | Source code exploration | lowercase |
| **Project** | 💼 | PROJ | Active projects | lowercase |
| **Archive** | 💎 | ARCH | Completed work | lowercase |
| **Scratch** | 🧪 | SCR | Temporary/experimental | lowercase |

**Critical zones** (Core, Workspace) display in **UPPERCASE** to signal caution.

---

## Usage

### As a Library
```rust
use faelight_zone::{current_zone, Zone};
use std::env;
use std::path::PathBuf;

let cwd = env::current_dir()?;
let home = env::var("HOME").map(PathBuf::from)?;

let (zone, display_path) = current_zone(&cwd, &home);

if zone.is_critical() {
    println!("⚠️ You are in a critical zone!");
}

println!("{} {}", zone.icon(), display_path);
```

### As a CLI Tool
```bash
faelight-zone
# Output: 🦀 RUST-TOOLS/FAELIGHT-ZONE
```

**Integrated with Starship prompt** - shows current zone automatically:
```
🔒 0-CORE 📦 root ⚠ 0 open
```

---

## Philosophy

### "The forest knows where it is"

Different parts of the filesystem have different rules:
- **0-core/** requires Intent Ledger entries for changes
- **rust-tools/** requires cargo check before commit  
- **1-src/** is read-only exploration
- **/tmp/** is ephemeral, no safety checks

**faelight-zone** makes this context explicit and programmatically accessible.

### Why UPPERCASE for Critical Zones?

**Visual signal** - when you see `🔒 0-CORE`, you KNOW:
- Changes affect system configuration
- Intent Ledger tracking required
- Extra caution needed

Contrast with `🛠 1-src/linux-kernel` - safe to explore, no consequences.

---

## Zone Detection Rules

**Priority order** (most specific first):

1. `~/0-core/rust-tools/*` → **🦀 WORKSPACE** (UPPERCASE)
2. `~/0-core/*` → **🔒 CORE** (UPPERCASE)
3. `~/1-src/*` → **🛠 SRC** (lowercase)
4. `~/2-projects/*` → **💼 PROJECT** (lowercase)
5. `~/3-archive/*` → **💎 ARCHIVE** (lowercase)
6. Everything else → **🧪 SCRATCH** (lowercase)

---

## Integration

### Starship Prompt
Shows current zone in every prompt:
```toml
[custom.zone]
command = "faelight-zone"
when = true
```

### Future Tools
- **core-plan** - Simulate changes based on zone rules
- **core-constraints** - Enforce zone-specific policies
- **intent-guard** - Require intents in critical zones

---

## API
```rust
pub enum Zone {
    Core,      // 🔒 System configuration
    Workspace, // 🦀 Active development  
    Src,       // 🛠 Source exploration
    Project,   // 💼 Active projects
    Archive,   // 💎 Completed work
    Scratch,   // 🧪 Temporary/experimental
}

impl Zone {
    pub fn short_label(&self) -> &'static str;
    pub fn icon(&self) -> &'static str;
    pub fn is_critical(&self) -> bool;
}

pub fn current_zone(cwd: &Path, home: &Path) -> (Zone, String);
```

---

## Examples

**Working in core configuration:**
```bash
cd ~/0-core
faelight-zone
# 🔒 0-CORE
```

**Developing rust tools:**
```bash
cd ~/0-core/rust-tools/faelight-bar
faelight-zone
# 🦀 RUST-TOOLS/FAELIGHT-BAR
```

**Exploring source code:**
```bash
cd ~/1-src/alacritty
faelight-zone
# 🛠 1-src/alacritty
```

**Temporary experiments:**
```bash
cd /tmp/test-stuff
faelight-zone
# 🧪 /tmp/test-stuff
```

---

## Why This Matters

**Before faelight-zone:**
- Easy to run dangerous commands in wrong directory
- No visual indication of context
- Accidents happen

**With faelight-zone:**
- Immediate awareness of where you are
- Visual warning for critical zones
- Foundation for zone-aware tooling

---

## Part of 0-Core

One of 30+ Rust tools in the Faelight Forest ecosystem.

Enables "spatial awareness" - your tools know where they're running and can adapt behavior accordingly.

See: https://github.com/WidkidoneR2/0-Core

## Quick Wins (v2.1.0)

### New CLI Options
```bash
# Default: icon + path
faelight-zone
# Output: 🦀 RUST-TOOLS/FAELIGHT-ZONE

# Icon only (for prompts)
faelight-zone --icon
# Output: 🦀

# Label only (for scripts)
faelight-zone --label
# Output: WORK

# JSON (for integrations)
faelight-zone --json
# Output: {"zone":"Workspace","label":"WORK","icon":"🦀","path":"...","critical":true}

# Health check
faelight-zone --health
```

### Scripting Examples

**Shell Prompt Integration:**
```bash
# Add to .zshrc
ZONE_ICON=$(faelight-zone --icon)
PS1="%F{cyan}${ZONE_ICON}%f %~ %# "
```

**Conditional Commands:**
```bash
# Only run if in critical zone
if [ "$(faelight-zone --json | jq -r .critical)" = "true" ]; then
    echo "⚠️  Critical zone - proceed with caution"
fi
```

**Zone-Based Aliases:**
```bash
# Different git behavior per zone
if [ "$(faelight-zone --label)" = "CORE" ]; then
    alias gp="git push --no-verify"  # Skip hooks in core
fi
```

## Installation
```bash
cd ~/0-core
cargo build --release -p faelight-zone
cargo install --path rust-tools/faelight-zone
```

Binary installs to: `~/.cargo/bin/faelight-zone`

