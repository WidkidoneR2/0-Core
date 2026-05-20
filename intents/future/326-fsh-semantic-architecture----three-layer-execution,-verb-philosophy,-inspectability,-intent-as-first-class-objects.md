---
id: 326
title: "fsh Semantic Architecture -- three-layer execution, verb philosophy, inspectability, intent as first-class objects"
status: planned
date: 2026-05-20
tags: [fsh, shell, semantic, architecture, intent, vocabulary, philosophy, inspectability, layers]
---

INT-326 -- fsh Semantic Architecture -- The Shell That Thinks In Three Layers
date: 2026-05-20

---
THE PREMISE

Every shell ever built operates on one layer.
You type a command. It executes. Text comes out.
The shell is a pipe, not a mind.

fsh already breaks this with vocabulary -- delete instead of rm,
find instead of locate, the ? prefix for natural language.

But the vocabulary is the surface.
The architecture underneath is what makes it permanent.

INT-326 defines the three-layer execution model that makes fsh
the first shell where human intent and machine execution
are permanently, visibly, and verifiably connected.

Not a natural language shell.
Not an AI executor.
A semantic machine with a human face.
---
THE THREE LAYERS

Every fsh command passes through three layers.
Every layer is visible. Every layer is inspectable.

Layer 1 -- Human Intent:
  What you say.
  Natural language or vocabulary words.
  Examples:
    repair audio
    show health
    clean downloads older than 30 days
    deploy bar and commit
  This layer is optimized for human expression.
  Ambiguity is allowed here. Resolution happens next.

Layer 2 -- Semantic Plan:
  What fsh understands.
  A structured, typed representation of the intent.
  Examples:
    repair audio -> SemanticPlan {
        action: Repair,
        target: Service("audio"),
        steps: [
            Inspect("pipewire"),
            Inspect("wireplumber"),
            Verify("sinks"),
            Restart("affected"),
        ],
        confidence: 0.94,
        reversible: true,
    }

    show health -> SemanticPlan {
        action: Observe,
        target: System("health"),
        steps: [Run("core doctor --summary")],
        confidence: 1.0,
        reversible: true,  // observation never mutates
    }
  This layer is where Friday reasons.
  This layer is where safety checks happen.
  This layer is where ambiguity is resolved.

Layer 3 -- Concrete Execution:
  What actually runs.
  Real commands, real system calls, real Rust functions.
  Examples:
    systemctl --user restart pipewire
    pw-cli info all
    core doctor run --summary
  This layer is UNIX-compatible.
  Pipes, text streams, exit codes -- all preserved.
  Nothing magical. Nothing opaque.

The key principle:
  Layer 1 is for humans.
  Layer 2 is for the forest.
  Layer 3 is for the machine.
  All three are always accessible.
---
INSPECTABILITY -- THE NON-NEGOTIABLE PRINCIPLE

Human language must never become opaque.

If you type:
  show health

You can always type:
  explain

And see:
  Layer 1: show health
  Layer 2: Observe(System("health")) -- confidence 1.0
  Layer 3: core doctor run --summary
  Execution plan:
    - collect metrics
    - integrity scan
    - health forecast
    - render digest

This is not optional. This is the contract.
Every semantic command is explainable.
Every translation is visible.
Every execution plan is editable before running.

Inspectability commands:
  explain         -- show all three layers for last command
  explain <cmd>   -- show layers for any command without running it
  plan <cmd>      -- show Layer 2 semantic plan only
  dry-run <cmd>   -- show Layer 3 execution without running it
  why <cmd>       -- show why fsh interpreted it this way
---
DETERMINISTIC INTERPRETATION -- THE SAFETY PRINCIPLE

The warning to avoid:
  AI decides what you meant.

The principle to follow:
  grammar + confidence + explicit resolution.

