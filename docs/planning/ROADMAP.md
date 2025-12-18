# 🗺️ Faelight Forest Development Roadmap

**Current Version:** 3.3 - Foundational Intelligence ✅  
**Last Updated:** December 17, 2025  
**Roadmap Version:** 5.0 - Architectural Refinement

---

## 📋 **v3.3.2 PLAN FOR TOMORROW:**

````
SESSION TIME: 1.5-2 hours (fresh, focused)

PHASE 1: Convert Config (45 min)
├── Environment variables (5 min)
├── All aliases (15 min - mostly copy/paste)
├── Functions (25 min - careful conversion)
   ├── ya (yazi cd-on-quit)
   ├── weekly-check
   ├── update-check
   ├── notes/keepass launchers
   └── fish_command_not_found equivalent

PHASE 2: Test in Zsh (30 min)
├── Launch zsh (don't switch default yet)
├── Test aliases
├── Test functions
├── Test autosuggestions
├── Test theme/colors
├── Verify everything works

PHASE 3: Switch & Document (15 min)
├── chsh -s /usr/bin/zsh
├── Update docs
├── Version bump to v3.3.2
├── Commit & push

TOTAL: ~1.5-2 hours, DONE!

📋 v3.3.3 - GITHUB POLISH PLAN:

1. Screenshots (HIGH PRIORITY) 📸
   bash# Create screenshots directory:
   mkdir -p docs/screenshots

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
**Package:** shell-fish v3.3.0

---

## 🔒 Core Protection

```fish
lock-core          # Lock 0-core (immutable)
unlock-core        # Unlock for editing
edit-core <pkg>    # Edit package with auto-lock
core-status        # Check lock status
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

## 📋 **v3.3.2 PLAN FOR TOMORROW:**
```

SESSION TIME: 1.5-2 hours (fresh, focused)

PHASE 1: Convert Config (45 min)
├── Environment variables (5 min)
├── All aliases (15 min - mostly copy/paste)
├── Functions (25 min - careful conversion)
├── ya (yazi cd-on-quit)
├── weekly-check
├── update-check
├── notes/keepass launchers
└── fish_command_not_found equivalent

PHASE 2: Test in Zsh (30 min)
├── Launch zsh (don't switch default yet)
├── Test aliases
├── Test functions
├── Test autosuggestions
├── Test theme/colors
├── Verify everything works

PHASE 3: Switch & Document (15 min)
├── chsh -s /usr/bin/zsh
├── Update docs
├── Version bump to v3.3.2
├── Commit & push

TOTAL: ~1.5-2 hours, DONE!

```

### **For v3.3.4 (Next Week):**

```

1. 📸 Take screenshots
2. 📚 Create ALIASES.md
3. 🧹 Polish all files
4. 🔐 Update gitleaks & hooks
5. ✨ Final review

```

### **For Open Source (After v3.3.2):**

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

**Current Status:** Version 3.3 Complete ✅
**Next Action:** v3.3.1 - Theme Engine Foundation
**Vision:** Infrastructure as Poetry 🌲✨

---

_Last Updated: December 17, 2025_
_Roadmap Version: 5.0 - Architectural Refinement_

```

```
