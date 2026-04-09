---
id: 205
date: 2026-04-08
type: planned
title: "Core v17 — Pattern Weight Engine"
status: planned
tags: [core, v17, patterns, weight, intelligence, friday, prediction, strategy]
---
Core v17 is not a new feature.
It is the reasoning layer that makes every existing feature smarter.
Right now the forest observes, predicts, and strategies.
But it treats all patterns equally.
A pattern seen once has the same voice as one seen a hundred times.
A pattern from last week competes equally with one from this morning.
A pattern that caused failure is weighted the same as one that caused success.
v17 fixes this. Every pattern gets a weight. Every weight is earned.
Every weight is explainable.
The unit of intelligence is not an event.
It is a pattern — behavior + context + outcome history over time.
A pattern that:
- occurs frequently
- occurred recently
- caused real consequences
- is trending worse
- behaves consistently
- comes from reliable data
...is a pattern that deserves to be heard.
The weight engine makes that precise.
Every pattern is scored across six dimensions:
```rust
pub struct PatternMetrics {
    frequency:   f64,  // how often does this occur? (0.0 → 1.0)
    recency:     f64,  // how recent? (decay-adjusted, 0.0 → 1.0)
    consequence: f64,  // severity of outcomes (0.0 → 1.0)
    trend:       f64,  // direction (-1.0 improving → 1.0 worsening)
    volatility:  f64,  // consistency vs chaos (0.0 stable → 1.0 chaotic)
    confidence:  f64,  // data reliability (0.0 → 1.0)
}
```
Each dimension is normalized. No raw counts. No unbounded values.
Every input is honest and bounded.
The coefficients are not fixed. They are context-dependent.
Different decisions require different emphases.
Starting values are educated guesses — calibrated over time by outcome tracking.
```rust
pub struct ContextWeights {
    frequency:   f64,
    recency:     f64,
    consequence: f64,
    trend:       f64,
    volatility:  f64,
}
```
Starting contexts and coefficients:
deployment:
consequence: 0.40   // failures hurt most here
recency:     0.25
frequency:   0.15
trend:       0.15
volatility:  0.05
work_rhythm:
recency:     0.40   // recent patterns define current state
frequency:   0.25
trend:       0.20
consequence: 0.10
volatility:  0.05
prediction:
confidence:  0.35   // data reliability is paramount
frequency:   0.25
recency:     0.20
trend:       0.15
consequence: 0.05
health:
consequence: 0.35
trend:       0.25
recency:     0.20
frequency:   0.15
volatility:  0.05
These are starting points. Trial and error. Honest calibration.
When outcomes prove the weights wrong — adjust. No pretending.
```rust
pub fn compute_weight(m: &PatternMetrics, w: &ContextWeights) -> f64 {
    let base =
        (m.frequency   * w.frequency)   +
        (m.recency     * w.recency)     +
        (m.consequence * w.consequence) +
        (trend_factor(m.trend) * w.trend) +
        (stability_factor(m.volatility) * (1.0 - w.volatility));
    base * m.confidence
}
```
The confidence multiplier is critical.
A perfectly computed weight on unreliable data is still unreliable.
Confidence scales the entire result.
```rust
pub fn trend_factor(trend: f64) -> f64 {
    // trend: -1.0 (improving) → 1.0 (worsening)
    // Worsening amplifies faster than improving dampens.
    // A system getting worse deserves more attention than one getting better.
    match trend {
        t if t > 0.0 => 0.5 + (t * 0.5),  // worsening → amplify
        t             => 0.5 + (t * 0.2),  // improving → dampen slowly
    }
}
```
```rust
pub fn stability_factor(volatility: f64) -> f64 {
    // Unstable patterns are dangerous even when frequent.
    // A pattern that works 60% of the time but unpredictably
    // is more dangerous than one that works 50% consistently.
    1.0 - (volatility * 0.7)
}
```
```rust
pub fn apply_decay(weight: f64, age_hours: f64) -> f64 {
    // High-weight patterns that stop occurring lose influence.
    // Memory without decay is accumulation, not learning.
    let decay_rate = 0.015;
    weight * (-decay_rate * age_hours).exp()
}
```
```rust
pub fn apply_identity_alignment(weight: f64, alignment: f64) -> f64 {
    // alignment: 0.8 (conflicts with declared values)
    //            1.0 (neutral)
    //            1.2 (aligns with declared values)
    //
    // "understanding over convenience" violation → amplify weight
    // Acting in accordance with values → dampen urgency
    weight * alignment
}
```
Raw numbers mean nothing without interpretation:
```rust
pub enum WeightClass {
    Ignore,    // < 0.25  — not worth surfacing
    Weak,      // 0.25–0.45 — mention only if asked
    Moderate,  // 0.45–0.65 — suggest
    Strong,    // 0.65–0.80 — recommend
    Critical,  // > 0.80    — challenge / interrupt
}
pub fn classify_weight(weight: f64) -> WeightClass {
    match weight {
        w if w < 0.25 => WeightClass::Ignore,
        w if w < 0.45 => WeightClass::Weak,
        w if w < 0.65 => WeightClass::Moderate,
        w if w < 0.80 => WeightClass::Strong,
        _              => WeightClass::Critical,
    }
}
```
| WeightClass | Friday Behavior                        |
|-------------|----------------------------------------|
| Ignore      | Silent — no output                     |
| Weak        | Mentions only if directly asked        |
| Moderate    | Suggests during relevant context       |
| Strong      | Recommends proactively                 |
| Critical    | Challenges / interrupts current action |
Most weight systems only score failures.
This one scores both directions.
Positive patterns use the same formula with:
- consequence = success impact (not failure severity)
- trend inverted (improving = positive direction)
This lets Friday say:
"Continuing this approach is working.
Prediction accuracy improved 12% over the last 30 days."
Friday must notice what is going well, not just what is going wrong.
Weight must always be decomposable.
Friday must be able to say exactly why a pattern matters:
"This matters because:
it occurs frequently (frequency: 0.82)
it happened recently (recency: 0.91)
it caused failure (consequence: 0.85)
the pattern is worsening (trend: 0.6)
confidence in data: 0.95
→ weight: 0.84 → CRITICAL"
If the weight cannot be explained in these terms — it is not valid.
This protects the core philosophy: understanding over convenience.
```rust
pub struct Pattern {
    pub id:             String,
    pub description:    String,
    pub context:        PatternContext,  // deployment | work_rhythm | prediction | health
    pub metrics:        PatternMetrics,
    pub weight:         f64,
    pub class:          WeightClass,
    pub decay_adjusted: f64,
    pub last_updated:   i64,
    pub outcome_log:    Vec<PatternOutcome>,
}
```
Stored in state.db — `pattern_weights` table.
| Engine          | How v17 improves it                                    |
|-----------------|--------------------------------------------------------|
| core predict    | Predictions ranked by pattern weight, not just recency |
| core strategy   | Priorities emerge from high-weight patterns            |
| core goals      | Goals generated from Critical-class patterns           |
| core partner    | Disagreements grounded in weight evidence              |
| friday          | Tone, interruption, and pushback driven by WeightClass |
| contextd        | Insights filtered through weight before surfacing      |
Starting weights are guesses. Calibration makes them true.
After each context decision:
1. Record the weight that drove the recommendation
2. Record the actual outcome
3. Compare: did the weight correctly predict importance?
4. Adjust context coefficients by ±0.02 based on outcome
5. Log calibration history — never hide adjustments
This is trial and error done honestly.
Not pretending the first guess was right.
Not hiding when it was wrong.
Iterating toward truth.
⬜ PatternMetrics struct defined and stored in state.db
⬜ compute_weight function with context-sensitive coefficients
⬜ trend_factor and stability_factor implemented
⬜ apply_decay function live — patterns lose weight over time
⬜ apply_identity_alignment integrated with v15 values (when ready)
⬜ WeightClass thresholds with behavior mapping
⬜ Positive reinforcement weights (not just failure patterns)
⬜ Explainability output — every weight fully decomposable
⬜ pattern_weights table in state.db
⬜ core predict uses weights to rank predictions
⬜ core strategy uses weights to prioritize
⬜ Friday WeightClass → behavior mapping live
⬜ Calibration protocol logging context coefficient adjustments
⬜ At least one Critical-class pattern correctly identified and acted on
⬜ deploy core and d passes 100%
"A system that treats all patterns equally
has not yet learned to pay attention.
Weight is not judgment.
Weight is earned attention —
proportional to frequency, recency, consequence,
and the direction things are heading.
The forest does not shout about everything.
It speaks loudest about what matters most.
And when it speaks — it can tell you exactly why." 🌲

