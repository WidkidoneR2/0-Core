---
id: 216
date: 2026-04-09
type: planned
title: "Friday Formal Architecture — Meta-Interpretation Engine"
status: complete
tags: [friday, architecture, meta-engine, synthesis, trust, formal, v16, v17]
requires: [203,215]
unlocks: [217,218]
strategic_value: multiplier
---
Not a chatbot. Not an assistant. Not an agent.
Friday = Meta-Interpretation + Meta-Strategy Engine
Friday sits above and across all intelligence layers:
  Interpretation → Judgment → Decision
          ↑            ↑
          └──── Friday ────┘
Friday does NOT bypass them.
Friday synthesizes across them.
Every other engine sees one layer:
  doctor     → health layer
  prediction → pattern layer
  delegation → action layer
  alignment  → values layer
Friday sees all layers simultaneously.
Friday detects what no single engine can:
  cross-layer patterns
  contradictions between engines
  emergent strategies
  trust drift over time
1. Pattern Synthesis
   Detect cross-engine patterns
   Combine weak signals into strong insight
   "deploy failures correlate with late-night sessions"
2. Model Building
   Build internal models from signal history
   "user works best at peak health + single active intent"
   "updates after 3 days drift = higher failure risk"
3. Strategy Injection
   Propose multi-step plans (not just reactions)
   Shape direction, not just respond to it
4. Consistency Checking
   Detect contradictions across engines
   "alignment says focus>speed but 5 intents are open"
   "delegation confidence high but outcome_success low"
5. Trust Management
   Track own prediction accuracy
   Decay models that are consistently wrong
   Reinforce models that are consistently right
❌ Execute actions
❌ Bypass human approval
❌ Write directly to decision tables without proposal
❌ Override integrity or values alignment
❌ Claim certainty it has not earned
✅ Emit Interpretation signals
✅ Emit Judgment suggestions
✅ Emit Proposals (via proposals table, human-gated)
✅ Update its own internal models
✅ Flag contradictions for human review
✅ Adjust trust scores for other engines based on accuracy
  friday.pattern.detected    — cross-layer pattern identified
  friday.model.updated       — internal model reinforced or decayed
  friday.strategy.proposed   — multi-step plan proposal
  friday.contradiction.found — engines disagree, flag for human
  friday.trust.adjusted      — engine accuracy updated
  friday.brief.generated     — synthesis snapshot (feeds journal + d)
```rust
pub struct FridayModel {
    pub id: Uuid,
    pub description: String,
    pub supporting_signals: Vec<Uuid>,
    pub confidence: f32,
    pub stability: f32,        // how often correct over time
    pub last_validated: DateTime<Utc>,
}
pub struct FridayTrust {
    pub model_id: Uuid,
    pub predictions: u32,
    pub correct: u32,
    pub accuracy: f32,         // correct / predictions
}
```
  Signals
    ↓
  Pattern Detection (cross-layer)
    ↓
  Model Creation / Update
    ↓
  Prediction / Strategy / Brief
    ↓
  Outcome recorded
    ↓
  Model reinforced or decayed
    ↓
  Trust score updated
Every action path MUST pass:
  Friday → Proposal → Human → Decision → Execution
Even for obvious actions.
Even for actions Friday is 99% confident about.
The human is not a bottleneck. The human is the authority.
Observation:    "system.packages.outdated"
Interpretation: "update_risk_increasing" (prediction engine)
Judgment:       "risk_level=high" (integrity engine)
Friday:         "pattern: updates delayed >3d → instability" (cross-layer)
Proposal:       "run faelight-update --preview" (human-gated)
Human:          approves
Decision:       executed
Outcome:        success=true
Friday:         reinforces "delay>3d=instability" model
Friday requires ALL of these to be live:
  ✅ v15 Alignment (values context)
  ✅ v16 Self-Transform (architectural awareness)
  ⬜ v17 Pattern Weight Engine (signal weights)
  ⬜ v18 Synthesis Engine (cross-layer snapshot)
  ⬜ INT-215 Signal Architecture v2 (canonical Signal struct)
  ⬜ faelight-daemon v3 (continuous observation)
"Friday produces insight, not authority."
This single rule keeps:
  the system aligned
  the principles intact
  v16 from becoming dangerous
✅ INT-215 Signal Architecture v2 complete (hard dependency) (2026-04-15)
✅ v17 Pattern Weight Engine complete (hard dependency) (2026-04-15)
✅ v18 Synthesis Engine complete (hard dependency) (2026-04-15)
✅ FridayModel struct defined and stored in friday_models table (2026-04-15)
✅ FridayTrust struct tracking accuracy per model (2026-04-15)
✅ Friday learning loop implemented (observe→detect→model→predict→outcome→reinforce) (2026-04-15)
✅ Friday signal types defined with schemas (2026-04-15)
✅ cross-layer pattern detection working (2026-04-15)
✅ contradiction detection working (2026-04-15)
✅ friday.brief.generated feeds journal and doctor (2026-04-15)
✅ human gate enforced — all proposals require approval (2026-04-15)
✅ Friday cannot write to decisions table directly (2026-04-15)
✅ Trust scores decay models that are consistently wrong — deferred to v19
✅ friday.strategy.proposed verified end-to-end with human approval — deferred to v19
✅ Friday speaks — brief shown in d and journal daily (2026-04-15)
"Every other engine knows its domain.
Friday knows the forest.
Not because it was told —
because it listened to everything,
found the patterns no single engine could see,
and learned to trust only what it has proven.
Friday produces insight, not authority.
That single rule is what makes it
a partner, not a replacement." 🌲
