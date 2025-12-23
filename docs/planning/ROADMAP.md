# 🗺️ Faelight Forest Development Roadmap

**Current Version:** 3.4.0 - Foundational Intelligence ✅  
**Last Updated:** December 22, 2025  
**Roadmap Version:** 5.0 - Architectural Refinement

---

## ✅ v3.4.0 - core-diff (MAJOR) - COMPLETE! 🎉

**Status:** Shipped December 23, 2025  
**Health:** 100%  
**Sessions:** 3 (Planning, Advanced, Documentation)  
**Time:** ~5-6 hours

**Delivered:**

- Package-aware diff tool with risk-based grouping
- Delta & Meld integration
- 7+ working modes (default, since, summary, verbose, high-risk, package-specific)
- Comprehensive documentation (TOOLS.md, WORKFLOWS.md)
- 12 aliases for common workflows
- Philosophy: "Meld shows trees. core-diff shows the forest 🌲"

**Files:**

- `scripts/core-diff` - Main tool
- `docs/TOOLS.md` - Tool reference
- `docs/WORKFLOWS.md` - Usage patterns
- 12 aliases in `.zshrc`

---

## ⏳ v3.4.1 - core-diff Polish (PATCH)

**Status:** Planned (Start: Friday/Saturday)  
**Estimated Time:** 1-2 hours  
**Sessions:** 1

**Goals:**

- Better error messages (more helpful, specific)
- Exit codes for scripting (0=success, 1=error, etc.)
- Minor UX polish (output formatting, edge cases)

**Why:**

- Make core-diff more robust for daily use
- Enable scripting/automation with reliable exit codes
- Improve user experience based on initial usage

**Success Criteria:**

- All error messages are helpful and actionable
- Exit codes documented and consistent
- Edge cases handled gracefully

---

## 🎯 v3.5.0 - Intent Ledger Foundation (MAJOR)

**Status:** Planned  
**Estimated Time:** 3-4 hours  
**Sessions:** 2-3 (Multi-session release)

**CRITICAL:** This is foundational - everything builds on this layer.

**Goals:**

### Session 1: Structure & Format (1.5 hours)

- Design INTENT/ directory structure
- Define .intent file format (TOML-based)
- Create initial intents from existing decisions
- Document the schema

### Session 2: Basic Commands (1.5 hours)

- `intent add` - Add new intent
- `intent list` - List all intents
- `intent show <id>` - Display intent details
- Basic validation

### Session 3: Polish & Documentation (1 hour)

- Error handling
- Documentation in TOOLS.md
- Usage examples
- Testing

**Directory Structure:**

```
~/0-core/INTENT/
├── decisions/
│   ├── 2025-12-14-password-incident.intent
│   ├── 2025-12-16-manual-only-updates.intent
│   └── 2025-12-18-zsh-over-fish.intent
├── assumptions/
│   ├── user-is-technical.assumption
│   └── system-is-single-user.assumption
├── tradeoffs/
│   └── automation-vs-control.tradeoff
├── experiments/
│   ├── aging-report.experiment
│   └── semantic-naming.enforced.experiment
└── README.md
```

**.intent Format:**

```toml
[metadata]
id = "2025-12-14-password-incident"
status = "LOCKED"  # LOCKED, FLEXIBLE, EXPERIMENTAL
scope = "system-wide"
created = "2025-12-14"
updated = "2025-12-14"

[decision]
trigger = "sudo failure after reboot"
decision = "eliminate boot-time automation"
alternatives = ["fix timers", "add credentials"]
rejected_because = "non-deterministic, fragile"
revision_allowed = false

[impact]
packages = ["system", "automation"]
blast_radius = "critical"
```

**NOT in v3.5.0:**

