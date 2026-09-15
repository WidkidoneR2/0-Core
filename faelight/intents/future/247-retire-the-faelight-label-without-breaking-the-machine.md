---
id: 247
date: 2026-09-09
type: future
title: "retire the Faelight label without breaking the machine"
status: planned
tags: [naming, migration, infrastructure, zero-core]
---

## Vision

The project is **Project 0**. `faelight` is a name it used to have, and one day nothing in the
tree says it except history. Getting there costs nothing if it is done slowly and could cost the
machine if it is done fast.

⭐ THIS IS DELIBERATELY NOT URGENT, AND THAT IS THE MOST IMPORTANT LINE IN THIS FILE. The label
costs nothing to keep. The shell is the priority, and the failure mode of rushing a rename is a
machine that does not boot or a ledger that silently reads empty. Slow is not a compromise here --
it is the requirement.

## THE NAME, DECIDED 2026-09-14 -- AND THE TOOLING DECIDED HALF OF IT

The name was chosen in four registers, not one, because a name that only works in conversation
gets quietly abandoned the first time it will not compile.

    REGISTER          FORM          WHY
    ---------------   -----------   ------------------------------------------------------
    spoken, docs      Project 0     What it IS: a minimal starting point, not a rebuilt
                                    ecosystem. "Faelight Forest" promised a world;
                                    "Zero Core" promised a framework; this promises only
                                    what is needed.
    repo and root     0-core        UNCHANGED. The riskiest thing to move does not move.
    the CLI           0             Measured: a binary named `0` runs fine. And it is a
                                    lovely thing to type.
    crates            zero-*        FORCED -- see below.
    env vars          ZERO_*        FORCED -- see below.

