---
id: 067
date: 2026-01-21
type: future
title: "Post-Linus Evolution - Next-Gen Causality & Planning Tools"
status: planned
tags: [rust, architecture, causality, simulation, post-v8.0.0]
related: [001, 061, 063]
---

## The Vision

After completing v8.0.0 audit and Linus presentation, build next-generation tools that answer:
- **"Why is the system in this state?"** (causality)
- **"What will happen if I do this?"** (simulation)  
- **"How do I prevent this class of mistakes?"** (constraints)

These tools formalize understanding into executable code.

## Critical Timing Rule

⚠️ **DO NOT START BEFORE:**
1. ✅ v8.0.0 released (all 29 tools audited)
2. ✅ Linus presentation complete
3. ✅ Feedback processed
4. ✅ Rest period taken

**Starting these tools before v8.0.0 = scope creep = incomplete presentation.**

## The Tools

### Tier 1: Core Planning Suite

#### 1. core-plan
**Purpose:** Safe future-state simulation

**What it does:**
- Simulate stow changes, package installs, profile switches
- Risk-scored execution plan
- Zone impact analysis (deploy/, govern/, observe/)
- **NEVER mutates state**

**Example:**
```bash
core-plan profile gaming

🌲 core-plan — Simulation Only
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Action: profile switch → gaming
Intent: #042 (GPU performance mode)

Zones Affected:
  🔴 DEPLOY   3 changes
  🟠 GOVERN   1 change

Planned Changes:
  + enable power profile: performance
  + override GPU clocks
  ~ replace sway config (wm-sway)
  ~ restart services: sway, faelight-bar

Risk Summary:
  Score: 9 / 12
  Level: HIGH
  Recovery: Snapshot available ✔

Constraints:
  ⚠️ VPN not connected (policy allows)
  ⚠️ Battery < 30%

No changes will be made.
```

**Why it fits:**
- Manual control over automation
- Recovery over perfection
- Makes "thinking before acting" first-class

**Implementation:**
- Zone detection (path → semantic meaning)
- Intent enforcement (DEPLOY requires intent)
- Stow + profile simulation
- Risk scoring with explainable math
- Constraint checking (read-only)

#### 2. core-why
**Purpose:** Explain why the system is in its current state

**What it does:**
- Traces causality chains
- Links intent ledger + git history + package metadata
- Answers "why" questions with evidence

**Example:**
```bash
core-why sway

→ Installed by profile: work
→ Required by intent #035 (Sway migration)  
→ Modified in v7.0.0 (architectural refactor)
→ Stowed from: 0-core/stow/wm-sway
→ Last changed: 2025-01-18 (keybind updates)
```

**Why it fits:**
- Understanding over convenience
- Makes archaeology actionable
- Zero automation

**Implementation:**
- Package provenance tracking
- Intent cross-referencing
- Git history integration
- Stow ownership mapping

#### 3. core-constraints
**Purpose:** Prevent classes of mistakes, not incidents

**What it does:**
- Declarative constraints in TOML
- Evaluated before safe-update, dotctl, profile switch
- Hard fail with explanation

**Example constraints:**
```toml
# 0-core/govern/constraints/system.toml

[[constraint]]
name = "no-core-unlock-on-vpn-down"
rule = "if vpn.down then forbid core.unlock"
message = "Core unlock requires VPN connection"

[[constraint]]
name = "no-kernel-update-on-battery"  
rule = "if battery.percent < 50 then forbid update.kernel"
message = "Kernel updates require >50% battery"

[[constraint]]
name = "no-deploy-when-locked"
rule = "if core.locked then forbid zone.deploy.*"
message = "Deploy zone changes require core unlock"
```

**Why it fits:**
- Intent → enforceable rules
- Explicit system behavior
- Human-aligned guardrails

**Implementation:**
- Constraint parser (TOML)
- System state queries (VPN, battery, core lock)
- Pre-execution hooks
- Clear error messages

### Tier 2: Deep Introspection

#### 4. intent-linter
**Purpose:** Enforce quality of thinking

**What it checks:**
- Orphaned intents (no implementation)
- Tools without intents (undocumented)
- Conflicting philosophy statements
- Missing success criteria
- Stale "in-progress" intents

**Example:**
```bash
intent lint

⚠️ Intent 052: In progress for 45 days
⚠️ Intent 055: No implementation found
❌ Tool 'faelight-dmenu': No intent references
✅ Philosophy consistency: OK
```

**Why it fits:**
- Makes philosophy executable
- Prevents "cargo cult" config growth
- Keeps 0-Core intentional

#### 5. faelight-trace (Advanced)
**Purpose:** Capture what actually happened

**What it does:**
- Wraps commands
- Records: files touched, services restarted, env mutations
- Stores trace as structured data

**Use cases:**
- Incident reconstruction
- Teaching integration
- Explaining side effects

**Example:**
```bash
faelight-trace safe-update

📊 Trace Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Command: safe-update
Duration: 42s

Files Modified (23):
  /etc/pacman.conf (backup created)
  /usr/bin/sway (package update)
  ...

Services Restarted (2):
  sway.service
  faelight-bar.service

Trace saved: ~/.local/share/faelight/traces/2026-01-22-safe-update.json
```

## Filesystem Evolution (Optional)

