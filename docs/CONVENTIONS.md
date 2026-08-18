# Conventions

Small rules that keep the forest honest. Each one is here because something broke without it.

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
  claimed Phase 1 was *"substantially complete"* while `nix/profiles/` had never existed. Wrong
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

`faelight/scripts/dev/fpatch.py` — its `_refuse` is the reference implementation. INT-192 is the
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
