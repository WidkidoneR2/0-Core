---
id: 218
date: 2026-04-09
type: planned
title: "Friday Knowledge Engine — Situated Learning and Conflict Resolution"
status: in-progress
tags: [friday, knowledge, learning, rust, arch, python, situated, conflicts, v19-prep]
requires: [203,216,217]
unlocks: [219,220]
strategic_value: multiplier
---
A general AI knows Rust.
Friday needs to know YOUR forest's Rust.
These are not the same thing.
A general Rust expert knows the language specification.
Friday needs to know:
- rusqlite double unwrap_or causes specific type error — fix: remove one
- String replacement on Rust files with Unicode = catastrophic corruption — fix: use git checkout then redeploy
-
- The build workflow: mod.rs → commands.rs → parser.rs → cli/mod.rs → dispatcher.rs
- deploy core takes 13 seconds — if under 3s, build did not recompile
- fsh multiline python3 -c fails — write to /tmp instead
- faelight-term keyboard changes can silently break all input — test in isolation first
This is situated knowledge.
It only exists in the context of this specific forest.
It is worth more than any documentation.
It was earned through failure.
Friday's job is to make sure those failures only happen once.
All forest knowledge sources indexed and queryable:
  - intents/incidents/ — every system failure and resolution
  - runtime/journal/ — session narratives, lessons learned
  - intents/complete/ — every completed intent with gates
  - events table — past commands and their outcomes
  - git log — what changed when, what broke what
Index structure:
```rust
pub struct KnowledgeEntry {
    pub id: Uuid,
    pub source: KnowledgeSource,   // Incident | Journal | Intent | Event
    pub domain: String,            // "rust", "arch", "python", "shell", "niri"
    pub error_signature: Option<String>,  // normalized error pattern
    pub resolution: String,
    pub confidence: f32,           // how reliable is this resolution?
    pub occurrence_count: u32,     // seen this before?
    pub last_seen: DateTime<Utc>,
}
```
When a build fails or a command errors:
1. Extract error signature (normalized — strip file paths, line numbers)
2. Query knowledge index: "have we seen this before?"
3. If yes: return resolution with confidence score
4. If no: log as new pattern, return generic guidance
Error signatures are normalized:
  "error[E0277]: `X` doesn't implement `Debug`"
  → normalized: "error[E0277]: missing_derive_debug"
  → known resolution: "add #[derive(Debug)] to enum"
  → confidence: 0.99 (seen 47 times)
**Rust / Cargo:**
- Build error patterns → resolutions
- Clap derive macro requirements
- Trait bound errors → missing derives
- Move/borrow errors → common patterns
- Lifetime issues → workarounds used in this forest
- Cargo.toml patterns (bundled rusqlite, feature flags)
**Arch Linux / Pacman:**
- Update conflict patterns → resolutions
- Systemd service failures → fix history
- Package dependency conflicts
- AUR helper patterns (paru)
- Kernel/driver conflicts
**Python (in this forest):**
- fsh multiline limitation → write to /tmp
- Common script patterns used
- Library versions in use
**fsh specifics:**
- Known bugs (inline env var assignment, tab completion)
- Pipe interception behavior
- Heredoc limitations
- Which commands fall back to sh
**Niri:**
- Config syntax patterns
- Keybind conflicts
- Session restart requirements
**state.db:**
- Schema evolution history
- Known query patterns
- WAL mode requirements
- SQLite version constraints
When Friday detects a conflict (build error, system error, unexpected behavior):
Step 1: Extract error signature
Step 2: Query knowledge index (confidence >= 0.8 → apply immediately)
Step 3: If < 0.8 confidence → present options ranked by confidence
Step 4: Human chooses → outcome recorded → knowledge updated
Step 5: If novel error → log as incident, return generic guidance
Friday never silently applies a fix.
Friday presents the fix with: evidence, confidence, past occurrence count.
Human approves. Outcome recorded. Model updated.
Every time a conflict is resolved:
1. Resolution recorded in knowledge_entries table
2. Confidence updated based on outcome
3. Error signature pattern strengthened
4. Related patterns cross-referenced
Every time Friday is WRONG:
1. Negative outcome recorded
2. Confidence decayed for that pattern
3. Alternative resolution explored
4. Lesson stored as high-priority entry
  core knowledge search <term>    — find relevant past lessons
  core knowledge show <id>        — full entry with resolution history
  core knowledge add              — manually add a lesson
  core knowledge patterns         — show known error patterns by domain
  core knowledge accuracy         — Friday resolution accuracy by domain
When Friday is active and a build error occurs:
  1. Knowledge engine queries automatically
  2. If confidence >= 0.85: Friday presents resolution inline
  3. If confidence < 0.85: Friday presents top 3 options
  4. Human confirms → applied
  5. Outcome → knowledge updated
Example:
  Build error: "error[E0277]: missing Debug derive"
  Friday: "This is the missing derive pattern (seen 47 times, 99% confidence).
           Fix: add
           Confirm? (y/n)"
```sql
CREATE TABLE knowledge_entries (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    domain TEXT NOT NULL,
    error_signature TEXT,
    description TEXT NOT NULL,
    resolution TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.5,
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    success_count INTEGER NOT NULL DEFAULT 0,
    failure_count INTEGER NOT NULL DEFAULT 0,
    last_seen INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE TABLE knowledge_outcomes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id TEXT NOT NULL,
    outcome TEXT NOT NULL,
    was_correct INTEGER NOT NULL,
    recorded_at INTEGER NOT NULL
);
```
On first run, seed with known forest lessons:
- rusqlite double unwrap_or error → fix
- clap
- String replacement Unicode corruption
- fsh multiline python3 limitation
- faelight-term keyboard regression pattern
- Build workflow order (canonical)
- git -C ~/0-core checkout <file> for recovery
- deploy takes 13s for core, 1-2s for cached
These are the lessons bought with real failures.
They should never have to be learned again.
✅ knowledge_entries and knowledge_outcomes tables created (2026-04-15)
✅ KnowledgeEntry struct defined with all fields (2026-04-15)
✅ Error signature normalization working (strip paths/line numbers) (2026-04-15)
⬜ core knowledge search <term> — queries by domain and signature
⬜ core knowledge show <id> — full entry with resolution
⬜ core knowledge add — manual lesson recording
⬜ core knowledge patterns — error patterns by domain
⬜ core knowledge accuracy — resolution accuracy by domain
⬜ Seed knowledge loaded — 10+ known forest lessons pre-loaded
⬜ Build error → auto-query knowledge engine
✅ Friday presents resolution with confidence + occurrence count -- deferred to v19
⬜ Outcome recording — correct/incorrect updates confidence
✅ Cross-domain pattern detection (same error, different domain) -- deferred to v19
⬜ Integration with Friday active mode — speaks on conflict detection
"The forest has made every mistake once.
Friday's purpose is to make sure
it only needs to make each mistake once.
Not because the forest is fragile —
but because time is finite
and every repeated failure
is time that could have been spent building.
Friday does not know Arch Linux.
Friday knows THIS forest on Arch Linux.
That is worth more." 🌲
