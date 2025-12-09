# 🗺️ Faelight Forest Development Roadmap

**Current Version:** 2.8.5 - Foundational Intelligence ✅  
**Last Updated:** December 09, 2025  
**Roadmap Version:** 5.0 - Architectural Refinement

---

**Next:** v2.8.6 enhances diagnostic tooling!

---

## 🔧 Version 2.8.6 - Advanced Dotfile Intelligence Suite

### Goals: Enhance Existing Tooling

**Workflow:**
````
Enhance dot-doctor → Enhance keyscan → Add New Tools → Automate → Document
````

---

### Enhanced dot-doctor

**Add Advanced Checks:**

**Tasks:**
- [ ] Font dependency verification:
````fish
  function _check_fonts
      # Check for required fonts
      fc-list | grep -i "Hack Nerd Font" > /dev/null && echo "✅ Hack Nerd Font" || echo "❌ Hack Nerd Font missing"
      fc-list | grep -i "Inter" > /dev/null && echo "✅ Inter" || echo "⚠️  Inter missing"
  end
````
- [ ] Orphaned file detection:
````fish
  function _check_orphans
      # Find configs without symlinks
      # Find old backup files
      # Report orphaned directories
  end
````
- [ ] Theme consistency checker:
````fish
  function _check_theme_consistency
      # Verify all theme files exist
      # Check dark theme complete
      # Check light theme complete
      # Check wallpaper theme if generated
  end
````
- [ ] Keybinding integration:
````fish
  function _check_keybindings
      # Call keyscan
      # Report if conflicts found
  end
````
- [ ] Git status check:
````fish
  function _check_git_status
      cd ~/dotfiles
      if test (git status --porcelain | wc -l) -gt 0
          echo "⚠️  Uncommitted changes in dotfiles"
      end
  end
````

**Add --fix Flag:**

**Tasks:**
- [ ] Implement auto-repair mode
- [ ] Recreate broken symlinks
- [ ] Install missing fonts (with confirmation)
- [ ] Remove orphaned files (with confirmation)
- [ ] Document what --fix does

**Add --report Flag:**

**Tasks:**
- [ ] Generate HTML health report
- [ ] Include graphs (using plotly or matplotlib)
- [ ] Email option (using mailx or similar)
- [ ] Save to ~/dotfiles/reports/

---

### Enhanced keyscan

**Expand Parsing:**

**Tasks:**
- [ ] Parse Waybar on-click handlers:
````fish
  function _parse_waybar_bindings
      # Read waybar config.jsonc
      # Extract on-click, on-scroll handlers
      # Report any that might conflict
  end
````
- [ ] Parse Kitty keybinds:
````fish
  function _parse_kitty_bindings
      # Read kitty.conf
      # Extract map bindings
      # Check for conflicts with Hyprland
  end
````
- [ ] Parse Fish key bindings (if any):
````fish
  function _parse_fish_bindings
      # Check for bind commands in config.fish
  end
````

**Beautiful Output:**

**Tasks:**
- [ ] Add color-coded categories (use set_color)
- [ ] Add ASCII art boxes (using unicode box drawing)
- [ ] Export to markdown table:
````fish
  function keyscan --argument --export
      # Generate KEYBINDINGS.md
  end
````

**Danger Zone Warnings:**

**Tasks:**
- [ ] Detect terminal vs system conflicts:
````
  ⚠️  DANGER: SUPER+C conflicts between:
     - Hyprland: VSCode
     - Kitty: Copy (Ctrl+Shift+C recommended instead)
````
- [ ] Detect common application conflicts
- [ ] Suggest alternatives

**Auto-Generate Documentation:**

**Tasks:**
- [ ] Add `keyscan --export` command
- [ ] Generate `docs/KEYBINDINGS.md` automatically
- [ ] Include descriptions from bindd
- [ ] Group by category
- [ ] Keep updated

---

### Additional Tools

**1. dot-diff - Visual Diff:**
````fish
function dot-diff --argument package
    # Compare current config vs dotfiles version
    if command -v meld &> /dev/null
        meld ~/.config/$package ~/dotfiles/$package/.config/$package
    else
        diff -r ~/.config/$package ~/dotfiles/$package/.config/$package
    end
end
````

