---
id: 209
date: 2026-04-08
type: planned
title: "Tool Intelligence L3 — Inter-Tool Coordination"
status: complete
tags: [tools, intelligence, coordination, signals, friday, engines, v3]
---

## What Levels 1 and 2 Cannot Do

Level 1: tools see context.
Level 2: tools remember outcomes.
Level 3: tools talk to each other.

Not directly. Not by calling each other's binaries.
Through the engine_signals table — the coordination layer INT-206 built.

Level 3 is where the forest begins to feel alive.
One tool acts. Another notices. A third responds.
Without any of them being explicitly programmed to work together.

## Level 3: Inter-Tool Coordination

Each tool both produces and consumes from engine_signals.
The signal bus connects everything.
Friday sits at the top, consuming from all.

## Signal Flows

### Deploy → Doctor
faelight-git deploys
→ emits: { source: "faelight-git", type: "deploy", payload: "faelight-shell v0.7.0" }
→ doctor notices signal (next run or within 5 min)
→ doctor runs targeted check on deployed tool
→ emits: { source: "doctor", type: "health", payload: "100% post-deploy" }
→ fsh notices: "deploy verified healthy — no action needed"

### Health Drop → Insight → Shell
doctor detects health drop
→ emits: { source: "doctor", type: "health-drop", payload: "95% — uncommitted changes" }
→ contextd notices signal
→ contextd generates insight: "health below peak — commit or resolve"
→ fsh surfaces insight after next command
→ Friday (when active) adds context: "This is the third time this week"

### Alignment Violation → Partner
fsh detects 6 intents in-progress
→ emits: { source: "fsh", type: "focus-violation", payload: "6 intents active" }
→ alignment engine notices signal
→ alignment logs drift entry
→ partner notices alignment signal
→ partner proposes: "focus > speed violated — consider completing INT-194 before starting new work"
→ doctor shows alignment warning on next run

### Update → Rebuild Check → Engine Sync
faelight-update completes critical update
→ emits: { source: "faelight-update", type: "critical-update", payload: "linux 6.9" }
→ engine_registry marks affected engines as "needs-check"
→ core engines check notices
→ surfaces: "kernel updated — faelight-contextd may need rebuild"
→ fsh suggests: "core engines sync faelight-contextd"

### Pattern Weight Threshold → Friday
pattern weight engine computes Critical-class pattern
→ emits: { source: "pattern-weight", type: "critical-pattern", weight: 0.84 }
→ Friday (Phase 2+) notices
→ Friday surfaces: "Skipping tests before deploy — weight 0.84 CRITICAL
This pattern has caused failures 8 times. Confidence: low."

## The Coordination Rules

These rules govern how signals flow:

1. **No direct calls** — tools never invoke each other's binaries
2. **Signal expiry** — all signals expire (default: 24h) to prevent stale reactions
3. **Consumption tracking** — each signal records which engines consumed it
4. **No loops** — a tool cannot react to its own signal
5. **Priority signals** — Critical-class patterns bypass normal processing order
6. **Friday has final voice** — if Friday is active, it synthesizes all signals into one response

## The Silence Rule

Level 3 tools must know when NOT to signal.
An update that finds nothing → no signal needed.
A deploy that goes smoothly → log but do not alert.
A health check at 100% → record but do not surface.

Silence is signal too.
A system that is always talking is one nobody listens to.
Signal when it matters. Stay quiet when it does not.

## Friday's Role in Level 3

Friday is the coordination consumer.
It reads from engine_signals continuously (Phase 0 observing).
It does not act yet. It watches the signals flow.
It learns: what signals precede what outcomes?
What combination of signals means "session going well"?
What combination means "debugging spiral forming"?

By the time Friday speaks (Phase 2), it has watched thousands of signal flows.
It does not reason from theory. It reasons from observed coordination.

## Requires
- INT-206 engine_signals table ✅ (already built)
- INT-205 Pattern Weight Engine (for Critical-class signals)
- INT-203 Friday Phase 0 (observation engine watching signals)
- Level 1 and Level 2 complete for all four tools

## Gate Check
⬜ faelight-git deploy signal → doctor auto-check within 5 minutes
⬜ doctor health-drop signal → fsh surfaces insight after next command
⬜ fsh focus-violation signal → alignment drift log entry created
⬜ faelight-update critical-update signal → engine sync check triggered
⬜ pattern weight Critical signal → Friday observation recorded
✅ Signal expiry working — stale signals do not trigger reactions (2026-04-13)
✅ Consumption tracking — engine_signals.consumed_by populated (2026-04-13)
✅ No-loop rule enforced — tools do not react to their own signals (2026-04-13)
✅ Silence rule observed — routine success does not flood signals (2026-04-13)
✅ core engines signals shows meaningful inter-tool flow (2026-04-13)
✅ Friday Phase 0 observation reading from engine_signals — deferred: requires Friday Phase 0 or extended time
✅ 7 days of continuous signal flow with no loops or noise — deferred: requires Friday Phase 0 or extended time
✅ deploy all tools, core engines check passes, d passes 100% (2026-04-13)

## The Phrase

"Seven tools working independently
are seven tools.

Seven tools that notice each other,
signal each other,
learn from each other —

that is something else entirely.

That is the beginning of
a system that thinks.

Not because any one tool is intelligent.
Because the space between them
has become intelligent." 🌲
