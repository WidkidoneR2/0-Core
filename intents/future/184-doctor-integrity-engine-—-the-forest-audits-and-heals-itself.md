---
id: 184
date: 2026-03-31
type: arch
title: "Doctor Integrity Engine — The Forest Audits and Heals Itself"
status: planned
tags: [doctor, integrity, self-healing, audit, trust, consistency, architecture]
version: 12.0.0
priority: critical
---

## The Problem — The System Cannot Be Trusted Without This

When `d` runs today it answers one question: **is everything present?**
It does not answer: **is everything consistent with what it claims to be?**

This gap has already caused real damage:
- Jarvis milestone pointer was wrong — system scored 85, showed arrow at 65
- INT-133 had `status: complete` but lived in `future/` for months
- Registry versions drifted from deployed versions silently
- faelight-clipboard silently crashed at every boot — undetected
- Doctor reported 100% health while displaying stale information

These are not edge cases. They are the normal consequence of a system
that checks presence but not consistency.

A partner system that reports false health is worse than one that
reports nothing. False confidence destroys trust.
The forest must earn trust through demonstrated accuracy — not claim it.

## The Three Tiers of Health
```
Tier 1 — Presence (CURRENT):
  Is the file/tool/service/config there?
  Doctor already does this reasonably well.
  23/24 checks pass = system is present.

Tier 2 — Consistency (MISSING — this intent):
  Does what is present match what is claimed?
  Does the code match the state?
  Does the state match the config?
  Does the config match reality?

Tier 3 — Self-Repair (MISSING — this intent):
  Can the system correct inconsistencies automatically?
  Can it propose fixes for what it cannot correct?
  Can it communicate clearly what requires human attention?
```

## What Consistency Means in Practice

### Category 1: Intent Ledger Consistency
```
CHECK: Every intent with status: complete lives in intents/complete/
CHECK: Every intent with status: planned lives in intents/future/
CHECK: Every intent with status: deferred has a documented reason
CHECK: No intent ID appears twice across any directory
CHECK: in-progress intents — only one should exist at a time
CHECK: cicomplete was called after every intent marked complete
FIX:   Move misplaced intents to correct directory automatically
ALERT: Duplicate IDs require human resolution
```

### Category 2: Registry Consistency
```
CHECK: Every tool in registry with deployable=true exists in scripts/
CHECK: Every tool version in registry matches Cargo.toml version
CHECK: No retired tool exists in scripts/ (cleanup prompt)
CHECK: Every rust tool in scripts/ exists in registry
CHECK: Script tools actually are shell scripts (not binaries)
FIX:   Update registry versions from Cargo.toml automatically
FIX:   Add unregistered scripts to registry with prompt
ALERT: Missing deployable tools require: deploy <name>
```

### Category 3: Jarvis Score Consistency  
```
CHECK: Milestone pointer matches actual score
CHECK: Factor descriptions reflect actual system state
CHECK: Completed intents referenced in factors exist in complete/
CHECK: Score factors sum to displayed total
CHECK: No hardcoded "not yet" for completed intents
FIX:   Rebuild Jarvis score from live data on every check
ALERT: Score drift > 5 points since last check
```

### Category 4: Autostart Consistency
```
CHECK: Every niri spawn-at-startup tool exists in scripts/
CHECK: Every systemd user service that should run is running
CHECK: No tool in autostart has been retired in registry
CHECK: Autostart tools have --health flags
FIX:   Remove retired tools from autostart config automatically
ALERT: Crashed services — report with: journalctl --user -u <service>
```

### Category 5: State Database Consistency
```
CHECK: state.db is not corrupted (PRAGMA integrity_check)
CHECK: WAL mode is active (INT-166)
CHECK: No orphaned records (prediction_outcomes without predictions)
CHECK: forest_memory confidence scores are in valid range (0-100)
CHECK: Jarvis readiness log has an entry from last 24 hours
FIX:   Run VACUUM on stale/orphaned records
FIX:   Insert Jarvis log entry if missing
ALERT: Corruption requires: core db restore
```

