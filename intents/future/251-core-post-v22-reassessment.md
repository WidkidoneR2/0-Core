---
id: 251
title: "Core v23 -- Friday Becomes Central"
status: planned
date: 2026-04-24
last_revised: 2026-04-28
type: arch
tags: [core, v23, friday, architecture, central, nervous-system, leap, post-v22]
version: TBD
---

This intent began as a stub on 2026-04-24 holding the question:
"is v22 as drafted the right leap, or just the next step by inertia?"

The reassessment happened on 2026-04-28 in the context of preparing
for the NY presentation in mid-July 2026. The honest framing of that
presentation -- not a demo, but a prototype that succeeds means
the conversation becomes "what comes after Linus" -- forced a
sharper answer than v22 could give as originally drafted.

The decision: v22 is reframed (see INT-244 last_revised 2026-04-28)
to ship the four pillars Friday can honestly support without an LLM.
v23 is reserved for the bigger leap.

This intent is now v23.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
THE LEAP
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

v18-v21: Friday observes. Friday predicts. Friday plans.
v22: Friday becomes useful (doc steward, cartographer, memory, voice).
v23: Friday becomes the system's nervous system.

The mental model:

  v22 = Friday is a tool inside the forest.
        It sits alongside the other 50 tools, doing useful work.

  v23 = Friday IS the forest's nervous system.
        Every tool emits to it, consumes from it, is mapped by it.
        The 50-tools-becoming-25 trajectory is enabled by Friday
        absorbing what the retiring tools did.

This is not a feature addition. It is an architectural reorganization
where Friday goes from "one tool among many" to "the connective tissue
that ties the forest together."

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
WHY THIS MATTERS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Today the forest has 50 tools. Each has its own state, its own logs,
its own way of expressing health. Friday observes all of them but
the observation is one-directional -- Friday reads, never writes
back to the system's behavior.

This is a ceiling. With 50 tools and 199 intents and 2400+ commits
maintained by one human, the human is the integration layer. The
human is what holds the system coherent. That worked at 30 tools.
It is straining at 50. It will break at 80.

v23 makes Friday the integration layer instead. Friday becomes:
- The single place every tool reports to
- The single place every tool reads system state from
- The reasoning engine that watches the whole system
- The coordinator when tools need to cooperate

When Friday is the nervous system:
- Tools shrink (they shed bookkeeping that Friday does centrally)
- Health becomes one coherent signal, not 50 partial views
- Predictions span the whole system, not single workflows
- New tools cost less to add (they plug into the nervous system,
  not into 50 ad-hoc integrations)
- The system can answer "how am I doing?" with one mind asking,
  not the human stitching together output from doctor + audit +
  intent list + git status + deploy registry + faelight-bar

This is what enables the "post-Linus" conversation. Not because
the system is bigger -- because it is more coherent. A
single-builder system can stay coherent past 50 tools only if
the builder externalizes the integration layer. That is what
v23 does.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 1: UNIFIED EVENT BUS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Today: tools emit signals to friday-context, write logs to runtime/,
update state.db tables, fire D-Bus notifications, write atomic files.
Five different mechanisms for "something happened."

v23: one event bus. friday::events::emit(kind, payload, source).

All tools migrate to it. Old mechanisms retire as tools are touched.
Friday consumes the event stream as the canonical source of truth
about what is happening in the system.

Schema (proposed):
  events table:
    id, ts, source_tool, kind, payload (JSON), correlation_id

Kinds (initial):
  build_started, build_completed, deploy_started, deploy_completed,
  intent_state_change, health_check, knowledge_added, prediction_made,
  user_interaction, decision_recorded, doc_proposal_offered

Friday queries events for prediction. Doctor reads events for health.
Bar reads events for status. Term reads events for the panel.
Everything else stops needing direct access to other tools' state.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 2: THE REASONING ENGINE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

v22 has forward-chaining inference (1-2 facts derive 1 conclusion).
v23 generalizes this into a proper reasoning engine that watches
the event stream and produces:

- System-wide observations (not workflow-local predictions)
- Causal chains: A happened, then B happened, then C; if A happens
  next session, expect B then C
- Anomaly detection: this event sequence has not been seen before;
  flag it
