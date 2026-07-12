---
id: 148
date: 2026-07-12
type: future
title: "doctor: first-class Status::Unknown -- checks that CANNOT run render as UNKNOWN, excluded from health math (the general fix deferred from INT-146)"
status: planned
tags: [doctor, health-check, engine, observability, status-enum]
---

## Vision
Every doctor check reports one of THREE honest outcomes -- Pass / Fail(Warn) / Unknown -- where
UNKNOWN means "this check could not be run" (session bus unreachable, tool missing, permission
denied, wrong context), distinct from a real failure. UNKNOWN never counts against health %, never
fires health_drop, never drags the forecast. The health system distinguishes "broken" from
"unmeasured".

## The Problem
`Status` (faelight/engine/src/domains/doctor/mod.rs:10) has only Pass / Warn / Fail. A check that
CANNOT RUN is forced to lie -- pick Pass (false green) or Fail/Warn (false red). There is no way to
say "I could not measure this."

INT-146 hit this concretely: `check_services` treated a bus-unreachable `systemctl --user` (Err) as
"service down" (.unwrap_or(true)), producing a false 0/3 during deploy activation. 146 fixed it
LOCALLY -- probe the bus, fake a Pass with a "not checked in this context" note. That works for one
check but does not scale: other checks have the same latent trap (a tool that isn't installed, a
path that isn't mounted, a context without privileges), and each would have to hand-roll its own
workaround. The type system should carry this, not each call site.

Downstream, the health math and persistence only know pass/warn/fail:
- renderer maps Status -> glyph (mod.rs:1109: `Status::Pass => "✅"` etc.) -- no UNKNOWN glyph.
- health % is computed from passed/warned/failed counts -- UNKNOWN must be EXCLUDED from the
  denominator (not-measured, not "passed", not "failed").
- the SQLite `health_patterns` schema (doctor/mod.rs ~279) has columns checks_passed / checks_warned
  / checks_failed -- no checks_unknown; a schema change or explicit exclusion is required.

## Scope (VERIFIED 2026-07-12, not estimated)
This is a type-wide change with a persistence change, NOT a one-liner:
- ~118 `Status::` references across doctor/{mod,checks,cockpit,schema}.rs (grep count).
- enum definition: doctor/mod.rs:10.
- renderer: doctor/mod.rs:1109 (glyph mapping) + cockpit.rs (panel rendering).
- health-% computation + the pass/warn/fail tallies at the bottom of `d`.
- SQLite `health_patterns` schema + INSERT (doctor/mod.rs ~279-288): add checks_unknown or exclude.
- every `match status` arm -- the compiler will enumerate these once the variant is added
  (exhaustive-match is the safety net: it won't build until every site handles Unknown).

## The Solution
Add `Status::Unknown` and thread it honestly:
1. Add the variant to the enum (mod.rs:10). Let the compiler list every non-exhaustive match.
2. Renderer: distinct glyph (e.g. ⚪ / "couldn't verify") -- visually different from ✅ red ❌.
3. Health %: exclude UNKNOWN from BOTH numerator and denominator (a check that could not run neither
   helps nor hurts the score). Confirm: N pass + M unknown out of N+M+... reports the same % as if
   the M unknowns did not exist.
4. health_drop / forecast: UNKNOWN does not trigger a drop and does not trend the forecast down.
5. Persistence: add checks_unknown to health_patterns (or exclude cleanly); migrate the INSERT.
6. Migrate INT-146's `check_services` local workaround (bus-probe -> Pass-with-note) to return
   `Status::Unknown` when the bus is unreachable -- replacing the fake-Pass with the real state.
7. (Optional, evaluate) audit other checks for latent could-not-run cases and adopt UNKNOWN where
   honest (tool-missing, path-absent) -- adopt or defer per the fsh filter.

## Success Criteria
- [ ] Status::Unknown added; project builds (every match arm handles it -- exhaustive match enforced)
- [ ] renderer shows a distinct UNKNOWN glyph, visually separate from Pass/Warn/Fail -- demonstrated
- [ ] health % EXCLUDES UNKNOWN from the denominator -- demonstrated: a run with K unknowns reports
      the same % as the same run with those K checks removed
- [ ] UNKNOWN does NOT fire health_drop and does NOT drag the forecast -- demonstrated
- [ ] health_patterns persistence handles UNKNOWN (checks_unknown column or clean exclusion) -- schema
      + INSERT updated, verified no write error
- [ ] check_services (INT-146) migrated from the local Pass-with-note workaround to Status::Unknown
      on bus-unreachable -- demonstrated: bus-less `core doctor run` shows services as ⚪ Unknown
- [ ] a genuinely failing check still reports Fail/Warn (no regression) and a passing one still Pass
      -- demonstrated

## Relationship
Deferred from: INT-146 (fsh/health-check false-red). 146 fixed the SYMPTOM locally in check_services;
this fixes the CLASS in the type system. When this lands, 146's workaround is migrated (criterion above)
and that line in 146 noted as superseded.
Cousin to: INT-118 (Friday engine resumption). 118 emits honest `health_check_failed` EVENTS; this
renders honest STATES. Both serve "do not record fiction." SEQUENCING NOTE: land INT-148 before 118
wires health events onto the bus -- otherwise 118 inherits cry-wolf data (a check that merely could not
run would emit a false health_check_failed). 148 is arguably a prerequisite for 118's event honesty.
Filter: a health system that distinguishes "broken" from "unmeasured" deepens trustworthy control;
conflating them corrupts every number downstream (health %, drop, forecast, events). Squarely in-filter.

## Notes
- Root pattern (Christian's framing): the engine's checks carry Arch-era assumptions that misread the
  NixOS/systemd-user reality -- same class as the tuigreet lockout and the EDITOR-in-two-places issue.
  UNKNOWN is the general antidote: encode "I could not measure this" instead of guessing.
- The INT-146 check_services fix is the TEMPLATE -- bus-probe then honest state. This generalizes that
  one-off into the type system so no future check hand-rolls it.
- Exhaustive match is the safety rail: adding the variant breaks the build at every unhandled site,
  which is how we FIND them all rather than hunting by grep. Work WITH the compiler here.
