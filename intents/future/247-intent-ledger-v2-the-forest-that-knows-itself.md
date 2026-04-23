---
id: 247
title: "Intent Ledger v2 -- The Forest That Knows Itself"
status: planned
date: 2026-04-22
tags: [intelligence, intent-ledger, friday, core, planning, velocity, dependency, retrospective, awareness, v2]
---
The current Intent Ledger is a filing cabinet.
It stores what you planned.
It records what you did.
It does not think.
The forest has 195 completed intents.
537 Friday observations.
2300+ commits.
Patterns everywhere.
And the ledger sees none of it.
Intent Ledger v2 is not an upgrade.
It is the forest becoming self-aware.
It knows what you are building.
It knows what is blocking you.
It knows what you should do next.
It knows what you should NOT do next.
It tells you before you ask.
"The forest does not stumble into its future.
It grows toward the light it has already mapped."
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
THE PROBLEM WITH THE CURRENT LEDGER
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
The current ledger is passive markdown files in a directory.
It has no memory beyond what you manually wrote.
It has no awareness of what is happening in the system.
It cannot see patterns across 195 completed intents.
It cannot warn you when you are making a mistake.
It cannot tell you which intent will unblock three others.
It cannot measure your real velocity versus your planned velocity.
It cannot tell you when you are in a flow state versus grinding.
It cannot see that INT-232 took 6 sessions when you planned 3.
It cannot see that every intent involving Wayland took 2x longer than planned.
It cannot see that intents tagged [v2] have a 94% completion rate.
It cannot tell you: "Stop. You have 3 active intents. Pick one."
The ledger is blind.
v2 opens its eyes.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
VISION: WHAT THE LEDGER BECOMES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
The Intent Ledger v2 is a living intelligence layer.
Before every session:
  The ledger greets you with a brief.
  "Good morning. You have 1 active intent.
   INT-232 Phase 3 is next. Friday estimates 2 sessions.
   Your velocity this week: 8 commits/session (above average).
   Recommended: continue INT-232. Do not start anything new."
During every session:
  The ledger watches. Through Friday. Through state.db.
  When you drift from the active intent, it notices.
  When you hit a pattern that previously blocked you, it surfaces the fix.
  When you have been on the same gate for 45 minutes, it asks if you need help.
After every session:
  The ledger writes a retrospective automatically.
  "Session 3 of INT-232: 5 commits. Phase 1 complete.
   Actual time: 4h. Planned: 2h. Delta: +2h.
   Cause: copy/paste architecture conflict (2 approaches tried).
   Learning: Wayland clipboard requires subprocess approach, not in-process.
   Friday confidence: +3% on Wayland complexity estimates."
When you complete an intent:
  The ledger generates a full post-mortem.
  Gates completed. Gates deferred. Time delta. Key decisions made.
  What future intents should know.
  Automatically seeds Friday knowledge engine.
When you plan a new intent:
  The ledger challenges you.
  "You are planning INT-248. It depends on INT-234 (not started).
   Similar intents took 3-5 sessions. You have 52 days to NY presentation.
   INT-234 + INT-248 = ~8 sessions. That leaves 12 sessions for everything else.
   Is this the right sequence?"
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 1: DEPENDENCY INTELLIGENCE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Every intent has dependencies. Currently they are buried in text.
v2 makes dependencies first-class citizens.
DEPENDENCY GRAPH:
  Every intent declares: depends_on: [INT-219, INT-234]
  The ledger builds a live dependency graph.
  Stored in state.db. Queryable. Visualizable.
  core intent-graph
  -> Full ASCII dependency tree
  -> Critical path highlighted in amber
  -> Blocked intents marked in red
  -> Ready-to-start intents marked in green
  core intent-deps INT-248
  -> INT-248 depends on: INT-234 (not started), INT-232 Phase 3 (in-progress)
  -> Blocked by: INT-234
  -> Unblocks when complete: INT-249, INT-251, INT-255
  -> Critical path: INT-232 -> INT-234 -> INT-248 -> INT-244
DEPENDENCY ENFORCEMENT:
  cistart INT-248 fails if INT-234 is not complete.
  Not silently. With explanation and override option.
  "Cannot start INT-248: depends on INT-234 (planned, not started).
   Complete INT-234 first, or: cistart INT-248 --override reason"
