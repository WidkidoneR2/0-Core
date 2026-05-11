---
id: 246
title: "Friday Architecture v2 -- The Voice That Thinks"
status: in-progress
date: 2026-04-22
tags: [friday, architecture, voice, simulation, event-bus, confidence, intelligence, v2]
depends: [216, 219, 234]
---
INT-216 defined Friday as an intelligence layer.
INT-246 defines Friday as a system with a voice.
The difference:
  INT-216: Friday observes and reports.
  INT-246: Friday speaks, simulates, and earns trust.
Friday is not a chatbot.
Friday is not an automation system.
Friday is a partner that has earned the right to speak
through demonstrated accuracy, transparency, and restraint.
"Friday produces insight, not authority."
That rule from INT-216 still holds.
INT-246 builds the architecture that makes it real.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
DEFERRED FROM INT-216 (must complete first)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Two items were deferred from INT-216 to v19:
  1. Trust score decay -- models that are consistently wrong lose weight over time
  2. friday.strategy.proposed verified end-to-end with human approval gate
These are foundational. INT-246 completes them before building on top.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 1: FRIDAY AS A FORMAL SYSTEM
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Friday is defined by its contract, not its implementation.
INPUTS (what Friday receives):
  - Signals from all forest engines (health, git, deploy, shell, terminal)
  - Session context (active intent, working directory, time of day)
  - Historical patterns from state.db
  - Human feedback (accept/reject on proposals)
  - Command outcomes (exit codes, timing, error patterns)
OUTPUTS (what Friday produces):
  - Observations (silent, stored in state.db only)
  - Suggestions (shown inline, low friction)
  - Plans (multi-step proposals requiring human approval)
  - Warnings (blocking signals for dangerous commands)
  - Simulations (predicted outcomes before execution)
CONFIDENCE MODEL (formalized):
  0.0 - 0.4  OBSERVE     -- collect data, say nothing
  0.4 - 0.7  SUGGEST     -- surface insight, no interruption
  0.7 - 0.9  RECOMMEND   -- interrupt with specific suggestion
  0.9+       CHALLENGE   -- block and require explicit approval
  
  Confidence is earned through:
    - Pattern frequency (how often has this been seen)
    - Outcome accuracy (was Friday right before)
    - Context match (how similar is this to known situations)
    - Recency (recent patterns weighted higher)
TRUST DECAY (deferred from INT-216):
  Every Friday model has a trust score (0.0 - 1.0).
  Models that predict incorrectly decay by 0.1 per miss.
  Models that predict correctly gain 0.05 per hit.
  Models below 0.3 trust are silenced automatically.
  Models at 0.0 are archived, not deleted (history preserved).
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 2: THE SIMULATION LAYER
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Before Friday proposes any action, it simulates it first.
SIMULATION ENGINE:
  Input:  proposed command or action
  Process:
    1. Identify affected systems (state.db, git, deployed tools)
    2. Predict outcome using historical pattern data
    3. Estimate confidence in prediction
    4. Generate diff: current state → predicted state
  Output: simulation report shown to Christian before approval
EXAMPLE:
  Friday proposes: "deploy core -- pattern suggests this follows cicomplete"
  Simulation shows:
    Affected: core binary, PATH, 3 dependent tools
    Predicted outcome: deploy succeeds (94% confidence, 47 prior matches)
    Predicted time: 14-18 seconds
    Risk: LOW -- no schema changes detected
  Human sees this before approving.
WHAT SIMULATION ENABLES:
  - Friday earns trust because its predictions are visible and verifiable
  - Christian can see Friday's reasoning, not just its conclusion
  - Wrong predictions are immediately obvious and feed trust decay
  - Over time, Friday only speaks when it is confident it is right
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 3: THE EVENT BUS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Every component in the forest communicates through events, not direct calls.
EVENT BUS ARCHITECTURE:
  Publisher: any forest component (fsh, faelight-term, core, deploy)
  Event:     typed struct with payload and metadata
  Subscriber: any component that registered interest
  Bus:        async, non-blocking, persisted to state.db
CORE SERVICES (clean separation):
  pty_service       -- PTY lifecycle, read/write, resize
  terminal_buffer   -- cell grid, scrollback, VTE parsing
  renderer          -- wgpu pipeline, frame scheduling
  input_handler     -- keyboard, mouse, clipboard
