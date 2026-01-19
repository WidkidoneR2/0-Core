---
id: 061
date: 2026-01-15
type: future
title: "v7.5-v8.0 Roadmap - The Path to the Observant Forest"
status: planned
tags: [roadmap, architecture, vision, v8.0]
version: 8.0.0
---

## Vision: The Observant Forest 🌲👁️

**A system that watches, learns, and suggests — but NEVER acts without explicit permission.**

**Core Principle from Incident 001 (2025-12-14):**
> "Automation + privileged operations + boot timing = disaster"
> "Nothing runs unless a human explicitly authorizes it"

---

## Release Roadmap

### v7.5.0 - "Foundation & Security" (Feb 2026)
**Theme:** Build secure infrastructure

**Major Features:**
- ✅ Intent 058: Security Audit Framework (observe, report, suggest fixes)
- ✅ Intent 059: Monorepo Unification (structure, not automation)
- ✅ Intent 060: Universal Search Foundation (tool, user-driven)
- ✅ Intent 051: Decision Log (documentation)
- ✅ Intent 055: wm-sway v1.1 updates

**Philosophy Alignment:**
- Security scanner REPORTS vulnerabilities
- User CHOOSES which fixes to apply
- No automatic patching

---

### v7.6.0 - "Time Machine" (Mar 2026)
**Theme:** Perfect recovery, user-controlled

**Major Features:**
- ✅ Intent 053: Configuration Registry (tracks changes)
- ✅ Intent 054: System Timeline (shows history)
- ✅ Intent 056: archaeology v2.0 (explores past)
- 🆕 Intent 062: BTRFS snapshot integration
- 🆕 Intent 063: Config restoration (manual trigger)

**New Capabilities:**
```bash
timeline show                    # View system history
timeline at 2025-01-01          # Show state at date
timeline suggest-restore wifi   # SUGGESTS rollback, ASKS permission
archaeology --what-broke wifi   # Shows what changed
archaeology restore <snapshot>  # USER triggers restore
```

**Philosophy Alignment:**
- System SHOWS you history
- System SUGGESTS restores
- USER makes all decisions
- Nothing happens automatically

---

### v7.7.0 - "The Observant Core" (Apr 2026)
**Theme:** Intelligence through observation, NOT automation

**CRITICAL:** This version was redesigned to avoid violating Incident 001 lessons.

