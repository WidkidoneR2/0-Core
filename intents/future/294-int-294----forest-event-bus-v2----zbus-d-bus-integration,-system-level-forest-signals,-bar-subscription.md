---
id: 294
title: "Forest Event Bus v2 -- zbus D-Bus integration, system-level forest signals"
status: planned
date: 2026-05-12
tags: [event-bus, zbus, dbus, signals, friday, bar, forest, ipc]
---

The current forest event bus (forest_events_v2) lives entirely in state.db.
It is append-only, typed, and persisted.
It works for within-process communication.

The gap: no system-level IPC.
faelight-bar cannot subscribe to forest events.
faelight-fm cannot know when a deploy just happened.
External tools cannot observe the forest state.

zbus solves this.

---

WHAT IS ZBUS

zbus is a pure-Rust D-Bus library.
D-Bus is the standard Linux IPC system.
Every desktop environment uses it.
systemd uses it. GNOME uses it. KDE uses it.
COSMIC Desktop uses it heavily.

With zbus, the forest can:
  Publish signals that any tool can subscribe to
  Expose properties (health %, active intent, Friday confidence)
  Receive method calls from external tools
  Integrate with system D-Bus (notifications, power events, etc.)

---

FOREST D-BUS SERVICE

Service name: org.faelight.Forest

Interfaces:

org.faelight.Forest.Health
  Properties:
    health_percent: u32      -- current health %
    integrity_percent: u32   -- current integrity %
    trend: f64               -- forecast trend
  Signals:
    HealthChanged(old: u32, new: u32)
    IntegrityChanged(old: u32, new: u32)

org.faelight.Forest.Intent
  Properties:
    active_intent: String    -- current focus intent
    active_intent_id: u32    -- intent ID
    intents_complete: u32    -- total complete count
  Signals:
    IntentChanged(old: String, new: String)
    IntentCompleted(id: u32, title: String)

org.faelight.Forest.Friday
  Properties:
    confidence: f64          -- current confidence level
    usefulness_score: f64    -- 30-day acceptance rate
    facts_count: u32         -- total facts stored
    patterns_count: u32      -- active patterns
  Signals:
    FridaySuggested(message: String, confidence: f64)
    FridayProposed(proposal_id: u32, action: String)
    SimulationResult(command: String, predicted: String, confidence: f64)

org.faelight.Forest.Deploy
  Signals:
    DeployCompleted(tool: String, version: String, duration_ms: u64)
    DeployFailed(tool: String, error: String)

org.faelight.Forest.Git
  Properties:
    commits_total: u32
    branch: String
    is_clean: bool
  Signals:
    CommitMade(hash: String, message: String)
    PushedToOrigin(commit_count: u32)

---

HOW BAR SUBSCRIBES

faelight-bar v3 subscribes to forest signals via zbus:

  let conn = zbus::Connection::session().await?;
  let proxy = ForestHealthProxy::new(&conn).await?;
  
  // Subscribe to health changes
  let mut health_stream = proxy.receive_health_changed().await?;
  while let Some(signal) = health_stream.next().await {
    let args = signal.args()?;
    bar.update_health(args.new());
  }

Bar always shows current state without polling state.db.
Friday signals appear in bar the moment they fire.
Deploy completions update bar instantly.

---

HOW FRIDAY PUBLISHES

Friday daemon emits D-Bus signals alongside state.db writes:

  // When Friday generates a suggestion:
  forest_bus.emit_friday_suggested(&message, confidence).await?;
  
  // When deploy completes:
  forest_bus.emit_deploy_completed(&tool, &version, duration_ms).await?;

This makes Friday's voice available to any tool on the system.
Not just fsh. Not just faelight-term.
Any future tool can listen.

---

STUDY SOURCES

zbus (primary):
  Pure Rust D-Bus library
  Source: github.com/dbus2/zbus
  Focus: zbus::interface macro, zbus::proxy macro
  Examples: zbus/examples/ directory

COSMIC Desktop D-Bus usage:
  cosmic-comp uses zbus for session management
  cosmic-panel uses D-Bus for applet communication
  Study: how COSMIC exposes compositor state via D-Bus

systemd D-Bus integration:
  logind for session events (lock, sleep, wake)
  Forest can subscribe to system power events
  Lock screen when system suspends

---

IMPLEMENTATION PLAN

Phase 1 -- Core D-Bus service:
  forest-daemon crate or add to faelight-daemon
  Expose Health and Intent interfaces
  faelight-bar subscribes
  Gate: bar updates when health changes without polling

Phase 2 -- Friday D-Bus signals:
  Friday emits via D-Bus alongside state.db writes
  faelight-bar shows Friday signals in real time
  Gate: Friday suggestion appears in bar within 100ms

Phase 3 -- Full signal coverage:
  Deploy, Git, all forest events on D-Bus
  Any tool can observe the forest
  Gate: external script can subscribe to forest events

Phase 4 -- System integration:
  Subscribe to logind for power events
  Subscribe to NetworkManager for connectivity
  Forest knows about system state, not just forest state
  Gate: forest health updates on network change

---

DEPENDS ON

INT-239 (faelight-bar v2) -- bar must exist to subscribe
INT-295 (faelight-bar v3) -- bar v3 built with libcosmic

---

GATES

[ ] zbus studied -- interface and proxy macros understood
[ ] org.faelight.Forest D-Bus service running
[ ] Health and Intent properties exposed
[ ] faelight-bar subscribes to HealthChanged signal
[ ] Friday suggestions appear in bar via D-Bus
[ ] Deploy completions update bar in real time
[ ] System power events (suspend/wake) reach forest
[ ] Full signal coverage: health, intent, friday, deploy, git

---

"The forest has always known its own state.
Now the forest can speak that state
to any tool that listens.
D-Bus is the forest voice at the system level." 🌲