- Health synthesis: 50 tools' worth of signals reduced to one
  coherent assessment

The engine is rule-based, not generative. No LLM. Rules can be:
- Hardcoded (v23 ships with a starter set)
- Learned from event-stream patterns (Friday proposes new rules,
  human approves)
- Imported from friday_knowledge entries (knowledge becomes rules)

This is the layer that lets Friday say "deploy core failed and
build_completed for faelight-shell happened 3 minutes earlier --
the dispatcher likely was not rebuilt." That is reasoning, not
retrieval.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 3: TOOL ABSORPTION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

The 50-tools-becoming-25 trajectory.

Today's tools that exist because nothing else integrates the data:
- archaeology-0-core (planned retirement)
- workspace-view (audit candidate)
- entropy-check (audit candidate)
- bin-doctor (audit candidate)
- dotctl, faelight-digest, faelight-gen, faelight-niri-bridge,
  faelight-palette, faelight-pulse, faelight-vault, faelight-zone
  (10 tools at audit-stale right now)

These tools exist as separate processes because there is no central
nervous system. v23 changes that. Several of them become commands
within Friday rather than standalone binaries:
  core friday workspaces  -- replaces workspace-view
  core friday entropy     -- replaces entropy-check
  core friday vault       -- replaces faelight-vault
  core friday zones       -- replaces faelight-zone

Not every tool absorbs. Some have legitimate independent reasons to
exist (faelight-shell, faelight-term, faelight-bar, faelight-fm,
faelight-git -- these are user-facing, with their own UX). But
infrastructure-shaped tools that exist mostly to surface system state
become Friday subcommands.

Target: 50 tools -> 25-30 tools by end of v23. Real reduction in
maintenance surface. Real coherence gain.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 4: COORDINATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

When tools need to cooperate, today the human coordinates. Deploy
expects build to complete first. Faelight-term expects faelight-shell
to be deployed before reading from it. The human knows the order.

v23: Friday coordinates. Tools declare their dependencies and their
output kinds. Friday's coordinator:
- Knows what depends on what (from Pillar 4 of v22 -- the cartographer)
- Schedules cooperative work in correct order
- Holds dependent work until prerequisites complete
- Surfaces blocks: "deploy faelight-term is waiting for faelight-shell
  to finish building"

This is not parallel execution (that is INT-245 Pillar 1 / INT-255).
This is correctness in serial cooperation. Today it works because
the human holds the order in their head. v23 externalizes that.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 5: THE ONE-MIND ANSWER
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Today, "how am I doing?" requires:
  doctor              -- 22-23 health checks
  audit scan          -- tool intelligence scoring
  intent list         -- intent state across 5 categories
  git status          -- working tree state
  fg risk             -- recent commit risk score
  faelight-bar status -- live system signals

The human stitches these together into one mental answer.

v23: core status (or a similar single command) returns one coherent
narrative. Built from:
- Event-stream synthesis (Pillar 1)
- Reasoning engine inference (Pillar 2)
- Cartographer state (from v22 Pillar 4)
- Decision history (from v22 Pillar 3)

The narrative is structured but readable:
  "Healthy. INT-232 in 14-day daily-driver trial, day 6.
   No regressions in fsh since 097ded6d. faelight-notify
   collision warnings stopped after ecced2c0 (yesterday).
   Active contradiction: 3 in-progress intents while values
   declare focus>speed -- closing INT-249 today reduced this
   to 2. Recommendation: focus on INT-232 validation; closing
   it would resolve the contradiction fully."

One paragraph. Forty seconds of reading. The whole forest's state
condensed by one mind that has been watching all of it. That is
the leap.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
WHAT v23 IS NOT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

v23 is NOT:
  - An LLM integration (still no internet, still no generative AI)
  - Autonomous action (Friday proposes, human approves -- always)
  - A rewrite of any existing tool from scratch
  - Centralization for its own sake (only where it earns coherence)
  - Designed to look impressive in a demo

v23 IS:
  - Architectural reorganization where Friday earns "central" status
  - The infrastructure that lets one human maintain a 50+ tool system
  - The integration layer the human currently is in their own head
  - The foundation for whatever comes after the NY presentation
  - The honest answer to "what does v23 need to be that v22 does not"

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
RELATIONSHIP TO v22 (INT-244)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

