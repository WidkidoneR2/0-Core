---
id: 247
date: 2026-09-09
type: future
title: "retire the Faelight label without breaking the machine"
status: in-progress
tags: [naming, migration, infrastructure, zero-core]
---

## WHERE THINGS ARE

This file is the RECORD of the rename, not just its plan -- what was decided, what was
measured, and why. It is long because the reasoning is the point; a record that has to be
assembled from three files is worse than one that has to be scrolled.

    sections, in order -- search the NAME, not a line number:
    Vision
    THE NAME, DECIDED 2026-09-14 -- AND THE TOOLING DECIDED HALF OF IT
    The Problem
    The Solution -- FIVE LAYERS, ONE PER WEEK AT MOST
    TWO RULES THE LEDGER NEEDS BEFORE LAYER 1 -- both learned 2026-09-15
    THE DECISION TEST -- ask these in order, every time
    WHAT NOT TO DO -- each with the reason, so it survives being re-argued
    ⭐ DEVBOX IS THE CENSUS INSTRUMENT
    THE MENTAL MODEL THIS RENAME IS FOR
    LAYERS 1 AND 2 -- DONE 2026-09-15
    THE PATH AUDIT -- DONE 2026-09-15, BEFORE LAYER 3 STARTS
    Success Criteria
    Relationship
    The Rule

READ FIRST, depending on why you are here:

    picking up the work    -> THE SOLUTION (the five layers), then the audit
    about to edit a file   -> TWO RULES THE LEDGER NEEDS, and WHAT NOT TO DO
    naming something new   -> THE NAME, and LAYER 0 -- the freeze is a RULE
    starting Layer 3       -> THE PATH AUDIT. It splits the layer in two, and the
                              order matters: consolidate BEFORE moving.

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

## LAYERS 1 AND 2 -- DONE 2026-09-15

Both landed in one day. The pace rule said one layer per week; that rule was written when
there were fifty tools. There are twenty-six, and the INT-249 census had already done the
analysis both layers existed to perform.

⭐ THE PACE RULE STILL HOLDS FOR LAYER 3. Its teeth were never about VOLUME -- they are
about the state paths, the four inputs that must agree, and the silent empty ledger. That
risk did not shrink with the tool count.

### Layer 1 -- identity in what you read

The three documents disagreed with each other, which was the real finding:

    README    "Faelight Forest 1.0.0"       the oldest name
    ROADMAP   "Zero Core 1.0"               the middle name
    AGENTS    "Codename: Project 0" AND
              "Public name (eventual): Zero Core"   -- contradicting itself

Three files stopped at three different points in the same migration. All three now say
Project 0, and the Layer 0 freeze is written into AGENTS.md as a RULE.

⭐ AND THE README WAS NOT JUST MISNAMED -- IT WAS UNTRUE. Measured:

    claimed 30 tools, then 30 again, then 38     actual: 26
    claimed 125k lines across 239 files          actual: 141,230 across 284
    advertised `build ||| test`                  command not found
    advertised `deploy core`                     the Nix verb, gone
    listed Smithay and wgpu in the stack         nothing links them
    "a self-aware personal computing environment" -- the ecosystem promise
                                                 Project 0 explicitly rejects

The static half is rewritten to say what is true: one shell being made good, a sandbox,
and twenty-four tools that have not had attention since August.

### AND THREE GENERATORS WERE WRITING FICTION

    update_tool_counts()   ##[allow(dead_code)], no callers, AND `for old in 50..60`
                          against a real count of 26 -- it could not have worked
                          if woken. DELETED.
    faelight-docs         counted nine EXTERNAL cargo subcommands as tools this
                          project wrote -> 36
    faelight-release      the same bug, independently -> 30. AND `Err(_) => 0`,
                          which once wrote "0 custom Rust tools" onto the front page

Both counters now use one predicate -- `type = "rust"`, not retired, not cargo -- and
neither returns a number it did not measure.

### Layer 2 -- binaries and crates

THE `faelight` CLI IS RETIRED. Every gate of the decision test answered:

    used             4 invocations since the migration, two verbs tried
    health           `core doctor` -- and the alias `d` is what is typed
    intent           `core intent` -- `ints`, `inta`, `intl`
    profile          `core profile`
    launch           shimmed to omarchy-menu, and named faelight-launcher --
                     a tool already retired
    config           managed ~/.config/faelight/cli.toml -- a file for itself,
                     which never existed
    desktop          no Hyprland bind, no systemd unit, no .desktop file