**Major Features:**
- ✅ Intent 057: Workspace intelligence (observe, don't act)
- 🆕 Intent 065: Workspace Pattern Observer (NOT AI Engine)
- 🆕 Intent 066: Command Suggestions (NOT natural language automation)
- 🆕 Intent 067: Workflow Templates (manual creation/execution)
- ✅ Intent 052: Intent system enhancements

**What Changed from Original Design:**
- ❌ REMOVED: AI auto-applies learned patterns
- ❌ REMOVED: "Smart" automation
- ❌ REMOVED: Predictive execution
- ✅ ADDED: Pattern observation and reporting
- ✅ ADDED: Suggestion system (user confirms)
- ✅ ADDED: Template system (user triggers)

**New Capabilities:**
```bash
workspace observe
# → Shows detected patterns
# → Lists suggestions available
# → NEVER applies anything

workspace suggest
# → "You often open X, Y, Z at 9am"
# → "Create 'morning' profile? [yes/no/details]"
# → WAITS for explicit yes

workspace apply morning
# → Shows commands it will run
# → Asks for confirmation
# → USER types 'yes' to execute
```

**Philosophy Alignment:**
- System OBSERVES your patterns
- System BUILDS models
- System SUGGESTS optimizations
- System WAITS for permission
- System SHOWS what it will do
- USER explicitly triggers

**Guard Rails:**
- No boot-time execution
- No background automation
- Every action requires confirmation
- All commands shown before execution

---

### v7.8.0 - "Living Dashboard" (May 2026)
**Theme:** Visibility and awareness

**Major Features:**
- 🆕 Intent 068: Live system dashboard (display only)
- 🆕 Intent 069: Intelligent monitoring (detect, report, suggest)
- 🆕 Intent 070: UI/UX polish
- Enhancement: Typography improvements
- Enhancement: Unified theme system

**New Capabilities:**
- Real-time system visualization
- Pattern detection (NON-ACTING)
- Suggestion queue (USER reviews)
- Beautiful, informative interfaces

**Philosophy Alignment:**
- Dashboard SHOWS information
- Dashboard NEVER acts
- All interactions user-initiated

---

### v7.9.0 - "The Vigilant Core" (Jun 2026)
**Theme:** Resilience through awareness, NOT self-repair

**CRITICAL:** This version was redesigned to avoid automation disasters.

**Major Features:**
- 🆕 Intent 071: Drift Advisor (NOT Self-Healing)
- 🆕 Intent 072: End-to-end testing
- Enhancement: Guided drift resolution
- Enhancement: Risk analysis and reporting

**What Changed from Original Design:**
- ❌ REMOVED: Automatic drift correction
- ❌ REMOVED: Self-healing behaviors
- ❌ REMOVED: Background fixes
- ✅ ADDED: Drift detection and reporting
- ✅ ADDED: Guided recovery workflows
- ✅ ADDED: Risk assessment

**New Capabilities:**
```bash
doctor
# Detects drift automatically
# → "⚠️ Config drift detected in sway/config"
# → "Line 42: Manual edit at 14:23"
# → "Risk: Low (keybind only)"
# → "Actions: [diff] [restore] [keep] [ignore]"
# → WAITS for user choice

drift analyze
# → Shows all drift with risk levels
# → Explains each change
# → SUGGESTS fixes (never applies)

drift restore sway/config
# → Shows diff
# → "Restore to Intent 040 version? [yes/no]"
# → WAITS for explicit yes
```

**Philosophy Alignment:**
- System DETECTS problems
- System EXPLAINS risks
- System OFFERS solutions
- System WAITS for decision
- USER chooses action

**Guard Rails:**
- No automatic fixes
- All changes require confirmation
- Risk levels clearly shown
- Rollback always available

---

### v8.0.0 - "THE OBSERVANT FOREST" (Jul 2026) 🌲👁️
**Theme:** Intelligence WITH you, not FOR you

**The Grand Integration:**
- 🆕 Intent 073: Observant forest integration
- 🆕 Intent 074: v8.0 migration guide
- 🆕 Intent 075: Complete documentation

**What Makes It "Observant" (NOT "Sentient"):**

1. **Self-Aware** - Knows its state, reports it
2. **Pattern-Aware** - Learns your habits, shows them
3. **Context-Aware** - Understands situations, explains them
4. **Risk-Aware** - Analyzes changes, warns you
5. **Time-Aware** - Remembers history, offers rollbacks
6. **Suggestion-Ready** - Has recommendations, waits to share
7. **User-Controlled** - Respects your authority ALWAYS

**The Magic Moment (Redesigned):**
```bash
You: "Core, I broke something yesterday"

Core: 
"📊 Analysis of changes from 2026-01-17:
- 3 config modifications detected
- Most likely cause: sway/config line 42
- Risk level: Medium
- Intent affected: 040 (Launcher)

Options:
[d] Show detailed diff
[r] Restore to Intent 040 version  
[k] Keep current version
[t] Timeline view
[?] Explain risk assessment

Your choice: _"

[CORE WAITS. DOES NOTHING.]
```
```bash
You: [types 'd']

Core:
"Diff of sway/config:
- bindsym $mod+Space exec faelight-launcher
+ bindsym $mod+Space exec fuzzel

This changes your launcher keybind.
Likely consequence: Different launcher opens.
Risk: Low (functionality preserved)

Restore to Intent 040? [yes/no/keep-and-update-intent]: _"

[STILL WAITING]
```

**Key Differences from Original "Sentient" Vision:**

| Original (WRONG) | Corrected (RIGHT) |
|------------------|-------------------|
| Learns and applies | Learns and suggests |
| Predicts and executes | Predicts and offers |
| Fixes automatically | Reports and guides |
| Acts in background | Waits for foreground command |
| "I fixed it" | "I found it. What should I do?" |

**Philosophy Compliance:**

✅ "Nothing runs without explicit trigger"
✅ "Asks before acting"  
✅ "Speaks when uncertain"
✅ "Fails loudly"
✅ "Never lies about what it's doing"
✅ "We reject invisible complexity"
✅ "Intent over automation"
✅ "Recovery, not perfection"

**What It Will NEVER Do:**

❌ Run at boot without permission
❌ Apply changes in background
❌ Assume you want something
❌ Execute privileged commands automatically
❌ Hide what it's doing
❌ Make decisions for you

**Breaking Changes:**
- Old automation scripts incompatible
- All triggers now require explicit confirmation
- Configuration format updated
- Philosophy enforced in code

---

## Incident 001 Compliance

**Every feature checked against the 2025-12-14 lessons:**

### The Test Questions:
1. ❓ Could this run at boot? → **If yes, redesign**
2. ❓ Could this act without user present? → **If yes, redesign**
3. ❓ Could this assume context? → **If yes, redesign**
4. ❓ Could this lock me out? → **If yes, DELETE**

### Applied to Each Version:

**v7.5 Security Audit:**
- ✅ Runs on explicit `security-audit` command
- ✅ Reports findings, suggests fixes
- ✅ Never applies fixes automatically

**v7.6 Time Machine:**
- ✅ User browses history
- ✅ User chooses restore points
- ✅ System shows preview, asks permission

**v7.7 Pattern Observer:**
- ✅ Watches silently
- ✅ Reports when asked
- ✅ Suggests when prompted
- ✅ Acts only on explicit yes

**v7.8 Dashboard:**
- ✅ Display only
- ✅ No actions triggered
- ✅ Pure information

**v7.9 Drift Advisor:**
- ✅ Detects passively
- ✅ Reports on doctor run
- ✅ Suggests fixes
- ✅ Waits for user command

**v8.0 Integration:**
- ✅ All subsystems ask first
- ✅ No background automation
- ✅ User always in control

---

## Version Mapping

| Version | Existing Intents | New Intents | Theme |
|---------|-----------------|-------------|-------|
| v7.5.0 | 051, 055 | 058, 059, 060 | Foundation |
| v7.6.0 | 053, 054, 056 | 062, 063 | Time Machine |
| v7.7.0 | 052, 057 | 065, 066, 067 | Observer |
| v7.8.0 | - | 068, 069, 070 | Dashboard |
| v7.9.0 | - | 071, 072 | Vigilant |
| v8.0.0 | - | 073, 074, 075 | Integration |

---

## Success Criteria

**v8.0.0 is complete when:**
- [ ] All intents 051-075 implemented
- [ ] System health: 100%
- [ ] ZERO automatic actions
- [ ] Every feature asks permission
- [ ] Pattern detection working
- [ ] Suggestion system functional
- [ ] Time travel reliable
- [ ] Universal search <50ms
- [ ] Dashboard informative
- [ ] Drift detection accurate
- [ ] Documentation complete
- [ ] **Philosophy test: Pass all 4 questions**
- [ ] **Incident 001 test: Could this cause lockout? NO**

---

## Timeline Summary
```
v7.4.0 ──────► v7.5.0 ──────► v7.6.0 ──────► v7.7.0
  NOW      Foundation    Time Machine    Observer
  Jan        Feb             Mar            Apr

──────► v7.8.0 ──────► v7.9.0 ──────► v8.0.0
      Dashboard     Vigilant    OBSERVANT
         May            Jun           Jul
```

**Total Duration:** ~6 months  
**Releases:** 6 major versions  
**New Intents:** 17 (058-075, corrected)  
**Existing Intents:** 7 (051-057)

---

## The Philosophy

> "We build not just a system, but an advisor.  
> Not just tools, but awareness.  
> Not just code, but consciousness of what YOU want.  
> The forest watches, learns, and suggests."

**The journey from v7.4 to v8.0 is about creating intelligence that:**
- Observes without judgment
- Learns without assuming
- Suggests without insisting
- Waits without impatience
- Respects your authority absolutely

**The system that acts for you will eventually act against you.**  
**The system that thinks with you will never betray you.**

---

## Lessons from Incident 001

**What the 2025-12-14 incident taught us:**

1. Automation at boot = unpredictable context
2. Sudo in automation = authentication disaster
3. "Convenience" without consent = system lockout
4. Background jobs = invisible failures
5. Assumed context = brittle systems

**What this roadmap embodies:**

1. Everything explicit = predictable behavior
2. No privileged automation = no auth failures
3. Permission always = user in control
4. Foreground only = visible operations
5. Asked context = robust design

**The incident created 0-Core's soul.**  
**This roadmap honors that soul.**

---

_"The observant forest watches. The observant forest learns. The observant forest suggests. The observant forest waits."_ 🌲👁️