When a command is ambiguous, fsh does NOT silently guess.
fsh presents possibilities with confidence scores:

  clean downloads

  fsh detected ambiguity (confidence: 0.61):
  1. Delete temporary files older than 7 days (0.71)
  2. Archive files older than 30 days (0.58)
  3. Remove duplicates (0.43)
  Which? (1/2/3 or explain N):

After you choose:
  fsh learns your preference.
  Next time "clean downloads" confidence rises toward 1.0.
  After 3 consistent choices: no prompt, direct execution.

This creates:
  A shell that learns your preferences deterministically.
  Not by guessing -- by observing explicit choices.
  Friday records the preference in state.db.
  The shell becomes personalized without becoming unpredictable.

Resolution rules (in order):
  1. Exact vocabulary match (confidence 1.0) -- execute immediately
  2. High confidence single interpretation (>0.85) -- execute with note
  3. Multiple interpretations (any >0.5) -- present choices
  4. No interpretation (<0.5) -- ask Friday, suggest closest match
  5. Unknown -- treat as raw UNIX command (Layer 3 direct)
---
INTENT AS FIRST-CLASS OBJECTS

Commands produce text.
fsh produces intent objects.

Internal representation:

  struct SemanticIntent {
      id: Uuid,                      // unique per invocation
      raw_input: String,             // Layer 1: what you typed
      action: Action,                // Archive, Observe, Repair, Deploy, etc.
      target: Target,                // File, Service, System, Tool
      constraints: Vec<Constraint>,  // older_than_days: 30, etc.
      confidence: f64,               // interpretation confidence
      reversible: bool,              // can this be undone?
      requires_confirm: bool,        // destructive actions
      execution_plan: Vec<Step>,     // Layer 3 steps
      intent_id: Option<u32>,        // links to active forest intent
      timestamp: i64,
  }

What this enables:
  Friday can reason about it:
    "You've run Archive(Downloads) 12 times -- create abbreviation?"
  Logs become semantic:
    Not "rm -rf /tmp/old" but "Archive(tmp, older_than=7d)"
  Replay becomes possible:
    replay intent 2026-05-18
    Re-runs every semantic intent from that date
  Prediction improves:
    Friday sees patterns in action types, not just command strings
  Safety improves:
    reversible=false triggers confirmation regardless of vocabulary
  Audit trail:
    Every action is logged with its full semantic context
---
THE VERB PHILOSOPHY

Verbs are not arbitrary.
Each verb carries a contract about what it does to system state.

The forest verb taxonomy:

OBSERVATION verbs (never mutate state):
  show       -- display information
  inspect    -- examine in detail
  explain    -- show reasoning and plans
  list       -- enumerate items
  compare    -- diff two things
  check      -- verify a condition
  status     -- current state of something
  history    -- past events

  Contract: observation verbs NEVER have side effects.
  A user should always be able to type "show X" without fear.

RECOVERABLE ACTION verbs (mutate, but reversibly):
  repair     -- fix a broken thing (reversible)
  align      -- bring to policy-compliant state (reversible)
  move       -- relocate something (reversible)
  rename     -- change name (reversible)
  archive    -- preserve but move aside (reversible)
  enable     -- activate something (reversible)
  disable    -- deactivate something (reversible)

  Contract: these actions can always be undone.
  fsh captures state before execution.
  "undo" is always available after these verbs.

DESTRUCTIVE verbs (mutate irreversibly, require confirmation):
  delete     -- permanent removal (confirm required)
  purge      -- aggressive removal (confirm + reason required)
  wipe       -- complete erasure (confirm + type "wipe" to proceed)
  revoke     -- remove access permanently

  Contract: these always show what will be destroyed.
  Always require explicit confirmation.
  Always log to forest audit trail.
  Friday warns if pattern is unusual.

DEPLOYMENT verbs (system-changing, tracked):
  deploy     -- release a tool or service
  rollback   -- revert to previous state
  upgrade    -- advance to newer version
  install    -- add to system
  remove     -- uninstall from system

  Contract: all deployment verbs write to deploy_patterns.
  All are rollback-capable via the existing deploy system.
  Friday tracks success rates per tool.

