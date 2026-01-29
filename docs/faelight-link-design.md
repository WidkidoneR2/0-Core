# Faelight-Link Design
**Rust-based Symlink Manager - Inspired by lnko**

## Core Algorithm (from lnko)

### Phase 1: Planning
```rust
struct LinkPlan {
    tasks: Vec<Task>,
    conflicts: Vec<Conflict>,
}

enum Task {
    Link { source: PathBuf, target: PathBuf },
    Unlink { path: PathBuf },
    Mkdir { path: PathBuf },
    Backup { from: PathBuf, to: PathBuf },
}

enum Conflict {
    FileExists { path: PathBuf, existing: PathBuf },
    BrokenLink { path: PathBuf },
    PermissionDenied { path: PathBuf },
}
```

### Phase 2: Conflict Resolution
```rust
enum ConflictAction {
    Backup,
    Skip,
    Overwrite,
    Diff,
    Quit,
}

fn resolve_conflict(conflict: &Conflict, zone: &Zone) -> Result<ConflictAction> {
    // Zone-aware prompt
    println!("⚠️  Conflict in zone {} [{}]", zone.name(), zone.status());
    println!("Path: {}", conflict.path);
    
    // If zone is locked, only allow Skip or Quit
    if zone.is_locked() {
        println!("[s]kip  [q]uit? (zone is LOCKED)");
    } else {
        println!("[b]ackup  [s]kip  [o]verwrite  [d]iff  [q]uit?");
    }
    
    // Read user choice
    // ...
}
```

### Phase 3: Execution
```rust
fn execute_plan(plan: &LinkPlan, daemon: &DaemonClient) -> Result<()> {
    // Pre-flight health check
    run_health_check()?;
    
    // Execute each task
    for task in &plan.tasks {
        match task {
            Task::Link { source, target } => {
                std::os::unix::fs::symlink(source, target)?;
                
                // Track in daemon
                daemon.track_link(source, target, intent)?;
            },
            Task::Backup { from, to } => {
                fs::rename(from, to)?;
            },
            // ...
        }
    }
    
    // Post-flight health check
    verify_health()?;
    
    Ok(())
}
```

## Ecosystem Integration

### Zone Awareness
```rust
impl LinkPlan {
    fn check_zone_permissions(&self) -> Result<()> {
        for task in &self.tasks {
            let zone = Zone::detect(&task.target())?;
            
            if zone.is_locked() {
                return Err("Cannot modify locked zone");
            }
        }
        Ok(())
    }
}
```

### Intent Tracking
```rust
struct PackageIntent {
    name: String,
    purpose: String,
    zone: Zone,
    files: Vec<PathBuf>,
}

fn track_intent(package: &Package) -> Result<()> {
    let intent = PackageIntent {
        name: package.name.clone(),
        purpose: "nvim-config".into(),
        zone: Zone::detect(&package.source)?,
        files: package.files.clone(),
    };
    
    // Store in daemon
    daemon.register_package_intent(intent)?;
    Ok(())
}
```

## Configuration Format (TOML)
```toml
# ~/.config/faelight-link/packages.toml

[[package]]
name = "editor-nvim"
source = "~/0-core/stow/editor-nvim/.config/nvim"
target = "~/.config/nvim"
zone = "0-core"
intent = "nvim-config"
health_check = true

[[package]]
name = "shell-zsh"
source = "~/0-core/stow/shell-zsh/.zshrc"
target = "~/.zshrc"
zone = "0-core"
intent = "shell-config"
health_check = true

[options]
verify_before_link = true
backup_dir = "~/.local/share/faelight-link/backups"
format = "relative"  # or "absolute"
```

## CLI Interface
```bash
# Stow a package
faelight-link stow editor-nvim

# Unstow a package  
faelight-link unstow editor-nvim

# Show status
faelight-link status

# Clean orphaned links
faelight-link clean

# Audit all links
faelight-link audit

# Format links to canonical form
faelight-link format
```

## Implementation Phases

### v0.1.0 - Core Functionality (Week 1)
- [ ] Basic link/unlink
- [ ] Conflict detection
- [ ] Interactive prompts
- [ ] TOML config parsing

### v0.2.0 - Ecosystem (Week 2)
- [ ] Zone awareness
- [ ] Health checks (pre/post)
- [ ] Verification prompts

### v0.3.0 - Advanced (Week 3)
- [ ] Intent tracking
- [ ] Daemon integration
- [ ] Orphan cleanup
- [ ] Link formatting

