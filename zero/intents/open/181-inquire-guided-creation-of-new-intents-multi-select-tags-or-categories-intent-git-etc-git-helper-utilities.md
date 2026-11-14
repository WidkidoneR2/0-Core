---
id: 181
date: 2026-07-20
type: feature
title: "faelight-prompt: the forest's interactive layer (inquire) -- foundation + guided intent ledger"
status: planned
tags: [inquire, intent, ledger, git, helper]
---

## Vision
`faelight-prompt` becomes a FOUNDATIONAL pillar of the forest -- survivor-tier, in the same breath
as core, fsh, and friday. Not a helper that wraps inquire; the forest's single interactive voice.
Every place the system asks the human something -- creating an intent, choosing one to start,
Friday offering a suggestion, git staging a commit -- speaks through this one layer, and speaks
BEAUTIFULLY. The mandate is not "functional prompts." It is a workflow that flows like silk:
consistent, unhurried, colored in the forest's own palette, so that using the ledger feels less
like filling a form and more like walking a path that already knows where you are going.

178 evaluated inquire and said adopt. 181 is the adoption -- built as a pillar, first surface being
the intent ledger, which lives in `core`.

## The Problem
The ledger is typed blind. `inta` asks for a category by NUMBER, a title as one run-on line, tags
as a comma-string formatted from memory. Nothing shows the valid set, so `type: fsh` gets invented
(twice this session) and tags drift into typo one-offs. To START an intent you must recall its
number (`cistart 174`) with no sight of the list. Destructive actions (cancel/delete/archive) have
no confirmation. And Friday and git will each want the same "ask the human cleanly" capability --
hand-rolled three times, the forest fractures into three prompt styles and three copies of the
same code.

The deeper problem: there is no SHARED, BEAUTIFUL interactive primitive. Without one, every prompt
is bespoke and the forest speaks in three accents.

## The Solution
Build the pillar first, then two ledger surfaces on it. The pillar lives in `core` (where the
ledger already lives); fsh and friday consume it later.

### Part 1 -- `faelight-prompt`: the pillar (the craft lives here)
A first-class `core` module -- treated like a surviving tool, documented, themed, tested -- that
wraps inquire and knows the forest's LIVE vocabulary, so no caller ever passes a magic string:
- `pick_type()`   -> Select over the LIVE valid intent types, read from the actual taxonomy. An
  invalid type is not rejected -- it is simply never offered. `type: fsh` becomes IMPOSSIBLE.
- `pick_tags()`   -> MultiSelect over EXISTING ledger tags + an "+ new tag" affordance. Tagging
  stays consistent by construction.
- `pick_status()` -> Select over valid statuses.
- `pick_intent(filter)` -> Select over the LIVE intent list (planned / in-progress / ...), the
  number shown but never required. This is what makes `cistart`/`dc` show-and-pick.
- `confirm_destructive(action, target)` -> Confirm naming the action AND the target, for
  cancel / delete / archive. Nothing irreversible without a beat.
- `text(prompt)` / `editor(prompt)` -> single-line and long-form (title / vision).

THE BEAUTY IS NOT OPTIONAL -- it is the point:
- ONE `RenderConfig` themed in the forest palette (the greens, the ❄, the ▶/· glyphs the forest
  already uses) lives here and only here. Every prompt inherits it. Change the theme once, the
  whole forest re-skins.
- Prompts carry context: the current intent count, the focus, a one-line "why" under the question
  so the human is never guessing what a field means.
- Flow like silk: sensible defaults pre-selected, the common path is Enter-Enter-Enter, escape
  cancels cleanly and leaves nothing half-written. No dead ends, no raw error dumps -- a rejected
  input re-asks in place with a gentle reason.

### Part 2 -- the guided intent ledger (TWO surfaces, both from day one)
`inta` (create) and `cistart ###` (act) are different actions and both start here:
- `inta` -> a guided wizard: pick_type -> text(title) -> editor(vision) -> pick_status ->
  pick_tags. No numbers, no run-on titles, no invalid types, consistent tags. It writes a
  well-formed intent every time.
- `cistart` / `dc` with NO argument -> pick_intent shows the live list, you choose by sight. The
  numbered forms (`cistart 174`) still work untouched, for muscle memory and scripts.
- cancel / delete / archive -> confirm_destructive first.
- edit an existing intent's fields through prompts instead of hand-editing markdown.

### Sequenced siblings (filed, NOT built here -- they ride the pillar)
- Friday interaction: suggestions become interactive (Select "did you mean X/Y/Z?", Confirm
  "apply this?"); Friday can offer "file this as an intent?" -> drops into the guided `inta` flow.
- Git helper utilities: the add -> commit -> (rustfmt re-add) -> gp dance as a guided flow, file
  staging via MultiSelect, Confirm-before-push.
Each is its own intent so each ships completable, on a proven pillar.