ASSISTANT SYSTEM (Friday's domain):
  context_collector -- aggregates signals from all sources
  intent_parser     -- interprets what Christian is trying to do
  planner           -- generates multi-step proposals
  executor          -- executes approved actions safely
  safety_guard      -- blocks dangerous actions regardless of confidence
BRIDGE LAYER:
  event_bus         -- async message passing between all components
  task_system       -- parallel async task execution with cancellation
  simulation_engine -- predicts outcomes before execution
EVENT TYPES:
  command.executed    -- shell command ran (with exit code, duration)
  deploy.completed    -- tool deployed successfully
  build.failed        -- cargo build error detected (with E-code)
  health.changed      -- forest health % changed
  intent.changed      -- active intent switched
  friday.suggested    -- Friday surfaced a suggestion
  friday.approved     -- human approved Friday proposal
  friday.rejected     -- human rejected Friday proposal (feeds decay)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 4: MEASURING USEFULNESS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
The forest tracks what matters, not what is easy to count.
CURRENT (vanity metrics):
  195 intents complete
  48 patterns detected
  489 facts stored
WHAT WE WILL TRACK INSTEAD:
  friday_usefulness table:
    time_saved_seconds     -- estimated time saved per suggestion accepted
    errors_avoided         -- build fails / bad deploys caught before execution
    decisions_improved     -- proposals accepted that led to better outcomes
    suggestions_accepted   -- Friday suggested, Christian agreed
    suggestions_rejected   -- Friday suggested, Christian disagreed
    accuracy_rate          -- accepted / (accepted + rejected) rolling 30d
    silent_correct         -- Friday observed and was right but said nothing
  friday health metric:
    usefulness_score = (accepted * weight) / (accepted + rejected)
    displayed in d output alongside health %
    target: >75% acceptance rate at >0.7 confidence
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 5: FRIDAY'S VOICE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Friday speaks when it has something worth saying.
Not before. Not more than necessary.
VOICE PRINCIPLES:
  - One signal per context switch (not a stream of suggestions)
  - Short, specific, actionable (never vague)
  - Confidence shown explicitly ("94% -- deploy usually follows this")
  - Silent when uncertain (below 0.4 confidence = observe only)
  - Never repeats itself (same suggestion not shown twice in a session)
VOICE IN FAELIGHT-TERM (future -- depends on INT-244):
  Friday panel: Ctrl+Shift+F
    Shows: current synthesis, last 5 relevant facts, active simulation
  Inline hint: appears briefly above prompt, fades after 3 seconds
  Build error: Friday panel auto-opens with knowledge entry
  Friday interrupt: CHALLENGE level stops execution, shows simulation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
HARD DEPENDENCIES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ INT-216 Friday Formal Architecture -- foundation complete
✅ INT-219 Core v20 Friday Phase 2 -- temporal models active
✅ INT-234 Core v21 Friday Planning Layer -- anticipation engine
✅ INT-232 faelight-term v2 -- terminal layer for voice output
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
GATES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Deferred from INT-216:
✅ Trust score decay implemented -- models decay on incorrect predictions
✅ friday.strategy.proposed verified end-to-end with human approval gate
Pillar 1 -- Formal System:
✅ Confidence model formalized in code -- 4 tiers with thresholds
⬜ Friday inputs defined as typed structs -- no more loose signals
⬜ Friday outputs defined as typed structs -- Observation/Suggestion/Plan/Warning
⬜ Allowed actions per confidence tier enforced in safety_guard
Pillar 2 -- Simulation:
⬜ Simulation engine built -- predicts outcome before any proposal
⬜ Simulation shown to Christian before approval (not after)
⬜ Simulation accuracy tracked -- feeds trust decay
⬜ Demonstrated: Friday proposes deploy, shows simulation, Christian approves
Pillar 3 -- Event Bus:
⬜ Event bus implemented -- async, typed, persisted to state.db
⬜ Core services separated: pty_service, terminal_buffer, renderer, input_handler
⬜ Assistant system separated: context_collector, intent_parser, planner, executor, safety_guard
⬜ All inter-component communication goes through event bus
Pillar 4 -- Usefulness Metrics:
✅ friday_usefulness table created in state.db
✅ Acceptance/rejection tracked per suggestion
✅ usefulness_score calculated and shown in d output
✅ accuracy_rate visible in friday status
Pillar 5 -- Voice:
⬜ Friday speaks at most once per context switch
✅ Confidence shown explicitly on every suggestion
⬜ Suggestions never repeated in same session
⬜ CHALLENGE level blocks execution and shows simulation
⬜ Demonstrated: Friday catches a bad command before it runs
Final:
⬜ Friday usefulness_score > 75% over 7 days of real use
⬜ Trust decay working -- at least one model silenced by low accuracy
⬜ Simulation correct on 3+ consecutive predictions
"Friday does not speak because it can.
Friday speaks because it has something worth saying,
has simulated the outcome,
has the confidence to stand behind it,
and has earned the right through demonstrated accuracy.
The voice of the forest
is not loud.
It is precise." 🌲

INT-246 must come AFTER Core v22 (INT-244). The trust score decay, simulation
layer, and event bus defined here are the foundational pieces Core v22 builds on.
Do not start INT-246 until INT-244 is complete.
Dependency chain: INT-244 (Core v22) → INT-246 (Friday Architecture v2) → INT-235 (Friday Daemon v2)