`core` IS KEPT AND NOT RENAMED. It was never a faelight name, it is not being rewritten,
and Layer 2's own rule forbids renaming a crate for spelling. `0` stays unclaimed.

⭐ AND ONE REPORTED DEFECT WAS NOT ONE. "novashell is unregistered" was wrong: the
registry names BINARIES (`nsh`), the tree holds CRATES (`novashell`), and the crate's own
Cargo.toml says the split was deliberate. Two representations that were never meant to
match were compared and found to disagree.

### ⚠️ DEVIATION, RECORDED RATHER THAN HIDDEN

Layer 1 was meant to be DOCUMENTATION. It required code: a deleted function, two corrected
counters, and a NEW SUBCOMMAND (`faelight-docs readme-index`).

The new verb exists because `readme-generate` writes the catalog AND twenty-six per-tool
READMEs together, and the second half is destructive: it would replace novashell's
hand-written README -- the one explaining why nsh is not the login shell -- with a
metadata stub labelled "active (unregistered) -- uncategorized". That label is the SAME
crate-vs-binary confusion as above, this time made by the TOOL.

The alternatives were: leave a catalog that lists three deleted crates, or run a generator
that destroys real writing. Neither was acceptable, so the smallest third option was
built -- and `readme-generate` now carries a comment saying why it must not be run.

## THE PATH AUDIT -- DONE 2026-09-15, BEFORE LAYER 3 STARTS

The audit this intent calls "a deliverable in its own right". It changes the shape of
Layer 3, so it is recorded in full.

### The inputs are FIVE, not four, and they form TWO CHAINS

    state     XDG_STATE_HOME  ->  FAELIGHT_STATE_DB  ->  HOME
    config    NSH_CONFIG      ->  XDG_CONFIG_HOME    ->  HOME

The intent said four inputs must agree. There is a FIFTH -- XDG_CONFIG_HOME -- and
the important part is not the count but the SPLIT: a redirect that satisfies the
STATE chain leaves the CONFIG chain on its defaults. That is the silent-empty-ledger
mechanism, stated precisely.

Measured today: none of FAELIGHT_STATE_DB, NSH_CONFIG or ZERO_STATE_DB is set. Everything
currently derives from HOME, which is why nothing has broken.

### paths.rs is smaller than feared: SIX functions, FOUR that matter

    runtime_dir           ~/.local/state/faelight        the ledger -- state.db
    faelight_config_dir   ~/.config/faelight             config, profiles, themes
    shell_config          ~/.config/faelight-shell/...   the aliases
    health_status_file    ~/.cache/faelight/health-status

    faelight_dir          ~/0-core/faelight              NOT state -- a REPO directory,
                                                        and this intent says it stays
    sway_config           a suffix check, not a path

### ⚠️ AND THE REAL FINDING: TWENTY-SIX LIVE SITES BYPASS paths.rs

    14   ~/.cache/faelight/health-status    built BY HAND
     3   ~/.cache/faelight/friday.log
     2   ~/.cache/faelight/last-*           prompt state
     4   ~/.config/faelight-shell/...        scripts dir, nl-patterns
     1   ~/.config/faelight                  the DOCTOR's own check
     1   join("faelight")                     faelight-clipboard

**`health_status_file()` EXISTS IN paths.rs AND FOURTEEN CALLERS IGNORE IT.** The
doctor, the daemon, three release tools, faelight-docs, faelight-git, and the shell's
own prompt each build the path themselves. That is INT-195's shape: one owner declared,
fourteen sites deriving it independently.

⭠ WHICH MEANS LAYER 3'S RISK IS NOT THE MOVE. It is that the health cache has no
owner. Move the directory and fourteen readers keep looking at the old place, find
nothing, and `unwrap_or(100)` reports PERFECT HEALTH. The silent failure is already
loaded; the move would merely pull the trigger.

### SO LAYER 3 SPLITS IN TWO, IN THIS ORDER

    3a  CONSOLIDATE -- every site adopts the paths.rs accessor. NOTHING MOVES.
        Testable immediately: the paths resolve IDENTICALLY before and after, so a
        mistake is visible at once rather than after a relocation.

    3b  MOVE -- one change, in one file, with one place to verify.

Doing 3b first is exactly how the silent failure happens. 3a is safe work with no move
and can start whenever; the week-long alias gate belongs to 3b alone.

⚠️ THE TEST FIXTURES COUNT TOO. nsh-test builds a fake forest at
`.local/state/faelight` in five places. Those are CORRECT today -- a fixture should
mirror reality -- and they move with 3b, not before it.