## Scope guardrails
- fsh/core is the daily driver: NUMBERED and non-interactive command forms MUST keep working. The
  guided flow is ADDITIVE, never the only door -- scripts and muscle memory keep their path (same
  discipline as INT-190's `-c`).
- Build Part 1 (pillar) before Part 2 (surfaces). Build both ledger surfaces before filing weight
  onto Friday/git.
- No hand-rolled inquire calls outside `faelight-prompt`. A prompt the pillar cannot yet make is a
  reason to GROW the pillar, never to hand-roll around it. This one rule is what keeps the voice
  singular.
- Recon FIRST when this starts: locate the `core intent` command code and the taxonomy source
  (where valid types/statuses/tags actually live) before writing the module -- pick_type is only
  magic if it reads the REAL source.

## Success Criteria
- [ ] Recon done: the `core intent` command code, the taxonomy (valid types/statuses), and the
      live tag set are located in source; faelight-prompt's home crate/module is chosen with that
      in hand (it lives in core).
- [ ] `faelight-prompt` exists as a first-class core module: wraps inquire; exposes pick_type /
      pick_tags / pick_status / pick_intent / confirm_destructive / text / editor; ONE shared
      forest-themed RenderConfig. inquire in Cargo.toml, Cargo.lock (repo root) staged.
- [ ] pick_type reads the LIVE type set -- demonstrated by adding a type to the taxonomy and seeing
      it appear in the picker with NO change to faelight-prompt. `type: fsh` is unreachable.
- [ ] pick_tags = MultiSelect of existing ledger tags + add-new. Demonstrated.
- [ ] `inta` guided wizard runs end to end and writes a well-formed intent (valid type, clean
      title, consistent tags) -- shown by a real intent created through it.
- [ ] `cistart` / `dc` with no argument show the live list and act on the pick; numbered forms
      still work. Both demonstrated.
- [ ] Destructive actions confirm first via confirm_destructive.
- [ ] THE SILK GATE: the common create + start paths are demonstrably smooth -- sensible defaults,
      Enter-through happy path, clean escape leaving nothing half-written, in-place re-ask on bad
      input, forest-palette theme throughout. Shown with a walk-through (asciinema or described
      step-by-step). Beauty is a gate, not a hope.
- [ ] core still builds, fsh still boots/deploys, every existing non-interactive command form still
      works. fsh-test green on the deployed binary.
- [ ] The two siblings (Friday, git helpers) are filed as their own linked intents.
- [ ] Each gate carries evidence per INT-158.

## The Rule
"A pillar, not a helper. Build the forest's interactive voice ONCE, in `core`, in the forest's own
colors -- then intents, Friday, and git all speak it, and speak it beautifully. The workflow should
flow like silk: the human walks a path that already knows where they are going." 🌲

## RECON FINDINGS (2026-07-20) -- verify-first, before any build
- The `core` command is the `faelight` crate (`faelight/rust-tools/faelight/`, bin renamed to
  `core`); it depends on `faelight-core`. The intent COMMAND logic (cistart/cicomplete) is largely
  in `faelight-shell/src/commands/mod.rs` + `schema.rs`; paths + theme are in `faelight-core`.
- TWO UNCOORDINATED VOCABULARIES, neither schema-enforced:
  1. LIFECYCLE FOLDER (chosen by the `inta` wizard): future / experiments / philosophy / decisions
     / incidents / in-progress / complete / cancelled -- defined ad-hoc as one function each in
     `faelight-core/src/paths.rs` (intents_experiments(), intents_philosophy(), ...).
  2. `type:` FRONTMATTER (feature / fix / study / improvement / infrastructure / ...): pure
     FREE-FORM. Nothing enforces it -- which is exactly how `type: fsh` slipped through.
  These are different axes (bucket vs kind) and BOTH lack a canonical list in code.
- THEREFORE 181's true FIRST foundation gate is: AUTHOR THE CANONICAL TAXONOMY. The forest has
  never had a single source of truth for a valid intent `type:` (and the folder set is only
  implicit in paths.rs). pick_type is only "magic" if it reads a real authored source -- so 181
  must CREATE that source, not just read it. This makes 181 more foundational, not less.
- THE SILK GIFT: `faelight-core/src/theme.rs` ALREADY defines the forest palette (e.g.
  NEON_PURPLE = active intent/philosophy). faelight-prompt's RenderConfig pulls from theme.rs so
  every prompt inherits the exact colors already in use -- the beauty has a real source.
- LIKELY HOME: `faelight-prompt` as a new module in `faelight-core` (it already owns paths.rs +
  theme.rs -- the two things a vocabulary-aware, forest-themed prompt layer needs), consumed by
  faelight-shell's command code. Confirm against schema.rs when the build starts.
- NEXT-SESSION START: read `faelight-shell/src/commands/mod.rs` (the cistart/cicomplete/add code)
  and `faelight-shell/src/schema.rs` (does a partial schema already live here?), then decide where
  the canonical taxonomy is authored and where faelight-prompt lives.