---
id: 327
title: "Forest Self-Healing Runtime -- fridayd nervous system, incident reports, degraded mode, desktop graph, versioned APIs"
status: planned
date: 2026-05-20
tags: [forest, runtime, self-healing, fridayd, incident, degraded-mode, desktop-graph, ipc, versioned-apis, orchestration]
---

INT-327 -- Forest Self-Healing Runtime -- The Forest That Heals Itself
date: 2026-05-20

---
THE PREMISE

Most Linux desktops die the same way.
A service crashes. Nothing notices.
The user sees a frozen bar, a dead notification daemon, a blank workspace.
They restart. They lose their session. They lose their work.
They accept this as normal.

Faelight Forest does not accept this.

The forest already has pieces of a self-healing runtime:
  Health monitoring (INT-196)
  Deploy rollback (existing deploy system)
  State snapshots (fsh_snapshots, INT-322)
  Event bus (INT-294)
  Watchdog timers (faelight-daemon health_watchdog)
  Integrity checks (core doctor)

What is missing is the orchestration layer.
The thing that connects all these pieces into a system that
detects failures, isolates them, repairs them, and reports them
without the user ever seeing a frozen screen.

INT-327 builds that layer.
The forest nervous system.
---
WHAT THE FOREST ALREADY HAS (the foundation)

faelight-daemon: health watchdog, event polling, Friday learning loop
faelight-bar: live health display, lock status, intent tracking
state.db: events table, engine_signals, health_patterns, deploy_patterns
deploy system: version history, rollback capability, 5-version retention
fsh snapshots: state capture before destructive commands
org.faelight.Forest: D-Bus service (INT-294) -- health + intent on session bus
core doctor: integrity checks, health scoring

These are not separate tools.
They are the unconnected organs of a nervous system.
INT-327 connects them.
---
COMPONENT 1 -- FRIDAYD: THE FOREST NERVOUS SYSTEM

fridayd is not a new daemon.
It is the evolution of faelight-daemon into something that
understands the entire desktop as a system of connected parts.

Current faelight-daemon: polls state.db, serves Unix socket RPC, learning loop
fridayd: all of that + orchestration layer + service health graph + incident engine

fridayd responsibilities:
  1. Service health monitoring -- knows every forest tool's status
  2. Event aggregation -- receives from D-Bus, filesystem, socket
  3. Failure detection -- pattern-matches against known failure signatures
  4. Recovery orchestration -- knows how to restart each service safely
  5. Incident reporting -- creates structured incident reports
  6. Desktop graph maintenance -- live model of running surfaces
  7. Degraded mode management -- safe fallbacks when things break
  8. API versioning -- stable interfaces for all forest tools

The nervous system metaphor is precise:
  Sensors: health_watchdog, event bus, IPC monitors, filesystem watchers
  Nerves: D-Bus signals, Unix socket events, state.db changes
  Brain: fridayd orchestration engine
  Muscles: systemctl restart, deploy rollback, surface recovery
  Memory: state.db incident log, desktop graph snapshots
---
COMPONENT 2 -- THE DESKTOP GRAPH

The most powerful idea in this intent.

A live, persistent graph of everything running in the forest:

  struct DesktopNode {
      id: Uuid,
      kind: NodeKind,          // Compositor, Terminal, Bar, FM, Notification, etc.
      pid: Option<u32>,
      wayland_id: Option<u32>, // wl_surface id if applicable
      ipc_socket: Option<PathBuf>,
      dbus_name: Option<String>,
      health: NodeHealth,      // Healthy, Degraded, Failed, Recovering
      dependencies: Vec<Uuid>, // what this node depends on
      dependents: Vec<Uuid>,   // what depends on this node
      started_at: i64,
      last_heartbeat: i64,
      last_incident: Option<Uuid>,
  }

  enum NodeKind {
      Compositor,   // Niri / faelight-compositor
      Shell,        // fsh instances
      Terminal,     // faelight-term instances
      Bar,          // faelight-bar
      FM,           // faelight-fm
      Notify,       // faelight-notify
      Daemon,       // faelight-daemon / fridayd
      Login,        // faelight-login
      Menu,         // faelight-menu
      External,     // non-forest apps (foot, helix, etc.)
  }

The graph is stored in state.db (desktop_graph table).
fridayd updates it in real time.
Friday can query it: "what depends on the compositor?"
The bar can display it: node health colors in the compositor chrome.

Why the graph matters:
  Dependency-aware restart:
    If the compositor restarts, fridayd knows which surfaces to re-attach.
    If the bar crashes, fridayd knows it depends on the D-Bus service.
    Recovery happens in dependency order, not randomly.

  Cascade failure prevention:
    If faelight-notify crashes and takes a D-Bus name with it,
    fridayd knows not to let faelight-bar crash waiting for the signal.
    It bridges the gap while notify restarts.

  Intelligent recovery:
    fridayd knows: "faelight-bar has crashed 3 times in 10 minutes.
    This is not a random crash. Stop restarting. Enter degraded mode."