v22 ships first. Without v22, v23 has nothing to be central FOR.

v22 -> v23 transition:
- v22 ships friday_decisions (decision record) -- v23 reads from it
- v22 ships friday_map (cartographer) -- v23's coordinator uses it
- v22 ships per-session debrief -- v23's reasoning engine consumes
  the debrief as a calibration signal
- v22 ships voice/tone calibration -- v23 inherits the voice when
  it reports the one-mind answer

v22 is the demoable, NY-presentation-relevant work. v23 is the
post-presentation work that makes the prototype actually scale.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
HARD DEPENDENCIES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⬜ Core v22 (INT-244) shipped -- foundation
⬜ Event bus design ratified (Pillar 1 schema decided)
⬜ Reasoning engine rule format ratified (Pillar 2)
⬜ Tool absorption candidate list confirmed (Pillar 3)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
GATES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Pillar 1 -- Unified Event Bus:
⬜ events table created with schema
⬜ friday::events::emit API live in faelight-core
⬜ at least 5 tools migrated to emit via the bus
⬜ Friday consumes events as canonical source for predictions
⬜ Old signal/log mechanisms retired in migrated tools

Pillar 2 -- Reasoning Engine:
⬜ Rule format defined and documented
⬜ Starter rule set shipped (10+ rules from existing knowledge)
⬜ Engine produces system-wide observations from event stream
⬜ Anomaly detection live (unknown event sequences flagged)
⬜ Causal chain reasoning demonstrated on a real failure case

Pillar 3 -- Tool Absorption:
⬜ Audit-stale tools reviewed individually for absorb vs retain
⬜ At least 5 tools absorbed (functionality moves into core friday)
⬜ Tool count reduced from 50 to <40
⬜ No regression in absorbed tools' user-visible function

Pillar 4 -- Coordination:
⬜ Tools declare dependencies in registry metadata
⬜ Coordinator schedules cooperative work in correct order
⬜ Pre-action surface of blocked work ("waiting for X")
⬜ Demonstrated: a multi-tool workflow succeeds without human ordering

Pillar 5 -- One-Mind Answer:
⬜ core status command returns coherent narrative
⬜ Narrative built from event synthesis + reasoning + cartographer + decisions
⬜ Narrative readable in <40 seconds
⬜ Demonstrated: human verifies narrative matches their own assessment

Final:
⬜ Tool count: 25-30 tools by end of v23
⬜ "How am I doing?" answerable from one command
⬜ At least one event-stream-derived prediction proven correct in real session
⬜ The forest holds coherent at 50+ tools without manual integration

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
THE DECISION (recorded 2026-04-28)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

INT-244 was rewritten to drop the two LLM-dependent pillars
(natural language conversation, intent co-build) and replace
them with documentation steward and system cartographer. The
remaining four pillars (dual presence, persistent memory, self-
review, voice) survived from the original draft.

INT-251 was populated as the v23 design: Friday becomes the
system's nervous system. Five pillars (event bus, reasoning
engine, tool absorption, coordination, one-mind answer) that
together transform Friday from "tool inside the forest" to
"connective tissue across the forest."

The reasoning behind this split:

The NY presentation demands Friday demonstrating real work
TODAY -- not Friday philosophizing. v22 ships four pillars
that demonstrably reduce cognitive load on a single human
maintaining a 50-tool system. That is what the presentation
shows.

v23 is the post-presentation work. It is the architectural
move that lets the prototype actually scale -- the foundation
for what happens if/when the conversation becomes "what comes
after Linus."

The two removed v22 pillars (conversation, co-build) are not
abandoned -- they are deferred to a future intent that
honestly scopes their LLM dependency. Forcing them into v22
would have meant either compromising the no-internet principle
or shipping templated stubs that fall over fast. Neither
serves the thesis.

This decision is recorded here. Future sessions reading this
file see why v22 was reshaped and why v23 became the leap
intent rather than another increment.

"v22 is what Friday can honestly be today.
v23 is what Friday must become if this prototype is going
to outlive the presentation that introduces it.

The first earns trust.
The second earns relevance." 🌲
