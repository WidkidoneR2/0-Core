---
id: 206
date: 2026-04-08
type: planned
title: "Engine Coordination Layer — The Forest Thinks as One"
status: planned
tags: [coordination, engines, sync, architecture, core, friday, contextd, v17]
---
The forest has grown powerful engines:
- Core intelligence arc (v9 → v17)
- faelight-contextd — nervous system
- Delegation Engine (INT-187)
- Friday observation engine (INT-203)
- Pattern Weight Engine (INT-205)
Each was designed well. Each works independently.
But they were not designed to talk to each other.
As the forest grows, isolated engines produce contradictory outputs.
The prediction engine says one thing.
The strategy engine says another.
Friday gets confused signals and loses credibility.
contextd surfaces insights the weight engine would classify as Ignore.
The result: a forest that is intelligent in pieces but incoherent as a whole.
Not a rewrite. Not a new engine.
A coordination layer — the wiring between engines.
Thin. Explicit. Documented.
A set of contracts that define:
- What each engine produces
- What each engine consumes
- How they share state through state.db
- What must be updated when another engine upgrades
Think of it as: the forest thinks as one.
These are the engines that must coordinate:
| Engine              | Produces                        | Consumes                        |
|---------------------|---------------------------------|---------------------------------|
| core predict        | predictions, accuracy scores    | pattern weights, events         |
| core strategy       | priorities, roadmap             | predictions, pattern weights    |
| core partner        | proposals, disagreements        | strategy, values, weights       |
| core doctor         | health scores, integrity        | events, tool states             |
| faelight-contextd   | insights, signals               | events, pattern weights         |
| delegation engine   | simulations, trust contracts    | confidence scores, history      |
| friday              | suggestions, conversation       | all of the above                |
| pattern weight      | weights, weight classes         | events, outcomes, values        |
All coordination happens through state.db.
No engine calls another engine directly.
No engine depends on another engine's binary.
The contract is the schema, not the code.
```sql
-- Engine registry: what is running and at what version
CREATE TABLE IF NOT EXISTS engine_registry (
    name        TEXT PRIMARY KEY,
    version     TEXT NOT NULL,
    last_active INTEGER NOT NULL,
    status      TEXT NOT NULL  -- active | dormant | degraded
);
-- Engine outputs: standardized signal format
CREATE TABLE IF NOT EXISTS engine_signals (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    source      TEXT NOT NULL,     -- which engine produced this
    signal_type TEXT NOT NULL,     -- weight | prediction | insight | alert
    payload     TEXT NOT NULL,     -- JSON
    weight      REAL,              -- if applicable
    consumed_by TEXT,              -- which engines have read this
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER            -- NULL = permanent
);
-- Upgrade contracts: what must change when an engine upgrades
CREATE TABLE IF NOT EXISTS engine_upgrade_log (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    engine          TEXT NOT NULL,
    from_version    TEXT NOT NULL,
    to_version      TEXT NOT NULL,
    breaking_change INTEGER DEFAULT 0,
    affected_engines TEXT,         -- JSON array
    migrated        INTEGER DEFAULT 0,
    upgraded_at     INTEGER NOT NULL
);
```
When an engine upgrades, dependent engines must be notified.
The rule: **no silent upgrades.**
If core v17 adds pattern weights, then:
- core predict must consume them (or explicitly opt out)
- contextd must filter insights through them (or explicitly opt out)
- friday must use WeightClass for behavior mapping (or explicitly opt out)
Opting out is allowed. Silent ignorance is not.
Every upgrade that affects the shared schema:
1. Bumps the engine version in engine_registry
2. Records affected engines in engine_upgrade_log
3. Sets breaking_change = 1 if schema changed
4. Requires affected engines to acknowledge before next deploy
This is the right place to solve the versioning problem.
Right now: core is perpetually `2.0.0`.
That number means nothing. Every deploy is the same label.
The fix: **core version tracks the intelligence arc.**
core 2.0.0     — current (base engine)
core 3.0.0     — when v13 Autonomy ships
core 4.0.0     — when v14 Partnership ships
core 5.0.0     — when v15 Alignment ships
core 6.0.0     — when v16 Self-Transformation ships
core 7.0.0     — when v17 Pattern Weight Engine ships
Minor versions (x.1.0) for meaningful capability additions within a version.
Patch versions (x.x.1) for fixes and refinements.
This means:
- `core version` tells you exactly where the intelligence arc is
- deploy bumps the version when gates complete
- engine_registry reflects the real state
- Friday can report: "Core is at v5.0.0 — Alignment active"
Each non-core engine has its own version in the registry.
When core upgrades, non-core engines are checked:
- Are they consuming new core signals?
- Are their schemas compatible?
- Do they need to be rebuilt?
The coordination layer surfaces this automatically:
core engines status
Output:
Engine              Version    Status     Last Active
core                4.0.0      active     now
faelight-contextd   0.1.0      active     2 min ago
delegation          0.3.0      active     12 min ago
friday              0.0.0      dormant    never
pattern-weight      0.0.0      planned    —
⚠️  contextd v0.1.0 has not acknowledged core v4.0.0 upgrade
💡  Run: core engines sync contextd
core engines status          — show all engines and their sync state
core engines sync <engine>   — acknowledge upgrade, update contracts
core engines signals         — show recent cross-engine signals
core engines upgrade-log     — history of engine upgrades and migrations
core engines check           — verify all engines are consistent
Friday is the only engine that consumes from all others.
It is the tip of the intelligence stack.
For Friday to work correctly, all engines below it must be coherent.
The coordination layer ensures that coherence.
When Friday says:
"I recommend stopping work on INT-194 today.
Pattern weight: 0.78 (Strong).
Reason: deployment failures increased 40% in the last 2 hours,
contextd flagged focus-fragmentation,
strategy engine shows 3 higher-priority items pending."
That sentence is only possible if:
- pattern weight engine fed the weight
- contextd fed the insight
- strategy engine fed the priority
- Friday consumed them all coherently
Without coordination, Friday cannot say that.
With coordination, it is inevitable.
⬜ engine_registry table in state.db
⬜ engine_signals table in state.db
⬜ engine_upgrade_log table in state.db
⬜ core engines status command live
⬜ core engines sync command live
⬜ Core versioning tied to intelligence arc (v3=v13, v4=v14, etc.)
⬜ core version reflects actual intelligence milestone
⬜ Non-core engines registered and versioned
⬜ Upgrade contract enforced — no silent upgrades
⬜ contextd producing signals in standardized format
⬜ core predict consuming pattern weights (when v17 ships)
⬜ Friday consuming coordination signals (when Friday ships)
⬜ core engines check passes clean
⬜ deploy core and d passes 100%
"Seven instruments playing separately
make seven sounds.
Seven instruments coordinated
make music.
The forest does not need more engines.
It needs the engines it has
to know what the others know.
Coordination is not complexity.
Coordination is the difference between
a collection of tools
and a mind." 🌲
