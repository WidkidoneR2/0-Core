---
id: 235
date: 2026-08-27
type: arch
title: "a declared capability is never checked to resolve, so a contract can advertise a command that does not exist"
status: planned
tags: [architecture, rust, design]
---

## The Invariant

A DECLARED COMMAND, ACTION OR DELEGATION TARGET MUST RESOLVE TO AN EXISTING
SUPPORTED CAPABILITY.

Stated around resolution rather than execution, deliberately. Displaying a
command the user might run is legitimate. The defect is having no proof that
what the system claims can be invoked actually exists.

## The motivating case

delegate_contracts carries a rollback_action column. It appears six times in
delegate/mod.rs: the schema declares it, the insert stores it, a select
retrieves it, a destructure unpacks it, and two lines display it.

NONE OF THE SIX IS A CHECK. Nothing validates that the string names a real
command, and nothing executes it.

The auto-checkpoint contract declares its rollback action as
core checkpoint restore {id}. That subcommand was DELETED in 17fb12bb, and
nothing anywhere noticed. The contract is still rated LOW risk with a 0.85
confidence gate.

THE RISK RATING IS THE NASTY PART. LOW is accurate for a no-op and wrong for
what the contract claims. A broken action and an honest risk score produce a
system that looks correct from every angle it measures itself from.

## Proportion, measured

Seven contracts are seeded. Only TWO declare an action at all:
- auto-checkpoint -- core checkpoint restore {id} -- DOES NOT RESOLVE
- restart-service -- systemctl --user start {name} -- resolves

The other five carry an empty string with requires_rollback = 0.

One broken out of two is worth reporting. It is not a systemic collapse, which
is the argument for a check rather than a rewrite.

## Why this is its own invariant

Three distinct questions have surfaced from the same week of work, and
collapsing them would produce a detector that answers none of them well:

- INT-233, OWNERSHIP -- where does the knowledge live?
- INT-234, MUTATION -- how does state change?
- THIS, RESOLUTION -- does a declared capability actually exist?

## Related, and the same defect in another register

INT-231 -- code cites intents that do not exist and nothing notices. That is a
reference whose target can disappear silently. So is this. Intent citations and
delegation actions are the same failure in different registers, and a solution
to one may inform the other.

IMMEDIATE INSTANCE OF INT-231: five comments in faelight-daemon already cite
INT-235 (daemon.rs:42, 94, 449, 463, 950) for a DIFFERENT 235 that was never
filed. Third such collision in one day, after 233 and 234.

## Scope discipline

DO NOT BUILD A GENERIC EVERYTHING-MUST-RESOLVE FRAMEWORK. Capture the invariant,
use rollback_action as the concrete case, and see whether the same detector
naturally handles the next example when one arrives. A framework built from one
instance is a framework built from a guess.

## Success Criteria
- [ ] A declared action is checked against the real CLI surface before a
      contract can be activated
- [ ] Watch it fail first: run the check against auto-checkpoint as it stands
      today and confirm it reports the broken action
- [ ] restart-service passes, so the check discriminates rather than warning
      on everything
- [ ] Reports, never rewrites
- [ ] The mechanism is examined against INT-231 before being generalised