CRITICAL PATH ANALYSIS:
  The ledger knows the NY presentation deadline: mid-July 2026.
  It knows which intents are on the critical path.
  It surfaces this every session automatically.
  "5 intents on critical path. 3 not started.
   At current velocity: 2 sessions behind schedule.
   Recommend: deprioritize INT-236, focus on critical path."
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 2: VELOCITY INTELLIGENCE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
You think you know how long things take.
You do not.
The ledger does.
REAL VELOCITY TRACKING:
  Every intent: planned sessions vs actual sessions.
  Every gate: planned time vs actual time.
  Every phase: planned complexity vs actual complexity.
  All stored in state.db. All analyzed by Friday.
VELOCITY PATTERNS FRIDAY DISCOVERS:
  "Intents tagged [wayland] take 2.3x planned time."
  "Intents tagged [rust,new-crate] take 1.8x planned time."
  "Intents in [intelligence] category complete in 0.9x planned time."
  "Your velocity is highest Tuesday-Thursday afternoon."
  "Your velocity drops sharply after 3 consecutive sessions without rest."
PLANNING CALIBRATION:
  When you estimate 2 sessions for a new intent:
  "You estimated 2 sessions for similar intents 8 times.
   Actual average: 3.4 sessions.
   Calibrated estimate: 3-4 sessions. Adjust your plan."
BURN-DOWN AWARENESS:
  core intent-burndown            -- sessions remaining vs deadline
  core intent-velocity            -- your real pace this week vs historical
  core intent-estimate INT-245    -- AI-calibrated time estimate with history
  core intent-reforecast          -- recalculate all estimates with real data
FLOW STATE DETECTION AND PROTECTION:
  Friday watches commit frequency and session length.
  High commit rate + long session + low errors = flow state.
  "Flow state detected: 8 commits in 90 minutes.
   Suppressing non-critical Friday suggestions until flow breaks.
   Current gate: on track."
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 3: HEALTH CORRELATION ENGINE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
The forest has health. Intents affect health.
Currently no one knows which intents cause health drops.
v2 knows. And it warns you before the drop happens.
HEALTH IMPACT TRACKING:
  Every health check timestamped and correlated to active intent.
  Every health drop analyzed for cause.
  "INT-232 Phase 1: 3 health warnings (uncommitted changes).
   Pattern: develop -> commit frequency too low.
   Fix applied each time: gc. Lesson: commit every gate, not every phase."
INTENT RISK SCORING:
  Low risk: documentation, planning, minor improvements.
  Medium risk: new commands, tool upgrades, config changes.
  High risk: system changes, new dependencies, architecture shifts.
  Critical risk: shell loop changes, state.db schema, Wayland protocol.
  core intent-risk INT-245
  -> Risk: 72/100 (HIGH)
  -> Reasons: modifies core shell loop, affects all 359 aliases
  -> No rollback path without git restore
  -> Recommendation: full commit + d before starting
PRE-INTENT HEALTH GATE:
  cistart checks health AND risk before allowing start.
  "Health: 100%. Risk: 72/100 (HIGH).
   Last 3 high-risk intents: 2/3 caused health drops.
   Action required before proceeding:
   1. Run d to confirm 100% health
   2. Run gc to ensure clean working tree
   3. Confirm: proceed? (y/n)"
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 4: AUTOMATED RETROSPECTIVES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Currently: when an intent completes, nothing is systematically learned.
v2: every completion triggers a structured retrospective.
Automatically. No manual writing required. Ever.
AUTO-RETROSPECTIVE (generated by Friday from state.db on cicomplete):
  INT-232 RETROSPECTIVE -- auto-generated 2026-04-22
  -----------------------------------------------
  Planned: 3 sessions  |  Actual: 6 sessions  |  Delta: +3
  Commits: 87          |  Per session: 14.5 avg
  Health events: 2 drops, both self-resolved within same session
  Gates: 12 complete, 3 deferred, 3 in-progress
  -----------------------------------------------
  LONGER THAN EXPECTED:
  - Copy/paste: 3 sessions vs 1 planned
    Cause: Wayland clipboard cannot share event loop connection
    Lesson: subprocess approach (wl-paste) is the only correct path
  - Status strip: 1.5 sessions vs 0.5 planned
    Cause: Python-Rust file corruption pattern (str mode vs binary mode)
    Lesson: always binary mode when writing Rust files with escape sequences
  - Ligatures: deferred entirely
    Cause: per-cell rendering fundamentally incompatible with ligature context
    Lesson: ligatures require per-row shaping -- separate future intent
  -----------------------------------------------
  FASTER THAN EXPECTED:
  - 256-color + truecolor: single session
  - Scrollback: better than v1 within 1 session
  - Resize content preservation: first attempt
  -----------------------------------------------
  KEY DECISIONS MADE:
  - DEC-006: SHM rendering kept, wgpu deferred to Phase 4
  - DEC-007: Per-cell rendering chosen, ligatures deferred
  - DEC-008: wl-paste subprocess, not wl-clipboard-rs
  -----------------------------------------------
  SEEDED TO FRIDAY KNOWLEDGE:
  - Wayland clipboard: subprocess only
  - Python writing Rust: binary mode required
  - cosmic-text per-cell: correct alignment, no ligatures
  -----------------------------------------------
  SPAWNED: INT-245 (friction), INT-246 (Friday arch v2), INT-247 (ledger v2)
  DEFERRED: ligatures (future intent), wgpu (Phase 4)
  -----------------------------------------------