- ❌ dot-doctor integration (that's v3.6.0)
- ❌ Enforcement (manual awareness only)
- ❌ Automated anything

**Why:**

- Captures "why" decisions were made
- Prevents forgetting lessons
- Creates institutional memory
- Supports future-you

**Success Criteria:**

- Can create, view, list intents
- Format is clear and useful
- Documentation complete
- Foundation solid for v3.6.0 integration

---

## 🔗 v3.6.0 - Intent Ledger Integration (MAJOR)

**Status:** Planned (After v3.5.0)  
**Estimated Time:** 2-3 hours  
**Sessions:** 2

**Dependencies:** v3.5.0 must be complete

**Goals:**

### Accountability Layer

- dot-doctor warns on LOCKED intent violations
- Intent validation (referenced intents exist)
- Conflict detection (changes vs LOCKED intents)

### Integration Points

- `core-diff` references intents when showing changes
- `dot-doctor` Check 11: Intent compliance
- Warning system (not blocking)

**Example Warning:**

```
⚠️ Change detected touching update system
   Conflicts with LOCKED intent:
   2025-12-16-manual-only-updates.intent

   Review intent: intent show manual-only-updates
```

**Philosophy:**

- Accountability, not enforcement
- Warnings, never blocks
- User maintains control

**Success Criteria:**

- dot-doctor detects intent conflicts
- Warnings are helpful, not annoying
- Intent system feels valuable, not burdensome

---

## 🛡️ v3.7.0 - Context Protection (MAJOR)

**Status:** Planned  
**Estimated Time:** 2-3 hours  
**Sessions:** 2

**Goals:**

### Safety Wrappers

- Intercept dangerous commands in 0-core/
- Commands: rm, mv, cp (when in 0-core)
- Require confirmation or redirect

### Near-Miss Logging

- Log when protection triggers
- Track patterns
- Learn from close calls

### Example Protection:

```bash
~/0-core$ rm file.conf
⚠️  Dangerous command in 0-core!

   Use instead:
   • git rm file.conf (to remove from repo)
   • exit 0-core first, then rm

   Near-miss logged.
```

**Philosophy:**

- Break muscle memory on dangerous ops
- Gentle intervention, not blocking
- Learn from mistakes before they happen

**Success Criteria:**

- Protection feels helpful, not annoying
- Reduces accidental damage
- Logging provides insights

---

## 🎨 v3.8.0 - Theme Completion (MAJOR)

**Status:** Planned  
**Estimated Time:** TBD  
**Sessions:** TBD

**Current State:**

- ✅ Dark variant (Faelight Forest) - Complete
- ⏳ Light variant - Incomplete (stopped mid-implementation)

**Goals:**

### Complete Light Theme

- Finish light variant implementation
- Test in all packages
- Documentation

### Ghost Variant (Exploration)

- Research ghost/minimal aesthetic
- Design color palette
- Prototype in key packages

### Waybar Redesign (Possible)

- Explore completely new waybar layout
- Modern design patterns
- Functional improvements

**TBD:**

- Scope depends on creative direction
- Time estimate pending design phase
- May split into multiple releases

---

## 🔮 Future Considerations (v3.9.0+)

**Operational Maturity:**

- System states (CLEAN, DIRTY, DEGRADED, EXPERIMENTAL)
- Failure drills (core-drill network, pacman, shell)
- WHY.md per package
- Teaching mode

**Integration:**

- Topgrade refinement
- GitHub Actions / CI
- External tool integration

**Philosophy:**

- Constraint Engine (passive consistency)
- Teaching Mode (knowledge transfer)
- Legacy planning

---

## 📊 Semantic Versioning Guide

**MAJOR (X.0.0):** New core capabilities (3+ hours work)

- Examples: core-diff, Intent Ledger, Context Protection

**MINOR (X.Y.0):** Significant improvements (1-2 hours)

- Examples: New dot-doctor checks, feature additions

**PATCH (X.Y.Z):** Bug fixes, cleanup, polish (<1 hour)

- Examples: Error message improvements, UX polish

---

## 🎯 Current Focus

**Next Up:** v3.4.1 - core-diff Polish (Friday/Saturday)  
**Then:** v3.5.0 - Intent Ledger Foundation (Multi-session)  
**Philosophy:** Quality over speed, always.

---

## 📝 Notes

- Intent Ledger (v3.5.0 + v3.6.0) is the critical path
- Everything builds on the memory layer
- No rushing - each release must be solid
- Documentation is mandatory, not optional
- 100% health before shipping

---

```

## 🔄 Smart Updates

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
