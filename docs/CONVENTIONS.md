# Conventions

Small rules that keep Project 0 honest. Each one is here because something broke without it.

---

## Evidence-backed gates (INT-158)

**A ticked box is a promise. Evidence is the receipt. Make "completed" mean "proven".**

When you tick a gate, put the proof in an HTML comment on the next line:

```markdown
- [x] Secure Boot enforcing on metal with custom keys
<!-- evidence: commit f0d0a08e, 2026-07-16. bootctl status -> Secure Boot: enabled (user),
     Measured UKI: yes. db read from the efivar = exactly 2 certs (mine + Framework's),
     ZERO Microsoft. Reboot survived; dep signed gen 383 without complaint. -->
```

Anything that lets future-you check the claim: a **commit hash**, a **file:line**, a **log or
artifact path**, or **`demonstrated: <what and how>`**. Prose counts -- the point is that the
claim is checkable, not that it has a schema.

### The three limits

**Forward-only.** Never retrofit old intents. That is busywork with no payoff.

**Soft.** Nothing enforces this. It is a discipline, not gate-police. An intent that closes
without evidence is not rejected -- it is just less trustworthy, and you will find out later.

**Light.** Trivial self-evident gates need no artifact. "File created" does not need a receipt.
"The VM boots" does.

### Why this exists

This was not invented. INT-133 was already doing it, and the strongest intents in the ledger
all did some version of it. INT-158 wrote it down.

The cost of NOT doing it was measured on 2026-07-16. An audit of the 123 intents marked
complete found gates ticked green that were not true:

- **INT-119** said rustfmt was *"sandboxed, reproducible, unskippable"*. `.git/hooks/pre-commit`
  did not exist. Nothing was ever skipped because nothing ever ran -- ~30 commits landed that
  day alone with zero complaints. **INT-113 had been retired for the identical bug six days
  earlier.** The same defect, shipped twice, with "unskippable" in the comment both times.
- **INT-061** claimed the tree was *"still in the CURRENT layout"* long after it wasn't, and
  claimed Phase 1 was *"substantially complete"* while the profiles directory it described had never existed. Wrong
  in both directions at once.
- Three separate comments said a file *"mirrors framework16"*. All three were false, and one had
  the VM testing a different greeter than the laptop actually runs.

Every one of those would have been caught by a gate that had to cite something.

### The tell

**A gate you have only watched pass might be doing nothing.** The rustfmt hook "passed" for six
days by never running. When you can, prove a gate by watching it FAIL first -- stage something
broken and watch it get rejected -- then fix it and watch it pass. That is the difference between
a gate and a green light.

### Exemplars

INT-133 (the original), INT-161 (Secure Boot, 9 gates), INT-112 (RISK.toml, 6 gates), INT-061
(the v2 restructure), INT-027:58 -- which discharges a `(consider)` gate by **declining** it, with
four numbered reasons. A gate can be closed by deciding NOT to do the thing. That is still proof.

---

## Failure output (INT-199)

**A safe abort and a crash must not look the same. Lead with what did NOT happen.**

When a tool stops, say so in this order, before any internal detail:
==================================================================
PATCH REFUSED -- safe abort

Result
No changes written to src/main.rs

Reason
The anchor matched 3 lines. It must match exactly 1.

What was compared
marker: 'Phase 10'
line 1051: ' // Phase 10 — shell variable table'

Likely cause

The marker is not unique.

Recovery

Lengthen the marker until it is unique.

**Result first.** The absence of side effects is usually the most reassuring fact available and the
hardest to infer. It should never have to be deduced from knowing how the tool works.

**The message carries the diagnostic.** No error code to look up. A code needs a catalogue behind it,
which is a second artifact to maintain and a second one to go stale.

### The three limits

**Assertions are for bugs.** An assertion means the program reached a state that should never happen.
A missing search pattern means the requested operation cannot be completed safely. Different events,
different presentation. Keep the non-zero exit either way.

**Diagnostics are opt-in.** Structured output by default; the traceback behind a debug flag. Normal
use stays approachable without losing anything a maintainer needs.

**Recovery is part of the interface.** An error should begin the debugging workflow, not end it.
Numbered, runnable next steps, so external documentation is rarely needed.

### Why this exists

Measured 2026-07-29. `fpatch` aborted six times in one session and every abort printed a bare
`AssertionError` with a traceback. The tool was correct every time — it declined a patch whose anchor no
longer matched, and wrote nothing. But the fact that mattered, NOTHING WAS WRITTEN, appeared nowhere.
Twice that session a safe refusal was read as a broken tool, and the wrong recovery was attempted.

⚠️ This convention is written down AFTER the tool that follows it, which is the wrong order and worth
admitting. INT-199 asked for the convention first; `fpatch` got there before anyone wrote it. What is
recorded here is what the implementation proved worth having.

### The tell

**If you have to know how the tool works to tell a refusal from a crash, the message is wrong.**

On 2026-08-06 an anchor matched three lines instead of one. The refusal named all three with their
line numbers, said nothing had been written, and said to lengthen the marker. The fix took one edit
and no source reading. That is the whole intent working: the message alone was enough.

### Exemplars

`zero/scripts/dev/fpatch.py` — its `_refuse` is the reference implementation. INT-192 is the
sibling from the opposite direction: tools that cannot express an UNDETERMINED outcome, so a failed
check reports clean. That one is about silence; this one is about noise.

---

## Dependency edges (INT-213)

**`depends_on` means "cannot start until". It does not mean "related to".**

An edge is a claim that work is impossible, not that two things are connected. Write one only when
you can answer: *what would break if I started this anyway?*

```yaml
depends_on: [214]
```

Each edge names its reason in the depending intent, so a future reader can check it rather than
trust it.