**2. dot-benchmark - Performance Profiling:**
````fish
function dot-benchmark
    echo "📊 Performance Benchmark"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Fish startup time
    set -l fish_time (time fish -c exit 2>&1 | grep real | awk '{print $2}')
    echo "Fish startup: $fish_time"
    
    # Hyprland startup (from logs)
    echo "Hyprland startup: (check logs)"
    
    # Plugin load times
    echo "Analyzing plugins..."
end
````

**3. dot-update - Safe Update Workflow:**
````fish
function dot-update
    echo "🔄 Safe Dotfile Update Workflow"
    
    # 1. Snapshot
    dot-snapshot "Pre-update $(date)"
    
    # 2. Run dot-doctor
    dot-doctor || return 1
    
    # 3. Update system
    sudo pacman -Syu
    
    # 4. Run dot-doctor again
    dot-doctor
    
    # 5. Update dotfiles
    cd ~/dotfiles
    git pull
    
    # 6. Final check
    dot-doctor
end
````

**4. theme-validate - Validate Generated Themes:**
````fish
function theme-validate
    echo "🎨 Theme Validation"
    
    # Check contrast ratios using faelight-theme
    faelight-theme validate ~/.cache/faelight-themes/latest.json
    
    # Verify all configs exist
    # Run syntax checks
end
````

**Tasks:**
- [ ] Implement all 4 tools
- [ ] Write Fish functions
- [ ] Test each tool
- [ ] Add to config.fish
- [ ] Document in TOOLING.md

---

### Integration & Automation

**Pre-commit Hook (Optional):**

**File:** `~/dotfiles/.git/hooks/pre-commit`
````bash
#!/bin/bash
# Validate dotfiles before commit

echo "Running dot-doctor..."
fish -c dot-doctor || exit 1

echo "Running keyscan..."
fish -c "keyscan | grep 'Conflicts: 0'" || exit 1

echo "✅ All checks passed"
````

**Tasks:**
- [ ] Create pre-commit hook
- [ ] Make executable
- [ ] Test by making bad commit
- [ ] Document how to enable/disable

**Weekly Health Check Reminder:**

**Tasks:**
- [ ] Add to cron (optional):
````cron
  0 9 * * SUN /home/christian/dotfiles/scripts/weekly-health-check.sh
````
- [ ] Create weekly-health-check.sh script
- [ ] Email/notify if issues found
- [ ] Document in TOOLING.md

**Health Command Integration:**

**Tasks:**
- [ ] Update `health` alias to include dot-doctor summary
- [ ] Show last health check time
- [ ] Show any warnings

---

### Documentation

**Tasks:**
- [ ] Update TOOLING.md comprehensively:
  - All commands explained with examples
  - Usage patterns
  - Troubleshooting section
- [ ] Update COMPLETE_GUIDE.md:
  - Add "Advanced Tooling" section
  - Document weekly maintenance routine with new tools
- [ ] Create troubleshooting flowcharts (Mermaid diagrams)
- [ ] Document all aliases and functions
- [ ] Add to CHANGELOG.md

---

**Estimated Time:** 4-5 hours  
**Dependencies:** v2.8.5 complete ✅  
**Priority:** Medium (polish after theme engine)  
**Deliverables:**
- ✅ Enhanced dot-doctor with --fix and --report
- ✅ Enhanced keyscan with exports and warnings
- ✅ 4 new diagnostic tools
- ✅ Optional automation (pre-commit, cron)
- ✅ Complete professional tooling suite
- ✅ Comprehensive documentation

**Next:** v2.9 - Security & Backup Infrastructure!

---

## 📊 Version 2.8 - Summary

**Complete v2.8 Structure:**
````
v2.8.0 - Foundational Intelligence (COMPLETE ✅)
├─ dot-doctor (basic health monitoring)
├─ keyscan (conflict detection)
└─ Safety baseline established

v2.8.1 - Theme Engine Foundation (3-4 hours)
├─ Standalone Python package structure
├─ Research & design decisions
└─ Ready for implementation

v2.8.2 - Color Extraction (4-5 hours)
├─ Extraction algorithm
├─ Palette generation
├─ WCAG validation
└─ CLI implementation

v2.8.3 - Template System (4-5 hours)
├─ Jinja2 templates
├─ Rendering engine
└─ Complete standalone theme engine

v2.8.4 - FCM Integration (5-6 hours)
├─ FCM consumes theme engine
├─ Atomic wallpaper packages
├─ Application integration
└─ Theme system operational