## LAYER 3a -- DONE 2026-09-15/16. CONSOLIDATION ONLY; NOTHING MOVED.

All 26 sites from the audit above now name their paths through paths.rs. The directories are
exactly where they were, which is the point: 3b changes one file, and 3a is what makes that true.

### What was adopted

    14  the health cache        11 adopt read_health(), 3 the path only
     3  friday.log              new: friday_log()
     3  last-exit-status        new: last_exit_status_file() -- 1 writer, 2 readers
     3  shell config            new: shell_config_dir(), shell_scripts_dir(), nl_patterns_file()
     1  the doctor's config     adopted the existing faelight_config_dir()
     1  clipboard history       new: faelight_data_dir(), clipboard_history_file()

### THREE SITES KEPT THEIR OWN LOGIC, DELIBERATELY

read_health() was NOT adopted where a caller had something it does not know about:

    dbus.rs           reads /etc/faelight/HEALTH FIRST -- collapsing would drop that source
    faelight-docs     falls back to the state.db cache -- same
    faelight-update   absent is the string "?" -- the ONE honest reader of the fourteen, showing
                      that it could not read rather than inventing a number

Each took the PATH from paths.rs and kept its own reading. A consolidation that quietly changes
behaviour is not a consolidation.

### AND ONE READER FALLS BACK TO ZERO ON PURPOSE

doctor/mod.rs compares CACHED health against freshly computed health to decide whether the score
moved. For a DELTA, a missing file meaning "no previous score" is right, where 100 would
fabricate one. Preserved and commented rather than normalised away.

### ⚠️ DEVIATION: ONE BEHAVIOUR CHANGE, AGREED BEFORE MAKING IT

`local_data_dir()` did not consult XDG_DATA_HOME while state_home(), xdg_cache_home(), bin_dir()
and shell_config() all consult theirs. On a machine that sets it, paths.rs and the
`dirs::data_local_dir()` faelight-clipboard used returned DIFFERENT DIRECTORIES.

Nil effect here -- the variable is unset. "Nil on this machine" is exactly the reasoning that
left five readers pointed at /etc/faelight for three weeks, so it was fixed rather than inherited
by the accessors added beside it.

### TWO DEFECTS FOUND BY WALKING PAST THEM

Neither was caused by this work and neither is fixed by it:

  - INT-250: five /etc/faelight reads, dead since Omarchy, answering "" and 0. The D-Bus service
    has reported NO ACTIVE INTENT for three weeks.
  - INT-251: the prompt caret is green when the shell has lost track of the exit status.
    `unwrap_or(true)` reads unknown as success. Proven pre-existing by rebuilding the previous
    binary and reproducing it there.

⭐ AND `last-system-rev` GOT NO ACCESSOR. One reader, ZERO writers -- another NixOS-era casualty.
Naming a path nothing writes would dignify it; it belongs to INT-250.

### The shape all of it shares

    health cache      unwrap_or(100)          absent reads as HEALTHY
    /etc/faelight     unwrap_or_default()     absent reads as EMPTY
    the caret         unwrap_or(true)         unknown reads as SUCCESS

Three files, one habit: when the answer is not known, supply the happy one. That is INT-192's
collapse, and the reason Layer 3's own gate exists.

### What 3b now is

One change in one file, with one place to verify -- because every site asks paths.rs and no site
builds its own answer. The week-long alias gate still applies; it was never about the number of
sites.

## LAYER 3b -- THE CLOCK STARTED 2026-09-17

    ~/.local/state/zero  ->  faelight    (symlink, RELATIVE target)
    ~/.config/zero       ->  faelight

Nothing moved. 314M of state -- 303M of it state.db -- is exactly where it was, and BOTH names
resolve to it. The targets are relative so the links survive a home that moves.

### BOTH GATES GREEN ON DAY ONE

    nsh -c 'history'                      through faelight  -> 104 lines
    FAELIGHT_STATE_DIR=../zero nsh ...    through zero      -> 104 lines
    core doctor run through zero          clean, 92%, trend stable

### ⭐ AND STEP 3 WAS ALREADY DEAD, WHICH REMOVED THE LANDMINE

runtime_dir() resolves in four steps, and step 3 is `faelight_dir()/runtime` -- INSIDE THE REPO.
Had that directory existed, the flip could have silently adopted a different state directory
when step 2 stopped matching.

Checked before creating anything: it does not exist. Step 3 cannot fire, so the flip has no
hidden branch.

