---
id: 039
date: 2026-06-08
type: feature
title: "friday-daemon: persistent Friday process across sessions"
status: planned
tags: [friday, daemon, systemd, ipc, event-bus, persistence, rust, nixos]
version: TBD
---

## Vision
Friday today is born and dies with each shell session -- it spins up when a session starts,
holds its patterns and facts in memory, and loses that continuity when the session ends.
friday-daemon makes Friday a persistent, always-on process: one long-lived intelligence that
observes the forest continuously, keeps a single authoritative state across every session AND
rebuild, and does real work in the background instead of only when a prompt is open. This is
the spine the rest of Friday's ambitions stand on -- shell-context (INT-041), continuous
learning, proactive suggestion, and the event bus all assume an always-on Friday to live in.
The difference is qualitative: from a tool you invoke to an ambient layer that is simply there.

## Why Now
Friday has outgrown the per-session model. It already carries 13 patterns and 500+ facts,
detects cross-layer signals and contradictions, and tracks its own usefulness -- but every
session it starts cold, cannot observe anything while no shell is open, and has nowhere to run
idle-time work (consolidation, memory decay, prediction). INT-071 is restoring Friday's commit
learning; INT-034 wants triad tracking; both want a continuous Friday to feed. The daemon is
the unlock that turns Friday from episodic to continuous.

## What
A long-lived friday-daemon process, supervised declaratively, that:
- Runs as a systemd user service on NixOS -- starts on login, restarts on failure, logs to the
  journal. Declarative, in the framework16 config, the Nix way.
- Owns the authoritative Friday state on disk (the fact/pattern store) as a single source of
  truth that survives session end AND rebuild.
- Exposes an IPC surface (a unix domain socket) so fsh and the rust-tools query Friday (facts,
  patterns, suggestions) and push events (commands, commits, rebuilds, intent transitions).
- Observes continuously and runs idle-time work: always-on event ingestion plus background
  consolidation / memory-decay / prediction hooks when the machine is quiet.
- Degrades safely: if the daemon is down, fsh and the tools keep working Friday-less -- Friday
  never takes the shell down with it.

## Approach
NixOS-native: a systemd user service (Type=simple, Restart=on-failure) declared in the host
config. IPC over a unix domain socket in the user runtime dir with a small framed JSON protocol
(request/response + event push); only the user can reach it (local trust boundary -- the
security angle is real). State moves from per-session in-memory into a daemon-owned, disk-backed
sqlite store (Friday already persists facts/patterns) as the one source of truth, concurrency-safe
across multiple simultaneous shells. On a rebuild the service restarts; the daemon reloads state
from disk and reconnecting clients resume -- continuity lives in the store, not the process. Keep
the daemon lightweight; it is always on and must not tax the Framework 16.

## Phases
Phase 0 -- survey + boundary: how Friday starts/stops today, where state lives, in-memory vs disk;
  decide exactly what moves into the daemon. Record here.
Phase 1 -- daemon skeleton: friday-daemon as a declarative systemd user service; survives login.
Phase 2 -- IPC surface: socket + framed protocol; fsh queries and pushes events; prove graceful
  degradation (kill the daemon, fsh still works).
Phase 3 -- authoritative state: daemon owns the disk-backed store; concurrency-safe; survives rebuild.
Phase 4 -- continuous operation: always-on ingestion + one idle-time task demonstrated end-to-end.

## Gates
- [ ] Phase 0: current Friday lifecycle surveyed; daemon boundary (what moves in) recorded here
- [ ] friday-daemon runs as a declarative systemd user service; survives logout/login
- [ ] fsh talks to the daemon over the socket AND degrades gracefully when it is down
- [ ] daemon owns the disk-backed store as single source of truth; state survives a rebuild
- [ ] daemon ingests events continuously and performs one idle-time task end-to-end

## Notes
- Foundation intent: INT-041 and future proactive features build ON this. Sequence 039 before 041.
- The "fridayd" naming/idea (2026-07-02) folds into THIS intent -- fridayd IS friday-daemon. No separate intent needed; the daemon binary may simply be named `fridayd` or `friday-daemon` (decide at Phase 1).
- Distinct from INT-071 (parity restoration): 071 recovers what the Arch->Nix migration broke; 039
  is a NEW capability (persistence). They meet at the state store.
- Feeds / fed by: INT-034 (triad data), INT-071 (commit learning) once live.
- Security: the socket is a local-only trust boundary (user runtime dir) -- fits the hardening posture.
- Hard parts to respect: rebuild-survival (services restart on switch -- continuity must live in the
  store), concurrency (many shells, one daemon), and the rule that a dead daemon must never break fsh.
- How far this could go: the always-on daemon is the precondition for Friday as a true event bus --
  proactive notices, situated prediction (the v11 pillar), delegation (INT-186 lineage). This intent
  builds the spine, not the whole nervous system.

## The Rule
"An intelligence that sleeps between sessions is a notebook. Friday should stay awake." 🌲