v2.8.5 - Polish & Features (5-6 hours)
├─ Caching system
├─ Preview mode
├─ Presets (vibrant, muted, etc.)
├─ Gallery
└─ Full automation

v2.8.6 - Advanced Tooling (4-5 hours)
├─ Enhanced dot-doctor
├─ Enhanced keyscan
├─ Additional tools
└─ Professional polish
````

**Total Time:** 28-34 hours (spread over weeks/months)

**Key Architectural Decisions:**
- ✅ Theme engine is STANDALONE (reusable, testable)
- ✅ Clean CLI API (FCM is consumer)
- ✅ Separation of concerns (theme gen vs config management)

**After v2.8 Complete:**
Your system will generate beautiful themes from ANY wallpaper, with professional tooling to monitor health and prevent conflicts! 🎨🛡️🌲

---

## 🔐 Version 2.9 - Security & Backup Infrastructure

### Goals: Advanced Security + Complete Backup Strategy

**Total Estimated Time:** 6-8 hours

*(Keep existing v2.9 structure - no changes needed)*

---

## ⚛️ Version 3.0 - Faelight Config Manager Foundation

### Overview: Atomic Packages + Declarative Configuration

**CRITICAL:** Atomic package restructure is **MANDATORY** for v3.0. This is the foundation for everything else.

**Core Architecture:**
````
Atomic Packages (base + theme + machine)
         ↓
Package Metadata (.dotfile-meta.yaml)
         ↓
Python Dependency Resolver (networkx)
         ↓
Declarative Manifest (manifest.yaml)
         ↓
Git Tag Snapshots (not YAML files)
````

**Total Estimated Time:** 20-25 hours (spread over 2-3 weeks)

---

### Phase 1: Atomic Package Restructure (8-10 hours) **MANDATORY**

**Goal:** Break monolithic packages into composable units

**This is NON-NEGOTIABLE - everything else depends on this!**

**Current Structure (Monolithic):**
````
dotfiles/
├── fish/           # Config + aliases + functions + theme (all mixed)
├── hypr/           # Config + theme + machine-specific (all mixed)
└── waybar/         # Structure + styling + modules (all mixed)
````

**Target Structure (Atomic):**
````
dotfiles/
├── fish-base/              # Core config only
├── fish-aliases/           # Just aliases
├── fish-functions/         # Just functions  
├── fish-theme-dark/        # Dark prompt colors
├── fish-theme-light/       # Light prompt colors
├── fish-theme-wallpaper/   # Generated theme (from v2.8)

├── hypr-base/              # Core Hyprland config
├── hypr-laptop/            # Laptop-specific (battery, backlight)
├── hypr-theme-dark/        # Dark window colors
├── hypr-theme-light/       # Light window colors
├── hypr-theme-wallpaper/   # Generated colors (from v2.8)

├── waybar-base/            # Structure + modules
├── waybar-laptop/          # Laptop modules (battery, backlight)
├── waybar-theme-dark/      # Dark CSS
├── waybar-theme-light/     # Light CSS
├── waybar-theme-wallpaper/ # Generated CSS (from v2.8)

├── kitty-base/             # Core Kitty config
├── kitty-theme-dark/       # Dark terminal.conf
├── kitty-theme-light/      # Light terminal.conf
├── kitty-theme-wallpaper/  # Generated theme (from v2.8)

├── nvim/                   # Keep monolithic (LazyVim manages itself)
├── yazi/                   # Keep monolithic (simple)
├── mako/                   # Keep monolithic
└── gtk/                    # Keep monolithic
````

**Why Atomic Packages:**
1. **Composability** - Mix and match (base + dark + laptop)
2. **Theme Switching** - Just swap theme packages
3. **Machine Profiles** - Different laptop/desktop configs
4. **Testing** - Test packages in isolation
5. **Clarity** - Clean separation of concerns

---

#### Step 1A: Plan the Split (1 hour)

**Tasks:**
- [ ] Document current fish package contents
- [ ] Document current hypr package contents
- [ ] Document current waybar package contents
- [ ] Document current kitty package contents
- [ ] Create splitting strategy document
- [ ] Identify which files go in which atomic package
- [ ] Plan migration approach (all at once vs incremental)

---

#### Step 1B: Split Fish Package (2-3 hours)

