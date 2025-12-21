# 🗺️ Faelight Forest Development Roadmap

**Current Version:** 3.3.2 - Foundational Intelligence ✅  
**Last Updated:** December 20, 2025  
**Roadmap Version:** 5.0 - Architectural Refinement

---

## 📋 **v3.3.2 PLAN FOR TOMORROW:**

````
## v3.3.3 - GitHub Polish & Screenshots

### Overview
Prepare repository for open source release with professional visual showcase and final polish.

### Sessions
**Session 1 (COMPLETE - 1 hour):**
- ✅ Archived Fish to archive/shell-fish branch
- ✅ Removed shell-fish from main
- ✅ Created MIGRATION.md
- ✅ Updated README.md

**Session 2 (1.5 hours - Tomorrow):**
- Take professional screenshots (5-6 total)
  - Desktop overview (Hyprland + Waybar)
  - Terminal welcome (Zsh + Faelight)
  - dotctl status (package management)
  - dot-doctor (100% health)
  - Yazi file manager
- Optimize images (<500KB each)
- Add to README with descriptions
- Commit & push

**Session 3 (1 hour - Tomorrow):**
- Repository cleanup (temp files, .gitignore check)
- Documentation final pass (Zsh references)
- Version bump to v3.3.3 (using bump-system-version)
- Final commit & verification
- Health check 100%

### Deliverables
- Professional screenshots
- Clean repository structure
- Updated documentation
- Open source ready

### Time Budget
- Total: 3.5 hours
- Completed: 1 hour (Session 1)
- Remaining: 2.5 hours (Sessions 2 & 3)

### Tools Ready
- ✅ grim (screenshot capture)
- ✅ slurp (area selection)

# What to capture:

1. Desktop overview (Hyprland + Waybar)
2. dotctl status (blast radius colors)
3. dot-doctor (100% health)
4. Welcome message (dynamic version)
5. Yazi navigation (teleports)
6. Qutebrowser (Faelight theme)
7. Terminal (Fish + Starship)

# Then add to README:

## 📸 Screenshots: Later (will install tool when needed)

Format: PNG ✅
Aliases Doc: Hybrid (auto-extract + manual enhance) ✅

### Desktop

![Desktop](docs/screenshots/desktop.png)

### System Status

![dotctl status](docs/screenshots/dotctl-status.png)

### Health Check

![dot-doctor](docs/screenshots/health.png)

2. Aliases Documentation 📚
   bash# Create comprehensive aliases reference:
   cat > docs/ALIASES.md << 'EOF'

# 🎯 Alias Reference - 0-Core

> Complete list of all aliases and functions in shell-fish

**Last Updated:** Auto-generated on commit


## v3.3.4 - Quick Wins (Discipline Automation)

### Overview
Low-effort, high-value improvements that automate existing manual discipline and add awareness features.

### Time Estimate
1.5-2 hours total (single session or split into 2 shorter sessions)

### Features

**1. Immutable-by-Default Extension (30 min)**
- Auto-lock after:
  - `git pull` (via sync-0-core)
  - `install.sh` completion
  - `dotctl edit` completion
- Implementation: Add lock-core calls to existing scripts
- Benefit: Tightens existing discipline loop
- Philosophy: You already do this manually, just automate it

**2. Config Aging Report (30 min)**
- Add to dot-doctor output
- Show files not modified in:
  - 30 days (informational)
  - 90 days (review suggested)
  - 1 year (cleanup candidate)
- No enforcement, just awareness
- Helps identify stale configuration
- Example output:
```
  📅 Config Aging:
     Recent (< 30 days):    42 files
     Aging (30-90 days):    8 files
     Stale (90+ days):      3 files
     Ancient (1+ year):     1 file
```

**3. Intentional Defaults Checker (30-45 min)**
- Extend dot-doctor to flag:
  - Files without semantic names (e.g., "temp.txt", "new-file")
  - Unnumbered directories (breaks 0-core philosophy)
  - Packages without .dotmeta
  - Packages without README
- Passive enforcement of discipline
- Catches drift before it spreads
- Example output:
```
  🎯 Intentional Defaults:
     ❌ Found non-semantic filename: temp.txt
     ❌ Unnumbered directory: random-stuff/
     ✅ All packages have .dotmeta
     ⚠️  Package missing README: tools-misc
```