RETROSPECTIVE COMMANDS:
  core intent-retro INT-232       -- view any retrospective
  core intent-retro-all           -- summary across all retrospectives
  core intent-lessons             -- distilled lessons from all retros
  core intent-patterns            -- recurring patterns across intents
KNOWLEDGE AUTO-SEEDING:
  Every retrospective seeds friday_knowledge automatically.
  No manual core knowledge add required.
  The forest learns from every completed intent.
  Always. Without being asked.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 5: SESSION INTELLIGENCE BRIEFS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Currently: every session starts cold.
You remember what you were doing. Sometimes.
v2: every session starts with full context. Instantly.
SESSION BRIEF (shown on fsh start, generated in under 100ms):
  SESSION BRIEF -- Apr 23 2026 -- 09:14
  Last session: 14h ago -- 15 commits -- 4h
  -----------------------------------------------
  ACTIVE: INT-232 Phase 3 (Intelligence)
  Next gate: Friday panel (Ctrl+Shift+F)
  Estimated remaining: 1-2 sessions
  Deadline pressure: LOW (52 days to NY)
  Health: 100% -- stable
  -----------------------------------------------
  FRIDAY REMEMBERS:
  Last session you completed the status strip.
  Blocker hit: borrow conflict in render() with build_status_text.
  Fix applied: moved build_status_text call before render loop.
  Committed and pushed. Ready to continue.
  -----------------------------------------------
  RECOMMENDED FIRST ACTION:
  cistart INT-232 -- implement Friday panel gate
  Confidence: 89% -- similar gate completed in 1 session before
  -----------------------------------------------
BRIEF COMMANDS:
  core intent-brief               -- current session brief
  core intent-brief --full        -- extended brief with full context
  core intent-yesterday           -- detailed view of last session
  core intent-last-blocker        -- what was blocking you?
  core intent-resume              -- resume exactly where you left off
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 6: CONTRADICTION DETECTION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Friday already detects contradictions at 85% confidence.
The ledger makes these actionable. Not just observed. Resolved.
CONTRADICTION TYPES:
  FOCUS VIOLATION:
  "3 active intents. Values declare focus>speed.
   You have been in this state for 3 sessions.
   The forest is spreading too thin.
   Recommended action: cicomplete INT-232 before touching INT-245."
  SCOPE CREEP:
  "INT-232 has grown ~40% beyond original spec.
   Original: 5 phases. Now: 5 phases + caret color + shell changes.
   This pattern preceded rework in 4 previous intents.
   Recommended action: create INT-247a for overflow work."
  DEPENDENCY VIOLATION:
  "You are working on INT-245 (fsh v9).
   INT-245 depends on INT-234 (Core v21, not started).
   You are building on a foundation that does not exist yet.
   This caused full rework in INT-186 and INT-205.
   Recommended action: pause INT-245, complete INT-234 first."
  VELOCITY DEBT:
  "Missed estimated completion on last 3 intents.
   Cumulative debt: +8 sessions behind original plan.
   NY deadline: 52 days. At current pace: 2 intents short.
   Immediate action: deprioritize INT-236, INT-239 -- not on critical path."
COMMANDS:
  core intent-contradictions      -- all active with recommended actions
  core intent-resolve 1 reason    -- resolve with documented reason
  core intent-values              -- your declared values vs actual behavior delta
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 7: INTENT GENEALOGY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Intents do not exist in isolation.
INT-232 spawned INT-245 and INT-246 and INT-247.
INT-216 informed INT-220 and INT-246.
INT-133 was the foundation for INT-148 and INT-151 and INT-162.
Currently this lineage is invisible.
v2 makes genealogy a first-class citizen of the forest.
GENEALOGY TRACKING:
  Every intent frontmatter declares:
    spawned_from: INT-216
    spawns: [INT-245, INT-246]
  The ledger builds a living family tree in state.db.
  Queryable. Visualizable. Permanent.