---
COMPONENT 3 -- INCIDENT REPORTS

After every significant failure, fridayd generates a structured incident report.
Stored in state.db, surfaced by Friday, queryable by the user.

  struct Incident {
      id: Uuid,
      timestamp: i64,
      service: String,          // which tool failed
      failure_kind: FailureKind, // Crash, Freeze, MemoryLeak, IpcTimeout, etc.
      confidence: f64,          // how sure fridayd is about the cause
      impact: ImpactLevel,      // None, Minor, Major, Critical
      duration_ms: u64,         // how long the failure lasted
      recovery: RecoveryAction, // Restarted, Degraded, Manual, Failed
      regression_risk: RiskLevel, // Low, Medium, High
      suggested_gate: Option<String>, // what watchdog to add
      notes: String,            // human-readable summary
  }

Example incident report:

  Forest Incident 2026-05-20-001

  Service:      faelight-bar
  Cause:        IPC timeout waiting for org.faelight.Forest
  Confidence:   0.91
  Impact:       Minor (bar restarted in 2.1s)
  Duration:     2100ms
  Recovery:     Restarted successfully
  Regression:   Low
  Suggested:    Add D-Bus readiness check before bar startup

The incident log is the forest's self-reflection.
Over time: Friday learns failure patterns.
Friday can predict: "faelight-bar tends to fail after suspend.
  Consider adding a post-resume restart hook."

User commands:
  incidents              -- list recent incidents
  incidents show 001     -- full incident detail
  incidents for bar      -- all incidents for faelight-bar
  incidents stats        -- failure rate, MTTR, most unstable service
---
COMPONENT 4 -- DEGRADED MODE

The most undervalued reliability feature.

When a service fails beyond recovery, the forest does not die.
It enters degraded mode: a minimal, stable environment that
keeps the user working while the failure is investigated.

Degraded mode tiers:

  Tier 0 -- Normal:
    All services running. Full forest experience.

  Tier 1 -- Degraded (non-critical service failed):
    faelight-notify crashed: notifications suppressed, bar shows ⚠ icon.
    faelight-fm crashed: FM unavailable, file operations via terminal.
    faelight-bar crashed: bar hidden, status in terminal title.
    User notified. Automatic recovery attempted.

  Tier 2 -- Degraded (rendering issues):
    GPU renderer unstable: software fallback renderer activates.
    faelight-term falls back to CPU rendering.
    Bar falls back to minimal SHM mode.
    User sees: "Degraded graphics mode -- GPU renderer recovering"

  Tier 3 -- Compositor failure:
    faelight-compositor crashes: Niri activates on TTY1 (always kept ready).
    Session state preserved in desktop graph snapshot.
    When compositor recovers: session restored from snapshot.
    User never loses their workspace layout.

  Tier 4 -- Critical failure:
    Compositor + Niri both unavailable.
    fsh activates in bare terminal mode (TTY3).
    User has full shell access.
    Friday surfaces recovery instructions.
    "Run: forest recover -- or see: incident show latest"

  Safe Mode (boot option -- INT-325 dependency):
    Boot with minimal services only.
    Compositor: minimal Niri config.
    Shell: fsh with safe vocabulary only.
    Bar: time + health only.
    No FM, no notifications, no Friday.
    "Boot safe graphics" option in faelight-login.

The principle: the forest never goes completely dark.
Something always works. The user always has a path forward.
---
COMPONENT 5 -- VERSIONED INTERNAL APIS

Without versioned interfaces, every tool breaks when any other tool changes.
This is the silent killer of complex systems.

The forest IPC contract:

  // Stable forever after release:
  org.faelight.Forest.Health.v1.HealthPercent
  org.faelight.Forest.Intent.v1.ActiveIntent

  // When we need to change:
  org.faelight.Forest.Health.v2.HealthPercent  // new schema
  org.faelight.Forest.Health.v1.HealthPercent  // still works

  // v1 deprecated after all tools migrate to v2:
  org.faelight.Forest.Health.v1  // marked deprecated in D-Bus introspection

Unix socket protocol versioning:
  {"version": 1, "id": 1, "payload": {...}}  // faelight-daemon v1 protocol
  {"version": 2, "id": 1, "payload": {...}}  // v2 with new fields

  Daemon always handles both.
  Tools declare what version they speak.
  No breaking changes without version bump.

