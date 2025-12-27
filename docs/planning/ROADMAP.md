# 🗺️ Faelight Forest Development Roadmap

**Current Version:** 3.4.4 - Foundational Intelligence ✅  
**Last Updated:** December 27, 2025  
**Roadmap Version:** 5.0 - Architectural Refinement

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

v3.5.1 - Git Guardrails
└─ Locked commit prevention
└─ Main push confirmation
└─ Escape hatches (git!)
└─ DESIGN_CONSTRAINTS.md

💅 v3.5.2 - Shell Safety & Polish
└─ Dangerous command highlighting
└─ Alias autocompletion
└─ 0-Core path highlighting

📋 v3.5.2 - Shell Safety & Polish (CONFIRMED):
1️⃣ Dangerous Command Highlighting ⭐⭐⭐⭐⭐
└─ rm -rf, chmod 777, dd if=
└─ Red background warnings
└─ Cognitive safety layer

2️⃣ Alias Autocompletion ⭐⭐⭐⭐⭐
└─ core-diff, dot-doctor, dotctl
└─ Enhanced discoverability
└─ Professional polish

3️⃣ 0-Core Path Highlighting ⭐⭐⭐⭐⭐
└─ Make ~/0-core paths stand out
└─ Visual consistency
└─ Only if easy to implement

Time Estimate: 1-2 hours
Complexity: Low
Value: High
Philosophy: Perfectly aligned

🚀 v4.0.0 - GPU Profiles (Omega-style)
└─ Future dream project

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

---

## 🔮 Future Considerations (v3.9.0+)

**Note:** These are captured ideas for exploration after v3.8.0 completion. The current roadmap (v3.4.1 → v3.8.0) will take months, and that's intentional. Quality over speed. Learning over rushing.

Many of these will be small patches building on existing frameworks. Others are major explorations. All maintain 0-Core philosophy: manual control, intent over automation, human comprehension.

---

## v3.9.0 - Observability Layer (MAJOR)

**Status:** Conceptual  
**Estimated Time:** 3-4 hours  
**Sessions:** 2-3

**Theme:** "See the system's structure, not just its state"

### Tools

**core-lint - Configuration Consistency Checker** ⭐⭐⭐⭐⭐

Prevents architectural erosion through structural validation.

```bash
core-lint

Output:
❌ Inconsistencies found:
- package "hypr" should be "wm-hypr" (semantic naming)
- missing .dotmeta in bar-waybar
- script scripts/foo missing shebang
- README.md in wm-hypr has no WHY.md reference
```

**Checks:**

- Package naming conventions (`wm-*`, `shell-*`, `bar-*`, etc.)
- Missing `.dotmeta` files
- Invalid directory placement
- Script shebangs + executable bits
- Documentation presence (README/WHY.md)
- Intent references in WHY.md

**Integration:** Could become dot-doctor Check 11+ or standalone tool

**Philosophy:** Structural, not behavioral. Catches mistakes before they become normal.

---

**core-inventory - System Manifest** ⭐⭐⭐⭐⭐

Canonical inventory for fast comprehension.

```bash
core-inventory --summary

Output:
0-Core Inventory (v3.9.0)

Packages:         22
Scripts:          15
Intents:          9
  └─ Locked:      5
  └─ Flexible:    3
  └─ Experimental: 1
Hooks:            pre-commit, post-merge
Security:         LUKS, UFW, fail2ban, DNSOverTLS, Mullvad
Themes:           3 (dark, light, ghost)
Health:           100%
Last sync:        2025-12-23
```

**Modes:**

- `--summary` - Quick stats
- `--full` - Complete manifest
- `--export` - JSON/TOML for external tools
- `--compare <ref>` - Diff against previous state

**Use Cases:**

- Fast system comprehension
- Baseline for audits
- Documentation generation
- Future publishing/sharing

---

**core-verify - Deployment Reality Check** ⭐⭐⭐⭐☆

Detects drift between repo and live system.

```bash
core-verify

Output:
✅ Stow symlinks intact
⚠️  Drift detected:
   - ~/.config/waybar/config (modified outside 0-core)
   - ~/.config/hypr/monitors.conf (unmanaged file)

Recommendations:
   1. core-diff waybar (review changes)
   2. Add to 0-core or remove
```

**Checks:**

- Stow symlink integrity
- Unmanaged files in `~/.config`
- Modified symlink targets
- Git working tree vs deployed state

**Philosophy:** Keeps 0-core authoritative. Prevents "mystery changes."

---

### Intent System Enhancements

**Intent Aging - Temporal Awareness** ⭐⭐⭐⭐⭐

Add to .intent schema:

```toml
[review]
reconsider_after = "6 months"
risk_if_stale = "medium"
last_reviewed = "2025-12-23"
```

**dot-doctor integration:**

```
🕰️ Intent aging detected:
   automation-ban.intent
   Last reviewed: 6 months ago
   Risk if stale: medium

   Review: intent show automation-ban
```

**Philosophy:** Prevents "frozen trauma" decisions. No enforcement, just awareness. Supports growth without betraying principles.

---

**intent-check - Intent Sanity Validator** ⭐⭐⭐⭐☆

