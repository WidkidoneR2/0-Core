---
id: 146
date: 2026-07-12
type: future
title: "fsh drops XDG_RUNTIME_DIR from child env -- breaks systemctl --user in health check (services actually up; checker misreports 0/3). Fix: propagate XDG_RUNTIME_DIR + DBUS_SESSION_BUS_ADDRESS to spawned processes."
status: planned
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
A health check should report what it *knows*, and say so when it *can't know*. Today fsh
strips XDG_RUNTIME_DIR (and DBUS_SESSION_BUS_ADDRESS) from the environment it hands to child
processes, so `systemctl --user` launched from fsh can't reach the session bus -- and the
health check renders "couldn't ask" as "0/3 running, services down." Fix the cause (propagate
the vars) AND the symptom (a check that fails to run must look different from a check that ran
and found a real problem).

## The Problem
Two defects, one visible symptom.

1. **Cause -- env stripping.** fsh does not pass XDG_RUNTIME_DIR / DBUS_SESSION_BUS_ADDRESS
   into spawned children. Proven live 2026-07-12: `systemctl --user is-active` run from fsh
   fails with "Failed to connect to user scope bus via local transport: $DBUS_SESSION_BUS_ADDRESS
   and $XDG_RUNTIME_DIR not defined"; the SAME command with the two vars set by hand returns
   `active active active`. The services were never down -- fsh blinded the query.
   (Related evidence: fsh's `env` builtin also omits these vars, and omitted IN_NIX_SHELL/name
   in INT-134 devshell work -- fsh curates its child/reported environment.)

2. **Symptom -- the checker cannot express uncertainty.** The health check ("System Services"
   at <pin: file:line>) shells `systemctl --user`, and on ANY failure -- including "bus
   unreachable" -- records the services as down. Absence of evidence is rendered as evidence of
   absence. This cascades: 0/3 down -> health drops to 87% -> `health_drop 90` fires -> forecast
   trends negative -> "declining health, investigate." One missing variable wearing five costumes.

The deploy triage already classifies the noise correctly ("benign: safe to ignore") -- the
judgment exists; it's just drowned out by a full-red health panel that shouts louder.

## The Solution
Two layers -- fix the cause so the check CAN see, and fix the checker so it stops lying when it
still can't.

**Layer 1 -- propagate the runtime env (the cause).**
When fsh spawns a child process, ensure XDG_RUNTIME_DIR and DBUS_SESSION_BUS_ADDRESS are present
in the child's environment (inherited from fsh's own, or reconstructed from the uid:
`/run/user/$(id -u)` and `unix:path=/run/user/$(id -u)/bus`). Do NOT blanket-clear the env and
re-add a curated allowlist -- if `.env_clear()` is the culprit (<pin: file:line>), the fix is to
stop clearing, or to re-inject these two vars after clearing. Scope: the process-spawn path used
by external commands, not a global export.

**Layer 2 -- the checker distinguishes UNKNOWN from FAILED (the symptom).**
When `systemctl --user` returns a bus-connection error (not a service-state answer), the check
must render a distinct third state -- e.g. `⚪ couldn't verify (no session bus)` -- NOT `❌ 0/3
down`. Unknown is not failure: it must not count against the health %, must not fire health_drop,
must not drag the forecast. A check that failed to RUN is a gap in observation, not a system fault.

## Success Criteria
- [ ] fsh-spawned children receive XDG_RUNTIME_DIR + DBUS_SESSION_BUS_ADDRESS -- demonstrated:
      `systemctl --user is-active <svc>` run from a plain fsh (no hand-set vars) returns `active`,
      on the DEPLOYED binary (not just debug)
- [ ] the spawn-env fix does not blanket-clear the child environment (no `.env_clear()` regression);
      existing commands that rely on inherited env still work -- spot-checked live
- [ ] health check "System Services" reads services correctly from a normal `d` (no hand-set vars):
      shows 3/3 running when they are -- demonstrated on the deployed system
- [ ] health check renders a distinct UNKNOWN state (`⚪ couldn't verify`) when the bus is genuinely
      unreachable, separate from a real DOWN -- demonstrated by forcing the bus-unreachable path
- [ ] an UNKNOWN check does NOT reduce health %, does NOT trigger health_drop, does NOT drag the
      forecast -- demonstrated (health holds when a check is unknown vs. drops when one truly fails)
- [ ] deploy triage / health advisory wording reflects the UNKNOWN state so the panel stops
      shouting red for a checker gap

## Relationship
Builds on: nothing structural -- isolated fix. Touches the fsh process-spawn path and the health
check tool (faelight-insightd / doctor -- confirm owner at cistart).
Reuses: the INT-134 lesson that fsh curates its environment (env builtin omitted IN_NIX_SHELL/name);
this is the child-process analogue of that same curation.
Filter: a health check that reports UNKNOWN honestly deepens understanding + trustworthy control;
a checker that cries wolf erodes it. Squarely in-filter.

## Notes
- Discovered 2026-07-12 during the INT-134 Lane 2/3 deploy (gen 353). Deploy succeeded, 0 real
  failures; the scare was entirely this bug. Services verified up by hand.
- Do NOT fix on tired eyes -- filed at end of a long session precisely so it gets done with
  fresh recon, not scrambled tonight.
- Two-layer on purpose: Layer 1 alone still leaves the checker lying whenever the bus is
  legitimately unreachable (early boot, headless, activation context). Layer 2 is what makes the
  health check honest in general, not just for this one variable.
- Sequencing at cistart: Layer 1 first (cause), verify the false red clears, THEN Layer 2
  (make the checker robust for the general case). Each demonstrated on the deployed binary.
