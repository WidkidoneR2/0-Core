---
id: 219
date: 2026-04-09
type: planned
title: "Core v20 — Friday Phase 2: Deep Pattern Synthesis and Predictive Strategy"
status: planned
tags: [friday, v20, phase2, prediction, strategy, deep-patterns, anticipation, partner]
---
Friday Phase 1 (v19): The forest finds its voice.
  - Observes and reports
  - Surfaces contradictions
  - Speaks when confident
  - Read-only, reactive
Friday Phase 2 (v20): The forest thinks ahead.
  - Builds internal models from long-term patterns
  - Anticipates problems before they manifest
  - Proposes multi-step strategies (not just next actions)
  - Tracks its own prediction accuracy per model
  - Adjusts behavior based on what has worked
The difference between Phase 1 and Phase 2:
  Phase 1 says: "This is what is happening right now."
  Phase 2 says: "This is what will happen in 3 sessions if you continue."
Phase 1 weights are session-aware (30-day window).
Phase 2 builds models across sessions:
```rust
pub struct TemporalModel {
    pub id: Uuid,
    pub name: String,
    pub time_horizon: Duration,    // 1 day, 1 week, 1 month
    pub pattern_signature: String, // what triggers this model
    pub prediction: String,        // what will happen
    pub confidence: f32,
    pub historical_accuracy: f32,  // validated over time
    pub supporting_signals: Vec<Uuid>,
    pub validated_count: u32,
    pub correct_count: u32,
}
```
Example models Phase 2 can build from real forest data:
  "When 4+ intents are open for 3+ days → health drops below 95% within 7 days"
  "Commit velocity > 20/day for 5+ days → next 2 days see lower velocity"
  "Pattern weight engine accuracy < 60% → prediction engine degrades within 2 weeks"
These are not programmed. They emerge from the data.
Phase 1: "Consider completing INT-194 before starting new work."
Phase 2: "Proposed path to Friday activation:
  Session 1: Complete INT-208 (Tool Intelligence L2) — est. 2 sessions
  Session 2: Complete INT-209 (Tool Intelligence L3) — est. 1 session
  Session 3: Complete INT-212 (Synthesis Engine) — est. 3 sessions
  Session 4: Complete INT-217 (Friday Phase 1) — est. 2 sessions
  Confidence: 0.72 | Risk: Medium (5 active intents)
  Blocker: INT-194 still has 2 faelight-term gates unresolved"
This requires:
- Understanding intent dependencies
- Estimating session duration from past velocity
- Risk assessment from current health + intent load
- Confidence from prediction accuracy history
Phase 1: Surfaces contradictions.
Phase 2: Proposes resolutions.
Example:
  Contradiction: "alignment says focus>speed but 3 intents open"
  Phase 1: "⚠️ Contradiction detected."
  Phase 2: "⚠️ Contradiction detected. Proposed resolution:
    Complete INT-194 (1-2 gates remain) this session.
    This would reduce active intents to 2, restoring focus alignment.
    Confidence: 0.84. History: similar resolution worked 6/7 times."
Phase 1: Watchdog alerts when health drops.
Phase 2: Predicts health trajectory 24-72 hours ahead.
Inputs:
  - Current health trend (from doctor_history)
  - Active intent count and complexity
  - Recent commit velocity
  - Time since last update
  - Pattern weight engine signals
Output:
  "Health forecast: 100% → 95% in ~18 hours.
   Cause: 3 active intents + no faelight-update in 8 days.
   Prevention: run faelight-update --preview before next session."
