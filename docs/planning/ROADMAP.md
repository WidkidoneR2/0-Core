# 🗺️ Faelight Forest Development Roadmap

**Current Version:** 3.4.4 - Foundational Intelligence ✅  
**Last Updated:** December 28, 2025  
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

4️⃣ Starship Lock Status Integration ⭐⭐⭐⭐⭐
└─ 🔒 when locked, 🔓 when unlocked
└─ Replaces core_guard() verbose warnings
└─ Cleaner, more elegant

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

🎮 v4.0.0 - GPU Profiles (Omega Legacy)
└─ Omega-style GPU configuration
└─ Wayland + Vulkan support
└─ TUI selector
└─ Honor the past

Phase 1: Learn by doing (v5.0)
🌲 v5.0.0 - FAELIGHT-BAR (THE BIG ONE)
└─ Custom Rust Wayland bar
└─ Minimal, intentional, beautiful
└─ Hyprland-native
└─ Security-focused
└─ Philosophy embodied in code
└─ CELEBRATION OF THE JOURNEY
└─ Something RARE and UNIQUE

## 🦀 RUST TRANSITION PLAN:

faelight-bar will be your Rust teacher:

- Real project (not toy)
- Meaningful goal
- Forces you to learn properly
- Builds muscle memory

Phase 2: Rewrite existing tools (v5.1+)
After faelight-bar, rewrite in Rust:

- core-diff → Rust (faster, safer)
- dot-doctor → Rust (type-safe health checks)
- safe-update → Rust (better error handling)
- core-protect → Rust (security-critical)

Phase 3: Rust-first for new tools (v6.0+)
All new 0-Core tools written in Rust:

- Better performance
- Memory safety
- Modern practices
- Cross-compilation

## 📅 **REVISED TIMELINE:**

```
Phase 1: Foundation (v3.5.x - v4.x) - Bash/Shell
v3.5.0 - Intent Ledger (Bash)
v3.5.1 - Git Guardrails (Shell)
v3.5.2 - Shell Polish (Zsh/Starship)
v4.0.0 - GPU Profiles (Bash/Shell)

Why Bash/Shell:
✅ Proven technology
✅ Fast to ship
✅ Captures your ideas NOW
✅ Builds the foundation

Phase 2: Learning & Standalone (v5.0.0) - First Rust
v5.0.0 - faelight-bar (Rust)

Why this is the PERFECT start:
✅ Standalone project (doesn't break 0-Core if you struggle)
✅ Can coexist with Waybar during development
✅ Forces you to learn Wayland + Rust properly
✅ Testable, iterate, improve
✅ Celebration of your journey

Status: bar-faelight/ as NEW package
0-Core still runs on bar-waybar (safe fallback)

Phav5.1 - core-diff rewrite (Rust)
   └─ Start with most-used tool
   └─ Keep bash version as fallback
   └─ scripts/core-diff.rs alongside core-diff.sh

v5.2 - dot-doctor rewrite (Rust)
   └─ Type-safe health checks
   └─ Better error messages
   └─ Parallel checks (faster)

v5.3 - safe-update rewrite (Rust)
   └─ Critical tool = deserves Rust safety
   └─ Better snapshot management
   └─ Atomic operations

v5.4 - core-protect rewrite (Rust)
   └─ Security-critical
   └─ Memory safety matters here

The strategy is REPLACEMENT:
0-Core/scripts/
├── core-diff              # Original bash (v3.5.0)
├── core-diff.rs          # Rust version (v5.1)
├── Cargo.toml            # Rust dependencies
└── .version              # Tracks which is default

REVISED RUST TRANSITION (ALL-IN):
Phase 1: Foundation (Bash) - Q4 2025 to Q1 2026
v3.5.0 - Intent Ledger (Bash)
v3.5.1 - Git Guardrails (Bash)
v3.5.2 - Shell Polish (Zsh/Starship)
v4.0.0 - GPU Profiles (Bash)

Status: Pure Bash/Shell
Announcement: "v5.0 brings Rust rewrite"
Phase 2: First Rust (Standalone) - Q2 2026
v5.0.0 - faelight-bar (Rust)

Status: NEW package, doesn't replace anything yet
bar-waybar/ → stays (for now)
bar-faelight/ → new Rust bar

Why: Proves Rust works, isolated experiment
Phase 3: THE BIG CUT-OVER - Q3 2026+
v5.1.0 - Complete Rust Rewrite (THE TRANSITION)

EVERYTHING rewrites to Rust:
├── scripts/core-diff         → Rust
├── scripts/dot-doctor        → Rust
├── scripts/safe-update       → Rust
├── scripts/core-protect      → Rust
├── scripts/dotctl            → Rust
├── scripts/core-status       → Rust
├── scripts/bump-system-version → Rust
└── ALL bash scripts          → Rust

bar-waybar/ → REMOVED
bar-faelight/ → ONLY bar

Result: 100% Rust tooling
No bash fallbacks
Clean codebase

---

**Current Status:** Version 3.4.4 Complete ✅
**Vision:** Infrastructure as Poetry 🌲✨

---

_Last Updated: December 28, 2025_
_Roadmap Version: 5.0+ - RUST TRANSITION
```
