# Faelight-Link Research
**Rust-based Symlink Manager for 0-Core**

## The Problem with Stow
- Mysterious breakage
- Hard to debug
- Raises blood pressure
- Not zone-aware
- No intent tracking
- Silent failures

## Existing Alternatives Analysis

### lnko (Good ideas to borrow)
✅ Interactive conflict handling
✅ Orphan cleanup
✅ Status overview
❌ Not Rust
❌ Not ecosystem-aware

### stown (Python)
✅ Lighter than Stow
❌ Not Rust
❌ No ecosystem integration

### XStow (C++)
✅ Drop-in Stow replacement
❌ Not Rust
❌ Same conceptual model

## What's MISSING (Our Niche)
🎯 **Canonical formatting** - rustfmt for symlinks
🎯 **Zone awareness** - Links know their context
🎯 **Intent tracking** - Why does this link exist?
🎯 **Verification** - Ask before creating
🎯 **Health integration** - Don't break working system

## Faelight-Link Vision

### Core Features
```rust
// Zone-aware linking
faelight-link install editor-nvim
  → Checks: Zone 0-core (locked?)
  → Asks: "Link 37 files to ~? Proceed?"
  → Creates: Relative links (canonical format)
  → Tracks: Intent (nvim-config-stow)

// Orphan detection
faelight-link audit
  → Finds: Broken links
  → Shows: Where they came from (intent)
  → Asks: "Clean 3 orphans? Proceed?"

// Canonical formatting
faelight-link format
  → Converts: Absolute → Relative
  → Standardizes: All links to canonical form
  → Verifies: No breakage
```

### Ecosystem Integration
- **Daemon-connected**: Query/store link metadata
- **Zone-aware**: Refuse to link into locked zones
- **Intent-tracked**: Every link has purpose
- **Health-checked**: Validate before/after
- **Verified**: Ask permission for every action

### Advantages Over Stow
1. **Understandable**: Show exactly what will happen
2. **Debuggable**: Inspect link metadata
3. **Verifiable**: Confirm before action
4. **Fixable**: Canonical formatter
5. **Safe**: Zone + health awareness

## Implementation Plan

### v0.1.0 - Basic Replacement
- [ ] Parse stow-style packages
- [ ] Create/remove symlinks
- [ ] Detect conflicts
- [ ] Ask before action

### v0.2.0 - Formatting
- [ ] Canonical link format
- [ ] Absolute → Relative conversion
- [ ] Link validation
- [ ] Orphan detection

### v0.3.0 - Ecosystem
- [ ] Zone awareness
- [ ] Intent tracking
- [ ] Daemon integration
- [ ] Health checks

### v1.0.0 - Production
- [ ] Full Stow replacement
- [ ] Migration tool
- [ ] Documentation
- [ ] Testing

## Next Steps
1. Create minimal prototype
2. Test on ONE stow package
3. Compare behavior to Stow
4. Iterate based on findings

---
**No more blood pressure spikes.** 🌲

## 🦀 MAJOR FINDING: lnko (Rust!)
**Source:** https://github.com/luanvil/lnko

### THIS CHANGES EVERYTHING
**lnko is already written in Rust!** We can:
- Study the implementation
- Learn from their approach
- Fork/extend OR build inspired by

### What lnko Does Well

#### 1. **Interactive Conflict Handling**
```bash
lnko stow package
# If conflict exists:
  [B]ackup / [O]verwrite / [S]kip / [Q]uit?
```

#### 2. **Status Overview**
```bash
lnko status
# Shows what's linked where
```

#### 3. **Orphan Cleanup**
```bash
lnko clean
# Removes stale/broken symlinks
```

#### 4. **Stow-compatible**
Works with existing Stow directory structures!

### Where Faelight-Link Would EXTEND lnko

| Feature | lnko | Faelight-Link |
|---------|------|---------------|
| Create symlinks | ✅ | ✅ |
| Interactive conflicts | ✅ | ✅ |
| Orphan cleanup | ✅ | ✅ |
| Status view | ✅ | ✅ Enhanced |
| **Zone awareness** | ❌ | ✅ |
| **Intent tracking** | ❌ | ✅ |
| **Daemon integration** | ❌ | ✅ |
| **Health checks** | ❌ | ✅ |
| **Canonical formatting** | ❌ | ✅ |
| **Verification prompts** | Basic | ✅ Enhanced |
| **Git integration** | ❌ | ✅ |