### v1.0.0 - Production (Week 4)
- [ ] Full Stow replacement
- [ ] Migration tool
- [ ] Documentation
- [ ] Testing

## Dependencies
```toml
[dependencies]
clap = "4.5"          # CLI
serde = "1.0"         # Config parsing
toml = "0.8"          # Config format
walkdir = "2.5"       # Directory walking
faelight-zone = "*"   # Zone detection
colored = "2.1"       # Terminal colors
dialoguer = "0.11"    # Interactive prompts
```

## Key Differences from lnko

| Feature | lnko (Lua) | Faelight-Link (Rust) |
|---------|-----------|---------------------|
| Runtime | Lua + LuaRocks | Single binary |
| Speed | Interpreter | Native code |
| Zone awareness | No | Yes |
| Intent tracking | No | Yes |
| Daemon integration | No | Yes |
| Health checks | No | Yes |
| Verification | Basic | Enhanced |

---

**Learned from lnko. Built in Rust. Ecosystem-native.** 🦀🌲

## Relationship with faelight-stow

### Separation of Concerns
**faelight-stow** (existing):
- Health monitoring tool
- Read-only verification
- Quick --fix via stow -R
- Used by dot-doctor
- STAYS AS-IS

**faelight-link** (new):
- Link management tool
- Create/remove/audit links
- Zone + intent awareness
- Daemon integration
- Replaces manual `stow` usage

### Shared Code
Both use `faelight-symlink` library:
```rust
// ~/0-core/rust-tools/faelight-symlink/src/lib.rs
pub fn scan_links(root: &Path) -> Vec<SymlinkInfo>;
pub fn is_broken(link: &Path) -> bool;
pub fn verify_link(link: &Path, expected: &Path) -> bool;
pub fn count_links(package: &str) -> usize;
```

### Usage Patterns

**Development workflow:**
```bash
# 1. Create new config package
faelight-link stow editor-nvim
  → Checks zone (0-core is locked?)
  → Asks permission
  → Creates links with intent tracking
  → Syncs with daemon

# 2. Daily health check
doctor
  → Uses faelight-stow internally
  → Quick verification
  → Reports status
```

**Troubleshooting:**
```bash
# Quick fix (uses stow -R)
faelight-stow --fix

# Deep analysis (zone + intent aware)
faelight-link audit
  → Shows broken links
  → Shows zone conflicts
  → Shows missing intents
  → Offers canonical formatting
```

### Why Not Merge?

**Could merge, but shouldn't:**
- stow is stable, used by doctor (don't break)
- link will be complex (zones, intents, daemon)
- Different cadence (stow = daily, link = setup/changes)
- Different risk (stow = safe, link = powerful)

**Better:** Keep focused, share code via library

## Relationship with faelight-stow

### Separation of Concerns
**faelight-stow** (existing):
- Health monitoring tool
- Read-only verification
- Quick --fix via stow -R
- Used by dot-doctor
- STAYS AS-IS

**faelight-link** (new):
- Link management tool
- Create/remove/audit links
- Zone + intent awareness
- Daemon integration
- Replaces manual `stow` usage

### Shared Code
Both use `faelight-symlink` library:
```rust
// ~/0-core/rust-tools/faelight-symlink/src/lib.rs
pub fn scan_links(root: &Path) -> Vec<SymlinkInfo>;
pub fn is_broken(link: &Path) -> bool;
pub fn verify_link(link: &Path, expected: &Path) -> bool;
pub fn count_links(package: &str) -> usize;
```

### Usage Patterns

**Development workflow:**
```bash
# 1. Create new config package
faelight-link stow editor-nvim
  → Checks zone (0-core is locked?)
  → Asks permission
  → Creates links with intent tracking
  → Syncs with daemon

# 2. Daily health check
doctor
  → Uses faelight-stow internally
  → Quick verification
  → Reports status
```

**Troubleshooting:**
```bash
# Quick fix (uses stow -R)
faelight-stow --fix

# Deep analysis (zone + intent aware)
faelight-link audit
  → Shows broken links
  → Shows zone conflicts
  → Shows missing intents
  → Offers canonical formatting
```

### Why Not Merge?

**Could merge, but shouldn't:**
- stow is stable, used by doctor (don't break)
- link will be complex (zones, intents, daemon)
- Different cadence (stow = daily, link = setup/changes)
- Different risk (stow = safe, link = powerful)

**Better:** Keep focused, share code via library
