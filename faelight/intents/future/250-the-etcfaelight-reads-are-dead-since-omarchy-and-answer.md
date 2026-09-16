---
id: 250
date: 2026-09-15
type: future
title: "the /etc/faelight reads are dead since Omarchy and answer as empty"
status: planned
tags: [omarchy, migration, nixos-debt, state, dbus, int-192]
---

## Vision

Five sites read `/etc/faelight/*`. **That directory does not exist.** Every one of them falls
through to `unwrap_or_default()` and returns an empty string or zero -- silently, confidently,
and since 2026-08-26.

## Measured 2026-09-15 (found by the INT-247 path audit)

    $ ls -la /etc/faelight/
    -> No such file or directory (os error 2)

    SITE                              FILE       FALLS BACK TO
    --------------------------------  ---------  ---------------------------
    dbus.rs:173   read_health          HEALTH     the ~/.cache file (REAL)
    dbus.rs:185   read_intent          INTENT     ""  -- EMPTY STRING
    dbus.rs:195   read_intent_id       INTENT     0   -- ZERO
    autobiography/mod.rs:89            COMMITS    ""  -- EMPTY STRING
    faelight-docs/main.rs:403          COMMITS    ""  -- EMPTY STRING

Only ONE of the five has a real fallback. The other four return nothing and say so to nobody.

### THE D-BUS SERVICE REPORTS NO ACTIVE INTENT

`read_intent()` returns `""` and `read_intent_id()` returns `0`, while the ledger has THREE
active intents (129, 222, 247). Anything asking the daemon what the forest is working on has
been told "nothing" for three weeks.

## Why it happened, and why nobody saw it

NixOS generated those files DECLARATIVELY -- `environment.etc."faelight/VERSION"` and its
siblings. The rebuild wrote them; nothing in this repository ever did.

The machine has not been NixOS since 2026-08-26. Omarchy has no reconciler, so nothing recreates
them -- and because every read is wrapped in a fallback, nothing EVER REPORTED their absence.

THIS IS THE INT-192 SHAPE EXACTLY: an unreadable source answering as an EMPTY one. The same
collapse INT-247 Layer 3 exists to prevent, found ALREADY IN PROGRESS in a different directory.
The audit went looking for a future risk and found a live one.

## The data still EXISTS -- elsewhere

    HEALTH    ~/.cache/faelight/health-status  -- written by `core doctor run`
    INTENT    the intent ledger -- `core intent status` knows all three
    COMMITS   git -- `rev-list --count HEAD`, which faelight-release ALREADY RUNS

Nothing is lost. Five readers are pointed at a source that stopped existing.

## Success Criteria

- [ ] Every `/etc/faelight` read is gone. Not repointed to a different hardcoded path -- GONE,
      or reading the real source through an accessor
- [ ] `read_intent()` returns the actual active intent, demonstrated against a ledger with a
      known active intent -- not `""`
- [ ] `read_intent_id()` returns that intent's number -- not 0
- [ ] The commit counters show a number or say they cannot -- not `""`
- [ ] NO NEW SILENT FALLBACK. Where a source cannot be read, the caller SAYS SO. That is the
      whole lesson here; replacing one `unwrap_or_default()` with another would fix the path and
      keep the defect
- [ ] A grep for `/etc/faelight` returns nothing in `.rs` files, proven after the fix
- [ ] The NixOS-era assumption is written where a future session will see it: declarative
      generation is gone, and ANY `/etc/` path in this repository is suspect until checked

## Relationship

- FOUND BY INT-247's Layer 3 path audit, 2026-09-15. The audit's full report lives in 247
- INT-192 is the same defect class: unreadable answering as empty
- INT-247 Layer 3 is the FUTURE version of this bug; this is the one already happening
- The doctor cannot catch this today -- it checks that files it knows about are valid, not that
  a read which silently defaulted ever had a source. Worth asking whether it should

## The Rule

"A fallback is a decision about what absence MEANS. Four of these decided it meant zero, and
nobody was told. A source that disappears with the operating system should be noisier than a
source that was never there." 🌲