**Tasks:**
- [ ] Create new atomic package directories:
````bash
  mkdir -p fish-base/.config/fish
  mkdir -p fish-aliases/.config/fish/conf.d
  mkdir -p fish-functions/.config/fish/functions
  mkdir -p fish-theme-dark/.config/fish/conf.d
  mkdir -p fish-theme-light/.config/fish/conf.d
````
- [ ] Move core config.fish to fish-base/
  - Remove aliases (go to fish-aliases)
  - Remove functions (go to fish-functions)
  - Remove theme colors (go to fish-theme-*)
  - Keep only core settings
- [ ] Move all aliases to fish-aliases/
  - Create aliases.fish
  - Source from conf.d/
- [ ] Move all functions to fish-functions/
  - One file per function
  - Place in functions/ directory
- [ ] Extract theme colors to fish-theme-dark/ and fish-theme-light/
  - Create theme-colors.fish for each
  - Include prompt customizations
- [ ] Test each atomic package individually:
````bash
  cd ~/dotfiles
  stow fish-base
  stow fish-aliases
  stow fish-functions
  stow fish-theme-dark
  # Test fish loads without errors
````
- [ ] Verify all combinations work
- [ ] Update .stow-local-ignore if needed

---

#### Step 1C: Split Hypr Package (2-3 hours)

**Tasks:**
- [ ] Create atomic package directories:
````bash
  mkdir -p hypr-base/.config/hypr
  mkdir -p hypr-laptop/.config/hypr
  mkdir -p hypr-theme-dark/.config/hypr
  mkdir -p hypr-theme-light/.config/hypr
````
- [ ] Split hyprland.conf into:
  - hypr-base/ - Core config (no colors, no machine-specific)
  - hypr-laptop/ - Battery, backlight, laptop-specific settings
  - hypr-theme-dark/ - colors.conf with dark colors
  - hypr-theme-light/ - colors.conf with light colors
- [ ] Ensure hypr-base sources colors.conf:
````conf
  source = ~/.config/hypr/colors.conf
````
- [ ] Test each package:
````bash
  stow hypr-base hypr-laptop hypr-theme-dark
  hyprctl reload
  # Verify no errors
````
- [ ] Test all combinations:
  - base + laptop + dark
  - base + laptop + light
  - base + laptop + wallpaper (from v2.8)

---

#### Step 1D: Split Waybar Package (2-3 hours)

**Tasks:**
- [ ] Create atomic package directories:
````bash
  mkdir -p waybar-base/.config/waybar
  mkdir -p waybar-laptop/.config/waybar
  mkdir -p waybar-theme-dark/.config/waybar
  mkdir -p waybar-theme-light/.config/waybar
````
- [ ] Split into:
  - waybar-base/ - config.jsonc structure + base modules
  - waybar-laptop/ - Battery, backlight, network modules
  - waybar-theme-dark/ - style-dark.css
  - waybar-theme-light/ - style-light.css
- [ ] Ensure config.jsonc imports style.css:
````json
  "style": "~/.config/waybar/style.css"
````
- [ ] Test combinations
- [ ] Verify bar displays correctly

---

#### Step 1E: Split Kitty Package (1-2 hours)

**Tasks:**
- [ ] Create atomic package directories:
````bash
  mkdir -p kitty-base/.config/kitty
  mkdir -p kitty-theme-dark/.config/kitty
  mkdir -p kitty-theme-light/.config/kitty
````
- [ ] Split kitty.conf:
  - kitty-base/ - Core config (no colors)
  - kitty-theme-dark/ - current-theme.conf dark
  - kitty-theme-light/ - current-theme.conf light
- [ ] Ensure kitty.conf includes theme:
````conf
  include current-theme.conf
````
- [ ] Test all combinations

---

#### Step 1F: Cleanup & Verification (1 hour)

**Tasks:**
- [ ] Remove old monolithic packages:
````bash
  # Unstow old packages
  stow -D fish hypr waybar kitty
  
  # Archive old structure (don't delete yet!)
  mv fish fish-OLD-BACKUP
  mv hypr hypr-OLD-BACKUP
  mv waybar waybar-OLD-BACKUP
  mv kitty kitty-OLD-BACKUP
````
- [ ] Test full system with atomic packages:
````bash
  stow fish-base fish-aliases fish-functions fish-theme-dark
  stow hypr-base hypr-laptop hypr-theme-dark
  stow waybar-base waybar-laptop waybar-theme-dark
  stow kitty-base kitty-theme-dark
  stow nvim yazi mako gtk