### ⭐ THE CONSTRAINTS ARE MEASURED, NOT ASSUMED. Both were tested on this machine, 2026-09-14.

    export 0_FOO=bar
    -> bash: export: `0_FOO=bar': not a valid identifier

POSIX names are `[a-zA-Z_][a-zA-Z0-9_]*`. **An environment variable cannot begin with a digit.**
Not a style preference -- the shell refuses the assignment.

    [package] name = "0-core"
    -> error: invalid character `0` in package name: the name cannot start with a digit

**Cargo refuses the PACKAGE name**, not merely the lib identifier. No crate in this workspace can
ever be `0-*`.

What DOES work, also measured: directories (`~/0-core`), globs (`0-*/`), and an executable named
`0`. So the split above is not a compromise between tastes. It is the shape the tooling allows,
found by asking it instead of by arguing.

⭐ AND THE SPLIT IS A FEATURE THIS PROJECT HAS ALREADY PROVEN. `NovaShell` is what it is called;
`nsh` is what is typed. Nobody has ever been confused by that. "Project 0" and `0-core` is the
same arrangement, and it means the name can be beautiful in prose without having to be legal in
a linker.

### What this REMOVES from the work

The original plan implied moving the root, the repo, and every crate. Two of those are now off
the table by measurement rather than by decision:

    ~/0-core             stays. Every path in AGENTS.md, every intent citation, `ship`,
                         every muscle-memory `cd` -- untouched.
    WidkidoneR2/0-Core   stays. No remote rename, no broken clone URLs.

The remaining surface is smaller than it looked: documentation, the `faelight` CLI, crate names
as crates are rewritten anyway, and the state paths.

## The Problem

Load-bearing today, in rough order of how badly a mistake hurts:

    state paths     ~/.local/state/faelight/state.db, ~/.config/faelight,
                    ~/.config/faelight-shell/config.nsh
    env vars        FAELIGHT_STATE_DB, and XDG_STATE_HOME read INDEPENDENTLY of HOME
    unified CLI     `faelight`
    crate prefix    faelight-* across most of rust-tools
    docs            README, ROADMAP, the generated catalog, AGENTS.md

### Measured 2026-09-12, so the census has a starting line

    29 directories in faelight/rust-tools/
     7 already retired = true in tools.toml (core-diff, faelight-diff, faelight-bootstrap,
       faelight-browser, faelight-cleanup, faelight-menu, faelight-notify)
     1 is NEITHER: faelight-fm is deployable = false, retired = false

And the counts already disagree with each other -- the README says one number, the catalog says
another, the tree says a third. Fixing the count is part of the census, not a follow-up.

### Three months of evidence about what matters

Roughly 80% shell, 10% health, 5% everything else since the Omarchy migration. `faelight-gen`,
`faelight-vault`, `faelight-context`, `faelight-clipboard` and `faelight-zone` have not been
touched in that window. That is not an argument for deleting them tonight; it is the reason this
intent exists at all.

## The Solution -- FIVE LAYERS, ONE PER WEEK AT MOST

### RULE: PACE IS A GATE, NOT A SUGGESTION

  - ONE layer per week, maximum. Never two.
  - Shell and DevBox work continues in the SAME sessions. This intent is background work.
  - A layer that is not finished in its week TAKES ANOTHER WEEK. It is not compressed and it does
    not run alongside the next one.
  - ⭐ A LAYER MAY SIT UNSTARTED FOR A MONTH AND THIS INTENT IS STILL HEALTHY. A gap is not a
    stall. Written down so a future reader does not treat a pause as a failure and rush to catch
    up -- rushing is the only way this breaks something.

### LAYER 0 -- FREEZE THE NAME (effective on filing, costs nothing)

No new crate, binary, path, doc heading or config key begins with `faelight`. New work is named
`nsh`, `core`, `friday`, `devbox`, or `zero-*`.

This is free, permanent, and cannot break anything, which is why it lands FIRST and not as part
of the census. It stops the problem growing while everything else is still being decided.

### LAYER 1 -- IDENTITY IN WHAT YOU READ

README, ROADMAP, AGENTS.md, the catalog generator. Project name becomes **Project 0**; a subtitle
may honestly say *formerly Faelight Forest*.

⚠️ AND STOP GENERATING FICTION. `faelight-docs` currently produces a catalog listing
`faelight-shell` when the crate has been `novashell` since 2026-09-01. A generator that
resurrects old names is a machine for undoing this intent.

### LAYER 2 -- BINARIES AND CRATES YOU STILL RUN

⭐ DO NOT RENAME A CRATE FOR SPELLING. `faelight-git` does not become `zero-git` because the
prefix is ugly. When a crate is REWRITTEN OR SPLIT, the new crate gets the new name. That is
exactly how faelight-shell became novashell without anyone doing a rename pass.

Old crate directories stay until they are DELETED, not until they are renamed.

⚠️ AND THE NEW NAME IS `zero-*`, NOT `0-*`. Cargo refuses a package name starting with a digit --
measured above. A future session that reaches for `0-git` will get a compile error and should
read this line rather than re-deriving the rule.

### LAYER 3 -- STATE PATHS, LAST AND MOST CAREFULLY

⚠️ THIS IS THE ONE THAT CAN BREAK THE MACHINE. Alias first, flip later:

    ~/.local/state/zero  ->  ~/.local/state/faelight
    ~/.config/zero       ->  ~/.config/faelight

Only flip the code default after `core doctor` and `nsh history` both work through the alias for
A FULL WEEK.

⚠️ THE ENV VAR IS `ZERO_STATE_DB`, NOT `0_STATE_DB`. Measured: bash rejects the second as "not a
valid identifier". A rename that produced it would fail at the moment of use, in the one layer
that can silently empty the ledger.

⭐ AND IT IS FOUR THINGS THAT MUST AGREE, NOT ONE. nsh-test measured on 2026-09-05 that hiding
the forest takes HOME *and* XDG_STATE_HOME, because state_home reads XDG independently -- a bare
HOME redirect left health, focus.toml and the ledger visible. Add FAELIGHT_STATE_DB and
NSH_CONFIG and there are four independent inputs. A symlink that satisfies three of them and not
the fourth produces a SILENT EMPTY LEDGER, which has already happened once (0 done, 0 tools,
0 planned).

That failure is the same collapse INT-192, INT-245 and INT-246 are all about: an unreadable
source answering as an empty one. Whatever reads these paths must be able to say "I could not
read that" rather than returning nothing.

### LAYER 4 -- DESKTOP AND SHELL INTEGRATION

Hyprland binds, the bash login shim, systemd user units, PATH wrappers, completions. Grep the
WHOLE HOME as well as the repo, for `faelight`, `fsh`, and `faelight-fm`, before deleting any
crate.

## TWO RULES THE LEDGER NEEDS BEFORE LAYER 1 -- both learned 2026-09-15

### 1. A COMPLETED INTENT IS HISTORY. DO NOT REWRITE IT.

When `faelight-fm` and `faelight-glog` were retired, tWENTY-FOUR intents mentioned them.
Only THREE needed touching:

    complete/    16   what was true then -- LEAVE
    decisions/    4   records of decisions made -- LEAVE
    philosophy/   1   LEAVE
    cancelled/    1   already dead -- LEAVE
    future/       3   LIVE -- the only ones that matter

Layer 1 covers README, ROADMAP, AGENTS.md and the catalog generator -- documents that
describe the PRESENT. The intent archive describes the PAST. Rewriting history to match
the present destroys the reason the ledger exists, and a rename sweep is exactly the
kind of well-meaning tidying that would do it.

### 2. ⚠️ INTENT NUMBERS ARE NOT UNIQUE ACROSS DIRECTORIES.

There are TWO intents numbered 141:

    future/141-faelight-glog-v02-...md
    decisions/141-look-at-project-dawnwood-...md

A glob like `intents/*/141-*.md` matches both, and taking `[0]` silently picks whichever
sorts first. That happened on 2026-09-15: a cancellation note for the glog intent was
written into the Dawnwood intent instead, and the file it was meant for was moved to
`cancelled/` with its gates untouched. Both edits reported success.

*THE RULE: edit an intent by EXPLICIT PATH, and guard the write -- refuse unless the
file contains something only the intended target contains.* Layer 1 will touch many
intent files at once; without this, the same mistake happens quietly in a layer where
nobody is watching each edit.

### 3. THERE IS NO `cancel` VERB.

`core intent` has start and complete, and `intc` REFUSES to complete an intent with
open gates -- correctly. Cancelling is therefore a `git mv` plus a hand-written note,
and the gates must be deferred with the ledger's own format so they read as CLOSED
rather than FORGOTTEN:

    ⏸ gate description -- deferred: [reason] -- approved by: christian <date>

Layer 2 will retire more tools, and some will have a live intent behind them. Worth
building the verb if it happens more than twice more.

## THE DECISION TEST -- ask these in order, every time

    1. Did I run this after 2026-08-26, on Omarchy?
    2. Does NovaShell or DevBox depend on it?
    3. Does Omarchy, or Flea/Yazi, already do the job better?
    4. If I deleted the crate tonight, what breaks tomorrow morning?

If (1) is NO and (4) is "nothing", it goes to `retired/` that week.
If (3) is YES, the replacement lands FIRST and the delete follows in the SAME week -- not after
the old one is polished.

## WHAT NOT TO DO -- each with the reason, so it survives being re-argued

  - ⛔ DO NOT RENAME THE ROOT OR THE REPO. `~/0-core` and `WidkidoneR2/0-Core` are correct under
    the decided name and moving them buys nothing. This is the single biggest change from the
    original plan and it is a REDUCTION in work, which is the rarest kind of good news.
  - ⛔ DO NOT RENAME 30 CRATES IN ONE COMMIT. A month spent on identifiers is a month not spent on
    the shell, and the shell is what has a deadline.
  - ⛔ DO NOT COMMENT A CRATE OUT OF THE WORKSPACE. A suppression is not a decision. This is the
    same shape as `#![allow(dead_code)]` sitting on a module nobody calls: it survives another
    year because nothing forces the question. Delete it or move it to `retired/`.
  - ⛔ DO NOT KEEP faelight-fm "for the plugin system". Flea and Yazi are already the answer.
  - ⛔ DO NOT MOVE state.db IN THE SAME COMMIT AS ANYTHING ELSE. One commit, one risk.
  - ⛔ DO NOT START DEVBOX BY WRAPPING EVERY OLD CRATE. DevBox installs the SHORT list. If a crate
    cannot be installed by DevBox on a clean Omarchy box, that is evidence about the crate.

## ⭐ DEVBOX IS THE CENSUS INSTRUMENT

Last-invoked date is a weak signal. A stronger one: **a crate that cannot run in a clean room is
already halfway retired.**

DevBox runs a command with the forest invisible -- HOME, XDG_STATE_HOME, FAELIGHT_STATE_DB and
NSH_CONFIG all redirected. Proven on nsh-test 2026-09-10: nineteen of 194 cases turned out to be
probing the author's checkout rather than testing the shell, which was invisible for as long as
the suite only ran on one machine.

The same question asked of each crate: does it work on a machine that is not this one? That is
the question the whole Omarchy-default ambition rests on, and it is answerable today.

## THE MENTAL MODEL THIS RENAME IS FOR

    Omarchy                 the base. Someone else's good work, kept.
    └── Project 0           only what is needed, and nothing carried over out of habit.
        ├── the shell       NovaShell -- where the attention goes
        ├── the tools       what survived the decision test
        └── the memory      state, history, intents: the part no other shell has

Faelight Forest was an ecosystem, and rebuilding an ecosystem after changing distros is the exact
trap this avoids. **Omarchy provides the forest. Project 0 provides only what I need.**

⚠️ A DIRECTORY RESTRUCTURE (`core/ tools/ scripts/ config/ modules/`) IS A SEPARATE DECISION AND
IS NOT PART OF THIS INTENT. It would break every path in AGENTS.md, every intent citation, and
`ship`, in exchange for tidiness. If it happens it gets its own intent and its own pace gate.
Naming and moving are two risks; this intent takes one of them.

## Success Criteria

- [ ] LAYER 0 landed: the freeze is written into AGENTS.md or CONVENTIONS.md as a rule, not a
      plan. Nothing new is named faelight from that commit onward
- [x] The NAME is decided in every register it has to survive, with the constraints measured
      rather than assumed
      <!-- evidence: 2026-09-14. `export 0_FOO=bar` -> "not a valid identifier"; cargo -> "invalid character `0` in package name". A binary named `0`, the directory `0-core` and the glob `0-*/` all work. Project 0 / 0-core / 0 / zero-* / ZERO_*. -->
- [ ] Week 1 census exists as `docs/inventory.md`: every crate with keep / replace / retire, the
      four decision-test answers, last invocation, and whether it runs under DevBox
- [ ] The COUNT is reconciled. README, the generated catalog, tools.toml and the actual tree agree
      on how many tools exist. Today they do not
- [x] faelight-fm is gone -- workspace, PATH, docs, teach, Friday facts, command registry, and the
      Hyprland bind. Moved to `retired/` or deleted, NOT commented out
      <!-- evidence: 2026-09-15. Crate deleted, binary retired with `ship --retire`, registry marked retired = true, aliases fm/fmd removed, census case deleted. deadwood reports registry orphans clean. faelight-glog went with it on the same evidence. 9,893 lines removed; 193/193 green after. -->
- [ ] The NixOS-era crates are gone by the same standard. The machine has not been NixOS since
      2026-08-26
- [ ] `faelight-docs` cannot resurrect a retired tool. Proven by running it after a retirement and
      confirming the catalog does not list it
- [ ] Layer 3 is NOT started until layers 0-2 are done and a path audit lists every hardcoded
      reference. The audit is a deliverable in its own right
- [ ] The state alias runs for A FULL WEEK with `core doctor` and `nsh history` green before any
      code default changes. Evidence: the dates
- [ ] Whatever reads the state paths reports UNREADABLE as unreadable. A silent empty ledger is
      the failure this intent most needs to avoid, and it is INT-192's collapse in a new place
- [ ] The `faelight` unified CLI is decided: renamed to `0` with the same subcommands, or deleted
      with `nsh` and `core` called directly. Written down either way
- [ ] No layer was done in the same week as another. If one was, say so here and say why -- the
      pace rule is a gate and breaking it is a thing to record, not hide

## Relationship

- ⏭ PRIORITY: BELOW the shell, always. This is spelling. If a session has to choose, this loses.
  Recorded so the ordering survives the enthusiasm of whoever picks it up
- INT-245, INT-246 and INT-192 share this intent's central failure mode: something unreadable or
  unenforceable answering as though it were empty or applied. Layer 3's silent-empty-ledger risk
  is the same defect, which is why it is last
- DevBox (INT-167 lineage) is the instrument, per the section above
- The naming policy in the existing identity notes stands: Faelight is not renamed WHOLESALE. This
  intent is how it is retired gradually instead

## The Rule

"A name has to survive four registers: what you say, what you type, what compiles, and what the
shell will export. Three of those were decided by asking the tooling rather than by choosing.
`~/0-core` does not move, and the rename got smaller the day it got serious." 🌲