### What the week is FOR

Not ceremony. The alias must survive ordinary use -- every tool, every session, every reboot --
before a default changes. A path that works once in a test and fails on the third day is exactly
the failure this gate exists to catch.

    FLIP NO EARLIER THAN  2026-09-24

And the flip is one line in paths.rs, because Layer 3a made every site ask that file.

## 2026-09-23 -- THREE MORE RETIREMENTS, AND THE DOC PLAN

The flip's calendar gate opens 2026-09-24. This session did the work that does NOT touch state.

### The decision test, run on the nine crates untouched since the migration

Invocations since 2026-08-26, from shell_history, against Christian's ruling on each:

```text
    RETIRED
      faelight-wallpaper   0 invocations   Omarchy ships 70 themes
      faelight-vault       0               rebuildable from git when the password-manager
                                           project starts -- gen stays, vault does not
      faelight-zone        0               THE BINARY ONLY -- see below
    KEPT, with the reason recorded
      faelight-git      2010   used today
      faelight-gen         3   a self-built password GENERATOR; may go to Omarchy users,
                               which means it needs a name it will keep (Layer 0 applies)
      faelight-ade         3   the terminal Friday is chatted with -- Friday is OPERATIONAL,
                               it has simply had no attention since NixOS
      faelight-vm          8   PROVEN wired: novashell's `vm` builtin drives it (INT-077),
                               and vm_dispatch is commented LIVE AND STAYS
      faelight-insightd    2   possibly a Friday daemon -- undecided
      intent-guard         1   possibly the ledger's -- undecided
      db-browse            1   undecided
      faelight-context     0   possibly Friday
      faelight-clipboard   0   likely retire; INT-221 owns its `pick`/`sk` dependency
```

⚠️ THIS INTENT SAID context, clipboard, gen, vault AND ade WERE UNTOUCHED AND THEREFORE CANDIDATES.
That reasoning does not survive contact with what they are FOR. Friday exists and runs; ade is its
terminal. A crate waiting for attention is not the same as a crate nobody wants.

### ⭐ ZONE IS A LIBRARY WEARING A BINARY'S NAME

`faelight-zone` has 0 invocations as a command, and THREE live importers as a crate:

```text
    engine/src/app/dispatcher.rs:111      domains::zone::run(...)
    engine/src/domains/doctor/aliases.rs  faelight_zone::current_zone(...)   -- the DOCTOR
    engine/src/domains/fetch/mod.rs:78    domains::zone::detect(ctx)
```

So the binary retired and THE CRATE STAYS, with the reason written into tools.toml beside the
entry. Deleting it would have broken `cargo check` -- and "0 invocations" would have been the
evidence that did it.

### What a retirement actually costs, measured

`ship --retire` takes the binary off PATH and backs it up. It does not do the rest, and it SAYS SO:
"the registry still lists X as deployable; deadwood flags it as an orphan until that entry says
retired = true". Five more places, and the suite found the one that was missed:

```text
    registry tools.toml     retired = true, deployable = false
    the crate directory     DELETED for wallpaper and vault (not commented out)
    workspace members       vault was one; wallpaper never was
    devbox census case      deleted with the crate
    ⚠️ ALIASES              6 of them, in ~/.config/faelight-shell/config.nsh -- OUTSIDE the repo
```

★ nsh-test WENT RED AND NAMED IT: deadwood_strict_gate_passes failed on "Dead aliases (6 flagged)",
all six pointing at the tools just retired. The suite found what ship's warning did not cover, and
the config file is not in the repository, so the fix was not part of the commit.

### ⚠️ THE CATALOG COUNTER IS WRONG AGAIN -- FOUND, NOT FIXED

`faelight-docs readme-index` now says "24 active tools (plus 1 retired)". The registry marks THREE
retired. The counter only sees a retirement whose crate is still on disk, so deleting a crate
removes it from BOTH numbers. That is the third counter bug in this file, after the two Layer 1
fixed. The Retired section itself is correct -- faelight-zone is in it.

## THE DOCUMENTATION PLAN -- decided 2026-09-23

A census of every markdown file outside the intent archive: 80 files, dated May to today.

### THREE RULES, AND EVERY FILE FALLS UNDER ONE

```text
    1  docs/ is the SOURCE. docs/public/ is GENERATED -- never hand-edited.
    2  HISTORY IS NEVER REWRITTEN: intents, CHANGELOGs, incidents, the idea bank.
    3  Everything else is CURRENT or DELETED. There is no third category.
```

