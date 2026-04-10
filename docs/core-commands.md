# Core Commands Guide
---
**Purpose:** The forest anticipates what comes next. Session patterns, health trajectory, intent velocity, coupling risk, prediction accuracy.
**Key subcommands:**
  core predict next          — what intent ships next?
  core predict health        — health trajectory forecast
  core predict session       — current session pattern analysis
  core predict accuracy      — how accurate have predictions been?
  core predict velocity      — intent completion velocity
**Notes:** Feeds directly into core strategy. Accuracy builds over time — minimum 14 days for meaningful signal.
**Purpose:** The forest responds without being asked. Rules that fire automatically based on system state.
**Key subcommands:**
  core react story           — what has the forest been signaling today?
  core react rules           — list all active reaction rules
  core react status          — current rule firing status
**Notes:** Core v10. Rules fire on health drops, security aging, checkpoint staleness, intent overflow.
**Purpose:** The forest plans across multiple time horizons. Coherence, jarvis readiness, trust building.
**Key subcommands:**
  core strategy jarvis       — Jarvis readiness score breakdown (currently 105/100)
  core strategy horizon      — short/medium/long horizon planning
  core strategy coherence    — are active intents coherent with each other?
  core strategy trust        — trust-building trajectory
  core strategy now          — immediate priorities
**Notes:** Core v12. Jarvis score gates v14 Partnership activation (gate: 98/100).
**Purpose:** The forest sets its own goals. Generate, evaluate, accept, reject, prioritize.
**Key subcommands:**
  core goals generate        — forest proposes new goals from patterns
  core goals list            — all active goals
  core goals accept <id>     — accept a proposed goal
  core goals reject <id>     — reject with reasoning
  core goals prioritize      — rerank by live conditions
**Notes:** Core v9. Goals feed into planning and strategy.
**Purpose:** Mandate system and autonomous action engine. The forest chooses its own purpose within defined boundaries.
**Key subcommands:**
  core autonomy status       — current autonomy level and mandate
  core autonomy mandate      — view active mandate
**Notes:** Core v13. Complete. Jarvis 95/100 achieved.
---
**Purpose:** 5-phase collaborative intelligence. The forest thinks alongside you.
**Key subcommands:**
  core partner status        — Jarvis score, alignment, all 5 phases
  core partner propose       — forest proposes actions based on patterns
  core partner discuss <id>  — forest discusses an intent
  core partner disagree <id> — forest flags concerns with evidence
  core partner consult <q>   — ask the forest a question
  core partner reflect       — longitudinal pattern reflection
  core partner pattern       — work pattern analysis
  core partner growth        — forest growth metrics
  core partner pushback      — review disagreement history
  core partner roadmap       — co-authored roadmap view
**Notes:** Core v14. Active at Jarvis 105/100. All 5 phases engaged.
**Purpose:** Declare and manage your principles. The machine-readable conscience.
**Key subcommands:**
  core values list           — show all declared values with weights
  core values define <stmt>  — declare a new value (--weight N --scope S)
  core values remove <id>    — deactivate a value
  core values weight <id> N  — update priority weight
**Seed values (loaded automatically):**
  "manual control over automation"             weight 10
  "nothing runs without explicit human auth"   weight 10
  "understanding over convenience"             weight 9
  "recovery over perfection"                   weight 8
  "focus > speed"                              weight 8  scope: intents
  "ship consistently"                          weight 7  scope: commits
**Notes:** Core v15. Values feed alignment checking and partner disagreement grounding.
**Purpose:** Check behavioral consistency against declared values. Detect drift before it compounds.
**Key subcommands:**
  core align check <subject> — score an action against your values (0-100%)
  core align drift           — behavioral drift report for last 30 days
  core align report          — weekly alignment conscience check
  core align report --weeks-ago N  — report for N weeks ago
**Notes:** Core v15. Observations are strictly behavioral — never personal. Score above 80% = proceed.
**Purpose:** Trust contracts and safe autonomy simulation. Earn delegation before granting it.
**Key subcommands:**
  core delegate simulate <action>  — test without executing
  core delegate contracts          — list all trust contracts
  core delegate history            — what has been delegated
  core delegate accuracy           — simulation accuracy (3D: action/outcome/calibration)
  core delegate activate <contract>— enable real delegation (after accuracy gates pass)
  core delegate suspend            — pause all delegation instantly
**Activation requires:**
  action_match >= 0.85
  outcome_success >= 0.80
  calibration_error <= 0.10
**Notes:** Clock running since 2026-04-03. Gate requires 14+ days simulation data.
---
**Purpose:** Coordinate all forest engines. No silent upgrades. Forest thinks as one.
**Key subcommands:**
  core engines status        — show all 7 engines, versions, sync state
  core engines sync <engine> — acknowledge upgrade, update contracts
  core engines signals       — recent cross-engine signal flow
  core engines check         — verify all engines consistent
  core engines upgrade-log   — history of engine upgrades