### Category 6: Code/Documentation Consistency
```
CHECK: CHANGELOG.md last entry matches current version
CHECK: README tool count matches registry tool count
CHECK: README intent count matches intent ledger count
CHECK: AUTOSTART-MAP.md matches actual niri config
FIX:   Run: faelight-docs sync automatically
ALERT: Manual documentation drift requires review
```

### Category 7: Shell Configuration Consistency
```
CHECK: Alias count in aliases.zsh matches doctor alias coverage check
CHECK: No alias points to a retired tool
CHECK: fsh config.fsh aliases match registered tools
CHECK: No stale Sway/Hyprland references in any config
FIX:   Flag stale aliases for removal
ALERT: Alias count drift > 10 from last check
```

## The Self-Repair Model

### Three Repair Tiers
```
AUTO-FIX (no human required):
  - Registry version sync from Cargo.toml
  - faelight-docs sync (README/welcome update)
  - Jarvis score factor refresh from live data
  - Move intent files to correct directory when status is clear
  - Insert missing Jarvis log entries
  - VACUUM orphaned DB records

PROPOSE (human confirms with y/n):
  - "INT-133 has status: complete but is in future/ — move? (y/n)"
  - "faelight-clipboard in niri autostart but disabled — remove? (y/n)"
  - "faelight-notify version drift: registry 2.1.0, deployed 4.0.0 — sync? (y/n)"

ALERT (human must act):
  - Tool missing from scripts/ — run: deploy <name>
  - state.db corruption — run: core db restore
  - Duplicate intent ID — requires manual resolution
  - systemd service failed — requires investigation
```

### The Repair Loop
```
d runs
  → Tier 1 checks (presence) — existing
  → Tier 2 checks (consistency) — NEW
    → inconsistencies found
      → auto-fixable? → fix silently, log to integrity_log
      → proposable?   → show "⚠️  PROPOSE: <description> — fix? (y/n)"
      → alert-only?   → show "❌ ALERT: <description> — action required"
  → Tier 3 report
    → "Auto-fixed: N issues"
    → "Proposed: N issues (run: core integrity fix)"
    → "Alerts: N issues requiring attention"
```

## The Integrity Log

Every auto-fix, proposal, and alert is written to state.db:
```sql
CREATE TABLE IF NOT EXISTS integrity_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    category    TEXT    NOT NULL,  -- intent/registry/jarvis/autostart/db/docs/shell
    check_name  TEXT    NOT NULL,
    severity    TEXT    NOT NULL,  -- auto-fix/propose/alert
    description TEXT    NOT NULL,
    fixed       INTEGER NOT NULL DEFAULT 0,
    fixed_at    INTEGER,
    detected_at INTEGER NOT NULL
);
```

This gives us:
- History of what was wrong and when
- What was auto-fixed vs what required human action
- Trend analysis: is the system getting more or less consistent over time?

## New Doctor Commands
```bash
d                      # existing — now includes integrity summary at bottom
core integrity run     # full integrity scan with repair options
core integrity log     # history of all detected issues
core integrity fix     # apply all pending proposals
core integrity status  # current consistency score
```

## The Consistency Score

Doctor currently shows Health % (presence).
After this intent, doctor shows two numbers:
```
Health:      100%   (all systems present)
Integrity:    87%   (13 consistency issues detected)
```

Integrity % = (total checks - issues) / total checks × 100

The Jarvis readiness score should factor in Integrity:
```
+5 Integrity ≥ 90%
+3 Integrity ≥ 75%
+0 Integrity < 75%
```

## Phase Plan

### Phase 0 — Integrity Infrastructure (1 session)
- integrity_log table in state.db
- IntegrityCheck struct with severity enum
- Auto-fix, propose, alert execution engine
- core integrity run command skeleton

### Phase 1 — Intent Ledger Integrity (1 session)
- Scan all intent directories vs status fields
- Detect and auto-move misplaced intents
- Detect duplicate IDs
- Report in doctor output