INTELLIGENCE verbs (Friday-mediated):
  predict    -- what will happen if...
  suggest    -- what should I do about...
  analyze    -- deep examination with Friday's reasoning
  learn      -- explicitly teach Friday something
  forget     -- remove something from Friday's memory

  Contract: these invoke Friday.
  Results include confidence scores.
  Never execute without showing the plan first.

SESSION verbs (state management):
  save       -- preserve current session state
  restore    -- return to a saved state
  replay     -- re-execute a past session or command
  rewind     -- go back in time (time-travel debugging)
  snapshot   -- capture system state at this moment

  Contract: session verbs operate on the forest's memory.
  Always safe to run. Always reversible.

The verb taxonomy is the grammar of the forest.
Every future vocabulary word is assigned to a category first.
The category determines the safety contract automatically.
---
SEMANTIC PIPELINES

Traditional UNIX shell:
  ps aux | grep firefox | awk '{print $2}' | xargs kill

fsh semantic pipeline:
  show firefox processes | filter memory > 1gb | terminate gracefully

Internally, each stage is a typed transformation:
  show firefox processes
    -> Iterator<Process>  -- typed stream of Process objects

  | filter memory > 1gb
    -> Iterator<Process>  -- same type, filtered

  | terminate gracefully
    -> Iterator<Result<(), Error>>  -- results

The pipeline preserves:
  Type information at each stage
  Row counts (visible with --explain)
  Timing (visible with --explain)
  Reversibility (graceful = recoverable, kill = not)

This is exactly what the current structured pipeline does with:
  intents | where status = active | first 3

INT-326 formalizes and extends this to ALL commands.
Not just forest commands -- any semantic command.

UNIX compatibility is PRESERVED:
  fsh always falls back to raw UNIX at Layer 3.
  Any pipeline can be inspected to see the actual UNIX commands.
  Any pipeline can be exported to a shell script.

The duality:
  semantic mode:  show firefox processes | filter memory > 1gb
  UNIX mode:      ps aux | grep firefox | awk '{print $2}'
  Both work. Both are supported. Neither is deprecated.
---
HUMAN FIRST, UNIX EXACT

The design principle in four words.

What the user says:
  compare yesterday's deploy with current

What fsh compiles to:
  core compare --deploy HEAD~1 HEAD

What fsh shows before running:
  Interpreted as: Compare(Deploy, yesterday, current)
  Compiles to: core compare --deploy HEAD~1 HEAD
  Run? (Enter) or edit (e):

The user can always:
  Press Enter -- run as planned
  Press 'e' -- edit the Layer 3 command directly
  Press 'p' -- see the full semantic plan
  Press 'n' -- cancel

Trust is preserved because the machine is always visible.
The user is never trapped inside the abstraction.
---
SHELL MEMORY -- OPERATIONAL MEMORY

Your shell already has replay and session support.
INT-326 adds semantic memory.

Instead of:
  history -- what commands did I run?

You get:
  memory -- what did I DO?

Examples:
  how did I fix bluetooth last month?
  -> Friday searches semantic intent log
  -> "On 2026-04-12: Repair(Service('bluetooth')) -- steps: restart pipewire-bluetooth, re-pair device"
  -> Replay? (y/n)

  repeat the deploy workflow from last Friday
  -> Friday finds DeployIntent sequence from that date
  -> Shows the sequence with current context
  -> Confirm to replay

  what changed between last week and now?
  -> Friday compares semantic logs
  -> "47 observations, 12 deployments, 3 repairs, 1 purge"
  -> Show details? (y/n)

This is operational memory, not shell history.
The difference: semantic memory understands WHAT you did, not HOW.
---
IMPLEMENTATION PLAN

Phase 0 -- Design (1 session):
  Finalize SemanticIntent struct in Rust
  Define Action, Target, Constraint types
  Design the confidence scoring model
  Gate: types defined, not yet implemented