### Key Architectural Lessons from lnko

#### 1. **Package Structure**
```rust
// They use a Package abstraction
struct Package {
    name: String,
    path: PathBuf,
    files: Vec<PathBuf>,
}
```

#### 2. **Conflict Detection**
They check for:
- Existing files
- Broken symlinks
- Permission issues

#### 3. **User Interaction**
Interactive prompts for conflicts - we can ENHANCE this with:
- Zone context ("This is in 0-core [LOCKED]")
- Intent context ("This serves: nvim-config")
- Health impact ("Breaking this affects: 3 tools")

### Faelight-Link Fork Strategy

**Option 1: Fork lnko**
```bash
# Fork it, add ecosystem features
git clone https://github.com/luanvil/lnko faelight-link
cd faelight-link
# Add: zone awareness, intent tracking, daemon
```

**Option 2: Build Inspired By**
```bash
# Study their approach, build fresh with ecosystem-first design
# Pros: Clean architecture aligned with 0-core
# Cons: More initial work
```

### Recommendation: **FORK + ENHANCE**

**Why:**
1. ✅ Don't reinvent symlink creation
2. ✅ They solved basic problems (conflicts, cleanup)
3. ✅ Already Rust (fast, safe)
4. ✅ We add the ECOSYSTEM layer on top

**What We Add:**
```rust
// lnko's Package + our ecosystem
struct FaelightPackage {
    package: lnko::Package,      // Their work
    zone: Zone,                   // Our addition
    intent: Intent,              // Our addition
    health_check: bool,          // Our addition
    daemon_tracked: bool,        // Our addition
}

impl FaelightPackage {
    fn stow(&self) -> Result<()> {
        // 1. Check zone permissions
        if self.zone.is_locked()? {
            return Err("Cannot modify locked zone");
        }
        
        // 2. Run health check
        if self.health_check {
            run_doctor()?;
        }
        
        // 3. Use lnko's stow logic
        self.package.stow()?;
        
        // 4. Track in daemon
        if self.daemon_tracked {
            daemon::track_links(&self)?;
        }
        
        // 5. Verify health after
        if self.health_check {
            verify_health()?;
        }
        
        Ok(())
    }
}
```

### Implementation Plan (Using lnko as base)

#### Week 1: Fork + Study
```bash
# 1. Fork lnko
git clone https://github.com/luanvil/lnko ~/1-src/faelight-link
cd ~/1-src/faelight-link

# 2. Study codebase
cargo doc --open
cargo test

# 3. Understand their architecture
# Read: src/package.rs, src/stow.rs, src/conflict.rs
```

#### Week 2: Add Zone Layer
```rust
// Add zone detection
use faelight_zone::Zone;

impl Package {
    fn detect_zone(&self) -> Zone {
        Zone::detect(&self.path)
    }
    
    fn verify_zone_permissions(&self) -> Result<()> {
        let zone = self.detect_zone();
        if zone.is_locked() {
            Err("Cannot modify locked zone")
        } else {
            Ok(())
        }
    }
}
```

#### Week 3: Add Intent Layer
```rust
// Track why links exist
struct PackageIntent {
    name: String,
    purpose: String,  // "nvim-config", "shell-setup"
    created: DateTime,
    tools_affected: Vec<String>,
}
```

#### Week 4: Add Daemon Integration
```rust
// Connect to faelight-daemon
impl Package {
    fn register_with_daemon(&self) -> Result<()> {
        let client = DaemonClient::new()?;
        client.track_package(self)?;
        Ok(())
    }
}
```

### Benefits of This Approach

✅ **Don't Reinvent Wheel**
- lnko already handles symlink creation
- Conflict detection works
- Cleanup logic solid

✅ **Focus on Value-Add**
- Zone awareness
- Intent tracking  
- Daemon integration
- Health checks

✅ **Faster to Production**
- Week 1: Working symlink manager
- Week 2-4: Ecosystem features
- vs. Months building from scratch

✅ **Open Source Friendly**
- Credit lnko properly
- Contribute improvements back
- Show ecosystem thinking

### Next Steps

1. **Tonight:** Study lnko codebase
2. **Post-Friday:** Fork and start extending
3. **Week 1:** Basic zone awareness
4. **Week 2:** Intent tracking
5. **Week 3:** Daemon integration
6. **Week 4:** Replace Stow completely

---

**Standing on the shoulders of lnko to build faelight-link.** 🦀🌲