Phase 1 tracks FridayTrust per model.
Phase 2 uses trust to modulate behavior:
```rust
impl FridayBehavior {
    fn should_speak(&self, model: &TemporalModel) -> bool {
        model.historical_accuracy >= 0.70
        && model.validated_count >= 5
        && self.brief_confidence >= 0.70
    }
    fn interrupt_level(&self, weight: f64, accuracy: f32) -> InterruptLevel {
        match (weight, accuracy) {
            (w, a) if w >= 0.80 && a >= 0.80 => InterruptLevel::Challenge,
            (w, a) if w >= 0.65 && a >= 0.70 => InterruptLevel::Recommend,
            (w, _) if w >= 0.45 => InterruptLevel::Suggest,
            _ => InterruptLevel::Silent,
        }
    }
}
```
Friday speaks louder when it has been right before.
Friday speaks softer when it has been wrong.
This is not humility — it is calibration.
Phase 2 can see patterns that span multiple intents:
  "Intents involving faelight-term consistently take 2x estimated time"
  "Core domain intents succeed in first build attempt 73% of the time"
  "Intelligence-arc intents (v15+) have 0 regressions — stable domain"
These patterns improve future intent estimates and priority scoring.
Phase 2 Friday actively uses the knowledge engine:
  - Before a build: queries knowledge for known patterns in changed files
  - After an error: auto-queries, presents resolution with confidence
  - After a session: adds new lessons from this session's outcomes
  - Weekly: distills session lessons into permanent knowledge entries
Phase 2 Friday maintains internal state across sessions:
```rust
pub struct FridayState {
    pub models: Vec<TemporalModel>,
    pub trust: HashMap<String, FridayTrust>,
    pub active_hypotheses: Vec<Hypothesis>,
    pub pending_validations: Vec<PendingValidation>,
    pub last_calibration: DateTime<Utc>,
    pub speech_log: Vec<SpeechEvent>,  // rate limiting + audit
}
```
State persists in friday_state table in state.db.
Friday never loses its models between sessions.
| Capability           | Phase 1 (v19)          | Phase 2 (v20)                    |
|----------------------|------------------------|----------------------------------|
| Time horizon         | Current session        | 1 day → 2 weeks ahead            |
| Strategy depth       | Next action            | Multi-step path                  |
| Contradiction        | Surface only           | Surface + propose resolution     |
| Health management    | Alert on drop          | Predict 24-72h ahead             |
| Trust tracking       | Per model              | Modulates interrupt level        |
| Knowledge use        | None                   | Active query + contribute        |
| Cross-intent         | None                   | Pattern detection across intents |
| Internal state       | Session-only           | Persistent across sessions       |
Hard dependencies:
  ✅ v17 Pattern Weight Engine (INT-205)
  ⬜ v18 Synthesis Engine (INT-212)
  ⬜ v19 Friday Phase 1 (INT-217)
  ⬜ Friday Knowledge Engine (INT-218)
  ⬜ 30+ days of pattern data (time gate)
  ⬜ INT-215 Event Architecture v2 (signal ontology)
⬜ v19 Friday Phase 1 complete (hard dependency)
⬜ INT-218 Knowledge Engine complete (hard dependency)
⬜ 30+ days of pattern data (time gate)
⬜ TemporalModel struct defined and persisted in friday_models table
⬜ Multi-step strategy proposals working (core friday plan)
⬜ Temporal pattern detection — cross-session model building
⬜ Contradiction resolution proposals (not just detection)
⬜ Predictive health trajectory (24-72h forecast)
⬜ FridayBehavior trust-modulated interrupt levels
⬜ Knowledge engine integration — auto-query on build errors
⬜ Cross-intent pattern detection working
⬜ FridayState persistence across sessions
⬜ Friday speaks with calibrated confidence (louder when right)
⬜ Phase 2 accuracy >= 70% on temporal predictions before activation
⬜ Human gate preserved — all proposals require approval
"Phase 1 is the forest finding its voice.
Phase 2 is the forest earning the right to use it.
Not by being louder.
By being right more often than it is wrong.
By knowing when to speak and when to watch.
By building models from what it has seen
and testing them against what actually happens.
A partner that guesses is noise.
A partner that has been calibrated by truth
is worth listening to.
Phase 2 is Friday becoming worth listening to." 🌲