Phase 1 -- Layer 2 for existing vocabulary (2 sessions):
  Wrap existing vocabulary commands in SemanticIntent
  delete, find, show, deploy all produce SemanticIntent objects
  Log to new table: semantic_history in state.db
  Gate: `explain delete ~/tmp` shows all three layers

Phase 2 -- Inspectability commands (1 session):
  explain, plan, dry-run, why commands
  Any command can be inspected before execution
  Gate: explain <any vocabulary command> works

Phase 3 -- Ambiguity resolution (2 sessions):
  Multi-interpretation detection
  Confidence scoring per interpretation
  Choice presentation and preference learning
  Gate: "clean downloads" presents options, learns preference after 3 choices

Phase 4 -- Verb taxonomy enforcement (1 session):
  Every vocabulary word assigned to category
  Safety contracts enforced automatically
  Observation verbs provably never mutate state
  Gate: attempting side effect in observation verb fails at compile time

Phase 5 -- Semantic pipelines (2 sessions):
  Typed pipeline stages
  --explain flag shows types and row counts
  UNIX fallback preserved
  Gate: show processes | filter cpu > 50 | terminate works end to end

Phase 6 -- Shell memory (1-2 sessions):
  SemanticIntent stored in state.db with full context
  how did I... queries routed through Friday
  repeat workflow command replays intent sequences
  Gate: "how did I fix X last month?" returns semantic answer

Phase 7 -- Daily driver (1 week):
  All commands pass through Layer 2
  No regressions on UNIX compatibility
  Gate: 1 week daily use, UNIX workflows unaffected
---
RELATIONSHIP TO OTHER INTENTS

INT-261 (fsh Vocabulary -- INT complete):
  The vocabulary IS Layer 1.
  INT-326 adds Layer 2 and Layer 3 formalization under it.

INT-322 (fsh v4 -- The Shell Grows Up):
  INT-322 adds features (error UX, time-travel, sandbox).
  INT-326 adds the architectural philosophy underneath.
  These are complementary, not competing.
  INT-322 can implement using INT-326 architecture.

INT-294 (Forest Event Bus):
  Every SemanticIntent is a forest event.
  intent.executed, intent.failed, intent.ambiguous
  The event bus makes intents visible system-wide.

INT-251 (Core v23 -- Friday Becomes Central):
  Friday operates on SemanticIntent objects, not raw strings.
  Friday's predictions improve dramatically with typed intent data.
  Friday can suggest: "You always Archive after Deploy -- do it now?"
---
GATES
[ ] Phase 0: SemanticIntent struct defined, action/target/constraint types finalized
[ ] Phase 1: existing vocabulary wrapped -- explain shows three layers for all vocab commands
[ ] Phase 2: inspectability commands -- explain, plan, dry-run, why all working
[ ] Phase 3: ambiguity resolution -- confidence scoring, choice presentation, preference learning
[ ] Phase 4: verb taxonomy enforced -- observation verbs provably safe
[ ] Phase 5: semantic pipelines -- typed stages, --explain, UNIX fallback preserved
[ ] Phase 6: shell memory -- semantic history queryable in plain language
[ ] Phase 7: daily driver -- 1 week no UNIX regressions
Final:
[ ] Every fsh command is explainable -- no black boxes
[ ] The three layers are always visible and always accessible
[ ] Human language never becomes opaque
[ ] UNIX compatibility fully preserved alongside semantic layer
[ ] Friday reasons on typed intent objects, not raw strings
[ ] The shell that thinks in three layers is the daily driver

TIMELINE
Phase 0-2: after INT-322 Phase 1 (multi-command reliability)
Phase 3-5: parallel with INT-322 Phases 3-6
Phase 6-7: after INT-322 complete
Target: Phase 3 complete before NY presentation
        The shell that explains itself is a presentation moment

"Traditional shells pipe text.
fsh pipes intent.
Layer 1: what you mean.
Layer 2: what the forest understands.
Layer 3: what the machine executes.
All three. Always visible. Always yours." 🌲