⭐ AND docs/public/ IS NOT A DUPLICATE, WHICH THE DIFFS PROVED. Each pair differs by 2-5 lines, and
the generator explains exactly why: it copies each doc and STRIPS EVERY LINE CONTAINING "INT-"
unless the line starts with #. The rest is a missing trailing newline. The CHANGELOG records that
docs/public/ was once deleted as a duplicate and regenerated by its own generator -- deleting it
again would repeat a mistake already written down.

### The seven that matter, rewritten AFTER the flip in ONE session

README.md, AGENTS.md, ROADMAP.md, docs/ARCHITECTURE.md, docs/WORKFLOWS.md, docs/NOVASHELL.md,
docs/inventory.md -- then regenerate docs/public/. All saying the same thing: Omarchy is the base,
Project 0 is what is built on it, here is how it works and how to build in it.

⚠️ AND faelight/meta/README.md IS STALE WHOLESALE, WHICH LAYER 1 NEVER COVERED. It names Niri as
the compositor (Hyprland since August), claims 40+ tools (24), lists faelight-term, faelight-bar
and faelight-notify as if they exist, and its roadmap is entirely NixOS-era. Only the two lines
today's retirement made false were removed; fixing three lines would make an untrue document look
maintained.

### ⚠️ FIVE DELETED, TWO RESTORED -- AND THE RECON HAD ALREADY SAID SO

Deleted, each describing a machine that no longer exists: forest-typography, venus-architecture,
ARCHITECTURE-FUTURE, readme-morphwood-draft, labs/dependency-manifest.

RESTORED, because the deletion commit claimed "none is linked or generated" and that was FALSE:

```text
    design-system.md          IS in faelight-docs' public list (main.rs:147), and db-browse and
                              friday-chat both cite it as their palette source
    THEORY_OF_OPERATION.md    linked from README.md:190, PHILOSOPHY.md:359, POLICIES.md:404
```

★ THE RECON PRINTED BOTH FACTS BEFORE THE DELETION RAN. The failure was not missing evidence, it
was not acting on evidence already on screen. Both files join the post-flip rewrite set.

## LAYER 3b -- THE FLIP, ORDERED 2026-09-23 FOR 2026-09-24

The gate is met: the clock started 2026-09-17, five active days of ordinary use, ~6,600 commands,
`d` and `history` exercised daily.

⭐ AND THE ALIAS IS PROVEN BY INODE, NOT BY A LINE COUNT:

```text
    ~/.local/state/faelight/state.db    57:41436   315453440 bytes
    ~/.local/state/zero/state.db        57:41436   315453440 bytes
```

Same device, same inode, same size -- one file under both names. A count of history lines could
have agreed while pointing at two different databases; this cannot.

### The five steps, and what each is for

```text
    1  BASELINE      d, git status, nsh-test 198/198 -- never start on an unknown tree
    2  RECON         paths.rs step 2 and every runtime_dir() caller, READ-ONLY
    3  THE FLIP      state_home().join("faelight") -> join("zero"). ONE line, ONE commit,
                     nothing else in it
    4  BOTH DOORS    nsh -c, a PTY session, d, history -- IMMEDIATELY.
                     ⚠️ IF ANY LOOKS EMPTY, REVERT THE COMMIT. Do not fix forward on state.
    5  DEPLOY        cargo build, ship, nsh-test, then exec /home/christian/.local/bin/nsh
```

ROLLBACK, DECIDED BEFORE IT IS NEEDED: `git revert` the flip commit, then `ship`. The symlinks stay
either way, so both names keep resolving and nothing is stranded.

NOT part of the flip: the directory rename (INT-252 -- source one week, state the next), the doc
rewrite, and any crate rename.

## LAYER 3b -- THE RECON, 2026-09-23. THE FLIP IS FOUR FILES, NOT ONE LINE.

Step 2 of the ordered plan, run the evening before. It changed the plan, which is what recon is for.

### runtime_dir() resolves in FOUR steps and step 2 is the one that wins

```text
    1  $FAELIGHT_STATE_DIR          explicit override; fsh-test isolates per-case state with it
    2  state_home()/faelight        EXISTS, so it wins today                     <- line 165
    3  faelight_dir()/runtime       legacy; checked 2026-09-17: DOES NOT EXIST
    4  state_home()/faelight        fresh install, no repo required
```

The function's own comment states the design: "there is deliberately no window where the code
points somewhere the data is not."

### ⚠️ FOUR SITES, NOT ONE. THE RECON FOUND THREE MORE.