**Current engines:**
  core 3.0.0               active
  faelight-contextd 0.1.0  active
  delegation 0.3.0         active
  friday 0.0.0             dormant (building toward)
  pattern-weight 0.0.0     planned
**Notes:** Built in INT-206. First upgrade logged: core 2.0.0 → 3.0.0.
---
**Purpose:** 23-check health monitoring with forecast, trend, and early warning.
**Key subcommands:**
  core doctor run            — full health check (also: just type d)
  core doctor quick          — fast check, fewer checks
  core doctor history        — health history over time
**Output includes:** health%, integrity%, forecast 24h/7d, trend, active intents
**Notes:** Run after every deploy. Auto-fixed integrity issues are normal.
**Purpose:** 13-check consistency oracle and self-repair engine.
**Key subcommands:**
  core integrity run         — full integrity scan
  core integrity apply       — auto-fix detected issues
  core integrity heal        — guided healing for complex issues
**Notes:** Checks: intent ledger, jarvis freshness, schema validation, duplicate detection, version drift.
---
**Purpose:** The forest's goal and work tracking system.
**Key subcommands:**
  core intent list           — all intents by status
  core intent show <id>      — full detail for one intent
  core intent new            — create new intent
  core intent new --smart    — AI-assisted creation with context
  core intent start <id>     — mark in-progress (also: cistart <id>)
  core intent complete <id>  — mark complete (also: cicomplete <id>)
  core intent search <term>  — search across all intents
  core intent health         — health scoring per intent
  core intent burndown       — completion trajectory
  core intent velocity       — shipping cadence
  core intent deps           — dependency graph
  core intent deps --critical-path  — longest blocking chain
  core intent edit <id>      — open intent file in $EDITOR
  core intent branch         — create git branch for intent
  core intent auto-link      — link related intents automatically
  core intent predict        — predict completion date
  core intent stats          — aggregate statistics
**Notes:** 157 complete, 13 planned. The ledger is the memory of the forest.
---
**Purpose:** Decision ledger with context fingerprints and outcome tracking.
**Key subcommands:**
  core decision list         — all decisions and outcomes
  core decision record <d>   — record a new decision
  core decision outcome <id> success|partial|failure|unknown
  core decision show <id>    — full detail
  core decision hindsight    — what decisions look like in retrospect
  core decision advise       — judgment advisory for current state
  core decision heuristics   — auto-derived rules from decision corpus
  core decision lessons      — human-readable lessons summary
  core decision stats        — correlation statistics
  core decision patterns     — repeating decision patterns
  core decision friction     — decisions requiring repeated corrections
  core decision reversal     — architectural reversals detected
  core decision story        — 30-day narrative
**Purpose:** Shorthand for core decision record.
  core decide "description"  — record decision quickly
---
**Purpose:** Query the forest event ledger. Everything the forest has done.
**Key subcommands:**
  core events list           — events from today
  core events since <dur>    — events since duration (1h, 30m, 2d)
  core events filter <dom>   — filter by domain
  core events status         — event log status and size
  core events watch          — live event stream
  core events archive        — compress old events
**Purpose:** Causality engine. Why is the system in this state?
  core why                   — what led to current state?
**Purpose:** Full event trace with detail.
  core trace                 — last 10 events with full context
---
**Purpose:** State snapshots with full forest context.
**Key subcommands:**
  core checkpoint create     — snapshot current state
  core checkpoint list       — all checkpoints
  core checkpoint restore <name>  — restore to checkpoint
  core checkpoint diff       — what changed since last checkpoint
**Notes:** Auto-created before every cistart and cicomplete.
---
**Purpose:** Security audit, debt tracking, hardening verification.
**Key subcommands:**
  core security audit        — full security scan
  core security harden       — apply hardening recommendations
  core security debt         — outstanding security items
**Purpose:** Database backup and recovery.
**Key subcommands:**
  core db backup             — backup state.db
  core db vacuum             — optimize database
  core db stats              — database statistics
  core db query <sql>        — direct query (careful)
**Purpose:** Forest profile management.
**Key subcommands:**
  core profile list          — available profiles
  core profile switch <name> — switch active profile
  core profile create <name> — create new profile
---
**Purpose:** The forest story — how the system became what it is.
  core narrative             — full forest autobiography
**Purpose:** 30-day narrative of computing life.
  core story                 — what the forest has been doing
**Purpose:** How intents relate and evolved.
  core genealogy tree        — full intent family tree
  core genealogy show <id>   — ancestry of one intent
**Purpose:** The forest narrates its own goal history.
  core autobiography         — goal history as narrative