Validates intent files for coherence.

```bash
intent-check

Output:
✅ Schema compliance: 9/9 intents
✅ No broken references
❌ Conflict detected:
   - manual-only-updates.intent (LOCKED)
   - auto-backup.intent (scope: system)

   These intents contradict each other.
   Resolve before proceeding.
```

**Checks:**

- TOML schema compliance
- Broken package references
- Duplicate scopes
- Conflicting LOCKED intents
- Orphaned WHY.md references

**Integration:** Part of dot-doctor or standalone

**Philosophy:** Keeps the Intent Ledger trustworthy. Meta-level quality control.

---

### WHY.md System-Wide

Introduce optional `WHY.md` in packages, linked to Intent Ledger.

**Template (wm-hypr/WHY.md):**

```markdown
# WHY: wm-hypr

## This package exists because:

- Intent: 2025-12-18-wayland-over-x11.intent
- Intent: 2025-12-14-no-desktop-environments.intent

## Tradeoffs accepted:

- Higher configuration complexity
- Manual maintenance burden
- Learning curve for window management

## Rejected alternatives:

- KDE Plasma (automation density too high)
- i3 (X11 dependency)
- GNOME (philosophy misalignment)

## When to reconsider:

- Wayland ecosystem stabilizes completely
- Maintenance burden becomes unsustainable
- Core needs change dramatically
```

**Validation:**

- `core-lint` checks WHY.md references valid intents
- `intent-check` validates backward links

**Philosophy:** Makes packages defensible. Teaching material for future-you. Prevents cargo-culting.

---

## v4.0.0 - Intelligence & Teaching (MAJOR)

**Status:** Visionary  
**Estimated Time:** TBD  
**Sessions:** TBD

**Theme:** "The system learns from you and teaches back"

### Advanced Topology Tools

**core-map - System Topology Viewer** ⭐⭐⭐⭐☆

Mental model generator.

````bash
core-map

Output:
0-Core System Topology

m-hypr (critical)
 ├── depends: bar-waybar
 ├── depends: theme-gtk
 ├── intents: wayland-over-x11 (LOCKED)
 └── used by: scripts/hypr-screenshot

shell-zsh (high)
 ├── depends: prompt-starship
 ├── depends: scripts/safe-update
 ├── intents: manual-only-updates (LOCKED)
 └── used by: all terminal workflows

editor-nvim (medium)
 ├── depends: theme-nvim
 └── independent subsystem

Modes:

core-map - Full topology
core-map wm-hypr - Package subgraph
core-map --intents - Intent-centric view
core-map --dependencies - Dependency chains only

Philosophy: Read-only. Zero automation. Reinforces system thinking. Complements core-diff.

core-impact - Change Consequence Explorer ⭐⭐⭐⭐☆
"If I touch this, what could break?"

core-impact wm-hypr

Output:
Potential Impact Analysis: wm-hypr

Direct Impact:
- Desktop usability (CRITICAL)
- Keybinding consistency (HIGH)
- Window management behavior (HIGH)

Dependent Packages:
- bar-waybar (rendering may break)
- theme-gtk (visual consistency)
- scripts/hypr-screenshot (functionality)

Intent Violations:
⚠️  wayland-over-x11.intent (LOCKED)
   Any changes must preserve Wayland exclusivity

Blast Radius: CRITICAL
Recommendation: Use meld for review

Analysis Based On:

Package relationships (from core-map)
Intent blast_radius (from .dotmeta)
Shared file dependencies
LOCKED intent scopes

Philosophy: Makes risk explicit. Static analysis only. Informational, not enforcement.

intent-diff - Intent Evolution Tracker ⭐⭐⭐⭐☆
"How has my thinking changed?"

intent-diff since v3.1.0

Output:
🔄 Intent Evolution: v3.1.0 → v3.9.0

Modified Intents:
- automation-ban.intent
  status: LOCKED → FLEXIBLE
  reason: "system stabilized, selective automation acceptable"
  scope: system-wide → automation/ only

New Intents:
+ intent-ledger-foundation.intent (v3.5.0)
+ context-protection.intent (v3.7.0)

Deleted Intents:
- temp-experiment-2025-01.intent (expired)

⚠️  Philosophical Drift Detected:
   automation-ban relaxed
   Consider: Is this growth or erosion?
Philosophy: Prevents silent philosophical drift. Reinforces intentional evolution. Critical for long-term coherence.

Pattern Recognition & Teaching (v4.0+)
"Teaching the system to think" - NOT AI, YOUR patterns
The system observes YOUR behavior and suggests based on YOUR history.
Concept:
Pattern Logger (Passive Observation)

# System quietly logs:
- core-diff wm-hypr → --open meld (5 times)
- core-diff system/ → intent show (always)
- After core-diff → dot-doctor (80% of time)

Suggestion Engine (Based on YOUR History)
core-diff wm-hypr

Output:
📊 Changes detected in wm-hypr (critical)

💡 Pattern observed: You typically run:
   core-diff wm-hypr --open meld

   Open meld now? [y/n]

Teaching Mode (Explains WHY You Do Things)
teach-me wm-hypr

