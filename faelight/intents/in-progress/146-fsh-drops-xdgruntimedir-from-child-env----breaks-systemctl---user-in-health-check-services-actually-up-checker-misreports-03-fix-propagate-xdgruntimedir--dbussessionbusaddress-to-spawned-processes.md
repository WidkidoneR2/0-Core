---
id: 146
date: 2026-07-12
type: future
title: "fsh drops XDG_RUNTIME_DIR from child env -- breaks systemctl --user in health check (services actually up; checker misreports 0/3). Fix: propagate XDG_RUNTIME_DIR + DBUS_SESSION_BUS_ADDRESS to spawned processes."
status: in-progress
tags: [fsh, Runtime, session, XDG]
---

id	146
date	2026-07-12
type	future
title	fsh drops XDG_RUNTIME_DIR from child env -- health check misreports services as down
status	planned
tags	
fsh
runtime
health-check
xdg
observability

## Vision
The health check reports what it KNOWS and does not cry wolf when it simply COULD NOT ASK.
"System Services" must not render "down" when the real story is "the session bus wasn't reachable
to query." One honest check, three honest outcomes: running / genuinely-down / couldn't-verify.

## The Problem
CORRECTED after recon (2026-07-12) -- the original premise (fsh strips XDG_RUNTIME_DIR from
children) was WRONG. Verified: fsh's own process HAS XDG_RUNTIME_DIR + DBUS_SESSION_BUS_ADDRESS
(/proc/<pid>/environ), and no fsh spawn path calls .env_clear(). fsh is not in the failing path.

The real bug is in the ENGINE, not fsh:
- `faelight/engine/src/domains/doctor/checks.rs:38` -- `check_services()` runs
  `systemctl --user is-active --quiet <name>` and does `.status().map(|s| !s.success()).unwrap_or(true)`.
  When the command CANNOT RUN (session bus unreachable -> Err), `.unwrap_or(true)` records the
  service as DOWN. A failure-to-query is silently converted to a service-is-down. It cannot tell
  "asked, inactive" from "couldn't ask."
- Amplifier: `faelight/packages/faelight/scripts/deploy:42` runs `core doctor run` inside the
  deploy/activation context, where the user session bus is not reachable -> every is-active Errs
  -> all services read "down" -> 0/3. Run the SAME `core doctor` from the user session (plain `d`)
  and the bus is reachable -> 3/3. Same code, two contexts. That is the entire "scare" from the
  gen 353 deploy: services were up; the checker was blind and reported down.

## The Solution (Option 2 -- contained, no type change)
Fix the honesty INSIDE `check_services()` -- distinguish the Err (couldn't-query) case from a real
inactive, without touching the Status enum:
- All active (bus reachable, all is-active succeed) -> Pass "N/N services running".
- Bus reachable, some genuinely inactive -> Warn "down: X" (unchanged, correct).
- Bus UNREACHABLE (is-active returns Err -- distinct from an "inactive" answer) -> do NOT count
  those as down. Report Pass with an honest note (e.g. "N services (session bus unavailable --
  not checked in this context)"), so a bus-less context (deploy activation, early boot, headless)
  stops showing a false red.
Detection: separate the two failure modes -- `.status()` Err (spawn/connect failure) means
couldn't-query; a successful spawn returning non-success means genuinely inactive. Probe the bus
reachability once (a lightweight `systemctl --user is-system-running`/`show-environment` or the
first Err) and branch on it.

The architectural fix -- a first-class `Status::Unknown` across the whole doctor domain (renderer,
health %, health_drop, tallies, schema) -- is DEFERRED to its own intent (filed separately), since
it is a doctor-domain-wide type change, not a rider on this symptom fix.

## Success Criteria
- [ ] check_services() distinguishes bus-unreachable (Err) from genuinely-inactive; the Err case is
      NOT counted as "down" -- demonstrated in code + behavior
- [ ] `d` from the user session still shows 3/3 when services are up (no regression) -- live
- [ ] running the check in a bus-less context (simulate: unset XDG_RUNTIME_DIR/DBUS_SESSION_BUS_ADDRESS
      then run the check) no longer reports services as down -- demonstrated
- [ ] a GENUINELY inactive service (bus reachable, service stopped) still correctly reports down/Warn
      -- demonstrated (stop a service, confirm it's caught; restart it)
- [ ] engine rebuilt + deployed; a real `dep`'s post-rebuild health check no longer shows the false
      0/3 for services -- demonstrated on the next deploy

## Relationship
Builds on: nothing structural. Single-function fix in engine doctor.
Reuses: the INT-147 lesson -- verify by what the SPAWNED process / real context sees, not assumptions;
this whole intent was re-diagnosed by /proc/environ + reading the actual call site instead of trusting
the original premise.
Defers to: [new intent] doctor Status::Unknown -- the general "checks that can't run render as UNKNOWN,
excluded from health math" architecture.
Filter: a check that reports couldn't-verify honestly deepens trustworthy control; one that cries wolf
erodes it. In-filter.

## Notes
- Original premise (fsh env stripping, Layer 1) INVALIDATED by recon 2026-07-12. Root cause is
  checks.rs:38 .unwrap_or(true) + deploy:42 bus-less context. Kept this history deliberately -- the
  wrong turn is part of the honest ledger.
- Siblings checked: notify/mod.rs:36 already returns "unknown" on Err (fine); strategy/mod.rs:1575
  .unwrap_or(false) under-scores the insightd factor by 5 in a bus-less context but degrades
  gracefully (soft factor, no false red) -- noted as a minor follow-up, not fixed here.
- Deploy-context angle: the deeper "correct" fix could ALSO run `core doctor run` as the user with a
  reachable bus, but the checker being honest regardless of context is the right primary fix, since
  early-boot/headless legitimately have no bus.