State.db schema versioning:
  migrations/ directory (already exists pattern in the forest)
  Every schema change is a numbered migration.
  friday_patterns v1 -> friday_patterns v2: migration runs automatically.
  Old column preserved until all tools updated.

The rule: interfaces are contracts.
Breaking a contract requires a version bump.
Old contracts are supported for at least 2 major versions.
Friday tracks which tools are on which API version.
---
COMPONENT 6 -- SERVICE HEALTH SUPERVISOR

Like systemd, but specifically for the forest ecosystem.

core-healthd (or integrated into fridayd):
  Responsibilities:
    Watchdog timers per service (heartbeat expected every N seconds)
    Memory tracking (RSS growth triggers warning at +50MB unexplained)
    Crash reports (exit code, signal, backtrace if available)
    Restart policy (immediate / exponential backoff / degraded mode)
    Dependency graph (restart in correct order)
    Degraded mode handling (switch tiers automatically)

Watchdog registration (each tool self-registers):
  fridayd.register_watchdog(WatchdogConfig {
      service: "faelight-bar",
      heartbeat_interval: Duration::from_secs(30),
      heartbeat_timeout: Duration::from_secs(90),
      restart_policy: RestartPolicy::ExponentialBackoff {
          initial: Duration::from_secs(2),
          max: Duration::from_secs(60),
          max_attempts: 5,
      },
      degraded_after: 3,  // enter degraded mode after 3 failures
      dependencies: vec!["org.faelight.Forest"],
  });

faelight-bar sends heartbeat every 30 seconds:
  // In bar's main loop:
  fridayd.heartbeat("faelight-bar").await;

If heartbeat stops: fridayd restarts bar.
If 3 restarts in 5 minutes: bar enters Tier 1 degraded.
If dependency missing: bar waits (does not crash).
---
COMPONENT 7 -- ARCHITECTURAL DEBT TRACKING

The forest grows. Debt accumulates.
Without tracking, debt becomes invisible.
Invisible debt becomes system collapse.

  struct ArchitecturalDebt {
      id: u32,
      description: String,
      severity: Severity,      // Low, Medium, High, Critical
      blocks: Vec<String>,     // what intents this blocks
      introduced: String,      // which commit introduced it
      owner: String,           // which tool owns the debt
      estimated_fix: String,   // rough effort estimate
  }

Example entries:
  Debt-001:
    faelight-bar polls /etc/faelight/ files every second
    Severity: Low
    Blocks: INT-294 Phase 2 (bar D-Bus subscription)
    Fix: Subscribe to org.faelight.Forest.Health signal instead

  Debt-002:
    faelight-daemon Unix socket protocol has no versioning
    Severity: Medium
    Blocks: INT-327 versioned APIs
    Fix: Add version field to Message envelope

  Debt-003:
    fsh main.rs is 3600+ lines with no test suite
    Severity: High
    Blocks: INT-322 reliability, safe refactoring
    Fix: Extract modules, add integration test suite

core doctor shows debt:
  core doctor --debt          -- list all architectural debt
  core doctor --debt critical -- only critical items
  Friday monitors: debt growing faster than it's being paid = warning signal
---
BOOT FLOW EVOLUTION

Current:
  kernel -> systemd -> greetd -> faelight-login -> Niri -> tools

Target (INT-327 + INT-325 + INT-323):
  kernel
  -> initramfs (faelight-splash -- INT-325)
  -> systemd
  -> greetd
  -> faelight-login (session orchestrator)
  -> fridayd (nervous system starts)
  -> Niri / faelight-compositor (compositor)
  -> faelight-bar (after fridayd ready)
  -> faelight-notify (after compositor ready)
  -> faelight-daemon (already running)
  -> fsh as login shell

At each step:
  fridayd registers the new service in the desktop graph.
  fridayd verifies health before proceeding.
  If a step fails: degraded mode, not panic.
  The splash screen shows real boot progress.

Boot options (from faelight-login):
  Normal Mode       -- full forest, all services
  Safe Mode         -- minimal services, no GPU acceleration
  Recovery Mode     -- fsh only, fridayd only, repair tools
  Previous Session  -- restore from last known-good desktop graph snapshot
  Debug Session     -- all services with verbose logging
---
THE PLASMA/KDE DIRECTION (F-DWL -- INT-290)

This intent connects to the F-DWL vision in an unexpected way.

The forest self-healing runtime is COMPOSITOR AGNOSTIC.
fridayd does not care if the compositor is Niri, faelight-compositor, or KDE Plasma.
The D-Bus event bus works with any desktop.
The desktop graph works with any set of Wayland surfaces.

This means:
  If Faelight Forest eventually runs on KDE Plasma (via CXX-Qt):
  fridayd still manages the forest services.
  org.faelight.Forest is still on the session bus.
  fsh still owns the shell layer.
  The forest nervous system survives the compositor change.

