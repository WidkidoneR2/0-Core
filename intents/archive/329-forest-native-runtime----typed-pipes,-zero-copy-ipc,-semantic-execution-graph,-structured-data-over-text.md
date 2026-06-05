---
id: 329
title: "Forest Native Runtime -- typed pipes, zero-copy IPC, semantic execution graph, structured data over text"
status: planned
date: 2026-05-21
tags: [forest, runtime, pipes, ipc, semantic, typed, zero-copy, execution, graph, shell]
---

INT-329 -- Forest Native Runtime -- The Shell That Thinks in Types
date: 2026-05-21

---
THE PREMISE

UNIX pipes are one of the greatest ideas in computing.
They are also 50 years old.

The fundamental assumption of UNIX pipes:
  Everything is text.
  Tools read text, write text, pass text.
  Meaning is inferred by the next tool.

This works. It has worked for decades.
But it has a ceiling.

  ps aux | grep firefox | awk '{print $1}'

Three tools. Three text parsers. Three opportunities for misalignment.
The meaning of the data is destroyed at each pipe boundary
and reconstructed by the next tool from scratch.

The forest already knows more than this.
Friday knows about processes. Core knows about intents. fsh knows about commands.
But they communicate through text -- the lowest common denominator.

INT-329 replaces the assumption.

Not text pipes. Typed pipes.
Not string parsing. Structured data.
Not grep and awk. Forest query language.
Not PIDs. Processes with intent, ownership, confidence, risk.

  processes | where memory > 500mb | owned-by christian | kill-if idle > 30min

Internally:
  typed Rust structs
  shared memory transport
  zero-copy IPC
  semantic execution graph
  Friday-aware pipeline

This is the forest's native runtime.
The layer between the shell and the kernel
that makes the forest speak its own language.
---
THE UNIX ASSUMPTION AND WHY TO REPLACE IT

UNIX text pipes are brilliant because they are universal.
Any tool can pipe to any other tool.
The cost: meaning is lost at every boundary.

  ps aux                    -- outputs text
  grep firefox              -- searches text
  awk '{print $1}'          -- parses text
  kill $(...)               -- receives text

At no point does the system know:
  What a process IS
  Why it is running
  Who owns it in a meaningful sense
  What its relationship to other processes is
  Whether killing it is safe

The forest already models all of this:
  Processes have owners (intent system)
  Processes have context (Friday patterns)
  Processes have risk (health monitoring)
  Processes have history (state.db)

The typed pipe makes this knowledge flow through the pipeline
instead of being thrown away at each text boundary.
---
THE FOREST QUERY LANGUAGE (FQL)

FQL is the human-readable layer on top of typed pipes.
It feels like English. It compiles to typed operations.

Examples:

  processes | where memory > 500mb
  -- Returns: Vec<Process> filtered by RSS > 500MB

  files in ~/0-core | newer-than 2h | not committed
  -- Returns: Vec<File> with git status embedded

  intents | status in-progress | blocked-by INT-308
  -- Returns: Vec<Intent> with dependency graph

  deploys | last 7 days | failed | tool faelight-bar
  -- Returns: Vec<Deploy> from deploy_patterns

  friday | patterns | confidence > 0.8 | triggered today
  -- Returns: Vec<FridayPattern> from state.db

  system | health | trend last 30 days
  -- Returns: HealthTimeSeries with forecast

The pipeline is lazy: each stage is a Rust iterator.
Nothing is computed until the terminal stage consumes it.
Friday can inspect the pipeline before execution:
  "This pipeline will kill 3 processes. Confidence: 0.91. Proceed?"
---
THE TYPE SYSTEM

Core forest types (Rust structs, serializable):

  struct Process {
      pid: u32,
      name: String,
      owner: String,
      intent: Option<IntentRef>,    // which forest intent spawned this?
      memory_rss: u64,
      cpu_percent: f32,
      started_at: i64,
      last_active: i64,
      risk: RiskLevel,              // Low/Medium/High/Critical
      friday_context: Option<String>, // what Friday knows about this
  }

  struct File {
      path: PathBuf,
      size: u64,
      modified: i64,
      git_status: Option<GitStatus>, // Staged/Modified/Untracked/Clean
      intent: Option<IntentRef>,     // which intent owns this file?
      forest_role: FileRole,         // Source/Config/Data/Log/Temp
  }

  struct Intent {
      id: u32,
      title: String,
      status: IntentStatus,
      health: u8,
      blockers: Vec<u32>,
      dependents: Vec<u32>,
      velocity: f32,
      friday_signal: Option<String>,
  }

  struct Deploy {
      tool: String,
      version: String,
      outcome: DeployOutcome,
      duration_ms: u64,
      timestamp: i64,
      commit: String,
      friday_prediction: Option<f64>,
      friday_actual: Option<bool>,
  }

  struct HealthSnapshot {
      percent: u8,
      checks: Vec<CheckResult>,
      timestamp: i64,
      trend: Trend,
      forecast_24h: u8,
  }

These types flow through the pipeline without serialization loss.
Each stage receives typed data, transforms it, passes it forward.
---
ZERO-COPY IPC TRANSPORT

The transport layer between fsh stages:

Option A -- Shared memory (fastest):
  Producer writes typed struct to shared memory region.
  Consumer reads directly -- no copy, no serialization.
  Works within a single machine.
  Uses: memfd_create, mmap