Remove volatility from ContextWeights.
Use it as a pure penalty multiplier alongside confidence:
  base_score * m.confidence * stability_factor(m.volatility)
Weights = what matters dimensionally.
Modifiers = how trustworthy/stable the signal is.
  t > 0.0 => 0.5 + (t * 0.5)   // worsening: up to 1.0
  t <= 0.0 => 0.5 + (t * 0.4)  // improving: down to 0.1
Improving patterns are less predictive. Penalize harder.
frequency = occurrences_in_window / window_size_days
This approximates rate without needing explicit opportunity tracking.
Raw occurrence counts allow low-risk noise to outrank rare high-risk signals.
When a prediction misses:
  contribution = (dimension_weight * dimension_score) / total_score
Adjust only the top contributing dimensions, in proportion.
Blanket ±0.02 on all dimensions is too blunt.
Not [0.8, 1.2]. Values inform, they do not override.
At 0.8 the system can suppress a real signal by 20% — dangerous.
Tight clamp prevents "ignored reality because it didn't align with identity."
struct WeightBreakdown {
    base: f64,
    confidence_adjusted: f64,
    decay_adjusted: f64,
    identity_adjusted: f64,
    final_weight: f64,
}
Store alongside every computed weight.
Friday needs this to explain its reasoning.
When something feels wrong, trace which stage amplified or killed the signal.
weights = what matters (frequency, recency, trend, context)
modifiers = how trustworthy it is (confidence, volatility/stability)
These must stay architecturally separate.