**4. Documentation Updates (15 min)**
- Update THEORY_OF_OPERATION.md
- Add to POLICIES.md
- Commit and push

### Deliverables
- Auto-locking after key operations
- Aging awareness in dot-doctor
- Intentional defaults enforcement
- Updated documentation

### Philosophy Alignment
- ✅ Awareness, not automation
- ✅ Discipline enforcement without force
- ✅ Tightens existing manual practices
- ✅ Low complexity, high value

---

## v3.4.0 - core-diff (Major Feature)

### Overview
Package-aware diff tool that provides forest-level awareness to complement Meld's tree-level inspection. Answers "What should I care about right now?" rather than "What exactly changed?"

### Time Estimate
3-4 hours total (break into 2-3 sessions)

### Concept

**The Problem:**
- Meld is excellent for line-by-line inspection
- But requires knowing WHAT to inspect
- No package-level awareness
- No risk-based grouping
- Manual hunting for changes

**The Solution:**
```
Meld = microscopic (line-by-line inspection)
core-diff = forest view (package-level awareness)

Meld shows trees.
core-diff shows the forest 🌲.
```

### Core Capabilities

**1. Package-Level Awareness**
```bash
$ core-diff since v3.3.2

Changes since v3.3.2:
  [HIGH]   wm-hypr (3 files modified)
  [MEDIUM] shell-zsh (1 file modified)
  [LOW]    theme-gtk (2 files modified)

  Untouched:
  - editor-nvim
  - system
  - all other packages
```

**2. Risk-Based Grouping**
- Reads blast_radius from .dotmeta
- Groups changes by risk level (HIGH/MEDIUM/LOW)
- Shows package context, not just files
- Example:
```bash
  $ core-diff last

  Last commit changes:
  [HIGH] system/
    - /etc/fstab
    - /etc/sudoers

  [MEDIUM] shell-zsh/
    - .config/zsh/.zshrc (aliases)

  [LOW] theme-gtk/
    - .config/gtk-3.0/colors.css
```

**3. Working Tree Awareness**
```bash
$ core-diff working-tree

Uncommitted changes detected:
  - bar-waybar/config.jsonc
  - scripts/safe-update

Packages affected: 2
Risk level: MEDIUM
Action: Review and commit or discard
```

**4. Summary Mode**
```bash
$ core-diff summary

Files changed:     6
Packages touched:  3
Risk level:        MEDIUM
System impact:     None
Git commits:       2 since last tag

High-risk changes: None
Medium-risk:       bar-waybar, shell-zsh
Low-risk:          theme-gtk
```

**5. Meld Integration**
```bash
$ core-diff wm-hypr --open meld
# Opens Meld on EXACTLY the changed files in wm-hypr

Workflow:
1. core-diff since last-release  → "wm-hypr changed"
2. core-diff wm-hypr             → See specific files
3. core-diff wm-hypr --open meld → Inspect with Meld

Awareness → Decision → Inspection
```

### Implementation Phases

**Phase 1: Core Functionality (2 hours)**
- Read git diff output
- Parse package boundaries (stow structure)
- Group changes by package
- Read blast_radius from .dotmeta
- Color-coded output (HIGH=red, MEDIUM=yellow, LOW=blue)
- Basic commands:
  - `core-diff` (show all changes)
  - `core-diff <package>` (show package changes)
  - `core-diff since <ref>` (show changes since tag/commit)

**Phase 2: Advanced Features (1 hour)**
- `--open meld` flag
- `working-tree` mode
- `summary` mode
- Risk level calculation
- Enhanced output formatting

**Phase 3: Polish & Documentation (1 hour)**
- Comprehensive testing
- README section
- THEORY_OF_OPERATION update
- Usage examples
- Integration with existing workflows

### What core-diff Will NOT Do
Philosophy alignment - keep it pure:
- ❌ No auto-opening tools (user decides)
- ❌ No auto-commits (manual control)
- ❌ No git magic (transparent operation)
- ❌ No guessing intent (present facts only)

Only:
- ✅ Read state
- ✅ Present meaning
- ✅ Exit cleanly

### Usage Examples