CXX-Qt integration path (for F-DWL):
  Rust core (fridayd, fsh, faelight-daemon) unchanged.
  CXX-Qt bridge exposes forest state to QML/Kirigami.
  Plasma widgets become forest-aware via D-Bus.
  KRunner plugin powered by Forest Query Language (INT-279).
  fsh vocabulary available from Plasma launcher.

The forest does not need to replace Plasma.
The forest can make Plasma forest-aware.
That is a different and more sustainable goal.

KDE Rust targets that serve the forest:
  KRunner plugin: "ask friday" launcher
  Plasma widget: forest health/intent display
  KIO plugin: forest-aware file operations
  Notification center: faelight-notify via Plasma
  Clipboard manager: forest history integration
---
PHASES

Phase 0 -- Audit and design (1 session):
  Catalog all existing health/monitoring infrastructure
  Define DesktopNode, Incident, WatchdogConfig structs
  Design desktop_graph schema for state.db
  Define degraded mode tiers formally
  Gate: architecture document complete, structs defined

Phase 1 -- Desktop graph (2 sessions):
  desktop_graph table in state.db
  fridayd registers each forest service on startup
  Heartbeat system for each registered service
  Gate: core doctor shows live desktop graph with health status

Phase 2 -- Incident reports (1 session):
  Incident struct and incidents table in state.db
  fridayd generates incident on any service failure
  incidents command in fsh
  Gate: crash faelight-bar manually, incident report generated

Phase 3 -- Degraded mode Tier 1 and 2 (2 sessions):
  Tier 1: non-critical service failure handling
  Tier 2: GPU rendering fallback
  Bar shows degraded indicator (⚠)
  Gate: kill faelight-notify, bar shows ⚠, system continues working

Phase 4 -- Versioned APIs (1 session):
  Version field in D-Bus interfaces
  Version field in Unix socket protocol
  state.db migration framework formalized
  Gate: bump org.faelight.Forest.Health to v2, v1 still works

Phase 5 -- Architectural debt tracking (1 session):
  debt table in state.db
  core doctor --debt command
  Friday monitors debt growth rate
  Gate: all known debt items catalogued, visible in core doctor

Phase 6 -- Boot flow integration (INT-325 + INT-323 dependency):
  fridayd starts before compositor
  Boot options in faelight-login
  Previous session restore from desktop graph snapshot
  Gate: boot with one service missing, degraded mode activates correctly

Phase 7 -- Self-healing full cycle (1 week):
  Complete failure -> detection -> recovery -> incident -> learning cycle
  Friday predicts failures based on incident history
  Gate: system runs 1 week, every failure auto-recovered, incident logged
---
GATES
[ ] Phase 0: DesktopNode/Incident/WatchdogConfig structs designed, db schema defined
[ ] Phase 1: desktop graph live -- core doctor shows all forest services with health
[ ] Phase 2: incident reports -- crash triggers structured incident in state.db
[ ] Phase 3: degraded mode Tier 1+2 -- non-critical failure handled gracefully
[ ] Phase 4: versioned APIs -- v1 and v2 of Health interface coexist
[ ] Phase 5: debt tracking -- all architectural debt catalogued
[ ] Phase 6: boot flow -- degraded boot, session restore, safe mode working
[ ] Phase 7: full self-healing cycle -- 1 week, every failure auto-recovered
Final:
[ ] The forest never goes completely dark -- Tier 4 keyboard mode always available
[ ] Every failure generates a structured incident report
[ ] Friday learns from incident history and predicts future failures
[ ] Desktop graph is the live model of the running forest
[ ] Versioned APIs mean tool updates never break other tools
[ ] Architectural debt is visible and tracked, not hidden
[ ] The boot flow is owned end to end by the forest

DEPENDS ON
INT-294 (Forest Event Bus) -- D-Bus infrastructure -- in progress
INT-323 (compositor v3 -- session authority) -- boot flow integration
INT-325 (faelight-boot) -- splash + initramfs -- boot flow
INT-251 (Core v23 -- Friday Central) -- Friday incident learning

TIMELINE
Phase 0-2: after INT-294 Phase 2 complete (bar subscribing to D-Bus)
Phase 3-4: parallel with INT-322 (fsh v4)
Phase 5: any time, low dependency
Phase 6: after INT-323 + INT-325
Phase 7: long-running, post-presentation
Target: Phase 3 (degraded mode) before NY presentation
        A system that heals itself is a presentation moment

"Most systems fail loudly.
The forest fails quietly, heals itself, and tells you what happened.
Not because it is magic.
Because every failure was anticipated,
every recovery was designed,
and the forest never forgets what broke." 🌲
