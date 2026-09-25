---
id: 158
date: 2026-07-13
type: future
title: "Evidence-backed gates: formalize the gate-evidence convention already used in strong intents (133-style <!-- evidence: commit/file/log --> after each [x]) into a consistent, light standard. Convention not rebuild."
status: complete
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
- [x] the evidence convention is written down in a durable, discoverable place (docs/templates/
      a CONVENTIONS note) -- format: gate + `<!-- evidence: commit/file/log/demonstrated -->`
<!-- evidence: BOTH places named in this gate, 2026-07-16.
     1. docs/CONVENTIONS.md -- new file. The WHY: the format, the three limits, the exemplars,
        and the measured cost of not doing it (the 2026-07-16 audit).
     2. THE TEMPLATE ITSELF -- faelight/engine/src/domains/intent/mod.rs:1633. This is the part
        that matters. A CONVENTIONS.md is durable but nobody opens it; the template is where you
        are standing EVERY time you file an intent. The convention now propagates BY
        CONSTRUCTION rather than by memory -- the same principle that made greetd a real mirror
        the same evening (INT-061 Phase 2): one definition, imported, instead of copies
        maintained by vigilance.
     A note on scope: this intent fences out "rebuilding inta/cistart/cicomplete tooling: it
     WORKS." Editing a template STRING is not rebuilding the ledger engine. The template IS
     documentation that happens to live in Rust. -->
- [x] new intents completed after this adopt the convention (spot-check a few closes carry
      evidence pointers)
<!-- evidence: this gate was ALREADY SATISFIED before the convention was written, which is why
     158 is formalization and not invention. Every intent closed on 2026-07-16 carries evidence
     blocks naming commits, file:line, or command output:
       INT-160  7/7  rescue USB -- gate 7 walked on real hardware, rollback landed 378->377
       INT-161  9/9  Secure Boot -- bootctl status, efivar reads, sbctl output, commit f0d0a08e
       INT-112  6/6  RISK.toml -- risk-gate run BOTH directions by eye, commits b7342957/c97d40f7
       INT-061       every phase -- store-path comparisons proving the deploy was a no-op
     Fifteen hours of dogfooding before the rule existed. -->
- [x] the convention explicitly states: forward-only (no retrofit), soft (not enforced), light
      (trivial gates exempt)
<!-- evidence: all three appear in BOTH docs/CONVENTIONS.md ("The three limits") and the
     template comment at mod.rs:1633, verbatim:
       FORWARD-ONLY (never retrofit old intents -- busywork, no payoff)
       SOFT (a discipline, not gate-police; nothing enforces this)
       LIGHT (trivial self-evident gates need no artifact)
     Deliberately NOT built: a validator that rejects an evidence-less gate. The tooling to do it
     exists as of today -- INT-119 repaired the pre-commit hook (d9b9b4d7) and INT-112 shipped a
     working RISK.toml gate (c97d40f7), so a "gate [x] with no evidence" check is buildable in an
     hour. This intent says SOFT, and enforcement would contradict its own scope. If it ever
     grows a tooling ambition, that is a SEPARATE decision -- as this intent's Sequencing section
     already says. -->
- [x] this intent's OWN gates carry evidence (dogfood the convention on itself)
<!-- evidence: demonstrated -- you are reading it. Four gates, four receipts, each naming a file,
     a commit, or a command output.
     And the template was proven by RUNNING it, not by reading the diff: after `dep` (gen 386),
     `core intent new future study` created INT-167 and the file was BORN carrying the evidence
     convention in its Success Criteria section. Then deleted -- it existed only to prove the
     deployed binary does what the source claims.
     That distinction is the whole intent. INT-110's checklist warns about exactly this: "the
     command runs the Nix-DEPLOYED binary, not target/debug. A cargo build alone shows green
     while the live command still fails." A gate you have only watched pass might be doing
     nothing. -->

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