**Daily workflow:**
```bash
# Morning check
$ core-diff working-tree
# See what's uncommitted

# Before committing
$ core-diff summary
# Understand scope of changes

# Review specific package
$ core-diff wm-hypr
# See details

# Open in Meld if needed
$ core-diff wm-hypr --open meld
# Deep inspection
```

**Release workflow:**
```bash
# What changed since last release?
$ core-diff since v3.3.3

# Get summary for changelog
$ core-diff summary since v3.3.3

# Review high-risk changes
$ core-diff --high-risk
```

### Deliverables
- `core-diff` script in ~/0-core/scripts/
- Package-aware diff capability
- Risk-based grouping
- Meld integration
- Comprehensive documentation
- Updated workflows

### Philosophy Alignment
This perfectly matches existing patterns:
```
dot-doctor    ≠ fixing
safe-update   ≠ blind updating
lock-core     ≠ editing
core-diff     ≠ inspecting

Awareness first. Action second.
```

### Why This Is Transformative
- Answers questions Meld can't without manual setup
- Understands your package structure
- Respects risk-based thinking
- Integrates with existing tools
- Catches drift automatically
- Makes large refactors manageable

**Quote from future you:**
> "Once you have it, you'll never not want it —
> because it matches how you already think."

---

## Future Considerations

### v3.5.0+ Ideas (Not Yet Planned)
- Temporal tracking (incident-free windows)
- Advanced policy enforcement
- Snapshot verification automation
- Predictive warnings based on change patterns

### Long-term Vision
- Make 0-core a reference implementation
- Share philosophy through documentation
- Potential framework for others
- Always: manual control, awareness over automation

---

````

## 🔄 Smart Updates

```fish
safe-update        # Smart system update with recovery
weekly-check       # Prompted weekly maintenance
check-updates      # Check for updates (no install)
```

## 🎮 System Control

```fish
dotctl status      # Show package versions
dotctl bump        # Bump package version
dotctl history     # Show changelog
dotctl health      # Run health check
```

## 📂 Navigation (Numbered Structure)

```fish
core               # cd ~/0-core
src                # cd ~/1-src
work               # cd ~/2-work
keep               # cd ~/3-keep
tmp                # cd ~/9-temp
```

[... continue with all aliases ...]
EOF

# Add auto-generation script:

cat > scripts/generate-aliases-doc << 'EOF'
#!/bin/bash

# Auto-generate ALIASES.md from config.fish

# Extract all aliases and functions

# Parse shell-fish/.config/fish/config.fish

# Output to docs/ALIASES.md with categories

EOF

3. File Polish & Cleanup 🧹
   bash# Areas to review:

1. Remove any test/temp files
   find ~/0-core -name "_.backup" -o -name "_.tmp" -o -name "\*~"

1. Ensure all scripts have proper headers

   # Check scripts/ for consistent format

1. Check for TODOs/FIXMEs
   grep -r "TODO\|FIXME\|XXX" ~/0-core --exclude-dir=.git

1. Verify all .dotmeta files complete

   # Ensure all packages have .dotmeta

1. Check documentation links

   # Verify all internal links work

1. Remove personal info (final check)
   grep -r "christian\|@tuta\|personal" ~/0-core --exclude-dir=.git

1. Gitleaks Check & Update 🔐
   bash# Check current gitleaks version:
   gitleaks version

# Update if needed:

yay -S gitleaks

# Test current config:

cd ~/0-core
gitleaks detect --no-git -v

# Review .pre-commit-config.yaml:

cat > .pre-commit-config.yaml << 'EOF'
repos:

- repo: https://github.com/gitleaks/gitleaks
  rev: v8.21.2 # Check for latest version
  hooks: - id: gitleaks
  EOF

# Test the hook:

git add test-file
git commit -m "test" # Should scan

5. Git Hooks Review 🪝
   bash# Check current hooks:
   ls -la hooks/

# Update pre-commit hook if needed:

cat > hooks/pre-commit << 'EOF'
#!/bin/bash

# Enhanced pre-commit hook

echo "🔍 Running pre-commit checks..."

# 1. Gitleaks scan

echo "Scanning for secrets..."
gitleaks protect --staged -v

# 2. Check for large files

echo "Checking file sizes..."
git diff --cached --name-only | while read file; do
if [ -f "$file" ]; then
size=$(wc -c < "$file")
if [ $size -gt 1048576 ]; then # 1MB
echo "❌ File too large: $file ($(($size / 1024))KB)"
exit 1
fi
fi
done

