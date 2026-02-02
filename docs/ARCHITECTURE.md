# 0-Core Architecture

> **Philosophy:** Understanding over convenience. Manual control over automation.

This document explains the complete directory structure of 0-Core and how each component interacts.

---

## 📁 Directory Structure

### User Configuration Layer
```
stow/                    # 🎯 ALL dotfile packages (GNU Stow managed)
├── wm-sway/            # Sway window manager config
├── shell-zsh/          # Zsh + 188+ aliases
├── shell-nushell/      # Nushell configuration
├── editor-nvim/        # Neovim + Faelight theme + LazyVim
├── term-foot/          # Foot terminal emulator
├── fm-yazi/            # Yazi file manager
├── config-faelight/    # Typed TOML configs for Rust tools
├── browser-brave/      # Brave browser theming
├── browser-qutebrowser/# Qutebrowser config
├── prompt-starship/    # Starship prompt with lock status
├── tools-topgrade/     # Topgrade updater config
└── vcs-git/            # Git configuration + aliases
```

**Deployment:** `cd ~/0-core/03-interfaces/stow && stow package-name`  
**Purpose:** All user-level dotfiles, deployed as symlinks to `~/`

---

### Development Layer
```
rust-tools/             # 🦀 Rust workspace (ALL source code)
├── faelight/           # Unified CLI binary
├── dot-doctor/         # 14-check health monitor
├── bump-system-version/# Release automation
├── faelight-bar/       # Custom Wayland status bar
├── faelight-update/    # Interactive update manager
├── faelight-git/       # Git workflow governance
├── faelight-term/      # Terminal emulator (WIP)
├── faelight-core/      # Shared library (config, health, IPC)
└── [25 more tools]     # All production-ready
```

**Build:** `cd rust-tools && cargo build --release --workspace`  
**Output:** Binaries land in `rust-tools/target/release/`

---

### Deployment Layer
```
scripts/                # 📜 Compiled Rust binaries (GITIGNORED)
target/                 # 🚫 Cargo build artifacts (GITIGNORED)
backups/                # 🚫 Build debris (GITIGNORED)
```

**Purpose:** `scripts/` is added to `$PATH` for system-wide tool access  
**Build Flow:** `rust-tools/` → `cargo build` → `target/release/` → `cp` → `scripts/`

**Why gitignored?**
- Single-machine use = no need to commit binaries
- Rebuilding from source ensures freshness
- Keeps git repo small (~10MB instead of ~60MB)

---

### System Configuration Layer
```
system/                 # ⚙️ System-level configs (requires sudo)
├── security/           # Security hardening
│   ├── 99-hardening.conf   # sysctl kernel hardening
│   ├── jail.local          # fail2ban configuration
│   └── README.md           # Security documentation
└── systemd-user/       # User systemd services
    └── faelight-stow.service  # Auto-deploy on login
```

**Deployment:** `sudo cp system/security/* /etc/sysctl.d/`  
**Purpose:** Root-owned files that configure system behavior

---

### Automation & Deployment Layer
```
automation/             # ⏰ Scheduled maintenance
├── weekly-maintenance.sh   # System health checks

hooks/                  # 🎣 Git workflow automation
└── pre-commit          # Gitleaks secret scanning

installation/           # 🚀 Bootstrap & setup scripts
├── 01-backup.sh        # Backup existing configs
└── 02-bootstrap.sh     # One-command system setup

logs/                   # 📊 Runtime logs (GITIGNORED)
└── power.log           # Tool execution logs
```

---

### Documentation & History Layer
```
INTENT/                 # 🎯 Intent Ledger (78+ decisions)
├── decisions/          # Why we made architectural choices
├── experiments/        # What we tried and learned
├── philosophy/         # Core principles
├── future/             # Planned improvements
└── incidents/          # Problems and how we solved them

docs/                   # 📚 Documentation
├── ARCHITECTURE.md     # THIS FILE - system structure
├── BUILD.md            # Build & deploy workflow
├── THEORY_OF_OPERATION.md  # Philosophy manifesto
└── [more guides]       # Tools, workflows, keybindings

profiles/               # 👤 System state switching
packages/               # 📦 Package lists (official/AUR)
```

---

## 🔄 Build & Deploy Workflow

### Initial Setup
```bash
# 1. Clone repository
git clone https://github.com/WidkidoneR2/0-Core.git ~/0-core

# 2. Build Rust tools
cd ~/0-core/rust-tools
cargo build --release --workspace

# 3. Deploy binaries
mkdir -p ~/0-core/scripts
cp target/release/* ~/0-core/scripts/

# 4. Add to PATH (in .zshrc)
export PATH="$HOME/0-core/scripts:$PATH"

# 5. Deploy dotfiles
cd ~/0-core/03-interfaces/stow
stow wm-sway shell-zsh editor-nvim
```

### Updating Tools
```bash
# 1. Edit source
cd ~/0-core/rust-tools/dot-doctor
nvim src/main.rs

# 2. Build
cargo build --release

# 3. Deploy
cp target/release/dot-doctor ../../scripts/

# 4. Test
dot-doctor

# 5. Commit (only source)
cd ~/0-core
git add rust-tools/dot-doctor/src/main.rs
git commit -m "fix(dot-doctor): ..."
```

### Version Releases
```bash
bump-system-version 8.4.0
# Handles: VERSION, CHANGELOG, git commit, tags
```

---

## 🏗️ Design Principles

### 1. Source-First Strategy
- **Git tracks:** Source code only (`rust-tools/`)
- **Local builds:** Binaries in `scripts/` (gitignored)
- **Rationale:** Single-machine use = no need for binary distribution

### 2. Separation of Concerns
- **User configs** → `stow/` (reversible symlinks)
- **System configs** → `system/` (requires sudo)
- **Source code** → `rust-tools/` (development)
- **Binaries** → `scripts/` (deployment)

### 3. Stow-Based Deployment
- Each package is self-contained
- Deploy/undeploy with single command
- Easy to test individual components
- Symlinks make changes instant

### 4. Documented Decisions
- Every architectural choice lives in `INTENT/`
- Future you (and others) understand WHY
- Experiments documented even if they failed

---

## 🎯 Philosophy Alignment

This structure enforces 0-Core's core principles:

✅ **Manual control over automation**
- No automatic builds or deployments
- Every step requires explicit invocation

✅ **Intent over convention**
- INTENT/ as first-class directory
- Every design choice documented

✅ **Understanding over convenience**
- Build from source (no mystery binaries)
- Stow makes symlinks inspectable
- Clear separation reveals architecture

✅ **Recovery over perfection**
- `backups/` for safety during updates
- `installation/` for clean reinstalls
- `profiles/` for state switching

---

**Made with 🌲🦀 by Christian**  
*"The forest grows from source."*