Option B -- Unix socket with length-prefixed frames (simplest):
  Producer serializes to bytes (bincode or cap'n proto).
  Sends over Unix socket.
  Consumer deserializes.
  Works across processes, even across network (future).

Option C -- Tokio channels (current fsh architecture):
  Producer sends to tokio::sync::mpsc channel.
  Consumer receives typed value.
  Works within a single async runtime.
  Zero serialization overhead within process.

Phase 1 uses Option C (tokio channels within fsh).
Phase 2 upgrades to Option B (Unix sockets for cross-process).
Phase 3 explores Option A (shared memory for hot paths).

The API is the same regardless of transport.
The pipeline operator (|) hides the transport entirely.
---
THE SEMANTIC EXECUTION GRAPH

Every FQL pipeline compiles to an execution graph:

  processes | where memory > 500mb | kill

Becomes:

  Source(ProcessIterator)
    -> Filter(memory_rss > 524288000)
    -> Sink(KillAction { confirm: true, risk: High })

The graph is inspectable before execution:
  fsh can show the plan: "processes | where memory > 500mb | kill --dry-run"
  Friday can assess risk: "Killing 3 processes. Risk: Medium. Proceed? [y/N]"
  The plan can be saved: "core plan save cleanup-heavy-procs"
  The plan can be replayed: "core plan run cleanup-heavy-procs"

This is the difference between a command that runs and a command that reasons.
The execution graph is the forest's understanding of what it is about to do.
---
FRIDAY INTEGRATION

Friday is not separate from the typed pipe system.
Friday is a pipeline stage.

  processes | where memory > 500mb | friday assess | kill-if risk < medium

The "friday assess" stage:
  Receives Vec<Process>
  Queries friday_knowledge for context on each process
  Adds friday_context field to each Process struct
  Passes enriched Vec<Process> forward

  deploys | last 30 days | friday correlate-health | report

The "friday correlate-health" stage:
  Receives Vec<Deploy>
  Joins with health_patterns from state.db
  Adds correlation data: "faelight-bar deploys correlate with health dips"
  Returns enriched dataset

Friday becomes the semantic layer of the pipeline.
Not a separate AI call. A pipeline stage that adds meaning.
---
FSH INTEGRATION

FQL is built into fsh as the forest pipe operator.

Current fsh:
  cat /etc/os-release | grep VERSION
  -- Text pipe. UNIX. Works.

Future fsh:
  system | info | where key = VERSION
  -- Typed pipe. Forest. Knows what VERSION is.

Both work. UNIX pipes still work.
The forest pipe (|>) is a new operator, distinct from UNIX pipe (|).

  processes |> where memory > 500mb   -- forest pipe, typed
  ps aux | grep firefox               -- UNIX pipe, text, still valid

The forest pipe operator (|>) signals to fsh:
  "This is a forest-native pipeline. Use the typed transport."
  "Apply Friday context automatically."
  "Show the execution graph before running if risk > medium."

Backward compatibility is preserved.
UNIX pipes work forever.
The forest pipe is an upgrade path, not a replacement mandate.
---
PROTOTYPE PLAN (in R&D VM -- INT-328)

Phase 1 -- Core types and simple queries (VM):
  Define Process, File, Intent, Deploy structs in faelight-core
  Implement ProcessIterator (reads from /proc)
  Implement FileIterator (reads from filesystem)
  Implement IntentIterator (reads from state.db)
  Add |> operator to fsh parser
  Implement: processes |> where memory > 500mb
  Gate: query returns typed Vec<Process> in fsh, not text

Phase 2 -- Pipeline composition (VM):
  Implement Filter, Sort, Limit, Map stages
  Implement friday assess stage
  Implement dry-run execution graph display
  Gate: multi-stage pipeline works, graph shown before execution

Phase 3 -- Actions (VM):
  Implement Kill, Archive, Tag, Report sink stages
  Implement confirmation gates (Friday-aware)
  Gate: kill stage requires confirmation, Friday assesses risk

Phase 4 -- Cross-process transport (VM):
  Move from tokio channels to Unix socket transport
  Multiple fsh instances can share typed data
  Gate: pipeline works across two terminal windows

Phase 5 -- Graduate to real machine:
  All VM gates passed
  Add to fsh daily driver
  Friday learns pipeline patterns
  Gate: 1 week of daily use, no regressions
---
GATES
[ ] Process, File, Intent, Deploy types defined in faelight-core
[ ] |> operator added to fsh parser (distinct from |)
[ ] processes |> where memory > 500mb works, returns typed data
[ ] Multi-stage pipeline: processes |> where memory > 500mb |> sort memory desc
[ ] friday assess stage enriches pipeline data with Friday context
[ ] Execution graph shown for high-risk pipelines (--dry-run)
[ ] Kill/Archive/Report sink stages implemented with confirmation
[ ] Prototyped and validated in R&D VM (INT-328)
[ ] 1 week daily use on real machine, no regressions
[ ] UNIX pipe (|) still works unchanged

DEPENDS ON
INT-328 (R&D Environment) -- prototype happens in VM first
INT-326 (fsh Semantic Architecture) -- three-layer execution model
INT-261 (fsh Vocabulary) -- |> becomes forest vocabulary
INT-327 (self-healing) -- processes type shared with desktop graph

TIMELINE
Prototype: after INT-328 Phase 1 (VM forest installed)
Phase 1-2: 2-3 sessions in VM
Phase 3-4: 1-2 sessions in VM
Graduate: before NY presentation if Phase 1-3 complete
Full pipeline: post-presentation

"Text pipes were the right answer for 1974.
Typed pipes are the right answer for now.
The forest knows what a process is.
It should be allowed to say so." 🌲