# 3. Check for personal info (basic)

echo "Checking for personal info..."
if git diff --cached | grep -E "@tuta\.com|personal|private" > /dev/null; then
echo "⚠️ Warning: Potential personal info detected"
read -p "Continue? (y/N): " confirm
[ "$confirm" != "y" ] && exit 1
fi

echo "✅ Pre-commit checks passed!"
EOF

chmod +x hooks/pre-commit

```

---

## 🎯 **v3.3.3 COMPLETE PLAN (2-3 HOURS):**

### **Session Breakdown:**
```

PART 1: Screenshots & Visual (45 min)
├── Take 7-8 key screenshots
├── Optimize images (compress)
├── Add to docs/screenshots/
├── Update README with images
└── Test image links work

PART 2: Aliases Documentation (30 min)
├── Create docs/ALIASES.md
├── Extract all aliases from config.fish
├── Organize by category
├── Add usage examples
└── Link from README

PART 3: File Polish (30 min)
├── Remove temp files
├── Check for TODOs
├── Verify all .dotmeta complete
├── Check documentation links
└── Final personal info audit

PART 4: Security Updates (30 min)
├── Update gitleaks
├── Test gitleaks config
├── Review/enhance git hooks
├── Test hooks thoroughly
└── Document hook behavior

PART 5: Final Review (15-30 min)
├── Run through README
├── Test all links
├── Verify screenshots display
├── Check repo cleanliness
└── Ready for open source!

TOTAL: 2.5-3 hours

```

---

## 📅 **REVISED TIMELINE:**
```

THIS WEEK:
Day 1-2: Test v3.3.0 features
Day 3: v3.3.1 (Blast Radius) - 3-4 hrs
Day 4-5: Use v3.3.1, prepare screenshots

NEXT WEEK:
Day 1: v3.3.2 (GitHub Polish) - 2.5-3 hrs
├── Screenshots
├── Aliases doc
├── File polish
├── Security updates
└── Final review

Day 2: Final testing
Day 3: Privacy audit (one last check)
Day 4: OPEN SOURCE! 🌍

```

---

## ✅ **IMMEDIATE TODO LIST:**

## 📋 **v3.3.3 PLAN FOR TOMORROW:**
```

```
1. 📸 Take screenshots
2. 📚 Create ALIASES.md
3. 🧹 Polish all files
4. 🔐 Update gitleaks & hooks
5. ✨ Final review

```

### **For Open Source (After v3.3.4):**

```

1. 🔍 Final privacy audit
2. 📝 Repository description
3. 🏷️ Add GitHub topics
4. 🌍 Make public
5. 📢 Share on Reddit/HN

💡 ALIASES.MD STRUCTURE PREVIEW:
markdown# 🎯 Alias Reference

## Categories

- [Core Protection](#core-protection)
- [Smart Updates](#smart-updates)
- [System Control](#system-control)
- [Navigation](#navigation)
- [Package Management](#package-management)
- [Git Shortcuts](#git-shortcuts)
- [File Management](#file-management)
- [Development](#development)
- [Utilities](#utilities)

## Change Log

**v3.3.0:** Added dotctl, sync-0-core
**v3.2.1:** Added Qutebrowser aliases
**v3.2.0:** Added safe-update, weekly-check
**v3.1.0:** Added numbered navigation

---

## Core Protection

### lock-core

**Description:** Lock 0-core (filesystem immutable)
**Usage:** `lock-core`
**Added:** v3.1.0

[... detailed for each alias ...]

v3.4.0 (2-3 hrs): Policy Enforcement

- Safety gates
- Requirement checks
- --ack-critical overrides
- Basic temporal tracking

v3.5.0 (3-4 hrs): Temporal Intelligence

- Stability metrics
- Entropy tracking
- Predictive warnings
- Advanced safety analysis

v4.0.0: The Research Paper

- Academic documentation
- Published system design
- Community presentation

---

**Current Status:** Version 3.3.2 Complete ✅
**Vision:** Infrastructure as Poetry 🌲✨

---

_Last Updated: December 20, 2025_
_Roadmap Version: 5.0 - Architectural Refinement_

```

```

```