### The three limits

**Lifecycle only.** An edge resolves inside one id namespace. `decisions/`, `incidents/` and
`philosophy/` each own their own sequence, so decision 144 and intent 144 both exist -- an edge
pointing at "144" from a record dir means nothing. Only `future`, `in-progress`, `complete` and
`cancelled` share the counter that makes an edge resolvable.

**Forward-only.** The graph exists to order work that has not happened. Do not retro-file
dependencies onto complete intents; it is busywork with no payoff.

**Soft associations go in `relates`.** If it is worth reading together but not blocking, it is not
a dependency.

### What satisfies an edge

Decided in INT-213 G4, implemented in one helper that all five consumers call:

| Dependency status | Satisfies? | Effect |
| --- | --- | --- |
| `complete` | yes | unblocked |
| `cancelled` | yes | **unblocked, but flagged as questionable** |
| `planned` / `in-progress` / `deferred` | no | blocked |
| id names no intent | -- | a validation error, never a permanent block |

★ **Cancellation removes the blocking condition without retroactively making the assumption behind
the edge true.** That is why it clears and flags rather than doing one or the other.

### Why this exists

Measured 2026-08-09: 241 intents, `depends_on` populated on **one**. So `core intent blocked`
answered *"no blocked intents -- all dependencies satisfied"*, and that answer was **false rather
than empty**. A command that reports confidently on an empty graph is worse than one that reports
nothing, because it is trusted.

The cost is the pattern the intent exists to end: starting work whose prerequisite is not done,
discovering it mid-session, and going back over code from a previous pass. That is rework that
breaks work already proven.

⚠️ And the opposite failure is real too. On 2026-08-17 an edge was written pointing at INT-175 --
which is **cancelled**, and cancelled precisely because its premise was false. The edge encoded an
assumption that had stopped being true, and only `blocked` surfaced it.

### The tell

**If you cannot name what would break by starting anyway, it is not a dependency.**

A `blocked` list full of soft associations stops being read, and an unread list is worse than an
empty one -- it looks like diligence. **A false positive costs more than a false negative here.**

### Exemplars

**INT-167 depends_on INT-214** -- DevBox reconstructs a command causally from recorded events, and
no commit has ever created the events provenance columns, so a database built from source cannot
record one. Chosen as the first real edge because it was already proven rather than assumed.

**INT-212 depends_on INT-211** -- cicomplete cannot be reconciled against a gate format while the
ledger has no canonical document shape.

📍 Zero Core laws 1 and 2 (decisions/148): record what is true and derive what should happen; a
concern should not have several competing authorities. An edge is a fact. "Blocked" is a
conclusion. Only the fact is stored.

---

## Findings, fixes and seals (INT-266)

**A finding is something found and not fixed. It gets a name, so the fix can point back at it.**

When work turns up a problem that is not part of the change in hand, file it instead of describing
it in prose:

```text
core intent find "what is wrong, in one line" --at file:line
```

That writes a record, F-NNNN.md, in the zero/intents/findings folder. It holds what was found,
where, which intent found it (the focused one, unless `--by` names another) and when. A record is
never edited to say it was fixed, and never deleted.

A commit says what it touches in trailers, the last lines of its message:

```text
Intent: INT-266           the intent this commit belongs to
Fixes: F-0010             this commit closes F-0010
Finding: F-0012           this commit touches F-0012 without closing it
Seal: 369dd33f83163a00    the reviewed plan that produced this commit
```

Whether a finding is fixed is never stored. It is worked out when asked, from the `Fixes:`
trailers in git.

A seal is the first 16 hex characters of the sha256 of the plan text that was reviewed. The plan
text itself is kept as a git note under refs/notes/seals, so the seal can be checked again at
any time.

One command answers the questions:

```text
core intent trace INT-266            what did INT-266 find, and what is still open?
core intent trace F-0010             what fixed F-0010, and is its seal intact?
core intent trace 87e17913           which intent and findings does this commit touch?
core intent trace 369dd33f83163a00   which commit did this reviewed plan produce?
```

### The three limits

1. **Could not read is never none.** If git or the findings folder cannot be read, trace says so
   and exits 1. It never shows "open", "no findings" or "no commits" in place of an answer it did
   not get.
2. **Old commits answer as old.** A commit from before the trailer rule (8c4c4c15, 2026-09-28)
   answers "no Intent trailer". A commit with only a `Fingerprint:` trailer answers "predates the
   Seal rule". Neither reads as blank, and neither is rewritten.
3. **A seal covers the plan, not the layout.** The commit step formats code: measured 2026-10-04,
   trace.rs hashed differently before and after its commit, with nothing left uncommitted. So a
   commit holds rustfmt's layout of what the sealed plan wrote. The behaviour is the same; the
   bytes are not.

### Why this exists

Before INT-266, findings lived as prose in whichever intent file was open. When it was measured on
2026-09-26, INT-247 alone held 14 "Found, not fixed" sections, and none of them had a name, so a
later fix could only describe the problem again. Commits named intents in prose, so
`git log --grep` found every passing mention. Reviewed plans had fingerprints, but no commit
carried them, so a commit could not be tied back to the plan that was reviewed. Recovering how one
earlier step had been done took three searches of past conversations.

### The tell

You are about to write "Found, not fixed" into an intent file, or a commit message that says
"this fixes the thing from yesterday". File the finding, and put its name in a trailer instead.

### Exemplars

- 87e17913: the first commit to carry `Fixes:`. `core intent trace F-0010` names it and checks its
  seal, 369dd33f83163a00, as intact.
- F-0001 to F-0010: the first finding records, all found by INT-266.
- 6865d306: an intent-file commit with `Intent:` and `Seal:` trailers and its plan kept as a note.