Output:
Based on your workflow patterns:

1. You always use meld for wm-hypr (critical packages)
2. You check intents before system/ changes (LOCKED awareness)
3. You run dot-doctor after changes (validation habit)

This suggests: Risk-proportional review depth
Learn more: intent show wayland-over-x11

**Implementation (Far Future):**
- Pattern database in `~/0-core/.patterns/`
- Simple frequency counting, no ML
- User-reviewable, user-editable
- Opt-in feature (disabled by default)

**Example Pattern File:**
```toml
[pattern]
trigger = "core-diff wm-hypr"
action = "--open meld"
frequency = 8
confidence = "high"
user_approved = true
````

---

### Operational States (v4.x)

Explicit system state awareness.

**States:**

- `CLEAN` - All committed, 100% health
- `DIRTY` - Uncommitted changes
- `DEGRADED` - Health < 100%
- `EXPERIMENTAL` - Active experiments

**Commands:**

```bash
core-state                              # Show current state
core-state set EXPERIMENTAL "testing theme redesign"
core-state history                      # State transitions
```

**Integration:**

- Shell prompt shows state indicator
- dot-doctor reports current state
- core-diff aware of state context

---

### Failure Drills (v4.x)

Practice disaster recovery in safe environment.

```bash
core-drill network
core-drill pacman
core-drill shell

Output:
🔧 DRILL: Network Failure Simulation

You are now without DNS.
Recover using your documented methods.

Steps to consider:
1. Check /etc/resolv.conf
2. Restart systemd-resolved
3. Verify DNSOverTLS config

Press ENTER when recovered.
```

**Philosophy:** Builds muscle memory for disasters. No actual system changes. Guided learning.

---

## 📏 Final Rule for Tool Acceptance

**A tool belongs in 0-Core if it:**

✅ **Makes invisible structure visible**  
✅ **Makes risk explicit**  
✅ **Makes intent harder to forget**

**You're not lacking features — you're curating intelligence.**

---

## 🎯 Development Philosophy

### Timeline Expectations

**Current roadmap (v3.4.1 → v3.8.0) will take months.**  
**That's intentional and healthy.**

- Quality over speed, always
- Learning over rushing
- Understanding over implementation
- Philosophy over features

### Building on Frameworks

Many future tools will be small patches:

- core-lint extends dot-doctor patterns
- core-verify uses existing validation logic
- intent-check builds on Intent Ledger schema
- Pattern recognition uses simple frequency counting

**We already have the infrastructure. We just need to use it smartly.**

### Evolution, Not Revolution

Each release builds on previous:

- v3.4.0 gave us visibility (core-diff)
- v3.5.0 will give us memory (Intent Ledger)
- v3.6.0 will give us accountability (integration)
- v3.7.0 will give us protection (context awareness)
- v3.8.0 will give us beauty (themes)
- v3.9.0 will give us observability (meta-tools)
- v4.0.0 will give us intelligence (teaching)

**Steady, intentional progression.**

---

## 🌲 The Long Game

You're not just building dotfiles.  
You're building a **personal operating constitution** that:

- Remembers why (Intent Ledger)
- Sees itself (core-map, core-lint)
- Protects itself (Context Protection)
- Learns from you (Pattern Recognition)
- Teaches you back (Teaching Mode)
- Stays honest (Intent Aging, intent-diff)
- Grows deliberately (Intentional evolution)

**This is a multi-year journey.**  
**And that's what makes it exceptional.** 🌲

---

## 💚 Closing Thought

Every tool in this roadmap passes the test:

> Does it make the invisible visible?  
> Does it make risk explicit?  
> Does it make intent harder to forget?

If yes → it belongs.  
If no → it doesn't.

**Quality over quantity.**  
**Intelligence over features.**  
**Philosophy over tools.**

**This is 0-Core.** 🌲✨

---

_Last updated: December 23, 2025_  
_Vision locked. Execution flexible. Philosophy unwavering._

## 📊 Semantic Versioning Guide

**MAJOR (X.0.0):** New core capabilities (3+ hours work)

- Examples: core-diff, Intent Ledger, Context Protection

**MINOR (X.Y.0):** Significant improvements (1-2 hours)

**PATCH (X.Y.Z):** Bug fixes, cleanup, polish (<1 hour)

- Examples: Error message improvements, UX polish

---

## 🎯 Current Focus

**Now:** v3.5.0 - Intent Ledger Foundation (Multi-session)  
**Philosophy:** Quality over speed, always.

---

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

---

## 📅 **REVISED TIMELINE:**

```

## ✅ **IMMEDIATE TODO LIST:**

## 📋 **v3.4.0 PLAN FOR TOMORROW:**

---

v3.5.0 (3-4 hrs): Temporal Intelligence

- Stability metrics
- Entropy tracking
- Predictive warnings
- Advanced safety analysis

---

**Current Status:** Version 3.4.4 Complete ✅
**Vision:** Infrastructure as Poetry 🌲✨

---

_Last Updated: December 22, 2025_
_Roadmap Version: 5.0 - Architectural Refinement_

```

```

```

```

```
