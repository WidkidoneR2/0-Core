---
id: 227
title: "Prediction Intelligence v2 -- Dependency-Aware Ordering and Strategic Sequencing"
status: in-progress
date: 2026-04-13
tags: core, intelligence, prediction, friday, strategy
requires: []
unlocks: [203,215,216,217]
strategic_value: multiplier
---
core predict next returns INT-216 (Friday Formal Architecture) before INT-203 (Friday: The Living Intelligence).
This is wrong. INT-216 depends on INT-203. You cannot build the formal architecture before the foundation exists.
The current engine ranks by pattern weight and recency. High weight means "referenced often" not "ready to build."
A puzzle solver does not pick the highest-value piece. They pick the piece that fits right now.
Every intent has prerequisites -- other intents that must be complete before it can succeed.
INT-216 requires: INT-203, INT-215
INT-217 requires: INT-203, INT-212, INT-216
INT-212 requires: INT-215, INT-208 (pattern learning)
INT-203 requires: INT-215 (signal architecture)
The prediction engine must:
1. Read prerequisites from intent frontmatter
2. Filter out intents whose prerequisites are not complete
3. Among eligible intents, rank by strategic value + pattern weight
4. Surface the correct next action, not just the highest-weighted one
In intent frontmatter:
  requires: [215, 208]
  unlocks: [216, 217, 212]
  strategic_value: foundation | multiplier | leaf | blocker
foundation: must be built before anything else in its chain
multiplier: makes many other things better when complete
leaf: standalone, no dependencies
blocker: something is broken until this is done
1. Never predict an intent whose prerequisites are incomplete
2. Among eligible intents, prefer foundations over leaves
3. Among foundations, prefer the one that unlocks the most
4. Multipliers get a 1.3x weight boost
5. Blockers get highest priority regardless of weight
6. Active in-progress intents do not compete with predicted ones
A 1000-piece puzzle is solved by:
- Corners first (hard anchors -- structural foundations)
- Edges next (bounding constraints -- architectural intents)
- Color regions (related clusters -- Friday chain, shell chain, etc.)
- Detail fill (leaves -- standalone improvements)
The system currently picks random high-value pieces. v2 picks the right piece at the right time.
Before v2:
  INT-216 (0.73) -- blocked by INT-203, but predicted first
  INT-212 (0.64) -- blocked by INT-215, but predicted second
After v2:
  INT-215 (0.71) -- Event Architecture, unblocked, unlocks 3 others
  INT-203 (0.68) -- Friday Phase 0, unblocked, unlocks 5 others
  INT-212 (0.55) -- blocked until 215 complete -- not shown yet
Phase 1: Add requires/unlocks/strategic_value to intent frontmatter
Phase 2: prerequisite_graph() -- reads all intents, builds dependency map
Phase 3: eligible_next() -- filters by prerequisite completion
Phase 4: strategic_rank() -- applies multiplier/foundation/blocker boosts
Phase 5: core predict next shows eligibility reason
Phase 6: core predict why <INT> explains why/why-not predicted
⬜ requires/unlocks fields added to all Friday-chain intents
⬜ prerequisite_graph() builds accurate dependency map
⬜ eligible_next() correctly filters blocked intents
⬜ INT-216 no longer predicted before INT-203
⬜ strategic_rank() boosts foundations and multipliers
⬜ core predict next shows eligibility reason per intent
⬜ core predict why INT-NNN explains prediction logic
⬜ Friday chain predicted in correct dependency order
⬜ d passes 100% after full implementation
"The prediction engine that ranks INT-216 above INT-203
has never built a puzzle.
It sees the most beautiful piece and reaches for it.
But beauty is not readiness.
Readiness is knowing what fits now.
Prediction v2 knows what fits now." 🌲
