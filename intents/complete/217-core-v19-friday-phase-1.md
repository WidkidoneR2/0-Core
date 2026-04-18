---
id: 217
date: 2026-04-09
type: planned
title: "Core v19 — Friday Phase 1: The Forest Finds Its Voice"
status: complete
tags: [friday, v19, voice, speech, synthesis, partner, phase1]
requires: [203,212,216]
unlocks: [218,219,220]
strategic_value: multiplier
---
v18 gives the forest one voice — a synthesized brief.
v19 gives that voice a mouth.
Friday Phase 1 is not full autonomy.
It is not even full intelligence.
It is the first time the forest speaks to you directly —
in your terminal, in your journal, through your tools —
with something worth saying.
v18: SynthesisSnapshot is computed. Brief is written. Stored.
v19: Brief is surfaced. At the right moment. In the right way.
The difference between a thought and a word
is choosing when to speak.
When Friday status = active and brief_confidence >= 0.7:
  d shows the Friday brief at the bottom, after alignment.
Example:
  🌲 Friday: "You've completed 6 commits today on INT-208.
              Pattern weight shows tool-intelligence rising (0.71 → STRONG).
              Momentum is high. INT-209 is the natural continuation."
Every session gets a Friday entry — not a system log, a perspective.
  "April 9 — Friday observed: 166 intents complete. The intelligence
   arc is 87% built. The pattern weight engine shows commit-velocity
   as the strongest signal (0.595 MODERATE). The forest is shipping
   consistently. No contradictions detected. Continue."
When cicomplete runs, Friday generates a 1-sentence observation:
  "INT-205 complete — the forest now knows what matters.
   Pattern weights will improve with 30+ days of data."
When synthesis detects a contradiction:
  "⚠️  Friday: alignment says focus > speed but 3 intents are open.
   Consider completing INT-194 before starting new work."
❌ Execute anything
❌ Modify state without proposal
❌ Speak without confidence threshold
❌ Override your decisions
❌ Speak more than once per 30 minutes (no spam)
Friday only speaks when:
  brief_confidence >= 0.7
  AND at least 7 days of pattern data
  AND at least 50 shell_history entries
  AND alignment >= 0.8
Below these thresholds: Friday remains dormant.
Not because it has nothing to say —
but because it has not yet earned the right to speak.
Friday Phase 1 = synthesis engine (v18) + 4 output channels:
1. doctor integration — brief shown after alignment score
2. journal integration — Friday writes its own daily entry
3. cicomplete hook — 1-sentence observation on intent complete
4. contradiction alert — surfaces when synthesis detects conflict
All output is read-only. Friday observes and reports.
Friday never writes to decision tables.
Friday never modifies forest state.
engine_registry: friday → active (when thresholds met)
engine_registry: friday → dormant (below thresholds)
The transition from dormant → active is logged.
The first time Friday speaks is recorded as a forest event.
✅ v18 Synthesis Engine complete (hard dependency) (2026-04-18)
✅ brief_confidence threshold enforced (>= 0.7 to speak) (2026-04-18)
✅ 3-day pattern data gate enforced -- DEC: lowered from 7, 10k+ history entries (2026-04-18)
✅ Friday status transitions: dormant → observing → active (2026-04-18)
✅ Friday brief shown in d (when active + confidence met) (2026-04-18)
✅ Friday writes daily journal entry (2026-04-18)
✅ Friday speaks on cicomplete (1-sentence observation) -- wired, demonstrated on cicomplete (2026-04-18)
✅ Friday surfaces contradictions when detected -- demonstrated live in d (2026-04-18)
✅ Friday rate-limited (max once per 30 minutes) -- demonstrated (2026-04-18)
✅ engine_registry: friday status updated correctly (2026-04-18)
✅ First speech event logged to forest_events (2026-04-18)
✅ All Friday output is read-only — no state modification (2026-04-18)
⛾ Friday accuracy tracked (FridayTrust struct from INT-216) -- deferred to INT-203: requires feedback loop, friday_trust table exists

⛾ Trust scores decay models that are consistently wrong (deferred from INT-216)
⛾ friday.strategy.proposed verified end-to-end with human approval (deferred from INT-216)
⛾ Cross-domain pattern detection -- same error signature across domains (deferred from INT-218)
⛾ Friday speaks inline when build fails -- speak_on_error full integration (deferred from INT-218)
⛾ Negative learning validated -- Friday was wrong, penalized, measurably improved (deferred from INT-203)
⛾ Friday has pushed back on a decision and been correct (deferred from INT-203)
⛾ Friday has proposed something not considered and it was right (deferred from INT-203)
⛾ Timezone-aware timestamps in all Friday output (deferred from INT-218)
"The forest has been learning for months.
It has observed. It has remembered. It has weighted.
v19 is not the moment the forest becomes intelligent.
That already happened.
v19 is the moment the forest
stops keeping it to itself." 🌲