**Purpose:** Snapshot narrative — the forest writes its own story.
  core snapshot              — current moment narrative
---
  core predict coupling      — which tools change together?
  core predict risk          — what is most likely to break?
**Purpose:** Detect unexpected system changes.
  core anomaly               — scan for anomalies
**Purpose:** Tool health and intelligence audit.
  core audit                 — full tool audit with scores
**Purpose:** Architectural proposals from coupling and churn analysis.
  core evolution             — where should the architecture evolve?
**Purpose:** Simulate system scenarios.
  core simulate              — scenario simulation
**Purpose:** Dependency intelligence across the forest.
  core deps                  — full dependency map
**Purpose:** Stress test — verify stability before building further.
  core stress                — run stress test suite
---
  core version               — show core version and description
**Purpose:** Tool registry management.
**Key subcommands:**
  core registry list         — all registered tools
  core registry check        — verify registry consistency
  core registry reality-check — usage vs expected_usage
**Purpose:** Symbolic link management.
  core link                  — manage symlinks
**Purpose:** Workspace zone management.
  core zone                  — manage zones
**Purpose:** Application launcher integration.
  core launcher              — launcher management
**Purpose:** Core protection management.
  core lock                  — lock core (also: lock-core)
**Purpose:** Data fetching utilities.
  core fetch                 — fetch external data
**Purpose:** Forest notification system.
  core notify                — send notification
**Purpose:** Bootstrap intelligence — rebuild guidance.
  core bootstrap             — forest rebuild guide
**Purpose:** What the forest has learned.
  core lessons               — heuristics summary
**Purpose:** Judgment advisory for current state.
  core advise                — get forest judgment
**Purpose:** Decision hindsight summary.
  core hindsight             — what decisions look like now
**Purpose:** Show capability requirements for all domains.
  core capabilities          — domain capability map
**Purpose:** Break accepted goals into concrete steps.
  core plan                  — task planning
**Purpose:** Surface competing values in decisions.
  core tradeoff              — tradeoff analysis
**Purpose:** Dynamic prioritization by live conditions.
  core prioritize            — rerank goals now
---
| Need | Command |
|------|---------|
| Check system health | `d` or `core doctor run` |
| What to work on next | `core predict next` |
| Check alignment before acting | `core align check "description"` |
| See all intents | `core intent list` |
| Start an intent | `cistart <id>` |
| Complete an intent | `cicomplete <id>` |
| See engine status | `core engines status` |
| Jarvis score | `core partner status` |
| Record a decision | `core decide "description"` |
| Create checkpoint | `core checkpoint create` |
| Behavioral drift | `core align drift` |
| What caused this state | `core why` |
| Security status | `core security audit` |
---

**Purpose:** The forest writes its own story. Human-readable narrative entries written automatically at key moments.
**Key subcommands:**
  core journal today           — show today's journal entries
  core journal yesterday       — show yesterday's entries
  core journal week            — show this week's entries
  core journal search <term>   — search journal by keyword
  core journal show <date>     — show journal for YYYY-MM-DD
  core journal session-start   — write a session-start entry (auto-fired by fsh on login)
  core journal daily-summary   — write end-of-day summary (auto-fired on fsh exit)
**Auto-writes when:** session starts, session ends
**Storage:** ~/0-core/runtime/journal/YYYY-MM-DD.md
**Notes:** INT-195. The journal is memory. Friday will read from it.
**Purpose:** Access forest documentation from the terminal.
**Key subcommands:**
  core docs commands           — open the full core commands guide
  core docs list               — list all available documentation files
**Notes:** INT-202. Documents in ~/0-core/docs/. Living document.

**Purpose:** Core v16 — The forest redesigns itself. Architecture analysis, structural proposals, evolution tracking. The Prime Directive governs all: explain reasoning, expose uncertainty, defer to human, improve when wrong.
**Key subcommands:**
  core self map              — architecture coupling analysis (domains, events, engine health, signals)
  core self evolve           — generate structural proposals with confidence + risk + evidence
  core self apply <id>       — accept a proposal (use --dry-run first, --checkpoint to be safe)
  core self history          — evolution audit trail (all proposals + outcomes)
  core self learn <id> <outcome> — record success/failure of an applied proposal
  core self accuracy         — proposal acceptance and success rates over time
  core self calibrate        — adjust confidence thresholds based on rejection patterns
  core self challenge <INT-XXX>  — prove-me-wrong mode, stress-tests a plan
**Prime Directive (encoded):**
  1. Explain reasoning — every proposal cites evidence
  2. Expose uncertainty — confidence score on every proposal
  3. Defer final authority — you decide, always
  4. Improve when wrong — track outcomes, update model
