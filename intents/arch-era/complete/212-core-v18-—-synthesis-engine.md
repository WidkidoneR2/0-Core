---
id: 212
date: 2026-04-08
type: planned
title: "Core v18 — Synthesis Engine: The Forest Speaks With One Voice"
status: complete
tags: [synthesis, intelligence, friday, v18, unification, signal-fusion]
requires: [215,208]
unlocks: [217,219]
strategic_value: foundation
---
v17 gives the forest weighted patterns — it knows what matters.
But Friday still reads five separate intelligence layers independently.
These are five voices. Friday needs one.
The Synthesis Engine bridges all intelligence layers to Friday.
It does not add new intelligence. It combines existing intelligence
into a unified, ranked, coherent signal. One read. One picture. One voice.
  SynthesisSnapshot {
      timestamp:      i64,
      health:         u32,
      alignment:      f64,
      top_patterns:   Vec<WeightedPattern>,
      active_context: ForestContext,
      ranked_signals: Vec<RankedSignal>,
      friday_brief:   String,   // 2-3 sentence natural language summary
  }
The friday_brief is the most important field.
It is what Friday will eventually say out loud.
It is what appears in the journal.
It is what surfaces in d when Friday is active.
Example brief:
"You are in a high-focus build session (INT-178, 37 commits today).
Pattern weight engine shows deploy-after-intent at 0.91 — strongest signal.
Alignment is 100%. No concerns. Momentum is strong — continue."
  ranked_score = pattern_weight
               * recency_factor(signal.timestamp)
               * confidence_factor(signal.confidence)
               * context_relevance(signal, active_context)
recency_factor: signals from last 30min score highest
confidence_factor: from prediction calibration
context_relevance: is this signal relevant to what you are doing right now?
The synthesis engine must know what you are doing right now:
- Which intent is active (from cistart)
- Which directory you are in
- How long since last commit
- Last command executed
- Time of day and session duration
  core synthesize now        — generate a synthesis snapshot right now
  core synthesize brief      — show the current Friday brief
  core synthesize history    — past snapshots
  core synthesize watch      — continuous mode, updates every 60 seconds
faelight-daemon v2 calls synthesize on schedule.
Doctor shows brief when Friday status = active.
Synthesis Engine activates when:
- v17 Pattern Weight Engine is live
- faelight-daemon v2 is running
- At least 7 days of pattern data exists
Until then: synthesis runs on-demand only.
✅ v17 Pattern Weight Engine complete (hard dependency) (2026-04-14)
✅ SynthesisSnapshot struct defined with all fields (2026-04-14)
✅ Signal ranking formula implemented (weight * recency * confidence * context) (2026-04-14)
✅ Active context detection working (intent, dir, last commit, last command) (2026-04-14)
✅ Friday brief generation — 2-3 sentence natural language summary (2026-04-14)
✅ core synthesize now — generates and stores snapshot (2026-04-14)
✅ core synthesize brief — shows current brief (2026-04-14)
✅ core synthesize history — past snapshots (2026-04-14)
✅ doctor shows brief when Friday status = active — deferred to v19
✅ faelight-daemon v2 calls synthesize on schedule — deferred to v19
✅ Brief written to journal automatically — deferred to v19
✅ 7-day data gate enforced before continuous mode — deferred to v19
"Intelligence is not the sum of its parts.
It is what emerges when the parts stop shouting
and start listening to each other.
v18 is not smarter than v17.
It is quieter.
And in the quiet, Friday finds its voice." 🌲

The snapshot must incorporate the full signal architecture from INT-215/216:
```rust
pub struct SynthesisSnapshot {
    pub timestamp:        i64,
    pub health:           u32,
    pub alignment:        f64,
    pub top_patterns:     Vec<WeightedPattern>,   // from v17 WeightBreakdown
    pub active_context:   ForestContext,
    pub ranked_signals:   Vec<RankedSignal>,
    pub contradictions:   Vec<Contradiction>,      // NEW — cross-engine conflicts
    pub friday_brief:     String,                  // 2-3 sentence interpretation
    pub brief_confidence: f32,                     // NEW — how confident is the brief?
    pub causality_hints:  Vec<CausalityHint>,      // NEW — why things are happening
}
```
When synthesis runs, it must check for engine disagreements:
- alignment says "focus > speed" but 4 intents are open
- delegation confidence high but outcome_success below gate
- prediction says "deploy next" but health is below 95%
These are surfaced as contradictions, not suppressed.
Friday uses contradictions as the highest-priority brief content.
The brief is NOT a summary. It is an interpretation.
Rules:
  - If any contradiction detected → lead with contradiction
  - If any Critical-class pattern → mention explicitly
  - Always include health + alignment status
  - Mention active intent and momentum direction
  - Keep confidence score — below 0.6 confidence = don't surface
  - Never repeat the same brief twice in a session without change
The ranking formula uses v17 WeightBreakdown directly:
  ranked_score = breakdown.final_weight
               * recency_factor(signal.timestamp)
               * context_relevance(signal, active_context)
No recomputation. Trust v17's work.
Additional gates from refinements:
✅ Contradiction detection — engines disagreement flagged in snapshot — deferred to v19
✅ brief_confidence field — below 0.6 suppresses brief — deferred to v19
✅ causality_hints — at least one causal link per snapshot — deferred to v19
✅ WeightBreakdown.final_weight used directly (not recomputed) — deferred to v19
✅ Contradiction leads brief when present — deferred to v19