````
- [ ] Verify everything works
- [ ] Run dot-doctor
- [ ] Run keyscan
- [ ] Test for 1 week before deleting backups
- [ ] Once stable, delete -OLD-BACKUP directories

---

**Deliverables:**
- ✅ Fully atomic package structure
- ✅ Clean separation (base / theme / machine)
- ✅ All combinations tested
- ✅ Ready for metadata and dependency system

**This restructure is the FOUNDATION - don't skip it!**

---

### Phase 2: Package Metadata System (6-8 hours)

**Goal:** Add intelligence with metadata files

**Metadata Format:**
````yaml
# waybar-theme-dark/.dotfile-meta.yaml
name: waybar-theme-dark
version: 3.0.0
description: "Dark theme for Waybar with Faelight Forest colors"
author: Christian
created: 2025-12-03
updated: 2025-12-03

type: theme
category: waybar

depends:
  - waybar-base

conflicts:
  - waybar-theme-light
  - waybar-theme-wallpaper

provides:
  - waybar-theme-colors

files:
  generated: []
  handwritten:
    - .config/waybar/style-dark.css

required-binaries: []

required-fonts:
  - Hack Nerd Font
  - Inter

tags:
  - theme
  - dark
  - waybar
  - faelight-forest

compatibility:
  hyprland: ">=0.40.0"
  waybar: ">=0.10.0"
````

**Tasks:**
- [ ] Create `.dotfile-meta.yaml` template
- [ ] Write metadata for ALL atomic packages (30-40 files!)
  - [ ] All fish-* packages (6 packages)
  - [ ] All hypr-* packages (5 packages)
  - [ ] All waybar-* packages (5 packages)
  - [ ] All kitty-* packages (4 packages)
  - [ ] Remaining monolithic packages (nvim, yazi, etc.)
- [ ] Create validation script (YAML syntax checker)
- [ ] Create `dot-info` command (query metadata)
- [ ] Create `dot-search` command (find packages)
- [ ] Create `dot-list` command (list installed)
- [ ] Document metadata schema in FRAMEWORK.md

**Deliverables:**
- ✅ Every package has metadata
- ✅ Query/search tools working
- ✅ Foundation for dependencies

---

### Phase 3: Python Dependency Resolver (6-8 hours)

**Goal:** Use Python + networkx for graph algorithms

**Why Python:**
- ✅ networkx library (proven graph algorithms)
- ✅ Topological sort built-in
- ✅ Cycle detection built-in
- ✅ Fast and reliable
- ✅ Don't reimplement algorithms in Fish!

**File:** `~/dotfiles/scripts/resolve-deps.py`
````python
#!/usr/bin/env python3
"""
Faelight Config Manager - Dependency Resolver
Uses networkx for graph algorithms.
"""

import sys
import yaml
import networkx as nx
from pathlib import Path

def build_dependency_graph(dotfiles_dir: Path) -> nx.DiGraph:
    """Build directed graph of package dependencies."""
    G = nx.DiGraph()
    
    # Find all packages with metadata
    for meta_file in dotfiles_dir.glob("**/.dotfile-meta.yaml"):
        with open(meta_file) as f:
            meta = yaml.safe_load(f)
        
        pkg_name = meta['name']
        G.add_node(pkg_name, **meta)
        
        # Add dependency edges
        for dep in meta.get('depends', []):
            G.add_edge(dep, pkg_name)  # dep → pkg_name
    
    return G

def check_cycles(G: nx.DiGraph) -> list:
    """Check for circular dependencies."""
    try:
        cycles = list(nx.simple_cycles(G))
        return cycles
    except:
        return []

def resolve_dependencies(G: nx.DiGraph, package: str) -> list[str]:
    """
    Resolve dependencies for a package.
    
    Returns:
        List of packages in installation order (deps first)
    """
    if package not in G:
        raise ValueError(f"Package not found: {package}")
    
    # Get all ancestors (dependencies)
    deps = nx.ancestors(G, package)
    deps.add(package)  # Include package itself
    
    # Topological sort for install order
    subgraph = G.subgraph(deps)
    install_order = list(nx.topological_sort(subgraph))
    
    return install_order