GENEALOGY COMMANDS:
  core intent-tree INT-232        -- full ancestry and descendant tree
  core intent-ancestors INT-245   -- what led to this intent existing?
  core intent-descendants INT-133 -- everything this intent created
  core intent-siblings INT-232    -- intents from the same parent
  core intent-lineage             -- full forest genealogy visualization
IN EVERY RETROSPECTIVE:
  "This intent spawned: INT-245, INT-246, INT-247
   Deferred from this intent: ligatures (scope), wgpu (Phase 4)
   Forest impact: 3 intents created, 2 scopes deferred to dedicated intents"
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 8: PRIORITY SCORING ENGINE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Currently: priority is implicit. Christian decides. No data behind it.
v2: every intent has a computed priority score.
Transparent. Explainable. Challengeable. Always current.
PRIORITY SCORE COMPONENTS (0-100 total):
  deadline_pressure   (0-30): days to deadline vs sessions needed
  dependency_unlock   (0-25): how many intents does this unblock?
  health_risk         (0-20): inverse of health impact probability
  friday_confidence   (0-15): Friday confidence of success
  velocity_match      (0-10): fit with current pace and energy level
COMMANDS:
  core intent-priority            -- full ranked list with scores
  core intent-priority --explain INT-245  -- full score breakdown
  core intent-next                -- what should you work on right now?
NEXT RECOMMENDATION (never just a name, always an explanation):
  "Recommended next: INT-234 (Core v21 -- Friday Planning Layer)
   Priority score: 91/100
   Why this intent:
   - Unblocks 3 critical intents (INT-244, INT-245, INT-248)
   - 52 days to deadline, estimated 3 sessions -- fits comfortably
   - Friday confidence: 87% (similar Core intents completed successfully)
   - Pure Rust, no system changes -- health risk: LOW
   Why not INT-239 (faelight-bar v2):
   - Priority score: 45/100
   - Does not unblock anything on critical path
   - Good choice only if you need a lower-intensity recovery session
   Why not INT-245 (fsh v9):
   - Priority score: 67/100
   - Depends on INT-234 (not complete) -- dependency violation
   - Starting now means rework later"
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TECHNICAL ARCHITECTURE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
All intelligence lives in state.db.
No new files. No new daemons. No new services.
The ledger is an intelligence layer on top of what already exists.
NEW STATE.DB TABLES:
  intent_dependencies
    (from_id INT, to_id INT, dep_type TEXT, created_at INT)
  intent_velocity
    (intent_id INT, planned_sessions INT, actual_sessions INT, delta REAL)
  intent_retrospectives
    (intent_id INT, generated_at INT, content_json TEXT)
  intent_genealogy
    (parent_id INT, child_id INT, relationship_type TEXT)
  intent_health_events
    (intent_id INT, session_id TEXT, health_before INT, health_after INT, cause TEXT)
  session_briefs
    (session_id TEXT, generated_at INT, brief_json TEXT)
  priority_scores
    (intent_id INT, score REAL, components_json TEXT, computed_at INT)
NEW CORE COMMANDS:
  core intent-graph               -- ASCII dependency tree
  core intent-velocity            -- velocity analysis with patterns
  core intent-next                -- AI recommendation with full explanation
  core intent-retro INT-N         -- generate or view retrospective
  core intent-brief               -- session brief in <100ms
  core intent-contradictions      -- active contradictions with actions
  core intent-priority            -- priority rankings with score breakdown
  core intent-burndown            -- deadline forecast with session estimate
  core intent-tree INT-N          -- genealogy tree
  core intent-risk INT-N          -- health risk assessment before starting
  core intent-blocked             -- what is blocked and why
  core intent-unblocked           -- what just became startable
  core intent-critical-path       -- deadline-critical sequence
  core intent-lessons             -- distilled lessons from all retrospectives
  core intent-resume              -- resume exactly where you left off
CISTART / CICOMPLETE UPGRADES:
  cistart INT-N:
    1. Checks dependencies (blocks with explanation if violated)
    2. Checks health risk (warns, requires confirmation for HIGH risk)
    3. Shows session brief for this intent
    4. Starts velocity timer
  cicomplete INT-N:
    1. Stops velocity timer, records actual sessions
    2. Triggers auto-retrospective generation
    3. Seeds friday_knowledge from retrospective
    4. Updates priority scores for all dependent intents
    5. Shows what intents just became unblocked
    6. Recommends next intent
