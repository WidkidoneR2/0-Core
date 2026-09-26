---
id: 266
date: 2026-09-26
type: future
title: "Bug Report or Catalog Report of connecting the intent, finding with fingerprint"
status: planned
tags: [intent, fingerprint, organization, bug report]
depends_on: [247, 252, 265]
---

## Vision

A finding, the intent that found it, and the commit that fixed it point at each other by name.
Asking what is still open, what fixed something, or which reviewed plan produced a commit is one
command -- not a search through prose, past conversations or git log.

## Why this cannot start yet

Ruled by Christian 2026-09-26: this starts only after INT-247, INT-252 and INT-265 are complete.
The rename and its fix pass come first. The rule is in the frontmatter as depends_on, so
core intent blocked reports it and cistart refuses until all three are done. The ledger holds
the rule, not anyone's memory.

Filed as an idea -- in Christian's words, "an idea at this point". Nothing below is a design
until the rulings further down are made.

## The Problem -- measured 2026-09-26

    findings    INT-247 holds 14 "Found, not fixed" sections, spread across its dated session
                records; INT-252 and INT-262 hold 0. A pass that spans several intents files what
                it finds in whichever file is open
    no names    INT-265 lists every defect the crate pass walked past, each with file and line,
                and none has an ID. A later fix can only point back by describing it again
    resolution  when a finding is fixed, the record says so in prose in a later section
                ("RESOLVES the FOREST_TEST line ..."). Nothing marks the original line closed
    commits     messages cite INT- numbers in prose, so git log --grep finds every mention,
                including commits that only name an intent in passing
    plans       every change this month went through a reviewed plan with a FINGERPRINT --
                b218e73b9485 for the zero-core rename, e669c6b347ed for its test rewrite -- and
                neither is in commit c4634250. The commit cannot be tied back to the plan that
                was reviewed
    ids         intent numbers are not unique across directories: there are two 141s (INT-247,
                rule 2). Any new ID has to be unique across the whole ledger by construction
    recovery    on 2026-09-26, recovering how an earlier step had been carried out took three
                searches of past conversations

## The idea as it stands

Three things that already exist, linked by name:

    finding     something found and not fixed: what, where (file:line), found by which intent
                and when, and its state -- open, fixed, or closed without a fix and why
    intent      the work that owns the finding or its fix
    commit      the change, naming the findings and intent it touches and the fingerprint of the
                plan that produced it

Candidate mechanisms -- recorded, NOT decided:

    commits     git trailers: "Intent: INT-247", "Finding: <id>", "Fingerprint: <fp>" as the last
                lines of a message. Part of git itself, so no new tooling. Which trailer queries
                this box's git supports is checked before it is relied on
    findings    one small markdown file per finding, the way intents are stored; or rows in
                state.db beside the ledger's other data; or IDs added inside the intent files
    verbs       under core intent, the ledger's one owner, rather than a new tool

## Rulings needed before any work

    storage     where a finding lives: its own file, state.db, or inside the intent
    id          the ID's form -- unique across the ledger, never reused
    migration   whether the open findings in LIVE intents move into the new records, and how the
                old line is marked as moved. Completed intents are history and are never
                rewritten (INT-247 rule 1)
    trailers    whether commits carry trailers before this intent starts. Asked 2026-09-26, not
                yet answered
    scripts     plan scripts are deleted when their work ends. Keeping them is what would let a
                recorded fingerprint be re-derived later; deleting them keeps ~/.cache clean
    owner       the command surface: which verbs, under which existing tool

## The Solution

    1  Wait for INT-247, INT-252 and INT-265 to complete. depends_on enforces it.
    2  Recon: count every place findings live today -- each "Found, not fixed" section in a live
       intent, each INT-265 line -- and record the count as the starting line.
    3  Make the rulings above and write them into this file before any code.
    4  Build the smallest thing that answers the three questions below, inside the existing
       ledger, red first wherever a test can hold it.
    5  Write the convention into docs/CONVENTIONS.md beside evidence and dependency edges, and
       the method into AGENTS.md.

The three questions -- the test of whether this worked:

    a  what did INT-X find that is still open?
    b  what fixed finding F, and which reviewed plan produced that fix?
    c  given commit C, which intent and which findings does it touch?

## Success Criteria

- [x] depends_on: [247, 252, 265] is in the frontmatter, and core intent blocked names 266 as
      waiting on all three
      <!-- evidence: b01efe38, 2026-09-26. core intent blocked: INT-266 waiting on INT-247 (in-progress), INT-252 (in-progress), INT-265 (planned); core intent validate: all 303 intents valid. -->
- [ ] The starting line is recorded: every place findings live today, counted by kind
- [ ] Every ruling above is written into this file, with the date and who ruled, before any code
      is written
- [ ] A finding ID cannot collide: an attempt to create a duplicate is refused, proven red first
- [ ] A finding record states what, where, found by and when, and its state; a field that is not
      known says so rather than reading as empty (INT-192)
- [ ] Question a is answered by one command, on real data
- [ ] Question b is answered by one command, on real data, naming the fixing commit and the
      fingerprint of the plan that produced it
- [ ] Question c is answered by one command, on real data
- [ ] When git or the store cannot be read, every query says it could not read -- never "no
      findings" or "no commits". A class test covers it
- [ ] The convention is in docs/CONVENTIONS.md and the method in AGENTS.md
- [ ] Each gate carries its evidence on the line after it (INT-158)

## Not in scope

    history     completed intents, CHANGELOGs and past commit messages are not rewritten
    trackers    no external issue tracker, service or download -- only what is on the box
    past fps    fingerprints of plans already applied are not reconstructed

## Relationship

- Depends on INT-247, INT-252 and INT-265 (Christian, 2026-09-26)
- INT-265 is the first real set of findings the records would hold
- INT-158: the evidence convention -- a finding's fixing commit is a receipt of the same kind
- INT-213 and decision 142: store facts, derive conclusions. A finding's state is a fact; "still
  open for INT-X" is a query and is never stored
- INT-192: an unreadable source answers as unreadable, never as empty