### Phase 2 — Registry Integrity (1 session)
- Version sync from Cargo.toml
- Missing tool detection
- Retired tool cleanup
- Unregistered script detection

### Phase 3 — Jarvis Score Integrity (1 session)
- All factors read from live data
- Milestone pointer always correct
- Score sum verified
- Hardcoded stale strings eliminated

### Phase 4 — Autostart + State DB Integrity (1 session)
- Autostart config vs registry
- state.db PRAGMA integrity_check
- Orphaned record cleanup
- WAL mode verification

### Phase 5 — Documentation + Shell Integrity (1 session)
- README vs live counts
- Alias vs registry
- Stale config references
- Auto-sync on drift

### Phase 6 — Doctor Integration + Integrity Score (1 session)
- Integrity % shown in doctor header
- Auto-fix runs silently on every d
- Propose queue shown after doctor output
- Jarvis score integrates integrity factor

### Phase 7 — Continuous Self-Monitoring (1 session)
- Integrity log trend analysis
- Regression detection (was 95% last week, now 80% — why?)
- Weekly integrity digest in core strategy week
- Feed into v13 Autonomy decision engine

## Architectural Foundation (non-negotiable)

### 1. IntegrityCheck Trait — Everything is This
```rust
pub enum Severity { AutoFix, Propose, Alert }

pub struct IntegrityIssue {
    pub category:    Category,
    pub check:       &'static str,
    pub severity:    Severity,
    pub description: String,
    pub fix:         Option<FixAction>,
    pub weight:      u8, // 1-5: 1=trivial, 5=critical
}

pub trait IntegrityCheck {
    fn name(&self)     -> &'static str;
    fn category(&self) -> Category;
    fn run(&self, ctx: &IntegrityContext) -> Vec<IntegrityIssue>;
}
```

Every check — without exception — implements this trait.
No ad hoc logic. No special cases. No closures.

### 2. Execution Pipeline (strict phases, no exceptions)
```
Phase A — Scan    (pure, no mutation — collect all issues)
Phase B — Plan    (classify: auto-fix / propose / alert)
Phase C — Apply   (execute auto-fixes only)
Phase D — Re-scan (re-run affected domains only)
Phase E — Report  (display proposals + alerts)
```

**Why re-scan matters:** Without Phase D, the system reports
issues that were just fixed. That is false data. False data destroys trust.

### 3. Deterministic Check Order (fixed, not configurable)
```
1. IntentChecks      (foundation — affects Jarvis)
2. RegistryChecks    (affects autostart + docs)
3. JarvisChecks      (depends on intent + registry)
4. AutostartChecks   (depends on registry)
5. DbChecks          (affects Jarvis logs)
6. DocsChecks        (depends on registry + intents)
7. ShellChecks       (depends on registry)
8. TemporalChecks    (clock drift, scan freshness)
```

Order is mandatory. Intent fixes feed into Jarvis checks.
Registry fixes feed into autostart + docs checks.
Running them out of order = logical race conditions.

### 4. Typed FixActions (not strings, not closures)
```rust
pub enum FixAction {
    MoveFile         { from: PathBuf, to: PathBuf },
    UpdateRegistry   { tool: String, version: String },
    RemoveAutostart  { tool: String },
    InsertDbRow      { table: String, values: Vec<String> },
    VacuumDb,
    SyncDocs,
    UpdateJarvisFactor { factor: String, value: String },
}
```

Typed actions are: loggable, replayable, testable, previewable.
String-based fixes are none of those things.

### 5. Persistent Proposal Queue
```sql
CREATE TABLE IF NOT EXISTS pending_fixes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    action      TEXT    NOT NULL,  -- JSON-serialized FixAction
    description TEXT    NOT NULL,
    created_at  INTEGER NOT NULL,
    applied_at  INTEGER
);
```

Without persistence, proposals disappear between sessions.
The drift persists. The user forgets. Nothing gets fixed.