FRIDAY DEEP INTEGRATION:
  Friday reads intent_retrospectives -> seeds friday_knowledge automatically
  Friday reads intent_velocity -> calibrates all future estimates
  Friday reads intent_health_events -> predicts risk for new intents
  Friday reads session_briefs -> personalizes each session greeting
  Friday confidence scores feed directly into priority_scores
  Friday detects contradictions -> writes to contradiction queue
  Friday detects flow state -> suppresses non-critical interrupts
FAELIGHT-TERM INTEGRATION (INT-232 Phase 3+):
  Ctrl+Shift+I: intent brief overlay inside terminal
  Status strip: active intent + priority score + deadline pressure indicator
  Friday panel: session brief + next gate + last blocker + recommendation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
GATES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Phase 1 -- Foundation (state.db schema):
⬜ intent_dependencies table created and seeded from existing intents
⬜ intent_velocity table live -- tracking starts immediately on all intents
⬜ intent_genealogy table seeded from all known parent-child relationships
⬜ intent_health_events table correlated to existing health history
⬜ session_briefs table live -- brief generated on every fsh start
Phase 2 -- Core Intelligence Commands:
⬜ core intent-next -- priority recommendation with full explanation
⬜ core intent-priority -- full ranked list with score breakdown per intent
⬜ core intent-graph -- ASCII dependency tree renders correctly in terminal
⬜ core intent-velocity -- real vs planned with Friday pattern analysis
⬜ core intent-burndown -- deadline forecast with per-session projection
⬜ core intent-brief -- session brief generated from state.db in under 100ms
Phase 3 -- Dependency Enforcement:
⬜ cistart validates all dependencies -- blocked intents require --override
⬜ core intent-blocked -- shows what is blocked and the exact reason
⬜ core intent-unblocked -- shows what cicomplete just made startable
⬜ core intent-critical-path -- deadline-critical sequence clearly highlighted
⬜ Dependency graph updates automatically on every cicomplete
Phase 4 -- Retrospective Engine:
⬜ cicomplete triggers auto-retrospective generation without any manual input
⬜ Retrospective automatically seeds friday_knowledge entries
⬜ core intent-retro INT-N -- view full retrospective for any intent
⬜ core intent-lessons -- Friday-distilled patterns from all retrospectives
⬜ Retrospectives backfilled for last 20 completed intents from state.db history
Phase 5 -- Session Briefs:
⬜ Brief generated on every fsh start -- non-blocking, under 100ms
⬜ Brief shows active intent, next gate, last blocker, deadline pressure
⬜ Brief includes Friday memory of previous session blockers
⬜ core intent-resume -- picks up exactly where the last session ended
⬜ Brief shown in faelight-term status strip (Ctrl+Shift+I overlay)
Phase 6 -- Contradiction Engine:
⬜ Focus violation detected when more than 1 active intent for more than 2 sessions
⬜ Scope creep detected when gate count grows beyond original specification
⬜ Dependency violation detected when working on a blocked intent
⬜ Velocity debt detected and surfaced when schedule slippage accumulates
⬜ core intent-contradictions -- all active contradictions with recommended actions
Phase 7 -- Genealogy:
⬜ Genealogy fields added to intent frontmatter schema
⬜ core intent-tree -- full ancestry and descendant visualization
⬜ core intent-lineage -- forest-wide genealogy view
⬜ Retrospectives include spawn and defer accounting automatically
⬜ 195 completed intents genealogy backfilled where knowable
Final Validation Gates:
⬜ The ledger recommends the next intent BEFORE Christian asks
⬜ The ledger catches a real dependency violation and prevents rework
⬜ A retrospective seeds a Friday knowledge entry that helps the next session
⬜ Session brief surfaces a real blocker from the previous session
⬜ At least one contradiction detected, surfaced, and correctly resolved
⬜ Christian says: "The ledger knew something I had forgotten."
"The forest has always been growing.
Now it knows where it is going.
Not because you told it.
Because it remembers everything that came before
and can see everything that comes next.
195 intents completed.
Each one a lesson.
Each one a seed.
The forest does not repeat its mistakes.
The forest does not forget its victories.
The forest does not stumble into its future.
It knows its dependencies.
It knows its velocity.
It knows its health.
It knows what it has spawned.
It knows what it has learned.
It knows what comes next.
Intent Ledger v2.
The forest that knows itself." 🌲
