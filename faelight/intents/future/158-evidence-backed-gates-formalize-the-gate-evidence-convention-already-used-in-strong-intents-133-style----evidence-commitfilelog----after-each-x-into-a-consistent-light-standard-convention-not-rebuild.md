---
id: 158
date: 2026-07-13
type: future
title: "Evidence-backed gates: formalize the gate-evidence convention already used in strong intents (133-style <!-- evidence: commit/file/log --> after each [x]) into a consistent, light standard. Convention not rebuild."
status: planned
tags: [intent-system, gates, evidence, ledger, convention, discipline]
---

## Vision
Every gate cites its proof. "Completed" means "here is the receipt" -- a commit, a file, a log,
a demonstrated result -- not "trust me, I checked a box." Formalize the evidence-in-gates
convention the strongest intents ALREADY use into a consistent, light standard across the ledger.

## The reality (this is FORMALIZING, not building)
The practice already exists -- it is just inconsistent. INT-133 is the exemplar: each gate `[x]`
carries an HTML-comment receipt, e.g.
  - [x] Doctor "Nix Hygiene" check runs... <!-- 2026-07-11: check_nix_hygiene(core_root) in
    checks.rs shells out to deadnix...; live on gen 350 -->
Commit hashes, file:line, dates, demonstrated-not-declared proof -- all right there in the gate.
Other intents tick gates with lighter or inline evidence ("committed 6258837c"), and some with
none. The GAP is consistency and convention, NOT capability. The system already supports this
(markdown gates + HTML comments); strong intents already do it. This intent makes it a standard.

## In scope (formalize the convention)
- Documented convention: each gate `[x]` SHOULD carry an evidence pointer -- commit hash, file
  path, log/artifact path, or a one-line "demonstrated: <what/how>" -- in a `<!-- evidence: ... -->`
  comment after the gate (the 133 pattern). Prose demonstrated-not-declared counts as evidence.
- Applies GOING FORWARD only. Do NOT retrofit the 180+ existing intents -- that is busywork with
  no payoff.
- SOFT expectation, not enforcement. The goal is honest receipts and a ledger future-you can
  trust, not gate-police ceremony. Trivial self-evident gates need no forced artifact.
- Write the convention down somewhere durable (a short note in the intent-system docs / the
  templates / a CONVENTIONS section) so it is discoverable, not tribal knowledge.

## Explicitly OUT of scope (guardrails -- do NOT let this become a meta-project)
- Structured Tasks schema (Intent -> Feature/Design/Tasks/Gates as separate fields): the intent
  bodies ALREADY carry informal task/phase structure (087's P0-P4, 027's remaining-work, 156's
  scope breakdown). Not rebuilding that into a schema.
- CI / `nix flake check` AUTO-GENERATING and attaching evidence artifacts: heavy, and it depends
  on the VM-testing infra (INT-157) which is itself deferred until after Friday prereqs. The
  convention works with MANUAL pointers now; auto-generation is a much-later maybe.
- Rebuilding inta / cistart / cicomplete tooling: it WORKS. This is a documentation + convention
  layer on top, not a rewrite of the ledger engine.
- Mandatory testing-as-a-gate on every intent: too heavy; evidence is a discipline, not a
  requirement enforced by tooling.

## Sequencing
Low-cost, do-anytime -- it is mostly writing down a convention + applying it on new intents. But
NOT urgent and NOT pre-Friday-blocking. Slot it whenever; it needs no dedicated day. If it ever
grows a tooling ambition (e.g. cicomplete prompting "evidence for each gate?"), that is a
SEPARATE, later decision -- keep THIS intent to the convention only.

## Success criteria
- [ ] the evidence convention is written down in a durable, discoverable place (docs/templates/
      a CONVENTIONS note) -- format: gate + `<!-- evidence: commit/file/log/demonstrated -->`
- [ ] new intents completed after this adopt the convention (spot-check a few closes carry
      evidence pointers)
- [ ] the convention explicitly states: forward-only (no retrofit), soft (not enforced), light
      (trivial gates exempt)
- [ ] this intent's OWN gates carry evidence (dogfood the convention on itself)

## Relationship
- Origin: Christian's "Evidence section" idea (2026-07-13 brainstorm) -- recon revealed the
  practice already exists (133), so this is formalization, not invention. The heavier parts of
  the same idea (Tasks schema, CI-generated artifacts, mandatory testing gates) were consciously
  fenced OUT above as meta-project scope that would compete with Friday.
- Deepens the ledger-honesty value that runs through the forest (demonstrated-not-declared gates,
  doctor Status::Unknown, the debug-build marker): the system should tell the truth about what
  was actually proven.
- Companion discipline to INT-130 (the cicomplete gate-blocker / mis-gated-intents fix) -- same
  family: gates should mean something.

## The Rule
"A ticked box is a promise. Evidence is the receipt. Make completed mean proven." 🌲