def check_conflicts(G: nx.DiGraph, packages: list[str]) -> list[tuple]:
    """
    Check if packages conflict.
    
    Returns:
        List of (pkg1, pkg2) conflict pairs
    """
    conflicts = []
    
    for pkg in packages:
        if pkg not in G:
            continue
        
        pkg_conflicts = G.nodes[pkg].get('conflicts', [])
        
        for other in packages:
            if other in pkg_conflicts:
                conflicts.append((pkg, other))
    
    return conflicts

def main():
    if len(sys.argv) < 2:
        print("Usage: resolve-deps.py <package>", file=sys.stderr)
        sys.exit(1)
    
    package = sys.argv[1]
    dotfiles_dir = Path.home() / "dotfiles"
    
    # Build graph
    G = build_dependency_graph(dotfiles_dir)
    
    # Check for cycles
    cycles = check_cycles(G)
    if cycles:
        print(f"ERROR: Circular dependencies detected:", file=sys.stderr)
        for cycle in cycles:
            print(f"  {' → '.join(cycle)}", file=sys.stderr)
        sys.exit(1)
    
    # Resolve dependencies
    try:
        install_order = resolve_dependencies(G, package)
        
        # Output (one per line for Fish to consume)
        for pkg in install_order:
            print(pkg)
    
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
````

**Tasks:**
- [ ] Create resolve-deps.py script
- [ ] Install dependencies: `pip install networkx pyyaml`
- [ ] Test with sample metadata
- [ ] Handle edge cases (missing deps, cycles)
- [ ] Write unit tests (pytest)
- [ ] Document in FRAMEWORK.md

**Fish Integration:**
````fish
function dot-install --argument package
    echo "📦 Installing: $package"
    
    # Use Python to resolve deps
    set -l install_order (python3 ~/dotfiles/scripts/resolve-deps.py $package)
    
    if test $status -ne 0
        echo "❌ Dependency resolution failed"
        return 1
    end
    
    echo "Install order:"
    for pkg in $install_order
        echo "  - $pkg"
    end
    
    # Install each package
    for pkg in $install_order
        cd ~/dotfiles
        stow $pkg
        echo "✅ Installed: $pkg"
    end
    
    # Register active packages
    # (Add to manifest or tracking file)
end
````

**Tasks:**
- [ ] Create Fish wrapper functions
- [ ] Implement dot-install
- [ ] Implement dot-remove (check dependents)
- [ ] Implement dot-deps (show tree)
- [ ] Test complex dependency chains
- [ ] Document in FRAMEWORK.md

**Deliverables:**
- ✅ Python dependency resolver working
- ✅ Automatic installation order
- ✅ Cycle detection
- ✅ Conflict checking
- ✅ Fish integration complete

---

### Phase 4: Manifest System (6-7 hours)

**Goal:** Declarative configuration

**Manifest Format:**
````yaml
# dotfiles/manifest.yaml

profile: omarchy-laptop

active_packages:
  # Base layers
  - fish-base
  - fish-aliases
  - fish-functions
  - hypr-base
  - waybar-base
  - kitty-base
  - nvim
  - yazi
  - mako
  - gtk
  
  # Theme layer
  - fish-theme-dark
  - hypr-theme-dark
  - waybar-theme-dark
  - kitty-theme-dark
  
  # Machine-specific
  - hypr-laptop
  - waybar-laptop

environment:
  EDITOR: nvim
  BROWSER: brave
  TERMINAL: kitty

features:
  mullvad-vpn: true
  battery-management: true
  brightness-control: true

metadata:
  hostname: omarchy
  type: laptop
  last_applied: 2025-12-03T20:00:00Z
  version: 3.0.0
````

**Tasks:**
- [ ] Create manifest.yaml format spec
- [ ] Create omarchy-laptop profile
- [ ] Implement dot-apply:
````fish
  function dot-apply
      # Read manifest.yaml
      # Resolve all dependencies (via Python)
      # Check conflicts
      # Unstow conflicting packages
      # Install packages in order
      # Set environment variables
      # Update last_applied timestamp
  end
````
- [ ] Implement dot-profile (switch profiles)
- [ ] Create profile templates (laptop, desktop, server)
- [ ] Test profile switching
- [ ] Document in FRAMEWORK.md

**Deliverables:**
- ✅ Declarative manifest working
- ✅ One-command apply
- ✅ Profile system

---

### Phase 5: Integration & Testing (3-4 hours)