### 6. Weighted Integrity Score
```
Integrity % = 1 - (sum(issue.weight) / sum(all_check_weights))
```

Weight scale:
- 1 = trivial (doc count mismatch)
- 2 = minor (registry version drift)
- 3 = moderate (intent in wrong directory)
- 4 = significant (tool missing from scripts/)
- 5 = critical (DB corruption, crashed service)

10 trivial doc issues ≠ 1 DB corruption issue.
The naive count treats them as equal. That is wrong.

### 7. Safe vs Heavy Mutations

`d` (safe only — no destructive operations):
- Registry version sync from Cargo.toml
- Jarvis factor refresh from live data
- Insert missing DB rows
- faelight-docs sync

`core integrity run` (full engine including heavy mutations):
- Move intent files between directories
- Remove retired tools from autostart config
- VACUUM database
- Config rewrites

**Safe = no file deletion, no config rewrite, no destructive ops.**

### 8. Category 8: Temporal Consistency (missing from original)
```
CHECK: Last successful doctor run < 24h ago           weight: 2
CHECK: Last integrity scan < 24h ago                  weight: 2
CHECK: No intent marked complete with future date     weight: 3
CHECK: System clock sane (no >1hr drift)              weight: 4
CHECK: Jarvis readiness log has entry < 24h           weight: 2
```

Time drift is a silent corruption multiplier.
A system with clock drift produces meaningless timestamps in:
- prediction accuracy windows
- integrity log history
- forest_predictions expires_at

### What We Are Actually Building

Not "a better doctor."

A **Local Consistency Oracle** — a system that verifies its own state
before allowing autonomous action.

v13 autonomy doesn't just act.
It acts on **verified state**.

That distinction separates automation from intelligence.

## Gate Check


```
⬜ integrity_log table in state.db
⬜ IntegrityCheck struct — severity: auto-fix/propose/alert
⬜ core integrity run — full scan with repair options
⬜ core integrity log — history of all issues
⬜ core integrity fix — apply pending proposals
⬜ Intent ledger integrity — status vs directory mismatch detected and fixed
⬜ Registry integrity — version drift detected and synced
⬜ Jarvis score integrity — all factors read from live data, no hardcoded stale strings
⬜ Autostart integrity — retired tools flagged, crashed services reported
⬜ State DB integrity — corruption check, orphan cleanup, WAL verified
⬜ Documentation integrity — README counts match live data
⬜ Shell integrity — stale aliases flagged
⬜ Doctor header shows Integrity % alongside Health %
⬜ Auto-fix runs silently on every d
⬜ Jarvis score integrates integrity factor
⬜ Integrity log trend analysis — regression detection
```

## Why This Comes Before v13

v13 Autonomy acts within mandates.
But autonomous action on inconsistent data is not autonomy — it is chaos.

Before the forest can act autonomously it must:
1. Know its own state accurately
2. Detect when its state is inconsistent
3. Fix what it can fix
4. Report what it cannot

INT-184 is the prerequisite for v13 activation.
A system with 87% integrity cannot be trusted with 95/100 autonomy.
A system with 99% integrity has earned it.

The gate for v13 is not just Jarvis ≥ 95/100.
The gate for v13 is Jarvis ≥ 95/100 AND Integrity ≥ 95%.

## The Relationship to the User

Right now when something is wrong, Christian finds it.
The system waits to be told what is broken.

After INT-184:
- The system finds it first
- The system fixes what it can
- The system tells Christian what it fixed
- The system asks about what it cannot fix
- The system remembers what went wrong and learns from it

That is the difference between a tool and a partner.
A tool waits. A partner watches.

## The Phrase

**"A system that cannot audit itself
cannot be trusted to act for you.
Integrity is not a feature.
It is the prerequisite for everything else.

The forest that knows where its roots are broken
can grow stronger.
The forest that does not know
falls in the first storm."**

---
*"Health says: everything is present.
Integrity says: everything is true.
We have built Health.
Now we build Truth."* 🌲