```text
    paths.rs:165    state_home().join("faelight")      runtime_dir -- the state chain
    paths.rs:354    config_dir().join("faelight")      faelight_config_dir -- the CONFIG chain,
                                                       a second edit, aliased the same way
    doctor probes/files.rs:70   state_home().join("faelight")   ⚠️ ITS OWN COPY. The doctor does
                                NOT call runtime_dir(). Flip paths.rs alone and the doctor keeps
                                reading the old name -- one site that escaped Layer 3a
    nsh-test main.rs   SEVEN fixture sites building .local/state/faelight (231, 232, 234, 587,
                       683, 687, 1019)
```

★ AND THE FIXTURES ARE THE SUBTLE ONE. Flip the code and leave them, and they still pass THROUGH
THE SYMLINK -- so the suite stays green either way, which means it is not testing the flip at all.
A green suite that cannot go red is the same defect as a check that cannot fail.

`shell_config_dir()` keeps `faelight-shell` and is CORRECTLY out of scope: that directory has no
zero alias and belongs to INT-252.

### DECISION, Christian 2026-09-23: (b) -- THE DIRECTORY MOVES, NOT JUST THE CODE

```text
    (a)  flip the code, keep the symlink    the real directory stays named faelight forever,
                                            reached through a link. One line, reversible.
    (b)  flip the code AND rename the       the data takes the new name; the symlink REVERSES
         directory                          (faelight -> zero) so old references still work
```

(b) chosen. (a) would have left the label alive on disk indefinitely, which is the thing this
intent exists to end.

### THE ORDER, AND WHY IT IS FIVE STEPS RATHER THAN ONE

Measured before deciding: ONE process holds state.db -- nsh pid 9099, four handles. The daemon is
INACTIVE. Nothing else is attached. And a rename within one filesystem does not change the inode,
so an open handle follows the data to the new name.

```text
    1  BASELINE      d, git status, nsh-test 198/198
    2  CODE ONLY     the four files above, ONE commit, NOT pushed yet.
                     The code says zero, the data is still faelight, and the EXISTING
                     symlink resolves it -- so this step is safe on its own.
    3  BOTH DOORS    nsh -c, a PTY session, d, history. Proves the code change alone.
    4  THE DATA      from BASH, not nsh: rm the zero symlink, mv faelight zero,
                     ln -s zero faelight. Old references keep working in the other direction.
    5  DEPLOY        both doors again, ship, nsh-test, exec /home/christian/.local/bin/nsh
```

ROLLBACK AT EACH POINT: after step 2, `git revert`. After step 4, reverse the rename. A symlink is
present in one direction or the other throughout, so BOTH names resolve at every moment.

⚠️ IF ANY DOOR LOOKS EMPTY, STOP AND REVERSE. Do not fix forward on state.

## Success Criteria

- [x] LAYER 0 landed: the freeze is written into AGENTS.md or CONVENTIONS.md as a rule, not a
      plan. Nothing new is named faelight from that commit onward
      <!-- evidence: 2026-09-15. AGENTS.md section "LAYER 0 -- THE FREEZE. THIS IS A RULE, NOT A PLAN." -->
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
- [x] `faelight-docs` cannot resurrect a retired tool. Proven by running it after a retirement and
      confirming the catalog does not list it
      <!-- evidence: 2026-09-15. gather_all() reads rust-tools/*/Cargo.toml from DISK, so a deleted crate cannot appear. Proven by running readme-index after three retirements: faelight-fm, faelight-glog and faelight are all absent from the catalog. -->
- [ ] Layer 3 is NOT started until layers 0-2 are done and a path audit lists every hardcoded
      reference. The audit is a deliverable in its own right
      <!-- the audit is DONE 2026-09-15 -- see THE PATH AUDIT above: 6 functions in paths.rs, 26 live sites outside it, 14 of them the health cache. Layers 0-2 are also done. The gate is open for 3a. -->
- [x] The state alias runs for A FULL WEEK with `core doctor` and `nsh history` green before any
      code default changes. Evidence: the dates
      <!-- evidence: 2026-09-24. Alias created 2026-09-17, seven days of ordinary use. Measured today BEFORE any code default changed: d 92%, 25/28, 0 failed, Zero Alias probe green (state and config aliases resolve to the faelight directories); nsh -c history -> 104 lines; state.db is ONE inode under both names (59:41436, 319037440 bytes); sole holder nsh pid 123517. Baseline HEAD e9eb743a, pushed, tree clean, nsh-test 200/200. -->
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
