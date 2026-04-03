---
id: 186
date: 2026-04-03
type: arch
title: "Shell Intelligence Layer — Confidence, Causality, Memory Decay"
status: in-progress
tags: [shell, intelligence, confidence, causality, memory, decay, fsh, v14]
priority: critical
depends_on: [179, 185]
---

## The Problem
fsh currently predicts and suggests. But it does not say how sure it is.
A suggestion without confidence is indistinguishable from a guess.
A pattern without causality is noise that looks like signal.
History without decay becomes a liability.

## The Judgment Credibility Layer
Every intelligent output must include three things:
  Confidence: how sure am I? (0.0 - 1.0)
  Causality:  why do I believe this? (signal sources + weights)
  Decay:      how long is this valid? (expires_at)

Plus one more from external feedback:
  Counterfactual: what would make me wrong?

## Output Format (required on ALL suggestions)
💡 Suggestion: run d
   Confidence: 0.82
   Signal sources: history(0.6) session_pattern(0.3) recency(0.1)
   Causality: fg commit in 89% of cases precedes d (47 sessions)
   Expires: next command
   Might be wrong if: already ran d in last 5 min

## Phase 28 Gate (tightened)
Current threshold: too low — fires on weak patterns
New gate (non-negotiable):
  occurrences  >= 30
  confidence   >= 0.7
  accuracy     >= 80%
  cooldown     not active (no suggestion in last 3 min)
  context      allows (not mid-pipeline)

fn should_suggest(pattern) -> bool:
  pattern.occurrences >= 30 AND
  pattern.confidence >= 0.7 AND
  pattern.accuracy >= 0.80 AND
  !cooldown_active() AND
  context_allows()

## Memory Decay
Weekly distillation job:
  compress raw history -> patterns
  decay low-frequency sequences (half-life: 30 days)
  boost consistent workflows (reinforcement)
  prune sequences not seen in 60 days

Without decay: predictions degrade as noise outweighs signal.

## Causality Layer
observe causality (new command):
  "Commit frequency increased — Cause: 3x fg commit alias usage"
  "Health dropped — Cause: 2 failed deploy scripts at 14:00"
Not just WHAT changed. WHY it changed.

## Multi-Path Failure Recovery
Current: last_command retry (linear)
New: alternative resolution paths
  Error: E_PORT_IN_USE
  Options:
    1. kill process on :3000
    2. run on next free port  
    3. inspect with: lsof -i :3000

## Time-Based Patterns
Morning patterns vs evening patterns.
Build phase vs ops phase vs exploration phase.
Session phase detection feeds suggestion timing.

## Integration with contextd (INT-185)
fsh reads insights from forest_insights (written by contextd).
All suggestions include confidence + causality from insight payload.
Decay managed by contextd expiry system.

## Gate Check
✅ Phase 28 gate tightened: >=30 occurrences, >=0.7 confidence, >=80% accuracy, 3min cooldown (2026-04-03)
⬜ Every suggestion includes confidence score + signal sources
⬜ Every suggestion includes causality explanation
⬜ Every suggestion includes expiry/decay
⬜ Every suggestion includes counterfactual
⬜ observe causality command live
⬜ Multi-path failure recovery (alternatives not just retry)
⬜ Memory distillation: weekly decay + compression job
⬜ Time-based patterns: morning vs evening detection
⬜ Session phase detection: build/ops/exploration

## The Phrase
"Confidence says: I am 78% sure.
Causality says: because of these 12 signals.
Decay says: this expires in one session.
Counterfactual says: I am wrong if X.
That is not a suggestion. That is judgment." 🌲
