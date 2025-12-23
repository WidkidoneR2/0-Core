# 🗺️ Faelight Forest Development Roadmap

**Current Version:** 3.4.0 - Foundational Intelligence ✅  
**Last Updated:** December 22, 2025  
**Roadmap Version:** 5.0 - Architectural Refinement

---

## Future Considerations

### **v3.5.0 - Safe Context (2-3 hours)**

```
Context-aware command protection:
- rm wrapper for 0-core
- mv wrapper for 0-core
- Clear, simple, valuable
- Prevents muscle-memory mistakes

Implements: "Break Muscle Memory" idea
```

---

### **v3.6.0 - Dot-Doctor Enhanced (2-3 hours)**

```
Tier 1 features (MUST HAVE):
- Human-readable fixes
- Risk-grading display
- Historical health tracking

These three alone are transformative!
```

---

### **v3.7.0 - Predictive Safety (3-4 hours)**

```
Tier 2 features (VERY VALUABLE):
- Near-miss logging
- Session-aware validation
- Config smell detection

### Long-term Vision
- Make 0-core a reference implementation
- Share philosophy through documentation
- Potential framework for others
- Always: manual control, awareness over automation

---

```

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



PART 4: Security Updates (30 min)
├── Update gitleaks
├── Test gitleaks config
├── Review/enhance git hooks
├── Test hooks thoroughly
└── Document hook behavior

---

## 📅 **REVISED TIMELINE:**

```

## ✅ **IMMEDIATE TODO LIST:**

## 📋 **v3.4.0 PLAN FOR TOMORROW:**

```

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

---

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

**Current Status:** Version 3.4.0 Complete ✅
**Vision:** Infrastructure as Poetry 🌲✨

---

_Last Updated: December 22, 2025_
_Roadmap Version: 5.0 - Architectural Refinement_

```

```

```

```

```