**Current structure works.** But if still needed after Linus feedback:
```
0-core/
├── deploy/              # Things that change the system
│   ├── stow/
│   ├── artifacts/
│   │   ├── bin/
│   │   └── scripts/
│   └── system/
│
├── src/                 # Source code
│   └── rust-tools/
│
├── govern/              # Rules, intent, constraints  
│   ├── INTENT/
│   ├── INCIDENTS/
│   ├── profiles/
│   ├── packages/
│   ├── constraints/     # ← New for core-constraints
│   └── hooks/
│
├── observe/             # Things that measure/explain
│   ├── health/
│   ├── archaeology/
│   └── traces/          # ← New for faelight-trace
│
├── knowledge/           # Documentation
│   ├── docs/
│   └── infrastructure/
│
└── automation/          # Explicit, manual-trigger only
```

**Decision point:** Wait for Linus feedback before restructuring.

## Implementation Phases

### Phase 1: Planning & Design (1 week)
**After v8.0.0 + Linus + rest**

Tasks:
- Process Linus feedback
- Refine tool designs based on his input
- Create detailed specs
- Set up Rust workspace structure

**Success:** Clear implementation plan

### Phase 2: core-plan (2-3 weeks)
**First tool - highest value**

Tasks:
- Zone detection system
- Intent enforcement
- Stow simulation
- Risk scoring
- Output formatting

**Success:** `core-plan profile gaming` works correctly

### Phase 3: core-why (2 weeks)
**Second tool - causality**

Tasks:
- Package provenance
- Intent cross-reference
- Git history integration
- Stow ownership

**Success:** `core-why sway` explains complete causality chain

### Phase 4: core-constraints (2-3 weeks)  
**Third tool - prevention**

Tasks:
- TOML constraint parser
- System state queries
- Pre-execution hooks
- Integration with safe-update, dotctl, profile

**Success:** Constraints prevent mistakes before execution

### Phase 5: intent-linter (1 week)
**Fourth tool - quality**

Tasks:
- Intent parser enhancement
- Tool-to-intent mapping
- Philosophy consistency checks
- Reporting system

**Success:** `intent lint` catches quality issues

### Phase 6: Polish & Integration (1-2 weeks)
**System-wide**

Tasks:
- Comprehensive READMEs
- Health check integration
- Cross-tool testing
- Documentation

**Success:** All tools production-ready

## Estimated Timeline

**Total: 8-11 weeks post-Linus**

| Phase              | Duration  | Tools                    |
| ------------------ | --------- | ------------------------ |
| 1. Planning        | 1 week    | Design + specs           |
| 2. core-plan       | 2-3 weeks | Simulation tool          |
| 3. core-why        | 2 weeks   | Causality tool           |
| 4. core-constraints| 2-3 weeks | Policy enforcement       |
| 5. intent-linter   | 1 week    | Quality tool             |
| 6. Polish          | 1-2 weeks | Integration + docs       |

**Target:** Q1-Q2 2026 (after Linus presentation)

## Why These Tools Matter

### They Fill Missing Layers
Current system is strong in:
- Configuration
- Health monitoring
- UI (bar, launcher, notify)
- Governance (intent, git, versions)

Missing:
- **Causality** ← core-why
- **Simulation** ← core-plan
- **Constraint enforcement** ← core-constraints
- **Quality assurance** ← intent-linter

### They Show Systems Thinking
Linus will appreciate:
- Deep understanding of state management
- Thoughtful prevention over reaction
- Manual control with strong guardrails
- Philosophy encoded in tools

### They Scale Beyond One Person
These tools make the system:
- Self-explanatory under stress
- Teachable to others
- Resistant to drift
- Maintainable long-term

## Risks & Mitigation

### Risks
1. **Scope creep** - Could delay indefinitely
2. **Over-engineering** - Tools more complex than needed
3. **Maintenance burden** - More code to maintain
4. **Linus feedback conflict** - He might suggest different approach

### Mitigation
1. **Strict phases** - Build one tool at a time, ship before next
2. **MVP first** - Core functionality only, expand later
3. **Clear value** - Each tool must solve real pain point
4. **Feedback first** - Wait for Linus input before committing

## Success Criteria

**Phase 1 Success:**
- Linus feedback processed
- Clear specs written
- Implementation plan agreed

**Phase 2-5 Success (Per Tool):**
- Tool works correctly
- Comprehensive README
- Health check integration
- Production-tested

**Final Success:**
- All 4 tools production-ready
- System more understandable
- Mistakes prevented before they happen
- Philosophy executable in code

## Alternatives Considered

### Option A: Build Now
**Rejected:** Would prevent v8.0.0 completion and risk incomplete Linus presentation

### Option B: Build After v8.0.0 but Before Linus
**Rejected:** Not enough time, would rush tools, incomplete work

### Option C: Build After Linus (CHOSEN)
**Chosen:** Allows complete audit, gets Linus feedback, no rush, proper design time

### Option D: Never Build
**Rejected:** These tools genuinely improve system understanding and safety

## The Decision

**Wait until after Linus presentation, then build methodically.**

This is not deferral - it's sequencing.
First: Complete what exists.
Then: Expand with wisdom.

## Post-Linus Checklist

Before starting Phase 1:
- [ ] v8.0.0 released and stable
- [ ] Linus presentation complete  
- [ ] Feedback documented
- [ ] 1 week rest taken
- [ ] Specs refined based on feedback
- [ ] Implementation plan confirmed

---

_First finish the forest. Then grow new branches._ 🌲

**DO NOT START UNTIL v8.0.0 + LINUS + REST.**