**Tables:** self_proposals, self_evolution_log, self_accuracy in state.db
**Notes:** INT-189. v15 alignment must be complete before v16 (hard dependency). Proposals accumulate over time — the forest gets smarter with use.

**Purpose:** Trust contracts and safe autonomy simulation. Defines what the forest can propose autonomously, under what constraints, with what rollback guarantees. Activation requires passing three accuracy gates over 14+ days.
**Key subcommands:**
  core delegate simulate <action>     — test delegation without executing (always safe)
  core delegate contracts             — list all trust contracts with risk levels
  core delegate history               — past simulations and outcomes
  core delegate accuracy-report       — three-dimensional accuracy (action_match/outcome_success/calibration_error)
  core delegate counterfactuals       — ground truth log: proposed vs human action
  core delegate log-counterfactual    — record what you actually did vs what was proposed
  core delegate activate <contract>   — enable real delegation (only after all gates pass)
  core delegate suspend               — pause all delegation instantly
**Activation gates (ALL must pass):**
  action_match >= 0.85
  outcome_success >= 0.80
  calibration_error <= 0.10
  14+ days of simulation data
**Hard boundaries (enforced at execution layer, never delegated):**
  git commit/push, file deletion, protected paths, permission changes
**Typed capabilities:** RestartService, CreateCheckpoint, NotifyUser, RunDiagnostic, ClearCache
**Typed rollbacks:** RestartService, RestoreFile, RevertDb (RunCommand eliminated)
**Notes:** INT-187. Clock started 2026-04-03. Simulation only until gates pass.

**Purpose:** Query faelight-daemon v2 — the always-on background brain. Monitors health, pre-computes predictions, aggregates signals.
**Key subcommands:**
  core daemon status           — full forest context: health, alignment, intent, commits, prediction
  core daemon signals [n]      — last N engine signals (default 10)
  core daemon watchdog         — health watchdog status and alert count
  core daemon context          — raw forest context JSON from daemon
  core daemon neovim <file>    — neovim context for a specific file path
**Background tasks (runs always):**
  Health watchdog: every 60s — alerts if health drops below 95%
  Prediction pre-compute: every 30s — caches next likely command
  Signal aggregation: every 30s — summarizes engine signal counts
**Socket:** ~/.local/state/0-core/daemon.sock
**Service:** systemctl --user status faelight-daemon
**Notes:** INT-196. faelight-daemon v3.1.0. Background brain feeds Friday when it activates.

**Purpose:** Core v17 — Every pattern earns its weight. Frequency, recency, consequence, trend, volatility, confidence combine into a single explainable score. Powers Friday's WeightClass behavior mapping.
**Key subcommands:**
  core weight compute          — scan events, compute weights for all known patterns
  core weight list             — all patterns ranked by weight with class and confidence
  core weight top              — Critical and Strong patterns only
  core weight explain <id>     — full 5-stage breakdown: base → confidence → volatility → decay → identity
  core weight calibrate <id> <outcome> — record outcome for calibration learning
**WeightClass behavior:**
  IGNORE   (< 0.25) — silent, not worth surfacing
  WEAK     (0.25–0.45) — mention only if asked
  MODERATE (0.45–0.65) — suggest during relevant context
  STRONG   (0.65–0.80) — recommend proactively
  CRITICAL (> 0.80)   — challenge / interrupt current action
**Design principles:**
  - volatility = modifier (not a weight dimension)
  - trend asymmetry: worsening amplifies 0.5x, improving dampens 0.4x
  - frequency = rate (occurrences/window_days), not raw count
  - identity alignment clamped [0.9, 1.1]
  - WeightBreakdown on every weight — fully explainable
**Tables:** pattern_weights, weight_calibrations in state.db
**Notes:** INT-205. Feeds core predict next (weight-ranked), Friday WeightClass behavior, and future Tool Intelligence L2/L3.

---
## Tool Intelligence L2 -- Pattern Learning (INT-208)
**Purpose:** Every tool now remembers what it did. Structured learning feeds Friday before Friday wakes up.
**New tables in state.db:**
  health_patterns    -- every doctor run: health_pct, integrity_pct, checks, trigger_type
  commit_patterns    -- every commit: hash, message, intent_id, outcome, velocity_per_hour, session_depth
  session_patterns   -- every fsh session exit: day_of_week, hour_start/end, commit_count
  update_history     -- every update run: total_updates, outcome, health_after, drift_label
**Engine signals from tools:**
  engine_signals source=doctor           -- health signal on every doctor run
  engine_signals source=faelight-update  -- update signal on every update run
**Notes:** INT-208 (10/13 gates complete 2026-04-09). 30-day threshold for meaningful patterns. Friday inherits all on activation.
*Living document — updated with every new domain.*
*Last updated: Faelight Forest 11.7.0 — The Intelligence Arc*