**Tasks:**
- [ ] Integrate with theme-switch.sh
- [ ] Integrate with theme-from-wallpaper.sh
- [ ] Add Fish aliases
- [ ] Test everything together
- [ ] Write FRAMEWORK.md
- [ ] Update COMPLETE_GUIDE.md
- [ ] Add to CHANGELOG.md

**Deliverables:**
- ✅ Fully integrated FCM
- ✅ Complete documentation
- ✅ Ready for daily use

---

**Version 3.0 Summary:**

**What You Built:**
- ⚛️ Atomic package system (composable!)
- 📋 Package metadata (intelligent!)
- 🐍 Python dependency resolver (proven algorithms!)
- 📝 Declarative manifests (state what you want!)
- 🖥️ Machine profiles (laptop/desktop/server!)

**Impact:**
Dotfiles are now **managed infrastructure** with intelligence, automation, and safety!

---

## 🏗️ Version 3.5 - Advanced Configuration Management

### Phase 1: Git Tag Snapshots (5-6 hours)

**Goal:** Use Git instead of custom YAML

**Why Git Tags:**
- ✅ Proven technology
- ✅ Smaller footprint (no duplicate YAML)
- ✅ Better tooling (git tag, git show)
- ✅ Remote backup (push tags to GitHub)

**Implementation:**
````fish
function dot-snapshot --argument description
    set -l timestamp (date +%Y-%m-%d-%H%M)
    set -l tag "faelight-$timestamp"
    
    cd ~/dotfiles
    
    # Commit current state
    git add -A
    git commit -m "Snapshot: $description" --allow-empty
    
    # Create Git tag
    git tag -a "$tag" -m "$description"
    
    # Optional: lightweight metadata file
    echo "timestamp: $(date -Iseconds)" > .dotfile-snapshots/$tag.meta
    echo "description: $description" >> .dotfile-snapshots/$tag.meta
    echo "active_packages:" >> .dotfile-snapshots/$tag.meta
    # ... add package list
    
    echo "✅ Snapshot created: $tag"
    echo "💾 Git tag: git show $tag"
end

function dot-snapshots
    echo "📸 Available Snapshots (Git Tags):"
    git tag -l "faelight-*" --sort=-creatordate | head -20
end

function dot-rollback --argument tag
    cd ~/dotfiles
    
    echo "⏪ Rolling back to: $tag"
    
    # Checkout tag
    git checkout "$tag"
    
    # Reapply configuration
    dot-apply
    
    echo "✅ Rollback complete"
end
````

**Tasks:**
- [ ] Implement snapshot functions
- [ ] Test snapshot creation
- [ ] Test rollback
- [ ] Optional: Add lightweight .meta files for quick queries
- [ ] Push tags to remote: `git push --tags`
- [ ] Document in FRAMEWORK.md

**Deliverables:**
- ✅ Git-based snapshots
- ✅ Easy rollback
- ✅ Smaller, simpler than YAML files

*(Keep rest of v3.5 as planned)*

---

## 🔥 Version 4.0 - The Phoenix Framework

*(Keep as planned - no changes)*

---

## 🎯 Complete Development Timeline

### Immediate (Next 2-4 Weeks)
- **v2.8.1-2.8.6** - Theme Intelligence Engine (28-34 hours)

### Short-term (Next 2-3 Months)
- **v2.9** - Security & Backup (6-8 hours)
- **v3.0** - FCM Foundation (20-25 hours)

### Medium-term (3-6 Months)
- **v3.5** - Advanced Management (15-20 hours)
- **v4.0** - Phoenix Framework (10-15 hours)

**Total Investment:** 80-100 hours over 6 months  
**Result:** Professional configuration management framework

---

## 🌲 Faelight Forest Principles

1. **Separation of Concerns** - Theme engine standalone
2. **Right Tool for Job** - Python for algorithms, Fish for UX
3. **Composability** - Atomic packages are MANDATORY
4. **Declarative** - State what you want (manifest.yaml)
5. **Safety** - Git tags for snapshots, validation before apply
6. **Intelligence** - Dependency resolution, conflict detection
7. **Beauty** - Not just functional, delightful
8. **Excellence** - Professional quality

---

**Current Status:** Version 2.8.0 Complete ✅  
**Next Action:** v2.8.1 - Theme Engine Foundation  
**Vision:** Infrastructure as Poetry 🌲✨

---

*Last Updated: December 03, 2025*  
*Roadmap Version: 5.0 - Architectural Refinement*
