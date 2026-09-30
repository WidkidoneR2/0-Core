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

## LAYER 3b -- DONE 2026-09-24. THE DATA IS NAMED zero.

```text
    ~/.local/state/zero      REAL directory   state.db 59:41436, 319037440 bytes
    ~/.local/state/faelight  -> zero          kept so every old reference still resolves
    ~/.config/zero           REAL directory
    ~/.config/faelight       -> zero
```

Proven after the move: `d` 0 failed, Zero Alias reading `real: state=zero, config=zero`; nsh-test
200/200 on the committed tree; `nsh -c history` 104 lines; a fresh `exec nsh` opened state.db
through zero and showed the whole day. Pushed.

### The commits

```text
    d89e3638   zero_alias probe rewritten -- landed and deployed BEFORE the flip
    4cca17ad   full-week alias gate ticked with evidence
    7fdc90a0   the flip: runtime_dir and faelight_config_dir name zero; seven nsh-test
               fixture lines; the paths contract test; two lines of runtime_dir's doc comment
```

The data move is not a commit. It is on disk, and this section is its record.

### ⚠️ THE RECON'S FOURTH SITE WAS A CHECK THAT WOULD HAVE STOPPED CHECKING

doctor probes/files.rs:70 was not a copy of the state path. It was zero_alias(), the probe
watching this migration. It asserted ONE direction -- zero links to faelight -- and read the config
side through faelight_config_dir(). Flipping paths.rs:354 would have made that side compare
`~/.config/zero` with itself: a check that cannot fail. Moving the data would have turned the state
side red on the correct end state.

Rewritten around the invariant that holds at every moment of the migration: exactly one name is a
real directory, the other links to it, both resolve to the same place. Direction-agnostic, so it
landed BEFORE the flip as its own commit and watched every step after. Its pass message names the
real directory, so the flip showed in `d` as `real: faelight` becoming `real: zero`.

It also stopped reporting UNREADABLE as ABSENT -- this intent's own criterion, applied in the one
place that was in reach today. Three class-of-failure tests: both directions pass; two real
directories, two links, dangling, absent and resolves-elsewhere are each named; unreadable is not
absent.

### ⚠️ THE MOVE WAS NOT rm / mv / ln

The ordered plan above said: rm the zero link, mv faelight zero, ln -s zero faelight. That leaves a
window where zero does not exist -- and the deployed code asks for zero. A tool starting inside it
falls through runtime_dir() to "fresh install" and can create an empty zero; the mv then lands
faelight INSIDE it as zero/faelight. Split state: the silent empty ledger.

Replaced by an atomic swap, per chain:

```text
    mv -T --exchange faelight zero    one renameat2(RENAME_EXCHANGE); zero exists at every instant
    ln -sfT zero faelight             repoint the old name; only the unused name is briefly wrong
```

⚠️ -T IS REQUIRED. Without it mv follows the zero link to a directory, treats it as a target
directory, and tries to exchange faelight with zero/faelight. The first attempt failed exactly that
way (coreutils 9.11: "cannot exchange ... and .../zero/faelight"). Nothing moved -- set -eu stopped
at the first mv. The retry REHEARSED the exact commands in a mktemp directory and refused unless the
rehearsal produced the right layout.

### ⚠️ A GATE THAT PRINTS IS NOT A GATE

`cargo test ... | grep "test result"` matches FAILED as happily as ok.
paths::tests::test_numbered_gravity went red -- "unexpected runtime_dir:
/home/christian/.local/state/zero" -- and the chain carried on into ship. No harm: the data had not
moved and zero already resolved to it. The test accepted `runtime` or `faelight`; after the flip
runtime_dir can only return runtime or zero, so it now accepts those. Keeping faelight would have
been a branch that cannot fail.

★ THE RED TEST WAS THE FIRST RUNTIME PROOF THAT THE CODE ASKS FOR zero. The doors could not show it,
because both names reached one inode. From then on every gate matched the PASSING string
(`ok. 4 passed`, `200 / 200 passed`, ` 0 failed`), so a failure stops the chain instead of scrolling
past.

### Two smaller things the record needs

  - 7fdc90a0's nsh-test diff is not byte-identical to the file that was built and run 200/200. A
    formatter -- believed to be the pre-commit hook, NOT confirmed -- collapsed the fixture's
    write(...) onto one line once `zero` made it fit. Meaning unchanged, proven by the diff; the
    final ship rebuilt from the committed tree and ran 200/200 again.
  - The move was reversed once by mistake -- the undo block sat directly under the success
    criteria -- and redone. Lossless both ways. An undo command belongs in a reply only after it is
    needed.

### ⚠️ CORRECTION TO "LAYER 3b -- THE RECON" ABOVE

It says the fixtures are the subtle one because the suite "stays green either way, which means it
is not testing the flip at all". The conclusion holds for a different reason: no nsh-test case calls
runtime_dir() against the fixture. Lines 231-234 BUILD the fake forest, 587 checks the scaffolding,
687 and 1019 test tilde expansion. They MIRROR the layout and were never a test of it. What tested
the flip was the probe, the paths contract test and the doors.

### OPEN -- NONE URGENT

```text
    faelight links       ~/.local/state/faelight and ~/.config/faelight stay until Layer 4 has
                         swept every non-Rust reference: scripts, Hyprland, systemd, config.nsh
    third chain          faelight_data_dir() = ~/.local/share/faelight (clipboard history).
                         No alias exists; not flipped
    FAELIGHT_STATE_DIR   the override env var still carries the old name (-> ZERO_*)
    runtime_dir comment  lines 156-158 are INT-061's migration narrative, stale before today
    startup cwd          a fresh nsh starts INSIDE the state directory. Not caused by the flip --
                         every changed path resolves to the same physical directory. Its own
                         intent, not this one
    the docs             the seven-document rewrite, per THE DOCUMENTATION PLAN
```

## 2026-09-24, LATER -- THE RENAME IS A PROJECT. RULINGS, COMMITS, AND THE NEW CENSUS

### The rulings, Christian 2026-09-24

```text
    "Omarchy is my operating system. Anything else that has faelight is mine.
     I am changing it to Project 0 (Zero)."
    "changing faelight to 0 or to zero is important to me" -- a new project, for good.
```

What that supersedes, recorded here rather than deleted above:

```text
    Vision         "DELIBERATELY NOT URGENT"             -> the rename is a first-class goal
    Layer 2        "DO NOT RENAME A CRATE FOR SPELLING"  -> every faelight-* crate that is KEPT
                                                            becomes zero-*, one at a time
    Relationship   "PRIORITY: BELOW the shell, always"   -> OPEN. Not yet ruled: how the rename
                                                            shares time with the shell before
                                                            the October evaluation
    forest         DROPPED from Project 0's own screens and labels. This intent's own mental
                   model says "Omarchy provides the forest" -- Project 0 uses plain words.
                   Stored data keys that contain it (domain = 'forest' in state.db) are a
                   DATA MIGRATION, never a text edit.
    versions       No version number in a title: "Project 0 1.0.0" read as one number. The
                   same ruling faelight-release made for the README on 2026-09-15.
```

UNCHANGED: history is never rewritten; ~/0-core and WidkidoneR2/0-Core stay; one crate at a time,
never a repository-wide sweep; anything touching state follows Layer 3's rules.

### What landed -- each seen RED before it went green

```text
    7c446407 81f769c5            INT-261 COMPLETE: new terminals open in ~/0-core. The last_dir
                                 restore is retired. The OPEN item "startup cwd" below is DONE.
    9f5f3c3d                     No intent template defaults to the faelight tag. `inta` offered
                                 it on every new intent -- a Layer 0 breach. template_tests.
    2ed55e4f e4d7c773 ec64ce16   Zero Core -> Project 0 in 12 display strings, and the guard.
    954718fd                     Version dropped from four titles; the guard refuses the run-on.
```

### THE GUARD -- nsh-test no_retired_display_name_in_printed_strings

Reads every string literal under rust-tools/ and engine/ and fails naming each file:line that
prints a retired form. It found a 12th site the hand-written census had missed, and it goes red
on a planted probe. A form joins its list only when its pass is FINISHED, so the suite is green at
every commit and red the moment one comes back. forest and faelight join it as their passes land.

### WARNING -- A COMMIT MESSAGE THAT PROMISED WHAT WAS NOT THERE

2ed55e4f says the guard exists. The step that added it had not been run: the suite read 201/201,
not 202. The guard landed in e4d7c773, whose message was corrected before it was pushed -- the
pre-push gate had refused a red tree, which is how the gap stayed local. History is not
rewritten; this paragraph is the record.

### THE CENSUS, 2026-09-24 -- the new starting line

```text
    4,077 lines in 515 tracked files still say faelight
      history (never rewritten)   2,013   completed intents 1,444, CHANGELOGs 569
      live                        2,064   Rust 1,083, markdown 410, config 142,
                                          live intents 265, Cargo 101, docs/public 63
    646 of 722 tracked files sit under faelight/ (INT-252)
    15 faelight-* crate directories
    outside the repo  ~/.config/faelight-shell, ~/.local/share/faelight and ~/.cache/faelight
                      are REAL directories; state and config are links -> zero
    string literals   forest 479, faelight 311, Zero Core 0 (guarded)
```

The live lines come from roughly twenty NAMES -- the directory, the crates, a handful of
generators -- not from 2,000 separate edits.

### Found, not fixed -- each its own discussion

```text
    checkpoint/mod.rs:573   runs `sudo btrfs subvolume list` itself. Automation plus sudo is the
                            2025-12-14 lockout class. Nothing changed -- decide first.
    System Services probe   reads faelight-session.target, likely a NixOS-era unit on Omarchy.
                            Reports Unknown honestly.
    deps/mod.rs:308         categorize_tool groups by the faelight- PREFIX. The first crate
                            rename must change it in the same commit, or every zero-* tool
                            silently lands in "Utilities".
    ~/0-core/.config        a tracked symlink to dotfiles/helix/.config -- dead since INT-149
                            and INT-107.
    the inta prompt         prompt() falls back with unwrap_or_default, so an interrupted Tags
                            prompt may file a real intent rather than cancel. UNVERIFIED.
```

### Next, in order

```text
    1  forest labels -- through the guard, same method as Zero Core
    2  the sayings -- Christian rewrites them; they are his voice
    3  faelight-release (Forest DNA) and the README; the faelight-docs footer and command
       reference; deadwood's own header
    4  the outside directories, one per session, alias first
    5  crates -- decision test, then one at a time, faelight-core last
    6  INT-252 -- the directory
    7  the compatibility links
```

## 2026-09-24, EVENING -- FOREST IS FOUR LAYERS, NOT ONE. THE FIRST PASS.

### The census, measured

534 string literals in 289 .rs files contain forest, any case. The earlier 479 used a different
definition; the two are not comparable line for line.

```text
    DATA    115   SQL: forest_* table names and domain = 'forest' keys
    TOKEN    72   bare words: command names, D-Bus names, stored keys, nl.rs vocabulary
    PATH     26
    NAME      9   Faelight Forest
    PROSE   234   154 in the engine: Friday sentences, help text, the sayings
    LABEL    78
    identifiers   35 distinct, 287 uses (ForestDb alone 164) -- code names, crate-rename territory
```

THE BUCKETS ARE HEURISTIC. Two help strings were sorted DATA and PATH only because they mention
domain and state.db; one was sorted PROSE only for having five words. Sort by meaning, check by line.

### The four layers

```text
    1  SCHEMA      ten forest_* tables in state.db, counted read-only 2026-09-24:
                   forest_events 94,558 rows, forest_predictions 21,614, forest_insights 226,
                   forest_events_v2 6, forest_memory 4, forest_goals 1, forest_mandates 1,
                   forest_plans 1, forest_tradeoffs 1, forest_strategies 0.
                   A DATA MIGRATION under Layer 3's rules. Never a text edit.
    2  CONTRACTS   D-Bus org.faelight.Forest.*, daemon GetForestContext and ForestContext,
                   commands forest-stats cp-forest mv-forest forest-ade, the --forest flags,
                   the FOREST_ prefix, thirteen nl.rs words. Alias first, like a crate.
    3  PROSE       the sayings are Christian's (item 2); the rest is sorted per crate.
    4  LABELS      item 1 proper.
```

Layers 1 and 2 have no line in "Next, in order". Proposed as their own items -- NOT YET RULED.

### The vocabulary, approved by Christian 2026-09-24

```text
    forest meaning the whole project        ->  Project 0
    forest meaning state, health, events    ->  the word is dropped
    forest meaning the repo                 ->  repo
    the tree emoji                          stays until the sayings pass
```

### Pass 1 -- cf7db6b9

26 strings in novashell's help text: cheatsheet_tui.rs, completion.rs, registry.rs, schema.rs. Every
anchor was preflighted -- once in its file, on its census line -- before anything was written, and
fpatch verified each on disk. novashell 221 passed; nsh-test 202/202 after ship. One forest literal
remains in those files: the forest-stats command name, a contract.

It was applied in two halves. The first 18 were the census LABEL bucket; fpatch's own context then
printed a help string the bucket had missed, and eight more were found. THE COMMIT WAITED until the
four files were finished, so its message is true.

### The guard cannot take forest yet

no_retired_display_name_in_printed_strings matches with lit.contains(name), case-sensitive. A plain
forest entry would go red on the SQL literals and stored keys forever. It also reads one line at a
time, so a literal spanning lines is seen only on its first line, and it skips only whole-line
comments. How forest joins is decided when the label pass is finished, not before.

### Found, not fixed

```text
    forest_memory            a table with 4 rows that no code reads
    forest_lesson,
    forest_operations        named in code, no such table
    doctor section header    prints Forest, but not from a literal the guard can see;
                             faelight-doctor probes/mod.rs has a forest identifier. Next read.
    welcome banner           prints a forest label and the growing-fast saying; source not located
    cheatsheet_tui.rs        Reload fsh configuration (old shell name); Nix store operations (NixOS)
    completion.rs:810        faelight-git helper -- the faelight pass
```

## 2026-09-24, EVENING -- PASS 2: THE DOCTOR, AND A PAYLOAD THAT RAN IN HALF

### 2968a60a -- the doctor section Forest becomes Internals

The doctor's section titles are DATA, in faelight/registry/doctor/checks.toml -- six checks under
section = "\U0001F4CB Forest" -- not Rust. That is why neither the census nor the guard saw them. The
panel was already titled Project 0 by the Zero Core pass, so the section is named for what it holds:
the ledger, deadwood, config, the alias probe, schema and Friday. Name approved by Christian.

THE NAME IS A CONTRACT. novashell health_tui.rs reads the doctor's PRINTED output and finds sections
by substring (line 110, is_section; 191-192, parse_section_header). Renaming the registry alone would
have left the TUI unable to see the section -- a lookup that stops matching and answers with nothing.
Registry and parser changed in ONE commit. Proven: d prints Internals with its six checks, and the
health TUI shows Internals as its own section.

### a102eca3 -- five engine doctor strings

The rebuild guide (reconstruct Project 0 from first principles; what was happening before any
failure; Clone the repo), the integrity notification (Integrity below 80%) and the stable line
(Stable). The engine crate has TWO unit tests and neither covers these strings. What covers them is
the guard, which reads engine/, and what the doctor prints.

### THE GUARD CANNOT SEE THE REGISTRY

It reads .rs files under rust-tools/ and engine/. checks.toml is outside it, so nothing stops Forest
coming back as a section title. A registry-to-parser contract test is the right guard -- see below.

### A PAYLOAD THAT RAN IN HALF -- AND THE CLASS FIX

A reply split one base64 argument across two lines. Line one decoded a partial script; line two ran
as a command ("File name too long"). It stopped at a syntax error before any write -- by the luck of
where the cut fell. A cut at a statement boundary would have run half a pass.

THE RULE FROM THIS SESSION ON: an edit script does nothing at top level except define, and its ONE
call to act is the LAST line. A payload cut short either fails to compile, or defines main() and
never calls it. Proven on a stand-in tree: all 1,029 possible truncations of the pass 2 payload
wrote nothing to any target; only the whole payload patched. Importing fpatch writes the Python
bytecode cache -- that is not a target.

And the test runs after the failed step passed on the UNCHANGED tree. A green run after a failed
step proves nothing about the step; the empty status line is what showed nothing had been written.

### Found, not fixed

```text
    health_tui          recognises 5 of the doctor's 8 sections. Boot, System State and Runtime are
                        missing from its hand-written name and emoji lists, so each folds into the
                        section above it. A test that every registry section is one the parser
                        recognises would be red on three today. PROPOSED, not built.
    dispatcher.rs:56    a failed version read prints 13.0.0 -- an invented version, the class INT-250
                        found as v14.0.0
    version labels      Forest: at dispatcher.rs:57 and commands/mod.rs:12585 -- the next label pass
    welcome banner      built in novashell main.rs near 3472-3478, found by its clean-and-pushed
                        text; its forest label and shell percentage are read next. session.rs holds
                        three sayings (247, 255, 443) for item 2
```

## 2026-09-24, NIGHT -- PASSES 3 AND 4, TWO RULINGS, AND THE ORDER FOR NEXT SESSION

### a4be9088 -- the banner and the identity label

The welcome banner's forest labelled the HEALTH number; it now says health. Proven by reading the
chain: ONE writer (core doctor run -- d is an alias for it -- at engine doctor/mod.rs:840) and one
reader (banner -> core_integration::health -> paths::read_health). The number is a SNAPSHOT of the
last doctor run. That is why the banner said 84 while d said 92: a later run on a dirty tree wrote
84, and a doctor run rewrote it 84 -> 85 while this was measured. Lag, not a second measure.

### de8eabbb -- one version reader, and a test that was passing on an invented number

state.db holds NO forest version key -- a read-only query returned []. So core version printed an
invented "Forest: 13.0.0" on every run. paths::read_version() now reads meta/VERSION, the one owner,
and returns None when it cannot. core version and the identity label both use it, so they cannot
disagree, and an unreadable version prints ? rather than a number. Three tests pin missing and blank
to None. core version now prints Project 0: 1.0.0.

THE RED THAT PROVED IT: nsh-test and_chain_fsh_builtin expected "3.0.0" -- core's version when the
test was written -- and kept passing after core moved to 3.2.17 only because "13.0.0" contains it.
Two stale numbers vouching for each other. It now asserts the SHAPE: both sides ran, in order.

AND faelight-core's doc-tests had been failing unseen. Two doc tables in paths.rs (zero_state_dir,
focus_file) were indented four spaces, which rustdoc compiles as Rust. Nothing ran them: d checks
cargo doc, which does not run doc-tests, and the pre-push hook runs nsh-test. Fenced as text: 2
failed -> 0.

### RULINGS, Christian 2026-09-24

```text
    SCHEMA and CONTRACTS are their own items, to be filed with inta.
      schema     ten forest_* tables (~116k rows) renamed with ALTER TABLE, every SQL string in the
                 same commit, rollback rehearsed first -- Layer 3's rules
      contracts  D-Bus org.faelight.Forest.*, forest-stats cp-forest mv-forest forest-ade, the
                 --forest flags, the FOREST_ prefix, the nl.rs vocabulary -- alias first
    PATHS say Project 0 or zero, not faelight: the paths.rs accessor names, the three outside
      directories, and the repo directory (INT-252)
    FIX AND IMPROVE ON THE WAY: what the rename finds broken is fixed, so the brand starts clean --
      as the version reader and the doc-tests were today
    THE FINISH LINE: see Success Criteria
```

### NEXT SESSION, IN ORDER

```text
    1  re-run the census: forest and faelight counts by kind -- the new starting line
    2  inta: file SCHEMA and CONTRACTS as their own intents
    3  paths.rs accessor names -- faelight_dir, faelight_config_dir, faelight_data_dir -- to zero
       names; the compiler finds every caller
    4  the outside directories, ONE PER SESSION, alias first: ~/.cache/faelight (the health cache,
       regenerable) first, ~/.local/share/faelight next, ~/.config/faelight-shell (the aliases) last
    5  the remaining forest labels, about 60, one engine domain at a time
    6  the sayings (Christian's voice) and Forest DNA / the README (item 3)
```

### Found, not fixed

```text
    intelligence name   core version prints intelligence v53 (Forest Mind) -- a name STORED in
                        state.db; it belongs to the schema item
    guard blind spots   reads only .rs under rust-tools/ and engine/, one line at a time; TOML
                        registries and multi-line strings are outside it
    health_tui          still recognises 5 of the 8 doctor sections (pass 2)
    one unseen value    tail cut off the Health line of the measuring doctor run, so what it
                        printed on that run was not seen -- the cache change 84 -> 85 was
```

## 2026-09-24, NIGHT -- THE NAMING MAP, RULED BY CHRISTIAN

Every name the rename touches and what it becomes. Rows marked PROPOSED are recorded, not ruled.

```text
    WHAT                         BECOMES
    screens, docs, spoken        Project 0
    repo root ~/0-core           unchanged
    faelight-* tools/crates      zero-*, ONLY if they survive the decision test; otherwise retired.
                                 Tools already zero-* (zero-gate) stay as they are.
    FAELIGHT_* / FOREST_* env    ZERO_*
    org.faelight.Forest.*        org.zero.*
    ~/.cache/faelight            ~/.cache/zero
    ~/.local/share/faelight      ~/.local/share/zero
    ~/.config/faelight-shell     ~/.config/nsh -- NovaShell and DevBox config (devshell is in the shell)
    forest_* tables              plain words; zero_* only where a plain name collides (schema intent)
    one Project 0 command        0
    ~/0-core/faelight/           ~/0-core/zero/ -- PROPOSED, one-to-one under INT-252, not yet ruled
    FINISH LINE                  no live faelight or forest anywhere; history exempt
    METHOD (PROPOSED)            rename what is kept, retire what is not, rewrite pieces whose design
                                 is the bug, turn invented defaults into honest unknowns
```

### What the map overrides, recorded here rather than edited above

```text
    Layer 2          "0 stays unclaimed"            -> 0 is the one Project 0 command
    PATHS ruling     "Project 0 or zero"            -> ~/.config/faelight-shell is the exception:
                                                       it becomes ~/.config/nsh, because NovaShell
                                                       keeps its own name (ruled 2026-09-21) and
                                                       DevBox config lives inside the shell
```

## 2026-09-24, NIGHT -- THE CENSUS: THE NEW STARTING LINE

Measured at de954f49, tree clean, read-only. The definitions differ from the two earlier censuses,
so this is a new starting line, not a delta against 4,077 or 534.

```text
    LINES containing the name, any case, in tracked files
      faelight  history  2,013 in 212 files   complete 1,115, CHANGELOG 569, cancelled 145,
                                              decisions 134, incidents 34, philosophy 16
                live     2,104 in 303 files   rust 1,086, markdown 410, in-progress 164,
                                              future 124, config 102, cargo 101, docs/public 63,
                                              other 22, scripts 18, intents top 10, planned 4
      forest    history    841 in 157 files   complete 500, CHANGELOG 181, cancelled 83,
                                              decisions 73, philosophy 4
                live     1,513 in 223 files   rust 1,050, markdown 203, future 67, in-progress 63,
                                              docs/public 58, config 36, other 17, cargo 13,
                                              intents top 3, planned 2, scripts 1

    RUST, comments skipped
      faelight  439 literals in 78 files      D-Bus 7, other 432
                identifiers 12 distinct, 408 uses -- faelight_core alone 355
      forest    496 literals in 78 files      schema-shaped 126, D-Bus 7, other 363
                identifiers 34 distinct, 285 uses -- ForestDb alone 164
      Zero Core 0 literals (guarded)

    LARGEST LIVE FILES                        faelight  forest
      novashell commands/mod.rs                     98     190
      nsh-test main.rs                              69      38
      faelight-core paths.rs                        50       -
      faelight-daemon dbus.rs                       31      54
      engine domains/friday                          -      48
      forest literals by area: novashell 85, friday 48, faelight-daemon 34, strategy 33, partner 26

    TREE
      722 tracked files, 646 under faelight/ (INT-252)
      15 faelight-* crate directories, and all 15 package names carry it; zero-gate is the one zero-*

    OUTSIDE THE REPO
      ~/.local/state, ~/.config      zero is REAL, faelight -> zero
      ~/.cache, ~/.local/share       faelight is REAL, no zero yet
      ~/.config/faelight-shell       REAL, no nsh yet
      /etc/faelight                  absent
```

### How it was counted

  - HISTORY is intents in complete/, decisions/, philosophy/, cancelled/ and incidents/, plus every
    CHANGELOG -- rule 1 above and rule 2 of the documentation plan. The first run counted incidents
    as live; corrected before this line was recorded.
  - Paths come from git ls-files -z. Without -z git quotes non-ASCII paths, and the incident file
    whose name holds an em dash was skipped. The census printed it under NOT COUNTED rather than
    reading it as empty, which is how it was caught.
  - INT-247 is itself the largest live faelight file (164 lines). It stays live until it completes;
    that count is the record of the rename, not its debt.

### Fixed on the way

```text
    898c3d19   ~/0-core/.config removed. A tracked link (mode 120000) to dotfiles/helix/.config,
               dangling since helix went (INT-149, before Omarchy). Referenced only by INT-149,
               which is history, and by this file. A dangling link already reads as absent, so
               removing it could not change what any reader sees.
```

### Found, not fixed

```text
    FOREST_TEST=hello   set in the live shell. No startup file sets it (bashrc, bash_profile,
                        profile, config.nsh, environment.d, hypr) and no repo code reads it --
                        fsearch finds only nsh-test's forest_test function. Being traced.
```

## 2026-09-24, NIGHT -- WHERE THIS SESSION STOPPED, AND THE ORDER NEXT

READ THIS SECTION FIRST WHEN PICKING UP. It supersedes "NEXT SESSION, IN ORDER" in the PASSES 3 AND 4
section above.

### Done this session

```text
    de954f49   the naming map, ruled by Christian
    898c3d19   ~/0-core/.config removed -- dead link to dotfiles/helix
    382dfd8e   the census, the new starting line
    (data)     FOREST_TEST removed from shell_persist. It came from a test export on 2026-04-09
               and nsh re-exported it into every shell for five and a half months. Traced through
               the process tree (no ancestor carried it, nor nsh's own start environment) to
               shell_persist rowid 1; the startup restore was read before the row was deleted.
               RESOLVES the FOREST_TEST line under the census's "Found, not fixed"
    683a890b   INT-262 filed -- persist has no inverse and reports success it never checked.
               STARTED, THEN PARKED behind this intent. Its next step: how nsh-test's
               repl::run_repl starts the shell, and whether it already isolates state
```

### THE ORDER, ruled by Christian 2026-09-24 -- what he SEES first

```text
    1  System Services probe   d prints "could not read faelight-session.target" on every run:
                               a NixOS-era unit that does not exist on Omarchy, and d's only
                               Unknown. Recon what it is for, then fix or retire it
    2  forest labels           the remaining ~60, one engine area at a time, through the guard
    3  paths.rs accessors      faelight_dir, faelight_config_dir, faelight_data_dir to zero
                               names; the compiler finds every caller
    4  outside directories     ONE PER SESSION, alias first: ~/.cache/faelight -> zero first,
                               ~/.local/share/faelight -> zero next, ~/.config/faelight-shell
                               -> ~/.config/nsh last (it holds the aliases)
    5  inta                    SCHEMA and CONTRACTS as their own intents -- any time, a minute each
    6  the words               the sayings (his voice, the tree emoji with them), then README and
                               the seven-document rewrite
    7  the structure           crates through the decision test, faelight-core last; INT-252
                               the directory; the compatibility links last of all
```

### Still visible in d after this session

```text
    faelight-session.target    item 1
    faelight-sandbox deployed  a crate name -- goes when the crate is renamed or retired (item 7)
    the tree emoji on Friday   the sayings (item 6)
```

## 2026-09-24, NIGHT -- ITEM 1 DONE; ITEM 2 PASSES 5 AND 6

```text
    2d376e42   ITEM 1 DONE: System Services retired. d shows no Unknown and no
               faelight-session.target. INT-237 ruled on 2026-09-02 that Project 0 runs no user
               services on Omarchy, so the probe could only ever answer Unknown. 27 checks now;
               three count tests pin 27 checks, 27 probes and 25 judging
    cfd5961d   PASS 5: 27 NovaShell lines. forest builtin/script -> shell builtin/script, bash's
               own wording. The type test in nsh-test changed in the same commit as the text it
               checks
    c6b9a934   PASS 6: 33 lines across nine engine domains and deadwood. Deadwood's header also
               lost "Faelight"
```

### Left on purpose, and where each goes

```text
    the sayings     every personified line -- "The forest advises. You decide.", "...The human
                    decides.", "The forest observes.", "The forest remembers", "The Living
                    Forest", "sheds dead wood", "The forest is tidy", the banner's "growing
                    fast". Christian's voice: item 6
    box headers     Forest Health and the Forest box in NovaShell, Forest Narrative in the
                    engine -- the whole box is read before its title changes width
    spine terms     "forest value pipeline", "forest value verb": refusal reasons the migrate
                    audit may compare as text. Recon first
    contracts       the nl.rs phrases, the forest theme name, Forest ADE: the contracts intent
```

### Found on the way

```text
    ~/.local/share/forest-trash   a sixth outside directory, named at mod.rs:4016 and :4092.
                                  Joins item 4
    "handled natively by fsh"     type still prints the shell's pre-NovaShell name
    a check that hid its answer   cargo test piped to tail -5 showed only the doc-test line
                                  (0 tests) and scrolled the unit-test result away. Test runs
                                  now print every "test result" line through a filter
```

## 2026-09-24, NIGHT -- ITEM 2 PASSES 7 AND 8

```text
    82efb936   PASS 7: 70 lines -- strategy, partner, reaction, journal, autobiography, events,
               core --help, teach, faelight-docs, faelight-git, faelight-daemon
    618f3ec0   PASS 8: 43 lines -- Friday, autonomy, goals, status, tradeoffs, weight engine,
               core --help, db-browse, gen, insightd, update, daemon, deps, docs, engines,
               integrity, predict, stress, synthesis, friday-chat, knowledge
```

This session: item 1 done, and 173 display lines across passes 5 to 8 say Project 0, repo or plain
words. Every pass refused at least once on text it also found elsewhere, and each time the other
site was folded in rather than left behind.

### Fixed on the way

```text
    faelight-docs toolgen   generated tool READMEs said "nix develop ~/0-core#faelight-forest -c
                            cargo build" -- a NixOS command. Now "cargo build"
```

### Found, not fixed -- invented numbers and stale facts

```text
    journal/mod.rs:309      every session-end entry says "Health: 100%." as fixed text
    daemon.rs:861           health.unwrap_or(100) answers "All systems nominal" when health
                            could not be read -- unknown reported as perfect
    Friday seeds            friday/mod.rs:275 "all 22 checks" (27 now) and "never ships below
                            95%" (nothing enforces it); :1046 "50+ custom Rust tools" (22);
                            daemon.rs:886 still lists faelight-fm, retired 2026-09-15
    a goal title            goals/mod.rs:104 "Restore forest health to 95%+" is matched by title
                            in forest_goals (:143), so renaming it would re-propose the goal.
                            With the schema intent
```

### Still deferred, and why

```text
    multi-line prose        partner 428/435/508, Friday 1234/1240/1255/1272 -- the text is not on
                            the line the census reports, so the guard refuses them
    nsh-test fixtures       "echo forest | grep forest", "forest writes", repl_206_forest_home,
                            repl_230_absent_forest -- one small pass, tests and names together
    faelight-git hooks      three hook templates written into .githooks -- with that crate
    palette text            faelight-context "forest green" -- with the design work
    the census instrument   its line numbers drift by one in at least one file (insightd 321 was
                            322). The pass script now looks three lines either way and prints a
                            note for every adjustment
```

## 2026-09-24, LATE NIGHT -- ITEM 3: TWO ACCESSORS RENAMED, AND ~/0-core/zero RULED

### 4d4d245b -- faelight_config_dir -> zero_config_dir, faelight_dir -> source_dir

```text
    zero_config_dir   paths.rs 5 uses, doctor probes/files.rs 2 (the Zero Alias probe)
    source_dir        paths.rs 10 uses, engine app/context.rs 1
    18 uses in 4 files -- the recon count; the payload refused unless it matched exactly
```

The payload surveyed every .rs file first and refused on any mismatch: a different file set or
count, a new name already in use, or an old name inside a longer word. It renamed through fpatch,
re-surveyed, and no old name remained.

Proven: cargo test faelight-core 7, faelight-doctor 60, core 2 -- each 0 failed, no error line;
ship 17 shipped, 0 failed; d 0 failed, and Zero Alias green reading real: state=zero,
config=zero -- the probe that now calls zero_config_dir, so the rename is proven end to end.
Pushed.

source_dir is named so it is true both ways: today it returns ~/0-core/faelight, after INT-252
~/0-core/zero. Only the path inside it changes, never the name.

faelight_data_dir is NOT renamed. It still returns ~/.local/share/faelight, so it is renamed
together with that directory's move (item 4), and its name never says something untrue.

### RULED, Christian 2026-09-24: ~/0-core/faelight/ -> ~/0-core/zero/

The naming map row marked PROPOSED is now RULED. The ruling fixes the DESTINATION only. The move
stays INT-252: its own session, its own gate, done last -- it touches 646 files, every path in
AGENTS.md, ship, and intent citations.

### Item 3 of the order

```text
    faelight_dir          -> source_dir         DONE 4d4d245b
    faelight_config_dir   -> zero_config_dir    DONE 4d4d245b
    faelight_data_dir     unchanged             renamed with ~/.local/share/faelight, item 4
```

## 2026-09-24, LATE NIGHT -- ITEM 4 BEGINS: ~/.cache IS NAMED zero

```text
    ~/.cache/zero        REAL directory   health-status, friday.log, last-exit-status
    ~/.cache/faelight    -> zero          kept so every old reference still resolves
```

### The recon

Three small files; no process held them; nothing outside the repo named the path. In code, three
accessors in paths.rs each joined "faelight/<file>" onto xdg_cache_home() -- the cache directory had
no owner. read_health() already answers None for an absent file, so the move could not fabricate a
health number through it.

### The order -- alias, red, flip, swap

```text
    alias      ~/.cache/zero -> faelight (relative); all three files one inode under both names
    red        zero_cache_dir() and two tests added, accessors untouched: faelight-core
               7 passed, 2 failed -- the two new tests and nothing else
    004c0d16   the flip. zero_cache_dir() is the ONE function that joins onto xdg_cache_home();
               the three accessors join onto it; the nsh-test caret fixture reads tmp/zero.
               faelight-core 9 passed 0 failed; nsh-test 202/202 including the caret case;
               d 0 failed; health-status read through zero matched d
    swap       renameat2(RENAME_EXCHANGE), then faelight repointed to zero by an atomic replace --
               from python, not mv/ln. Rehearsed first in a mktemp directory inside ~/.cache, same
               filesystem; refused unless the tree was clean and the flip pushed.
               Inodes KEPT: health-status 46763, friday.log 290174, last-exit-status 596566
```

After the swap: d 92%, 25/27, 0 failed, tree clean and pushed; health-status read through zero said
92, matching d; a fresh exec nsh, then false, and last-exit-status read failure.

### The two tests

cache_files_live_under_zero_cache_dir -- every cache file's parent is zero_cache_dir().

xdg_cache_home_is_joined_by_one_owner -- the CLASS. paths.rs, read with include_str!, must join onto
xdg_cache_home() exactly once. A future accessor that joins its own directory name there goes red.
The needle is split with concat! so the test's own source does not match it.

### Found, not fixed

```text
    prompt.rs:242       reads ~/.cache/faelight/last-system-rev: one reader, zero writers since
                        NixOS, so .ok()? always answers None. NOT renamed -- naming a dead path zero
                        would dignify it. Belongs to INT-250: delete the read
    faelight-docs:430   health unreadable from both sources becomes "100" -- unknown reported as
                        perfect, the INT-192 class. Fix-on-the-way candidate, its own commit.
                        NOT YET RULED
    paths.rs tests      the new tests sit above use super::* in mod tests. Compiles; cosmetic
    docs                ARCHITECTURE.md:148 names ~/.cache/faelight -- the seven-document rewrite
```

### Item 4, what remains

```text
    ~/.cache/faelight             DONE -- zero real, faelight -> zero
    ~/.local/share/faelight       next -- with faelight_data_dir (clipboard history)
    ~/.local/share/forest-trash   found in passes 5-6, commands/mod.rs :4016 and :4092
    ~/.config/faelight-shell      last -> ~/.config/nsh; it holds the aliases
```

## 2026-09-24, LATE NIGHT -- TWO FIXES ON THE WAY, RULED BY CHRISTIAN

```text
    7b764b2c   system_drift removed from novashell prompt.rs, with its one call. It read
               ~/.cache/faelight/last-system-rev, which nothing has written since NixOS, so it
               answered None on every render and the prompt never showed its hint. Behaviour
               unchanged; one .git/HEAD read per prompt render gone. Deleted, not renamed --
               a dead path does not get the new name
    c1db03e2   faelight-docs: health unreadable from both sources is "?", not "100"
```

Proven: the prompt edit ran first on a copy and matched the planned file byte for byte; novashell
221 passed 0 failed; faelight-docs builds (it has no unit tests); ship 10 shipped 0 failed; nsh-test
202/202; the prompt renders as before.

These resolve the prompt.rs:242 and faelight-docs:430 lines under ITEM 4 BEGINS, Found, not fixed.
That section stays as written; this one records the fix.

Still open, the same class: daemon.rs:861 health.unwrap_or(100) answers "All systems nominal" when
health could not be read -- recorded under PASSES 7 AND 8.

## 2026-09-24, END OF SESSION -- superseded by the 2026-09-25 section below

READ THIS FIRST WHEN PICKING UP. It supersedes "WHERE THIS SESSION STOPPED, AND THE ORDER NEXT"
above, which was written earlier the same day.

### State at bb08745c -- tree clean, pushed, d 92%, 0 failed

```text
    state, config, cache      zero is the REAL directory; faelight -> zero links kept
    paths.rs accessors        source_dir, zero_config_dir, zero_cache_dir; faelight_data_dir remains
    outside, still faelight   ~/.local/share/faelight, ~/.local/share/forest-trash,
                              ~/.config/faelight-shell
```

### RULED, Christian 2026-09-24: faelight-clipboard is not needed -- RETIRE it

The decision test is answered: 0 invocations since the migration, and Christian does not need it.
The next session is therefore a RETIREMENT, not a migration:

```text
    1  baseline    git status, d
    2  recon       what is in ~/.local/share/faelight; every caller of faelight_data_dir(),
                   clipboard_history_file() and faelight_clipboard; its aliases in config.nsh;
                   its registry entry; INT-221's pick/sk site inside the crate
    3  retire      ship --retire, then the five places ship does not cover (2026-09-23, "What a
                   retirement actually costs"): tools.toml retired = true and deployable = false,
                   the crate directory deleted, workspace members, the devbox census case, its
                   aliases in config.nsh
    4  accessors   faelight_data_dir() and clipboard_history_file() lose their last callers --
                   DELETE them, do not rename them. A path nothing uses does not get the new name
    5  the data    if ~/.local/share/faelight holds only clipboard history, Christian decides:
                   delete, or archive with a date. Nothing is removed before he sees the listing
```

If the directory holds anything else, that part follows the cache method instead.

### THE METHOD -- how every step since the cache has been done

```text
    recon       fsearch for code; read-only python for disk. Look before touch
    transport   one argv word: python3 -c 'import base64,sys; exec(...)' <b64> <mode>
    payloads    define only; the ONE call is the last line, so a cut paste does nothing
    edits       through fpatch: patch() for ASCII anchors read in the same run; patch_between()
                where a region holds non-ASCII, rehearsed on a copy and compared byte for byte
    guards      every payload refuses unless its preconditions hold: anchors found once, tree
                clean, the previous step committed and pushed
    tests       red first, then green; a class test, not only the example
    moves       alias, flip the code, then an atomic renameat2(RENAME_EXCHANGE) swap rehearsed
                on the same filesystem; inodes compared before and after
    records     each INT-247 section checks its own claims against disk and git before writing
```

### Then, in order

```text
    after clipboard   daemon.rs:861 -- "All systems nominal" on unreadable health, the class
                      c1db03e2 fixed in faelight-docs
    next directory    ~/.local/share/forest-trash
    last directory    ~/.config/faelight-shell -> ~/.config/nsh (it holds the aliases)
    then              items 5, 6 and 7 of the order above: inta, the words, the structure
```

## 2026-09-25 (written 2026-09-24), END OF SESSION -- superseded by the section below

READ THIS FIRST WHEN PICKING UP. It supersedes every earlier "START HERE" and "where this
stopped" section in this file. Every claim below was checked against git and the disk by the
script that wrote it.

### A CORRECTION FIRST

The 2026-09-24 START HERE ordered faelight-clipboard retired. It had ALREADY been retired on
2026-09-23 (INT-260's evidence names it): no crate files, not on PATH, registry retired = true,
aliases only in .bak files. A session went into planning a retirement that was done.

RULE: before starting a step a START HERE names, check it against disk and git. A list can be
stale on the day it is written.

### Done this session

```text
    002bfa78   faelight_data_dir() and clipboard_history_file() deleted -- no callers
    2a514845   zero_data_dir() and teach_progress_file(); teach stops building its path from
               HOME. Class test: local_data_dir() is joined exactly once. Red, then green
    c7ca9b32   trash_dir() = zero_data_dir()/trash; the delete builtin uses it; help text and
               comment say zero. Red (compile error), then green. Proven on the deployed shell:
               delete put a probe file in ~/.local/share/zero/trash
    (disk)     ~/.local/share/zero is REAL, faelight -> zero. teach-progress.json kept its inode
               through the renameat2 swap. forest-trash never existed on this machine
    (tool)     cargo-audit installed; a fresh core security scan ran it -- 0 findings.
               d 96%, 26/27, 0 failed
```

### State

```text
    state, config, cache, share   zero is REAL; the faelight names are links to it
    outside, still faelight       ~/.config/faelight-shell only
    paths.rs                      no accessor is named faelight; FAELIGHT_STATE_DIR remains
```

### Next, in order

```text
    1  ~/.config/faelight-shell -> ~/.config/nsh. The last outside directory; it holds the
       aliases (config.nsh). Alias, flip, swap. Accessors: shell_config_dir(),
       shell_scripts_dir(), nl_patterns_file(). NSH_CONFIG is the override
    2  the crates, one at a time: kept -> zero-*, otherwise retired. From disk today:
       faelight-ade, faelight-context, faelight-core, faelight-daemon, faelight-deadwood
       faelight-docs, faelight-doctor, faelight-gen, faelight-git, faelight-insightd
       faelight-release, faelight-sandbox, faelight-update, faelight-vm, faelight-zone
       Decision-test answers for most are in "2026-09-23 -- THREE MORE RETIREMENTS".
       faelight-insightd and faelight-context are UNDECIDED -- Christian rules first.
       deps/mod.rs categorize_tool groups tools by the faelight- prefix: change it in the SAME
       commit as the first rename. faelight-core goes LAST (355 uses)
    3  items 5 to 7 of "THE ORDER, ruled by Christian 2026-09-24": inta for SCHEMA and
       CONTRACTS, the words, INT-252
```

### Found, not fixed

```text
    doctor security_audit   the pass line says "scanned today"; the skipped/warn line does not
                            say how old the scan is, so a scan from before a fix reads as the
                            current state. Christian noticed it too -- to look into
    Alias Coverage          ship and zero-gate have no aliases; zero-gate as a risk audit is
                            for another day
    the untrue facts        daemon.rs:861 and :886, friday/mod.rs:275, :1046, :1047, :1052,
                            journal/mod.rs:309, and 25 unwrap_or(100) sites, measured
                            2026-09-25. PROPOSED as their own intent (the INT-192 class),
                            not yet ruled
```

## 2026-09-24, NIGHT, END OF SESSION -- superseded by the 2026-09-25 section below

READ THIS FIRST WHEN PICKING UP. It supersedes every earlier "START HERE" in this file. Every
claim below was checked against git and the disk by the script that wrote it.

### A DATE CORRECTION FIRST

The section above headed 2026-09-25 was written on the night of 2026-09-24 -- measured: the
machine read 2026-09-24T20:14-05:00 at the start of this session. Left as written; this line is
the record.

### Done this session -- the last outside directory

```text
    c000b4dc   nsh_config_home() is the ONE place ~/.config/nsh is named. shell_config_dir()
               falls back to it; shell_config() joins config.nsh onto it. Class test: the
               retired directory name appears nowhere in paths.rs, and the nsh name is joined
               exactly twice -- the XDG and HOME branches of the owner. The default template
               config.rs writes and the devshell nshconf row follow. faelight-core 13 passed,
               novashell 221, ship 17 shipped 0 failed, nsh-test 202/202
    (disk)     alias first: ~/.config/nsh -> faelight-shell, config.nsh one inode (58:341584)
               under both names. Then one renameat2(RENAME_EXCHANGE), rehearsed in a mktemp
               directory inside ~/.config: ~/.config/nsh is REAL, faelight-shell -> nsh.
               5 entries, every inode kept
    (disk)     config.nsh's two stale header lines rewritten in place, inode kept
    doors      exec nsh after the flip and again after the swap: 256 aliases loaded; d 96%,
               26/27, 0 failed both times
```

RED, HONESTLY: the two tests landed first, and the flip refused unless they were present, but
the failing run was not captured in this session's log. Red was proven on a stand-in crate built
from the same regions: 2 failed, then 4 passed with XDG_CONFIG_HOME set, unset, and with
NSH_CONFIG set.

### State

```text
    state, config, cache, share   zero is REAL; the faelight names are links to it
    shell config                  ~/.config/nsh is REAL; ~/.config/faelight-shell links to it
    outside the repo              NOTHING is real under a faelight name. What remains are the
                                  compatibility links, removed last (item 7 of THE ORDER)
```

### Next, in order

```text
    1  the faelight-shell NAME in live code: file headers, help text, the teach module, and the
       dead /run/current-system/sw/bin/faelight-shell lookups (commands/mod.rs near 12755,
       main.rs near 2732). Then faelight-shell joins the display-name guard
    2  the crates, one at a time: kept -> zero-*, otherwise retired. faelight-insightd and
       faelight-context need Christian's ruling first. categorize_tool changes in the same
       commit as the first rename. faelight-core last
    3  inta for SCHEMA and CONTRACTS, the words, INT-252
```

### PROPOSED by Christian, not ruled

```text
    the shell branch   NovaShell, nsh-test and DevBox grouped as the shell in the tree, as in
                       THE MENTAL MODEL. A layout question, so it belongs with INT-252 -- and it
                       should be decided BEFORE INT-252 runs, so every path moves once
```

### Found, not fixed

```text
    devshell-lib:27          the state row snapshots .local/state/faelight -- a link to zero
                             now. Should name zero
    paths.rs near 730-750    shell_config()'s doc comment sits above shell_config_dir(): two
                             doc blocks run together over one function, none over the other
    ~/.config/nsh leftovers  config.fsh.bak-20260623T213049, config.fsh.stub,
                             config.nsh.bak-1790169521 -- Christian decides keep or delete
    still open               everything under "Found, not fixed" in the section above:
                             security_audit age, Alias Coverage, the untrue facts
```

## 2026-09-25, END OF SESSION -- superseded by the NIGHT section below

READ THIS FIRST WHEN PICKING UP. It supersedes every earlier "START HERE" in this file. Before
starting a step it names, check the step against disk and git.

### Done this session

```text
    1ac46d19   the old shell name: 37 header, help and comment lines say NovaShell or nsh
    408a731f   faelight-shell 48 -> 0 in live code. Events write nsh; diag, gaps and identity
               moved under nsh; exec and reload ask paths::bin_dir, so bare exec nsh works;
               four broken defaults fixed on the way. Guard + faelight-shell, red first 201/202
    9793e9f0   nsh identity says NovaShell; the snapshot manifest header says Project 0.
               Guard + Faelight Shell, red first 201/202
    bae75273   INT-263 (schema) and INT-264 (contracts) filed with details and gates
    7b79c725   INT-252: the source tree is zero/ -- the record is in INT-252
```

### State

```text
    outside the repo   nothing real under a faelight name; the old names are links, removed last
    source tree        ~/0-core/zero/ -- no faelight/ at the root
    the guard          Zero Core, "Project 0 {}", faelight-shell, Faelight Shell
    fpatch             ~/0-core/zero/scripts/dev/fpatch.py -- AGENTS.md still names the old path
```

### RULED, Christian 2026-09-25

```text
    faelight-insightd and faelight-context are KEPT and renamed zero-*
    INT-252 is not completed until the entire flip is: no live faelight or forest in the code
```

### Next, in order -- THE CRATE PASS

```text
    1  ONE crate per commit. Each: the directory, [package] and [[bin]] names, every import,
       tools.toml and aliases.toml, the aliases in ~/.config/nsh/config.nsh (outside the repo),
       ship --retire for the old binary name, doctor lines that name the binary. deps/mod.rs
       categorize_tool changes in the FIRST rename's commit
    2  the fifteen: ade context daemon deadwood docs doctor gen git insightd release sandbox
       update vm zone -- small leaf crates first, faelight-core LAST (355 uses)
    3  nsh-test main.rs:881 goes red on the first rename. That red is INT-252's open gate
    4  then INT-263 schema, INT-264 contracts, the compatibility links, the docs and README last
```

### Found, not fixed

```text
    intent/mod.rs ~1656   the fallback template default tags "faelight" -- a Layer 0 breach the
                          template test does not cover
    nsh identity          "Login shell since 2026-04-03" is untrue: bash is the login shell
    printed Faelight      9 strings, all inside faelight-* crates -- they go with each crate
```

## 2026-09-25, NIGHT -- THE CRATE PASS AT 6 OF 15 -- superseded by the MORNING section below

READ THIS FIRST WHEN PICKING UP. It supersedes the 2026-09-25 END OF SESSION section above.
Before starting a step it names, check the step against disk and git. Every claim below was
checked against git and the disk by the script that wrote it.

### Done this session

```text
    70d31161   AGENTS.md names zero/scripts/dev for fpatch (lines 539, 547, 743)
    33857379   1/15 faelight-gen -> zero-gen. categorize_tool groups zero- like faelight-,
               red first (zero_prefix_is_categorized_like_faelight). Fixed on the way: the gen
               box header, the README build line
    85ae8690   the tree emoji dropped from all 19 crate README headings -- RULED by Christian:
               README headings, now and for good
    fda6fb9f   2/15 faelight-vm -> zero-vm, and its own data names (no VM data existed on disk)
    443393a6   3/15 faelight-ade -> zero-ade
    4f9feeb5   4/15 faelight-context -> zero-context
    650a647c   5/15 faelight-insightd -> zero-insightd
    a7a6376a   6/15 faelight-deadwood -> zero-deadwood
```

Every crate went through the same doors before its commit: the renamed binary on PATH,
nsh-test 202/202, d 0 failed. ship --retire kept each old binary in ~/0-core/bin/<name>@<stamp>.

### THE METHOD -- one script, named by its own hash

```text
    SCRIPT ~/.cache/zero/crate-rename-d38dedd39c13.py  (the name is its sha256 prefix;
           delete it when the pass ends)
    plan <crate>          read-only: every edit; REVIEW on lines that build a path or run or
                          match a process; SKIPPED for systemctl, journalctl, is-active, qcow2,
                          org.faelight, .sock, .service; collisions; a FINGERPRINT
    apply <crate> <fp>    refuses unless the plan still matches what was reviewed; git mv plus
                          fpatch, each file verified against the plan; never commits
    alias <crate>         config.nsh; refuses until ~/.local/bin/zero-<crate> exists
    per crate             plan -> review -> apply -> README build line -> cargo test -> ship
                          -> alias -> ship --retire faelight-<crate> -> exec nsh -> doors -> commit
```

A REVIEW line is renamed only if the thing it names is renamed in the same commit. A line that
must stay (a dead path, a history entry a rename would make false) is put back through fpatch
after apply, before the build -- done for strategy/mod.rs:1628 and the insightd README.

THE HOME SWEEP RAN before ade: nothing outside the repo calls a faelight-* binary -- no Hyprland
bind, .desktop file, systemd unit, script in ~/.local/bin, rc file or crontab. The only hits were
two backups in ~/.config/nsh that nsh does not load.

### Next, in order

```text
    leaves    update, sandbox, release, docs, then daemon. daemon last of them: its systemd
              unit, D-Bus names and socket are contracts the SKIP list holds as data.
              release needs Christian's name for the Forest DNA README section --
              faelight-release readme.rs:166 writes it with the tree emoji
    engine    doctor, zone -- the engine imports both (zone as a library; its binary is retired)
    git       novashell imports it; its hook templates are written into .githooks
    core      LAST, its own reviewed payload -- the script refuses core. 101 importers
    then      INT-252's fixture case goes red with core (see INT-252), the compatibility links,
              INT-263 schema, INT-264 contracts, the docs and README last
```

### Found, not fixed -- each needs a ruling

```text
    strategy/mod.rs:1628  Factor 7 checks scripts/faelight-context and scripts/faelight-memory;
                          neither exists, so it always scores 0 of 7 and says "Neither context nor
                          memory built yet" while zero-context is built and deployed. Left
                          unrenamed: a dead path does not get the new name. With the untrue facts
    strategy/mod.rs:1564  systemctl is-active faelight-insightd -- a unit name; Project 0 runs no
                          user services on Omarchy (INT-237)
    zero-vm               run-faelight-vm-vm, faelight-vm.qcow2 in .gitignore and faelight-vm-swtpm
                          are names the NixOS build-vm runner made, and are kept. No VM image exists
                          and nothing builds that runner on Omarchy: does zero-vm drive anything?
    zero-vm state_dir()   builds ~/.local/state/zero-vm itself instead of asking paths.rs
    devbox census         zero-ade ignores its arguments, so zero-ade --version opens the ADE;
                          that census case cannot pass in a clean room
    README history        auto-seeded changelog lines in crate READMEs now say zero-*: an
                          anachronism, not a falsehood. The one a rename made false (insightd's
                          rename record) was restored
    AGENTS.md:743         still says the scripts are at zero/scripts/devshell TODAY and that
                          INT-252 renames that -- the move is done
    ~/.config/nsh         config.nsh.bak-1790169521 and config.fsh.bak-20260623T213049 name
                          faelight-* tools; nothing loads them. Keep or delete is Christian's
    a one-off payload     the vm data-name step wrote main.rs, then refused before .gitignore --
                          it checked after writing. Every check belongs before the first write;
                          the crate script already works that way
```

## 2026-09-25, MORNING -- THE CRATE PASS AT 14 OF 15 -- superseded by the 2026-09-26 section below

READ THIS FIRST WHEN PICKING UP. It supersedes "THE CRATE PASS AT 6 OF 15" above. Before starting
a step it names, check the step against disk and git. Every claim below was checked against git
and the disk by the script that wrote it.

### Done this session -- eight crates, each its own commit, each through the doors

```text
    02e29c06   7/15  faelight-update  -> zero-update
    40743cab   8/15  faelight-sandbox -> zero-sandbox
    a302a404   9/15  faelight-release -> zero-release   Forest DNA is Project 0 DNA
    09b9775e  10/15  faelight-docs    -> zero-docs      footer matchers moved WITH the footer
                                                        they read; 18 README footers and the
                                                        tools index name zero-docs; Part of
                                                        Project 0 in 20 READMEs
    3f22a1bc  11/15  faelight-daemon  -> zero-daemon    D-Bus, wire and unit names kept as data
    4c084e7a  12/15  faelight-doctor  -> zero-doctor    a library; probe module forest is
                                                        internals; the Zero Alias recovery hint
                                                        no longer points the link the wrong way
    06e4d6e8  13/15  faelight-zone    -> zero-zone      Core zone icon is the Project 0 mark
    e09dcaf2  14/15  faelight-git     -> zero-git       hooks shown not to call it before retire
```

INT-265 filed: every defect the pass walked past, with file and line, gates and rulings needed.

### RULINGS, Christian 2026-09-25

```text
    scope          until INT-247 and INT-252 are complete and closed, the crate pass renames
                   and rebrands ONLY. Everything else goes to INT-265 and waits for them
    Forest DNA     is Project 0 DNA (the README section zero-release writes)
    tree emoji     leaves each crate as that crate passes -- sayings keep theirs until the
                   sayings pass
    Core zone      its icon is U+25C9, the mark the startup banner shows beside Project 0
    READMEs        "Part of [Project 0]", not Faelight Forest
    aliases        z replaces the f prefix (fg -> zg, fu -> zu, fr- -> zr-, fdocs -> zdocs...);
                   the work is INT-265's
    checks.toml    the Zero Alias recovery hint was fixed inside the doctor commit: it was the
                   Layer 3b flip's own leftover, and following it would have pointed the link
                   the wrong way over the state directory
```

### THE METHOD, what this session added

```text
    non-ASCII     fpatch patch() refuses a non-ASCII anchor by design. Emoji edits build the
                  file's planned text in memory, apply each changed range bottom-up through
                  patch_between, and compare the file with the plan after writing. A range with
                  no unique line after it refuses (zero-git Cargo.toml) -- ASCII lines like that
                  go through patch() instead
    builds        every cargo test line is filtered for "test result|^error|FAILED|panicked",
                  so a red is always named. A novashell red whose names are all observe::tests
                  is INT-265's proven race, not the crate
    matchers      a line that searches data on disk is renamed only with the data it reads in
                  the same commit (the docs footer), or not at all (dead paths, unit names)
    history       a comment quoting a measured path keeps the path that ran (faelight-git
                  commit 370); an anachronism is left, a falsehood is put back
    retire        a crate with no binary (doctor, zone's library) needs no alias and no retire;
                  check ~/.local/bin for BOTH names before and after
```

### State

```text
    crates        14 of 15 renamed. faelight-core remains; the crate script refuses it
    ~/.local/bin  no faelight-* binary of the fourteen is on PATH
    zero-zone     on PATH although the registry says deployable = false -- INT-265, gate 1
    hooks         .githooks never called faelight-git; commits and pushes run through them
```

### Next, in order

```text
    1  faelight-core -> zero-core, as its own reviewed payload: the package, every
       faelight_core:: use, every Cargo.toml dependency, the nsh-test fixture at main.rs:212-216
       (INT-252's gate -- it goes red with this rename, on purpose). Recon first
    2  INT-252: regenerate and classify the audit; the fixture gate
    3  the remaining word passes: sayings, the tree emoji outside the renamed crates (Friday's
       banners, nsh type, d's Friday lines), faelight_data and the stale probe.rs:49 line with
       the compatibility links
    4  INT-263 schema, INT-264 contracts, the compatibility links, the docs rewrite, the guard
       for faelight and forest -- then the finish line
    5  INT-265, only after INT-247 and INT-252 are closed
```

## 2026-09-26, MORNING -- THE CRATE PASS IS DONE, 15 OF 15 -- superseded by the 2026-09-28 section below

READ THIS FIRST WHEN PICKING UP. It supersedes "THE CRATE PASS AT 14 OF 15" above. Before starting
a step it names, check the step against disk and git. The script that wrote this checked the
commits and the zero-core state against git and the disk; the test counts are from the runs made
in the session.

### Done this session

```text
    d7d561dd          the MORNING records for 247 and 252, written last session, committed
    c4634250  15/15   faelight-core -> zero-core, the library every tool links: 374 faelight_core
                      uses in 102 files, 19 dependency lines, the registry entry and the three
                      name matchers that read it, moved together. Reviewed plan b218e73b9485.
                      INT-252's fixture gate went red on the rename and green on the rewrite
    b01efe38          INT-266 filed: finding, intent and commit linked by name, with the plan's
                      fingerprint. depends_on 247, 252 and 265
```

### Proven for c4634250

```text
    cargo test --workspace   every result ok -- zero-core 13, novashell 221, zero-doctor 60
    ship                     21 shipped, 0 failed; exec nsh loaded 256 aliases
    nsh-test                 201/202 on the rename (tilde_nested_pipe), 202/202 after the rewrite
    d                        0 failed
    faelight_core            no live .rs file contains it -- checked again by this record's script
```

### Held on purpose -- these still name the old crate

```text
    census-core-coupling.py and novashell/CORE-COUPLING.md
                             INT-230 G1's instrument and its record. Re-run after the rename, the
                             script would count 0
    teach/src/main.rs.v2.0.0 a tracked backup cargo does not compile (INT-265, hygiene)
    zero-core/Cargo.lock     the stray lock, moved with the directory (INT-265)
    the docs                 AGENTS.md:281, docs/ARCHITECTURE.md, docs/inventory.md, the generated
                             rust-tools/README.md, novashell/README.md:58-59 -- the docs pass, each
                             for the reason given at review: half-true, NixOS-era, a dated census,
                             generator-owned, and the zero-git leftover beside it
```

### Found, not fixed

```text
    deploy line      "deploy # sudo nixos-rebuild switch" in 17 crate READMEs and in
                     zero/engine/README.md: false on Omarchy. zero-docs writes those READMEs, so
                     its template changes in the same commit or the next sync writes it back.
                     zero-core's own README was fixed in c4634250
    a word           c4634250's message calls teach/src/main.rs.v2.0.0 "untracked-by-cargo". Git
                     tracks it; cargo does not compile it. History is not rewritten; this line
                     is the record
```

### Next, in order

```text
    1  commit 2, zero-core only: FaelightError (error.rs, glyph.rs, the lib.rs re-export), the
       theme's faelight_default, faelight_dark and faelight_light, and FOREST_GREEN. Other crates
       call them: census of callers first, then plan, fingerprint, apply, doors, commit
    2  the NixOS deploy lines: the zero-docs template and the 18 READMEs, one commit
    3  the remaining word passes: the sayings, the tree emoji outside the renamed crates
    4  INT-263 schema; INT-264 contracts, FAELIGHT_STATE_DIR and FAELIGHT_STATE_DB among them;
       the compatibility links; the docs rewrite; the guard for faelight and forest
    5  the finish line. INT-265 starts only after INT-247 and INT-252 close
```

### Open, needing Christian

```text
    INT-252 gate 1   "the audit is regenerated and classified before anything moves" cannot be
                     ticked as written: the move ran in 7b79c725 under census 12cb33529e87.
                     Rewrite it by ruling to name that census, or defer it. Asked 2026-09-26
    trailers         whether commits carry Intent, Finding and Fingerprint trailers before
                     INT-266 starts. Asked 2026-09-26
```

## 2026-09-28 -- COMMIT 2 AND THE DEPLOY LINES -- superseded by the 2026-09-28 EVENING section below

READ THIS FIRST WHEN PICKING UP. It supersedes "THE CRATE PASS IS DONE, 15 OF 15" above. Before
starting a step it names, check the step against disk and git. The script that wrote this checked
the commits, their trailers, the five old code names and the README deploy lines against git and
the disk; the test counts are from the runs made in the session.

### Done this session

```text
    8c4c4c15   commit 2 -- zero-core's code names say zero. FaelightError is ZeroError; the
               theme's faelight_default, faelight_dark and faelight_light are zero_default,
               zero_dark and zero_light; the default theme's doc line says Project 0.
               FOREST_GREEN is DELETED, not renamed: the recon found no caller, and a name
               nothing uses does not get the new name. Plan c74e22229304. The first commit to
               carry trailers
    f6a15c25   18 READMEs (17 crates and the engine) say ship and cargo build -p <package>, not
               sudo nixos-rebuild or nix develop. Hand-edited to the exact text the zero-docs
               template already writes, so a later sync agrees; sync itself was not run, because
               it also regenerates docs/public. nsh-test's build line said -p fsh-test and now
               names its real package. Plan b739b4b3c8f1
```

### Proven

```text
    8c4c4c15   cargo check -p zero-core --features ui finished before AND after (glyph.rs is
               compiled only with ui); cargo test all ok; ship 15 shipped 0 failed; nsh-test
               202/202; d 0 failed; the five old names in live files: none
    f6a15c25   24 lines in 18 files, each equal to the plan; nsh-test 202/202; d 96%, 0 failed;
               no README outside the intents says nixos-rebuild
    trailers   git log reads Intent and Fingerprint back from both commits
```

### Rulings, Christian 2026-09-28

```text
    names        ZeroError, zero_default, zero_dark, zero_light -- approved
    FOREST_GREEN recon decides, he said; the recon found no caller, so it was deleted
    colours      none decided. Project 0's colours come from the Omarchy theme; the forest
                 names went with the forest
    trailers     YES. Every commit from 8c4c4c15 carries Intent and Fingerprint trailers;
                 Finding waits for INT-266 to give findings IDs
    INT-252      gate 1 REWRITTEN to name the census that ran (12cb33529e87, in 7b79c725) and
                 ticked -- done by this record, with its evidence in INT-252
```

### Found, not fixed

```text
    palette      zero-core still carries a fixed palette (NEON_GREEN and the rest) while Project 0's
                 colours come from the Omarchy theme. The colour work, not the rename
    template     zero-docs writes the binary deploy wording for every crate, libraries included.
                 zero-core's README has a library line by hand (c4634250); the next sync would
                 replace it. Teaching the template the difference is INT-265's
    a pipe       under nsh, cargo ... 2>&1 | grep left cargo's progress lines on the terminal
                 unfiltered and grep matched none of them (exit 1) -- seen 2026-09-26 and
                 2026-09-28. Observed, not investigated; the cause is not known
```

### Next, in order

```text
    1  the word passes: the sayings (Christian's voice -- the tree emoji goes with them), then the
       tree emoji outside the renamed crates
    2  INT-263 schema: the forest_* tables, by ALTER TABLE, rollback rehearsed first
    3  INT-264 contracts: FAELIGHT_STATE_DIR and FAELIGHT_STATE_DB, the D-Bus names, the forest
       commands and flags -- alias first
    4  the compatibility links, after every non-Rust caller is swept
    5  the docs rewrite: the seven documents, meta/README.md whole, docs/public regenerated,
       AGENTS.md:281, the tools index
    6  the guard for faelight and forest, reading every file type; then the finish line
    7  cicomplete 247 and 252 only when every gate is honestly ticked; then INT-265, then INT-266
```

## 2026-09-28, EVENING -- FOUR COMMITS; EVERY REMAINING NAME WAITS ON A RULING -- superseded by the 2026-09-28 NIGHT section below

READ THIS FIRST WHEN PICKING UP. It supersedes "COMMIT 2 AND THE DEPLOY LINES" above. Before
starting a step it names, check the step against disk and git. The script that wrote this refused
to write unless every commit below, its two trailers, and every finding marked CHECKED held on
disk and in git.

### Done this session

```text
    8b8e0865   the Layer 0 breach: the intent wizard's fallback repeated the future template
               as its own literal, tagged faelight. It now IS the future row, looked up in
               TEMPLATES; future_row_exists pins that the row exists. Plan f90d2344aa8e
    9e970d15   fm is gone from the code, as ruled ("anything that has FM needs to be
               removed"): core workspace fm, the plugin arm and plugins.toml block,
               SKIP_VERSION, fm in type's two builtin lists (type fm had claimed a builtin
               nothing handled), completion, deadwood, deps. yazi keeps its cwd handoff as
               yazi_cwd_file and is_yazi_cmd. Plan deac2753f14c
    c3ad4a8d   dead calls: nsh no longer spawns faelight-export at every shell start; the
               faelight-fetch plugin, aliases and deps suffix; the Friday notify seed; the
               events hint to a unit that never existed; the FAELIGHT-SPECIFIC alias section
               (swaymsg reload); the stray Cargo.lock files in zero-core and zero-git.
               Plan 4db6c8b5a649
    07d46c1a   stale names, text only: the REPL thread is nsh-repl; engine tests use
               /tmp/zero-test; current help examples; zero-core doc lines; zero-docs in
               changelog.rs. Plan 4afbeaf4f3d4
```

Every commit carries Intent and Fingerprint trailers. Doors on each: cargo test --workspace all
ok, ship 0 failed, nsh-test 202/202, d 0 failed. When a door's output was not seen (9e970d15),
the plan was replayed on a clean clone and its diff compared with the live tree: equal.

### The census at 07d46c1a -- same definition as the morning census at ad33fd97

```text
                    now       morning
    faelight live     1,119   1,162    in 159 files; 355 of those lines are INT-247 and INT-252 themselves
    forest live       1,318   1,320    in 198 files
    faelight hist     2,005   2,005
    forest hist         846     846
```

### Findings

```text
    fm gate       CHECKED. The gate "faelight-fm is gone -- workspace, PATH, docs, teach,
                  Friday facts, command registry" was ticked 2026-09-15 while the engine still
                  had core workspace fm, a plugin entry, SKIP_VERSION, completion and deadwood
                  entries and a Friday answer naming it. 9e970d15 finished what the gate
                  claimed. The tick stands; this is the correction. registry/tools.toml keeps
                  its retirement record pending ruling 1
    vm route      CHECKED. vm is an ALIAS in ~/.config/nsh/config.nsh (vm -> zero-vm) that
                  shadows the builtin. vm_dispatch is unreachable and would run
                  zero/packages/faelight/scripts/vm, deleted in 362d18d8 (INT-255). vm works,
                  through the alias; the 2026-09-23 "PROVEN wired" line credits the wrong
                  mechanism
    exit-2 label  nsh labels an external command's exit 2 "misuse of shell builtin" -- bash's
                  meaning -- as it did for core's argument-parser error. For INT-265; not fixed
    history       faelight history read 8 lines fewer in the morning census than on
                  2026-09-24. git diff --shortstat de954f49..HEAD over every history path: 0
                  insertions, 0 deletions. Nothing was edited; the two census scripts differ
    tools         22 crates on disk, none named faelight-*. Every faelight-<name> left in live
                  code is a stale name, a dead tool, or not a tool
    rm -rf        nsh's delete guard stops rm -rf mid-paste and asks for DELETE. Payloads
                  should not put rm -rf mid-block
    scripts       plan and record scripts exit 1 on REFUSE or MISMATCH, so && chains stop
```

### Rulings still open -- with the recommendation given

```text
    1  RETIREMENT RECORDS  14 registry/tools.toml entries for retired tools. Recommended
                           KEEP: history the registry and catalog use
    2  notify              the long-command notification (novashell engine.rs) calls
                           faelight-notify, which Quickshell replaced; it has never fired on
                           Omarchy. Recommended PORT to notify-send
    3  commit 7            nsh compare (faelight-diff), nsh cache (scripts deleted in
                           362d18d8), vm_dispatch (unreachable). Recommended DELETE all three
    4  D                   the faelight CLI gate. Recommended YES: the 2026-09-15 retirement
                           answers it; a 0 command becomes its own intent
    5  HISTORY IN COMMENTS dated measurements naming a tool as measured: friday planning.rs
                           :996, novashell main.rs:2147, ship main.rs:268, zero-core paths.rs
                           :478, zero-docs main.rs:329, zero-release changelog.rs:541, engine
                           notify/mod.rs:32 and :40, novashell engine.rs:2133 and :2148.
                           Recommended REWORD
    A  release manifests   meta/releases/*/manifest.toml are history
    B  live intents        the finish guard excludes zero/intents/; NixOS-era 048, 145 and
                           157 are cancelled with the ledger's method
    C  teach backup        delete the tracked teach/src/main.rs.v2.0.0
    E  dictionary forest   zero-gen's wordlist uses forest as a word, not the brand
    F  subtitle            zero-release writes "formerly Faelight Forest" under the README
                           title: Layer 1 allowed it, the finish line forbids it
    also                   zero-update --snapshot (faelight-snapshot); strategy Factors 7 and
                           8; zero-vm's runner names and .gitignore; risk.rs's "faelight
                           snapshot" advice; the faelight_root field and ForestDb (identifiers)
    gates                  PROPOSED, not ticked: "Layer 3 is NOT started until..." and the pace
                           gate, on evidence already in this file. Christian's word first
    to file               a tree-organization intent (inta), designed after 247 and 252 close
                           so every path moves once
```

### Next, in order

```text
    1  get the rulings above; each unlocks its own plan
    2  those plans, one concern per commit: recon, plan, fingerprint, apply, doors, commit
    3  the remaining word passes: the sayings (Christian's voice), then the identifiers
    4  INT-263 schema; INT-264 contracts (FAELIGHT_STATE_DIR, the D-Bus names, the forest
       commands and flags)
    5  the compatibility links; the docs rewrite; the guard for faelight and forest in every
       file type
    6  cicomplete 247, then 252 (its last gate is this intent's finish line); then INT-265
```

### The method, as used today

```text
    transport  zlib inside base64, one argv word; the writer refuses unless the stream
               decompresses and its sha256 matches the file name it writes
    scripts    ~/.cache/zero/<name>-<sha prefix>.py; plan prints every edit and a FINGERPRINT;
               apply refuses on any mismatch; deleted when the commit lands
    edits      fpatch patch() for ASCII spans read from the file in the same run;
               patch_between for spans with non-ASCII, markers found by the script; every file
               compared with the plan after writing
    doors      apply, cargo test --workspace and ship chained with &&; then nsh-test, d, and
               the door specific to the change; the commit is its own block
```

## 2026-09-28, NIGHT -- THE CENSUS, AND THE PASSES THAT FINISH IT -- superseded by the 2026-09-28 PASS 1 section below

READ THIS FIRST WHEN PICKING UP. It supersedes "FOUR COMMITS; EVERY REMAINING NAME WAITS ON A RULING"
above. Before starting a step it names, check the step against disk and git. The script that wrote
this refused to write unless every commit below exists with its trailers, zero-vm is gone from the
tree, PATH and config.nsh, the teach backup is gone, no live .rs names faelight-notify, and the tree
was clean and pushed.

### Rulings, Christian 2026-09-28

```text
    priority      the faelight, forest and NixOS removal comes BEFORE every other intent, the
                  shell included. Relationship is rewritten to say so
    the evening   adopted as recommended: 1 retirement records KEEP; 2 notify PORT; 3 compare,
    rulings       cache and vm_dispatch DELETE; 4 (D) the faelight CLI gate YES; 5 dated comments
                  REWORD; A release manifests are history; B the finish guard excludes
                  zero/intents/, and 048, 145 and 157 are cancelled by the ledger's method;
                  C the teach backup DELETE; E zero-gen's dictionary word forest is EXEMPT;
                  F the "formerly Faelight Forest" subtitle is REMOVED
    NixOS         everything NixOS-era goes: code, comments, the "WAS REMOVED HERE" tombstone
                  notes, docs. Project 0 is the new
    zero-vm       RETIRED (0d35dc3d). A VM, if wanted, comes from the Omarchy plugin site or a
                  new build under its own intent
    teach         KEPT -- Christian's own learning tool, built early (compare
                  DanWahlin/learn-omarchy). Its lesson content gets one recon for NixOS and
                  faelight text
    ForestDb      becomes StateDb, the handle to state.db
    the guard     registry entries with retired = true are EXEMPT: the registry's own history,
                  like a completed intent
    /etc/faelight becomes /etc/zero -- INT-268, filed and written today. INT-247 removes
                  /etc/faelight from every live file (the nine comments, pass 1). INT-268 designs
                  what /etc/zero holds and the system-run writer that writes it -- or closes
                  without creating it, if nothing needs to be system-wide
    zombie code   none: no call to a retired tool, no read of a path nothing writes, no code on
                  a path nothing reaches, no child left unreaped. It is a gate now
    INT-267       the tree intent: filed, designed any time, MOVED before the docs rewrite
    intents       Christian: "we are creating more intents than completing." This session filed
                  267 and 268 and completed none. The next session starts on pass 1
    still open    H: the two PROPOSED gate ticks (the Layer 3 gate and the pace gate)
```

### Done this session

```text
    2de8974d   INT-267 filed -- the shell branch and the tree, designed before anything moves
    9f8dbcb4   INT-268 filed -- /etc/faelight to /etc/zero, seven gates, the writer as the hard part
    99933962   compare, cache and vm_dispatch removed with everything that advertised them: the
               three dispatch arms, compare in where's vocabulary, the libvirt vm verbs, the
               ~/vms qcow2 completion, vm in completion's and deadwood's builtin lists, the false
               LIVE AND STAYS comment. compare had SHADOWED /usr/bin/compare. Plan 9becec1e1cc1
    87012d13   teach/src/main.rs.v2.0.0 deleted -- 1,112 lines cargo never compiled
    ce1df990   the long-command notification calls notify-send, not the retired faelight-notify.
               Plan dd869f21aefd. ITS POPUP DOOR IS RED for a reason that predates it -- findings
    d1d73903   eleven comments stop naming retired tools. Plan c39483c749f9
    0d35dc3d   zero-vm retired: crate, census case, .gitignore line, Cargo.lock entry, binary
               (kept in ~/0-core/bin), alias vm; the registry says retired and why.
               Plan 6f8ab1eb9f8d
```

Doors on every commit: cargo test --workspace all ok, ship 0 failed, nsh-test 202/202, d 0 failed,
and the door specific to the change. Every commit carries Intent, and Fingerprint where a plan ran.

### THE CENSUS -- the new starting line, measured at d1d73903

A line counts if it contains faelight or forest in any case. HISTORY is excluded: completed,
decision, philosophy, cancelled and incident intents, and every CHANGELOG.

```text
    2,269 live lines in 258 files
      668  markdown in live intents   excluded by ruling B; 454 are INT-247 and INT-252 themselves
      547  markdown, docs             62 files
      341  Rust strings               70 files
      319  Rust code and identifiers  42 files
      305  Rust comments              87 files
       68  TOML                       28 files
       21  scripts and other
    about 1,600 lines are the work, and five names carry most of the Rust: ForestDb 176, the
    forest_* tables about 150, FAELIGHT_STATE_DB and FAELIGHT_STATE_DIR 36, org.faelight.Forest 9,
    and the retired tool names. 279 distinct names in all
```

THE LESSON OF THE FORTNIGHT: taking a few names per commit is safe and does not converge. INT-252
converged because it started from one census of every occurrence. From here every pass takes ONE
KIND across the whole tree, in one plan.

### Findings, not fixed -- each with where it goes

```text
    zombie process   novashell main.rs:3658 -- Friday's "failed 3 times" notification spawns
                     notify-send and drops the child unreaped: a zombie per notification until
                     nsh exits. The class INT-299 fixed in engine.rs. It fired live this
                     session ("python3 failed 3 times today"). Pass 4
    dead path        execute_and_record (engine.rs:1821) is not reached by ordinary commands --
                     the spine runs them -- so INT-194's slow-command warning and the
                     long-command notification are silent. PROVEN: a logging notify-send first on
                     PATH saw no call after sleep 31 ran 31s; the newest TIMING row in
                     shell_history is 2026-09-10 23:28, after 110 TIMING:sleep rows. The routing
                     commit is in git log --since=2026-09-10 --until=2026-09-12 for novashell
                     main.rs. Pass 4, with INT-201 and INT-196
    skipped headers  when an && chain stops, nsh still prints the [n/m] header of every skipped
                     command; only the missing output shows it did not run. INT-265
    exit-2 label     an external's exit 2 is labelled "misuse of shell builtin" (seen again on ls
                     of a missing file). INT-265
    doctor trend     "Declining health with no active work" and a 7-day forecast of 88% follow
                     door runs; 88% is d's score on an uncommitted tree, so the trend is likely fed
                     by mid-change readings. UNCONFIRMED. INT-265
    /etc/faelight    nine live comment lines, INT-250's history notes. Pass 1
    INT-265 gate     "zero-update and zero-vm take th..." names a retired tool. Reword with 265
    teach            its lessons are unread for NixOS and faelight text. One recon, pass 9
    the sayings      the banner still says forest ("The roots hold. The branches grow." is fine;
                     "The forest remembers. The human decides." is not). Pass 9, his voice
```

### THE PASSES -- each one plan, one fingerprint, EVERY occurrence of its kind

```text
    1  comments      every live Rust comment naming faelight, forest or NixOS, the tombstone
                     notes included -- a note that only records NixOS history goes whole. The
                     2026-09-24 vocabulary: forest as the project -> Project 0; as state or
                     health -> the word dropped; as the repo -> repo; a retired tool -> a plain
                     description. Split by crate if the plan runs long
    2  identifiers   ForestDb -> StateDb, then ForestHelper, ForestDeployIface, faelight_root and
                     the rest, through the compiler, which finds every use
    3  strings       printed text, under the display guard extended to faelight and forest
    4  zombie code   every Command::new of a retired or absent binary, every read of a path
                     nothing writes, every function nothing reaches, every unreaped child --
                     deleted or rewired, and a guard that goes red on the first
    5  contracts     INT-264: FAELIGHT_STATE_DB and FAELIGHT_STATE_DIR -> ZERO_*, the
                     org.faelight.* D-Bus names, the forest commands and flags. Alias first
    6  schema        INT-263: the forest_* tables by ALTER TABLE, every SQL string in the same
                     commit, rollback rehearsed first -- Layer 3's rules
    7  the links     ~/.local/state/faelight, ~/.config/faelight, ~/.cache/faelight,
                     ~/.local/share/faelight and ~/.config/faelight-shell removed once every
                     non-Rust caller is swept
    8  the tree      INT-267 designed and moved
    9  the docs      the seven documents, meta/README.md, docs/public regenerated, teach's
                     lessons, the sayings
   10  the guard     one test reading every file type: no live faelight, forest or NixOS outside
                     the exemptions below. Seen red first. Then cicomplete 247 and 252
```

### Next session, in order

```text
    1  re-run the census at HEAD with the same definitions -- the line to beat
       PROPOSED, for Christian's ruling: no new intent until 247 closes, unless it holds a
       ruling 247 itself needs
    2  pass 1, the comments, the crate with the most first
    3  pass 2, ForestDb -> StateDb
    then the passes in order, one kind per commit
```

### The method, and what this session added

```text
    recon      read-only, rehearsed like any payload -- a one-line read with an escaped quote in
               an f-string failed on the machine because it was sent unrehearsed
    plans      every edit printed with a FINGERPRINT; apply refuses unless it matches
    apply      the whole edit set runs through fpatch on mktemp copies and must be byte-identical
               to the plan before a real file is touched; each real file is compared after.
               patch() for ASCII spans; patch_between where a span holds non-ASCII -- the edit
               routine decides by reading the span, not by guessing
    windows    a guard window is bounded by STRUCTURE, not a line count: a registry edit is
               confined to its own [[tool]] block after a +-5 window caught a neighbour
    doors      cargo test through a python filter (under nsh, cargo piped to grep once went
               unfiltered), ship, exec nsh, nsh-test, d, and the door for the change
    outside    files outside the repo are copied to ~/.cache/zero before the write, and the copy
               is deleted when the doors are green
    scripts    ~/.cache/zero/<name>-<sha prefix>.py, deleted with the commit
```

## 2026-09-30, 14 OF 18 -- START HERE

READ THIS FIRST. It supersedes the PASS 7 DONE section below; its rulings, findings and method stand
unless this says otherwise. Written only if HEAD was the COUNT commit, clean and pushed, a60298ba exists,
the Success Criteria read 14 ticked and 4 open, zero-zone is not on PATH, and the five directories are
real with no old name beside them.

### Done since the PASS 7 record -- pushed

```text
    4d722109   the PASS 7 record
    a60298ba   NO NIXOS ticked: the last two NixOS comments (friday/mod.rs) rewritten; the path-audit
               gate ticked from the 2026-09-15 audit and the Layer 3 dates
    (COUNT)    COUNT ticked at 18: registry, tree, PATH, catalog and README all measured 18
```

### Rulings, Christian 2026-09-30

```text
    exemptions   font files (third-party glyph names) and AGENTS.md (his file) join the guard's list
    AGENTS.md    his file: propose, never edit
    tool         a tool is a binary Project 0 builds and deploys. Libraries (zero-core, zero-doctor)
                 are not tools
    zero-daemon  KEPT, deployable -- it will be connected with Friday
    zero-zone    binary retired for real: [[bin]], src/main.rs and its devbox census case removed,
                 ship --retire took it off PATH; the library stays (engine and doctor import it)
```

### Gates -- 14 of 18. The four open, in order

```text
    1  inventory    docs/inventory.md EXISTS (113 lines, last written 2026-09-15, e780ab9a). Recon
                    first: does it cover today's 18 tools and 2 libraries, with keep/replace/retire,
                    the four decision-test answers, last invocation, and DevBox. Update, then tick
    2  UNREADABLE   one open_state_db() in zero-core that reports unreadable; the state readers move
                    onto it. The one real code change left. Recon every state reader first
    3  exemptions   ticks with the guard (pass 10): one test reading every file type, the exemption
                    list as ruled, seen red first
    4  FINISH LINE  no live faelight or forest anywhere: pass 8 (INT-267 tree, INT-252), pass 9
                    (docs, fsh names, nl.rs, ~/.config/zero themes), pass 10 (guard, kept aliases,
                    nsh-test names). Then cicomplete 247 and 252
```

### Found, not fixed

```text
    zero-docs     readme-index prints "index, 21 tools" -- it counts crates; the file it writes says 18
    aliases       alias f-daemon = zero-daemon -- the f prefix (z replaces f, INT-265 ruling)
    docs          ARCHITECTURE.md "51 total tools", THEORY_OF_OPERATION.md "30 Rust tools" (pass 9);
                  README.md: "Forest DNA" heading, a changelog link into faelight/meta, "~125k lines"
    zero-zone     its Cargo.toml still lists clap, which only the removed binary may have used
    carried       the PASS 7 DONE list: ~/.config/zero themes and 0444 files, checkpoint/mod.rs:580,
                  generate-history-inventory.py, binary-size-baseline.txt
```

### A new chat starts with

```text
    ints 247, and this section pasted. Then: recon docs/inventory.md against the 18 tools
```

## 2026-09-30, PASS 7 DONE -- SUPERSEDED 2026-09-30 by the 14 OF 18 section above

READ THIS FIRST. It supersedes the 6B3 DONE section below; that section's rulings, findings and method
stand unless this says otherwise. Written only if HEAD was 7a328a4a, clean and pushed, the four commits
below exist, each of the five directories is real with no old name beside it, the Success Criteria read
11 ticked and 7 open, and the NixOS count below matched.

### Done -- pushed

```text
    c4c09481   the 6B3 DONE record
    0475640d   PHILOSOPHY rewritten by Christian for Omarchy and Project 0; the comparison table restored
    aeffdd14   PHILOSOPHY names declarative Linux, not NixOS; its forest and Faelight lines gone;
               docs/public regenerated -- index.md's title still said Faelight Forest Documentation
    7a328a4a   PASS 7: the five compatibility links are gone
```

### Pass 7 -- what changed

```text
    probe        Zero Paths (id zero_alias): state, config, cache and share must be real zero
                 directories and ~/.config/nsh real, with the old name absent -- a link or a directory
                 under an old name is a finding; unreadable is reported as unreadable, never absent
    hint         the recovery hint no longer says to recreate a link
    devshell     devshell-lib snapshots .local/state/zero
    docs         AGENTS.md and PHILOSOPHY describe one name each
    disk         the five links removed after ship deployed the new probe; config.toml state_dir and
                 term.toml's path comment say zero (both files made writable for the edit, back to 0444);
                 the .bashrc comment names nsh only; six stale config backups deleted
    doors        d 0 failed, Zero Paths "5 directories, one name each"; zero-doctor 60 passed;
                 nsh-test 202/202 with both devshell cases
```

### Gates -- 11 of 18, unchanged. Pass 7 proves no gate by itself.

NO NIXOS is the nearest: 9 lines in 4 files remain, intents and CHANGELOGs excluded.

```text
    AGENTS.md                   1   :40, the missing Omarchy recovery runbook
    friday/mod.rs               3   :142, :1007, :1009 -- comments
    registry/tools.toml         3   :29, :93, :103 -- read whether each sits in a retired entry
    HackNerdFont-Regular.ttf    2   the font's own glyph names; zero-core paths.rs loads the font.
                                    Recommended: exempt as third-party data. NOT YET RULED
```

### Found on the way, not fixed

```text
    ~/.config/zero       config.toml, term.toml, themes.toml are 0444 by mode -- who sets it is not
                         known; config.toml theme = "faelight-forest" and the themes.toml sections;
                         term.toml belongs to the retired faelight-term; profiles.toml's header.
                         A config pass with reader recon first
    checkpoint/mod.rs    :580 keeps only btrfs lines containing faelight, after a sudo call -- pass 4 class
    novashell            generate-history-inventory.py:24 points at the faelight/ tree INT-252 moved
    zero/meta            binary-size-baseline.txt says faelight-shell=11.0; nothing reads it
    history              ~/.cache/zero/friday.log and the state checkpoints name old tools -- history
    the method           a plan must check write access on every target: the pass 7 apply wrote five
                         files and stopped at a 0444 file. The resume script checked modes first
```

### Next, in order

```text
    1  NO NIXOS: the 9 lines above and the font ruling -- then tick the gate with the count
    2  UNREADABLE: open_state_db() in zero-core, recon of every state reader first
    3  pass 8: INT-267 tree and INT-252
    4  pass 9: docs; the fsh comments and the nsh builtin description; nl.rs's 12 context lines;
       the ~/.config/zero config pass above
    5  pass 10: the guard, the kept aliases, nsh-test case names
    6  inventory, COUNT, path audit; cicomplete 247 and 252
```

## 2026-09-30, 6B3 DONE -- PASS 7 NEXT -- SUPERSEDED 2026-09-30 by the PASS 7 DONE section above

READ THIS FIRST. It supersedes the PASS 6 section below; that section's rulings, findings and method
stand unless this says otherwise. Written only if HEAD was 2bccddf0, clean and pushed, the three
commits below carry the subjects named, the five compatibility links were links on disk, the
Success Criteria read 11 ticked and 7 open, and the NixOS count below matched.

### Done since the PASS 6 record -- pushed

```text
    d6d7adb2   the PASS 6 record; it named 6B3 next
    692ed5ac   AGENTS.md trimmed to the working rules; tables restored, four facts corrected
    2bccddf0   6B3 DONE: ship runs nsh --refresh-cheatsheet after it ships nsh. The 6B3 run
               reported 245 aliases and 105 builtins in the registry
```

### Gates -- 11 of 18 ticked, unchanged

```text
    open   inventory, COUNT, path audit; UNREADABLE; NO NIXOS; exemptions and FINISH LINE
```

### NO NIXOS at 2bccddf0 -- 23 lines in 6 files, intents and CHANGELOGs excluded

```text
    docs/PHILOSOPHY.md          7   Christian's rewrite
    docs/public/PHILOSOPHY.md   7   generated from docs/ -- regenerated, never hand-edited
    friday/mod.rs               3   :142 a doc comment naming the nixos and forest domains;
                                    :1007 and :1009, the comment the PASS 6 record placed at :1011
    registry/tools.toml         3   :29 and :103 descriptions, :93 a comment -- whether each
                                    sits in a retired = true entry (exempt) is read before editing
    AGENTS.md                   1   :40, the missing Omarchy recovery runbook
    HackNerdFont-Regular.ttf    2   the font's own glyph names (dev-nixos, linux-nixos):
                                    third-party data. Exempt or not -- NOT YET RULED
```

### Next, in order

```text
    1  PASS 7 -- the compatibility links:
         ~/.local/state/faelight -> zero      ~/.config/faelight -> zero
         ~/.cache/faelight -> zero            ~/.local/share/faelight -> zero
         ~/.config/faelight-shell -> nsh
       Recon first: every caller outside the Rust code (config.nsh, Hyprland, ~/.local/bin,
       systemd user units, rc files, crontab -- guard for its absence), and zero-doctor's
       Zero Alias probe read before it is changed. The probe changes in the SAME step as the
       removal, so d stays green
    2  UNREADABLE: open_state_db() in zero-core
    3  pass 8: INT-267 tree and INT-252
    4  pass 9: docs; the fsh comments, the nsh builtin description with them; the NixOS lines
       above; nl.rs's 12 context "forest" lines (recon first); PHILOSOPHY is Christian's
    5  pass 10: the guard, the kept aliases, nsh-test case names
    6  inventory, COUNT, path audit; cicomplete 247 and 252
```

### Found, not fixed

```text
    carried   the three lines of the PASS 6 section: friday/mod.rs:258, events/mod.rs:221, d trend
```

## 2026-09-30, PASS 6 -- THE SCHEMA AND FRIDAY'S KNOWLEDGE -- SUPERSEDED 2026-09-30 by the 6B3 DONE section above

READ THIS FIRST. It supersedes the LATE NIGHT section below; its rulings and method stand unless this
says otherwise. Written only if HEAD was 15492582, clean and pushed, the four pass-6 commits carry
Intent INT-247, and state.db holds no forest table and no Friday row naming forest, faelight, nixos or fsh.

### Done -- all pushed, every commit through its doors

```text
    18e1f3d8   6A    nine forest_* tables renamed in one transaction: zero_events (events was taken),
                     events_v2, goals, insights, mandates, plans, predictions, strategies, tradeoffs;
                     143 SQL, name and label sites in 26 files. Backup state.db.pre-pass6-20260930T045915
    (data)           forest_memory dropped -- 4 rows, no code named it; the pass-6 backup holds them
    0000eac5   6B1   goal title; nsh::delete; knowledge source 'lesson'; the NixOS knowledge entry and
                     release_triad removed
    97bddede   6B2a  Friday's project domain is zero; seven seeds rewritten true; NixOS translations and
                     seeds gone; Arch facts restored and systemd facts moved to domain linux;
                     intelligence_name is Ground State
    15492582   6B2b  27 Friday rows about retired tools or untrue facts deleted; zero-git, zero-daemon,
                     events_v2 and vocabulary rows reworded; two duplicate seeds removed
```

### Gates -- 11 of 18 ticked, unchanged this session

```text
    open   inventory, COUNT, path audit; UNREADABLE; NO NIXOS (comments, docs, PHILOSOPHY);
           exemptions and FINISH LINE (pass 10)
```

### Rulings 2026-09-30

```text
    tables   plain words; zero_* only on a collision (zero_events). Log tables are history
    Friday   intelligence_name is Ground State. Project facts live in domain zero, Linux facts in
             linux. cistart starts an intent; cicomplete closes it once every gate is proven
    fsh      the name is nsh. The 260 fsh comment lines (37 files) are recommended for pass 9, with
             the 223 string and code lines as their own pass -- the timing is not ruled
```

### Next, in order

```text
    1  6B3 command_registry: 285 alias rows say config.fsh and 13 command rows say faelight-shell.
       Recon what rebuilds the table (novashell cheatsheet_tui.rs:185-290) before any plan
    2  pass 7: the compatibility links and zero-doctor's link probe
    3  UNREADABLE: open_state_db() in zero-core
    4  pass 8: INT-267 tree and INT-252
    5  pass 9: docs; the fsh comments; the NixOS comments (friday/mod.rs:1011); nl.rs's 12
       context "forest" lines (recon first); PHILOSOPHY is Christian's
    6  pass 10: the guard, the kept aliases, nsh-test case names
    7  inventory, COUNT, path audit; cicomplete 247 and 252
```

### Found, not fixed

```text
    friday/mod.rs:258   seed "fg commit after changes" -- stale, INT-265
    events/mod.rs:221   doc comment names `core events status`, which prints the JSONL event log;
                        StatusV2 is the events_v2 verb -- INT-265
    d trend             "Declining health with no active work" follows door runs on a dirty tree
```

### Failures this session, and the rule each earned

```text
    transport   a 7.4k one-line payload arrived damaged -> over ~2.5k, install in sha-checked chunks
                to ~/.cache/zero/<name>-<sha>.py and run it by name; fix an installed script with a
                small checked in-place line, never a resend
    formatter   6A committed +158/-160: the hook reformatted after the tests ran -> rustfmt the
                touched files inside the apply, before the build
    patterns    a seed regex assumed the line shape -> read the real bytes with repr first
    doors       a door named a verb from its doc comment -> read the dispatch before naming a door
```

### ~/.cache/zero: this session's b62 and b63 scripts were deleted by the script that wrote this.

## 2026-09-30 LATE NIGHT, NO ZOMBIE, /ETC/FAELIGHT, ZERO-RELEASE, 5E PARTS 1-2A -- SUPERSEDED 2026-09-30 by the PASS 6 section above

READ THIS FIRST. It supersedes the NIGHT record below; its rulings and method stand unless this says
otherwise. Written only if HEAD was 4ee4309c, pushed, clean, with /ETC/FAELIGHT and NO ZOMBIE ticked.

### Done -- all pushed, every commit through its doors

```text
    a8c82a88 a1e9aae6   generated READMEs: link CHANGELOG.md, never copy git subjects
    6fa86814            nsh: command -v defers to POSIX command
    3bc41fd0 d3201691   zero-release and core release bump-system retired
    b627f4d4 .. 2e7c6ebf  pass 4 (4A-4E): zombies, reaping class, bump-tool and workspace view retired,
                        zero-gate retired-spawns gate (seen red first), record_timing on the spine
    bc20fdb8            /ETC/FAELIGHT TICKED        efac04ac   NO ZOMBIE TICKED
    76c302bf e1f74501   5E parts 1 and 2a: user-facing text and nsh:: codes, /tmp/nsh-* files
    4ee4309c            forest-stats, fstats, fsh-gaps gone; dead pkg-search completion gone
```

### Gates -- 11 of 18 ticked

```text
    open   inventory, COUNT, path audit; UNREADABLE; NO NIXOS (PHILOSOPHY + Friday NixOS rows);
           exemptions and FINISH LINE (pass 10)
```

### Rulings 2026-09-29/30

```text
    bump-tool retired (its binary never existed; cicomplete never called it) -- Christian shows how
    tools are bumped after 247. forest-stats/fstats/fsh-gaps removed now. Zero compiler warnings is a door.
```

### Next, in order

```text
    1  5E remainder: fsh in comments -- ask Christian now or pass 9; nsh-test names stay for pass 10
    2  pass 6 schema with a rehearsed rollback
    3  UNREADABLE: open_state_db() in zero-core
    4  passes 7-10; PHILOSOPHY is Christian's
    5  inventory, COUNT, path audit; cicomplete 247 and 252
```

### Failures this session, and the rule each earned

```text
    traceback   hand-copied 5,000-char payload damaged -> small payloads, byte-check the reply text,
                installer guarded so damage prints REFUSE, never a traceback
    refuses     missed caller (goals/mod.rs) -> sweep the whole tree before retiring a name;
                5E counts from a comment-blind census -> count every fragment in the real tree first;
                d inside bash -c -> doors run in nsh; unused imports -> warnings checked every removal
```

### ~/.cache/zero: keep census, gaterecon, nixrecon2, pass5c, passreload; this session's one-offs go.

## 2026-09-29 NIGHT, STATEDB, PACE, NIXOS-ERA CRATES AND NO NIXOS A-C1 -- SUPERSEDED 2026-09-30

READ THIS FIRST WHEN PICKING UP. It supersedes the EVENING record below, whose rulings, held lines and
method still stand except where this section says otherwise. The script that wrote this refused unless
HEAD was 948040e5 and pushed, the tree was clean and the seven commits below exist as INT-247 commits.

### Done this session -- all pushed, every commit through its doors

```text
    82e23d2b   STATEDB        deps test renamed zero_prefix_is_categorized; census 0 lines of Rust code and
                              identifiers; gate TICKED
    f92c664d   NO NIXOS A     the engine nix domain (INT-088, a nixos-option wrapper) and core nix deleted; its
                              only caller, alias inspect, left config.nsh; ship deployed core
    1dd4b6f1   NO NIXOS B1    38 edits in 23 files: tombstones and comments, fsearch --nix, the prompt /nix
                              colour, nsh's NixOS PATH entries (~/.cargo/bin still leads PATH once per tree)
    eb1e002f   NO NIXOS B2    zero-release stops reading /nix and loses gc-check; zero-update drift reads
                              pacman.log; meta/packages.txt, the platform census script and its doc deleted
    794a9f29   pace           TICKED with its record: broken on purpose, and it says so
    de5aba4e   NixOS crates   TICKED: the nix domain was the last; zero/meta/README.md lost its Nix Inspector line
    948040e5   NO NIXOS C1    MIGRATION-RUNBOOK, NEW-CHAT-DIRECTIVES, ideas-parking and recovery-runbook deleted;
                              NixOS sections and lines out of docs/ and docs/public/; AGENTS.md keeps the
                              recovery gap as its own line; CHANGELOGs untouched
```

### THE GATES -- 9 of 18 ticked

```text
    TICKED   LAYER 0, NAME, faelight-fm gone, faelight-docs, state alias week, unified CLI, STATEDB,
             NixOS-era crates, pace
    open     inventory      docs/inventory.md not written
    open     COUNT          README, the catalog, tools.toml and the tree not reconciled
    open     path audit     the audit is a section in this intent; the gate asks for a deliverable in its own right
    open     UNREADABLE     gaterecon (a heuristic) found 23 silent, 35 unclear and 31 reporting state-path
                            sites. Design: one open_state_db() in zero-core that reports unreadable, and the
                            readers move onto it crate by crate
    open     NO NIXOS       live code and docs are clean except four held pieces: PHILOSOPHY (Christian
                            rewrites it), the generated tool READMEs (13 lines -- read zero-docs toolgen first),
                            the release docs (RELEASE.md, zero/meta/README.md, zero-release's README -- they go
                            with zero-release), Friday's NixOS rows (14 lines -- the schema pass)
    open     NO ZOMBIE      pass 4
    open     /etc/faelight  INT-268
    open     exemptions     pass 10, the guard
    open     FINISH LINE    the census below, then the schema, the tree, the docs and the guard
```

### Rulings, Christian 2026-09-29 (night)

```text
    tool axis     Project 0 upgrades tool by tool, never as a whole system. cicomplete and core release
                  bump-tool are the version path. zero-release and core release bump-system RETIRE. Callers
                  swept: config.nsh (bump, release, fr-history, fr-preview, fr-status), zero-docs (owns README
                  lines 1-37, registry entries), engine intent/mod.rs tool list and release/mod.rs, nsh
                  commands/mod.rs Cargo.toml map, devbox/census/zero-release.toml, tools.toml
    docs          the four NixOS-era documents are deleted; the recovery gap is stated in AGENTS.md and an
                  Omarchy runbook belongs to INT-225; PHILOSOPHY is Christian's rewrite; the generated tool
                  READMEs are re-examined before any edit
    zero-update   drift is days since pacman.log's last full system upgrade, unknown without one
    CHANGELOGs    never edited, by any pass, including link cleanup
```

### Next, in order

```text
    0   ints 247 and this START HERE
    1   generated tool READMEs: read how zero-docs toolgen writes its history lines, then fix the generator
        or the files -- whichever owns the text
    2   retire zero-release and core release bump-system (callers above); RELEASE.md, zero/meta/README.md
        and zero-release's README go with it
    3   5E fsh -> nsh, as ruled in the EVENING record
    4   pass 4 zombies, then the NO ZOMBIE gate (list in the EVENING record)
    5   pass 6 schema, with a rehearsed rollback: forest_* tables, Friday's forest/nixos domain and system
        values and NixOS seed facts, release_triad's generation column, the goal title, fsh::delete.
        NO NIXOS closes when this lands and PHILOSOPHY is rewritten
    6   UNREADABLE: open_state_db() and its readers
    7   passes 7 (links, zero-doctor's probe), 8 (INT-252 tree), 9 (docs), 10 (guard, kept aliases,
        nsh-test case names)
    8   inventory, COUNT, the path audit deliverable; then cicomplete 247 and 252
```

### Open questions for Christian (carried)

```text
    releases   11 zero/meta/releases/*/manifest.toml themes: exempt as history, or rewrite?
    theme      the stored prompt_theme is none of the four names
```

### What this session added to the method

```text
    recon first   a gate-tick script is preceded by its read-only sweep, so a leftover line is recon
    one run       a plan collects every refusal cause and lists them all at once
    transport     the byte-check is of the exact text in the reply, run through bash -- not the sandbox file
```

### Tools in ~/.cache/zero -- reuse, do not rewrite

```text
    census-6c5139205bd1.py        read-only census
    gaterecon-bd77dca4c79d.py     read-only: UNREADABLE site classes and the live NixOS lines
    nixrecon2-b98255a4f866.py     read-only: NixOS lines in their comment blocks, nix/store fns and callers
    pass5c-1048ccce0bff.py        line-entry engine; reuse for 5E
    passreload-5cdf8e6411e0.py    block engine
    one-offs from this session (statedb*, nixplan*, nixcommit*, pace*, crates*, docplan*, doccommit*,
    record*) are finished and can be deleted
```

### THE CENSUS at this record -- the line to beat

```text
    INT-247 CENSUS at 948040e5 INT-247 NO NIXOS C1: the NixOS-era docs go -- four 
    intent directories: cancelled(history), complete(history), decisions(history), experiments, future, in-progress, incidents(history), philosophy(history), planned
    
      1605 live lines in 184 files
        771  markdown in live intents     41 files
        489  markdown, docs               57 files
        227  Rust strings                 39 files
          0  Rust code and identifiers     0 files
         65  TOML                         27 files
         37  Rust comments                21 files
         16  scripts and other            11 files
       834 lines are the work.  160 distinct names
      of the Rust strings, 26 lines sit inside a string that began on an earlier line
    
    by extension: .md 1260, .rs 264, .toml 65, .py 6, .json 4, .sh 2, .txt 1, pre-commit 1, .conf 1, devshell-lib 1
    
    LIVE PATHS naming faelight or forest (12)
      docs/forest-resilience.md
      docs/public/forest-resilience.md
      zero/intents/future/026-forest-observatory-event-timeline.md
      zero/intents/future/048-forest-ci-local-ci-with-gitea-and-hydra-for-flake-builds.md
      zero/intents/future/145-fix-faelight-ade-to-work-in-nix-and-fsh.md
      zero/intents/future/165-apparmor-confinement-layer-and-how-it-relates-to-faelight-sandbox.md
      zero/intents/future/187-evaluate-gix-vs-git2-for-faelight-git-pure-rust-git-not-a-felt-need-yet.md
      zero/intents/future/218-faelight-deadwood-scopes-the-command-word-check-by-file-while-the-rule-it-enforces-is-defined-by-role-so-a-live-defect-escaped-in-a-file-outside-the-six-name-list.md
      zero/intents/future/263-the-forest-tables-and-old-event-rows-still-carry-the-retired-names----rename-them-under-a-rehearsed-rollback.md
      zero/intents/future/268-going-from-etcfaelight-to-etczero.md
      zero/intents/in-progress/247-retire-the-faelight-label-without-breaking-the-machine.md
      zero/intents/in-progress/252-the-source-tree-still-spells-faelight-in-162-live-places----consolidate-the-callers-onto-pathsrs-then-rename-the-directory-to-zero.md
    
    PASS 2 SCOPE -- Rust identifiers naming faelight or forest in CODE (0)
      identifier                          code string  comment files
    
    CENSUS DIFFERS from the pass-1 record: lines 1605 (record 2061), files 184 (record 232), work 834 (record 1346)
```

## 2026-09-29 EVENING, PASSES 5B-5D AND THE RELOAD FIX -- 5E NEXT -- SUPERSEDED 2026-09-29 NIGHT

READ THIS FIRST WHEN PICKING UP. It supersedes "PASSES 2, 3 AND 5A" below, whose rulings, held lines and
method still stand except where this section says otherwise. The script that wrote this refused unless
HEAD was e8427098 and pushed, the tree was clean and the five commits below exist as INT-247 commits.

### Done this session -- all pushed, every commit through the doors inside its own commit script

```text
    c68f1c28   5B   D-Bus org.faelight.Forest -> org.zero.Core, /org/zero/Core/*, GetContext / Context,
                    get_context, log tag bus. Plan d8d3ec93c192
    7c2753b3   5C   zero-stats / zstats (forest-stats, fstats kept as aliases); cp-forest, mv-forest and
                    --forest dropped; theme zero (forest an alias); dashboard overview (forest an alias);
                    ADE session and layout zero-ade (a forest-ade session still found); nl phrases; config
                    template and ~/.config/nsh/config.nsh say prompt_style = zero. Plan 7f882d61129f
    5253cc71   5D   207 tree icons removed by rule in 40 files. Plan 7c1b27fe599a
    de919976   5D2  the prompt, db-browse, zero-update and zones take the Project 0 mark; the two readers
                    that looked for the tree (health_tui, nsh-test) go. Plan 0eb17ebb9516
    e8427098   reload  reload compares /proc/self/exe with ~/.local/bin/nsh by (device, inode). Gone: the
                    /tmp/fsh-running-build writer (every nsh process rewrote it), the /tmp/fsh-reload-signal
                    poll (nothing writes it), running_build_identity and its test. resolve_nsh_binary,
                    reload_nsh, the last two tree icons. Plan d8c59b428a68
```

Doors on every commit: cargo test 28 of 28 binaries, rustfmt clean in the touched crates, nsh-test 202/202,
zero-gate as the pre-commit hook, the deployed binary checked for the new build; d 0 failed.

### Rulings, Christian 2026-09-29 (this session)

```text
    tree icons    remove all; the prompt, db-browse, zero-update and zones use the Project 0 mark U+25C9
    5E            folded into INT-247. fsh -> nsh everywhere a user reads it ("native fsh command" ->
                  "native nsh command"); fsh stays an ALIAS as a typed name (nsh|fsh meta-command, exec fsh,
                  completion, the fsh info / fsh doctor phrases); fsh-gaps -> nsh-gaps with an alias;
                  diagnostic codes fsh::spine::* and fsh::platform::* -> nsh::... with their two tests;
                  fsh::delete is written to state.db -> pass 6; /tmp/fsh-* -> /tmp/nsh-*, writer and
                  reader together; the .fsh script runner reads scripts/fsh, which does not exist and holds
                  no files -> pass 4 zombie, not a rename
    reload        a builtin, not an alias: after ship, type reload
    zero-gate     it runs as the pre-commit hook; commit scripts confirm that and refuse if it changes
                  the tree
```

### THE CENSUS at e8427098 -- the line to beat

```text
    INT-247 CENSUS at e8427098 INT-247: reload compares the running nsh with the d
    intent directories: cancelled(history), complete(history), decisions(history), experiments, future, in-progress, incidents(history), philosophy(history), planned
    1657 live lines in 193 files
    756  markdown in live intents     41 files
    547  markdown, docs               62 files
    227  Rust strings                 39 files
    1  Rust code and identifiers     1 files
    66  TOML                         28 files
    40  Rust comments                22 files
    20  scripts and other            13 files
    901 lines are the work.  165 distinct names
    of the Rust strings, 26 lines sit inside a string that began on an earlier line
```

### THE GATES -- none is fully proven yet, so none is ticked; where each stands

```text
    inventory     docs/inventory.md not written                                    not started
    count         README, catalog, tools.toml and the tree not reconciled           not started
    NixOS crates  the engine nix domain is still live (INT-093 comment, pass 4)      open
    path audit    the hardcoded-path audit before layer 3 is not a deliverable yet   open
    unreadable    whether state readers report UNREADABLE was not checked            open
    pace          passes 2 through 5D and the reload fix all landed 2026-09-29; the  record it
                  pace rule is broken on purpose to finish the rename -- say so at close
    finish line   901 lines of work left; the kept aliases (forest-stats, fstats, theme
                  forest, dashboard forest, forest-ade) go at pass 10                open
    statedb       ForestDb -> StateDb done (029c710e); the census still counts 1 line of
                  Rust code / identifiers -- find it first next session              nearly
    no NixOS      the nix domain and NixOS comments remain; the reload fix removed the
                  store-path and makeWrapper comments                                open
    no zombies    the reload-signal poll (read a path nothing writes) is gone; the rest
                  is pass 4                                                           in progress
    /etc/faelight INT-268                                                             open
    exemptions    pass 10 (the guard)                                                 open
```

### Next, in order

```text
    0   exec /home/christian/.local/bin/nsh once (the shell open at the end of this session is the old
        build, whose reload cannot see a new one), then reload must say "Already on the current nsh build"
    1   find the 1 remaining line of Rust code / identifiers; if it is the last one, tick STATEDB with
        the census as proof
    5E  fsh -> nsh, as ruled above: 81 non-comment lines in nsh, 161 comment lines
    4   zombie code: the FOREST_ env prefix, faelight-snapshot, faelight-memory / insightd / context
        checks, zero/packages/faelight in nsh-test, zero-git risk text, the ADE launcher
        (~/.config/zellij does not exist, so ade always stops at "layout not found"), the .fsh runner,
        the nix domain; the config.nsh aliases bar, bar-restart, daemon-log and daemon-status call the
        faelight-bar and faelight-daemon units
    6   schema: forest_* tables, friday_knowledge domain forest and the forest_stats key, the goal
        title "Restore forest health to 95%+", the fsh::delete event source; rehearsed rollback
    7   the compatibility links and zero-doctor's link probe
    8   the tree, INT-267 (INT-252: the source tree spells faelight in 162 live places)
    9   docs: the markdown lines, teach's lessons, AGENTS.md
    10  the guard, the kept aliases removed, the nsh-test case names; then the gates above, then
        cicomplete 247 and 252
```

### Open questions for Christian

```text
    releases   11 zero/meta/releases/*/manifest.toml themes ("The Forest Speaks"): exempt shipped
               release manifests as history, or rewrite them?
    theme      the stored prompt_theme is none of the four names, so theme marks none; predates 5C
```

### Tools in ~/.cache/zero -- reuse, do not rewrite

```text
    census-6c5139205bd1.py        read-only census
    pass5c-1048ccce0bff.py        line-entry engine (fragment once per line, or DEL); reuse for 5E
    passreload-5cdf8e6411e0.py    block engine: unique anchor, brace-matched block, comments above
    pass5d-288a37b4b56d.py        rule engine for the tree icons
    commit scripts                sent inline: changed files exactly the plan, the deployed binary
                                  checked, cargo test, rustfmt, nsh-test, zero-gate; any red refuses
```

### What this session added to the method

```text
    guards        a plan's final guard reads comments too, so a rename covers doc comments
    stand-ins     carry the file's doc comments and the goal and Friday lines, not just the target lines
    reload        after a reload fix the OLD process answers first; leave it once by the full path
    speed         one block per reply, the next step only
```


## 2026-09-29, PASSES 2, 3 AND 5A -- THE CODE SPEAKS PROJECT 0; 976 LINES OF WORK LEFT -- SUPERSEDED 2026-09-29 evening

READ THIS FIRST WHEN PICKING UP. It supersedes "PASS 1 -- THE COMMENTS ARE DONE" below, whose rulings,
held lines and method still stand except where this section says otherwise. The script that wrote
this refused unless the six commits below exist with their trailers, HEAD was 719cb38d and pushed,
the tree was clean, and the census counted exactly 1,705 lines in 199 files, 976 of them the work.

### Rulings, Christian 2026-09-29 (this session)

```text
    sayings       swap "The forest X" to "Project 0 X" where it reads true; the tree icon beside a
                  saying goes. The metaphors, rewritten (zero is where things restart): Built From
                  Zero (the Living Forest titles); A system that knows itself can survive anything;
                  Project 0 does not fear the crash. It knows how to start from zero.; Every commit
                  is a ring. Project 0 reads them all.; A healthy repo sheds dead code.; Back to zero
    vocabulary    forest value pipeline -> value pipeline; forest commands -> native commands;
                  forest meaning the repo -> repo; ForestHelper -> ShellHelper; forest_version ->
                  project_version; faelight_root -> source_root; emit_forest_event ->
                  emit_runtime_event (emit_event was taken); seed_forest_lessons -> seed_lessons;
                  the zero-daemon Iface structs, BusState and run_bus lose the word
    env vars      FAELIGHT_STATE_DB / FAELIGHT_STATE_DIR / FAELIGHT_ADE -> ZERO_STATE_DB /
                  ZERO_STATE_DIR / ZERO_ADE, no alias: the sweep of rc files, nsh config,
                  Hyprland, systemd user units, ~/.local/bin and desktop entries found no caller
                  outside the repo
    APPROVED,     D-Bus org.faelight.Forest -> org.zero.Core, paths /org/zero/Core/*, interfaces
    not applied   org.zero.Core.Health/Intent/Friday/Deploy; protocol GetForestContext /
                  ForestContext -> GetContext / Context; get_forest_context -> get_context; the
                  log tag forest-bus -> bus
    commit gate   every commit script runs cargo test --workspace --no-fail-fast, rustfmt on the
                  touched crates and nsh-test ITSELF and refuses on any red
```

### Done this session -- all pushed, every commit through the doors

```text
    029c710e   2A  ForestDb -> StateDb: 172 lines in 17 novashell files. Plan f19094273146
    e0ad1b8f   2B  23 Rust-only names (ShellHelper, value pipeline, repo_dir, native commands,
                   the daemon Iface structs ...). Plan 2499429d1119
    1c3eb646   3A  labels and test text: headers, Zero ADE, value pipeline, zero-context, the
                   README subtitle removed (ruling F), nsh-test echoes zero. Plan fe0659339df7
    9ec60a01   3A fix  parser.rs asserted kind.contains("forest"); pushed red in 1c3eb646
                   because that commit was not gated. One word; the gate was born here
    cb2a9cd8   3B  the sayings speak as Project 0, the metaphors rewritten: 71 edits in 23 files.
                   Plan f4ba1ff8900b
    719cb38d   5A  the env vars are ZERO_*. Plan f8dea0ee90a9
```

Doors on every commit from 9ec60a01 on ran inside the commit script: cargo test 28 of 28, rustfmt
clean in every touched crate, nsh-test 202/202; d 0 failed.

### THE CENSUS at 719cb38d -- the line to beat

```text
    1,705 live lines in 199 files (2,075 at the start of the session)
      729  markdown in live intents   excluded by ruling B
      547  markdown, docs             pass 9
      271  Rust strings
       25  Rust code and identifiers  all held names (below)
       67  TOML
       46  Rust comments
       20  scripts and other
    976 lines are the work (1,346 at the start of the session)
    History directories are complete, decisions, philosophy, cancelled, incidents. A line counts by
    code first, then strings, then comments; the census script follows strings across lines
```

### Next, in order

```text
    5B  the plan REFUSED on a false collision: zero/engine/src/domains/weight_engine/mod.rs:334 has
        an unrelated "Context" string. Narrow the engine collision check to
        zero/engine/src/domains/daemon/mod.rs, re-plan, apply, ship, gated commit. After ship:
        pgrep -af zero-daemon -- if it runs, restart it so both ends speak GetContext
    5C  typed commands and flags, alias first: forest-stats / fstats, cp-forest, mv-forest,
        --forest, dashboard forest, forest-ade, the forest theme and prompt_style, nl.rs phrases
        (check forest, forest health, forest tools, forest events); with them forest_stats_*,
        dashboard_forest, forest_flags. The sweep found none of these outside the repo
    4   zombie code: the FOREST_ env prefix (commands/mod.rs), faelight-snapshot (zero-update
        --snapshot), faelight-memory / faelight-insightd / scripts/faelight-context checks (engine
        strategy), zero/packages/faelight in nsh-test, zero-git risk "faelight snapshot", the tree
        icon arg on the ADE launcher; plus the pass-1 list. zero-daemon is not on the bus -- is it
        run at all? Friday's "failed 3 times" counts a grep with no match as a failure
    6   schema: the forest_* tables (about 137 SQL lines), friday_knowledge domain='forest' keys,
        Friday's seeded facts and proposal texts, the goal title "Restore forest health to 95%+"
        (deduplicated by title), the table-name labels. ALTER TABLE with a rehearsed rollback
    7   the compatibility links and zero-doctor's link probe
    8   the tree, INT-267
    9   docs: 547 markdown lines, teach's lessons, forest-resilience.md, zero-docs, AGENTS.md's
        ForestDb
    10  the guard; the retired-name tests; zero_prefix_is_categorized_like_faelight; the nsh-test
        case names repl_206_forest_home..., repl_230_absent_forest... (stored in state.db results:
        renaming breaks their history -- decide). Exempt: zero-gen's dictionary word forest.
        Then cicomplete 247 and 252
```

### Tools in ~/.cache/zero -- reuse, do not rewrite

```text
    census-6c5139205bd1.py   read-only census; must reproduce the line to beat before any pass
    pass5b-a539de398fd5.py   5B plan/apply, a fragment editor; its engine collision regex needs
                             narrowing (a new version, rehearsed)
    the apply engine         start and end markers searched independently over the whole file;
                             identical repeated blocks go as one counted fpatch call; an ASCII
                             patch() block at the end of a file. Rehearsed on copies first
    the commit gate          commit script = changed files exactly the plan, then cargo test
                             --no-fail-fast, rustfmt, nsh-test; any red refuses
```

### What this session added to the method

```text
    one reply     plan (read-only), then apply + ship + reload + gated commit in one reply
    the gate      a commit cannot land red, even when every block is pasted at once
    recon         every command guards for a missing binary (Omarchy has no crontab)
    pairing       a plan guard pairs identical literals only; a test checking the same concept
                  with a different literal is caught only by cargo test -- so the gate is a door
    stand-ins     must carry the real file's shapes: repeated identical blocks, box-drawn runs,
                  unrelated uses of a new name
    collisions    scope a collision check to the files that define the contract, not the crate
```

## 2026-09-28, PASS 1 -- THE COMMENTS ARE DONE; 79 HELD LINES WAIT FOR THEIR PASSES -- superseded by the 2026-09-29 PASSES 2, 3 AND 5A section above

READ THIS FIRST WHEN PICKING UP. It supersedes "THE CENSUS, AND THE PASSES THAT FINISH IT" above,
whose rulings, passes, findings and method still stand. The script that wrote this refused to
write unless the four pass-1 commits exist with their trailers, the pass-1 scope counts exactly
79, the census counts exactly 2,061 lines in 232 files, and the tree was clean and pushed.

### Rulings, Christian 2026-09-28 (this session)

```text
    new intents   RULED -- the PROPOSED line of the night record: no new intent until 247
                  closes, unless it holds a ruling 247 itself needs. None was filed this session
    the standing  do not break the system, the shell or the tools. Every pass-1 apply proved
    ask           mechanically that no code changed before it wrote a byte
```

### Done this session -- PASS 1, the comments

```text
    9ab65305   1A novashell: 141 hit lines, 120 handled, 21 held. Plan eda09f903349
    e6a3bfb8   1B engine: 130 hit lines, 98 handled, 32 held; core --help reworded with them.
               Plan 8364ac06ec68
    af845b54   1C the other 18 rust-tools crates: 120 hit lines, 91 handled, 29 held.
               Plan fc71a77080fd
    e302bbab   1D zero-gate's Why section: its 3 NixOS lines rewritten. Plan a7492be51129
```

Pass 1 took 391 comment lines, whole-line and trailing, naming faelight, forest or NixOS in 20
crates. 312 are handled. The 79 left are EXACTLY the held lines below, each owned by a later pass.
Doors on every commit: cargo test --workspace all ok, ship 0 failed, nsh-test 202/202, d 0 failed,
and the apply's own proof that the comment-stripped files are identical before and after.

### THE CENSUS at the end of pass 1 -- the line to beat

```text
    2,061 live lines in 232 files (2,311 at the start of the session, 2,269 in the night record)
      715  markdown in live intents   excluded by ruling B; 488 are INT-247 and INT-252 themselves
      547  markdown, docs
      329  Rust strings
      319  Rust code and identifiers  ForestDb alone is 181
       68  TOML
       63  Rust comments              was 305
       20  scripts and other
    1,346 lines are the work (was 1,596). 264 distinct names (was 279)
```

### The 79 held lines, by the pass that owns them

```text
    pass 2   12   ForestDb in comments, is_forest_pipeline, forest_present
    pass 3    8   golden.rs's test input; the ruled-kind label "forest value pipeline", a string in
                  plan.rs and migrate_audit.rs; deps' faelight- prefix test; nsh-test's fixture;
                  readme.rs's subtitle note (ruling F)
    pass 4   20   comments on live zombie code -- the list below
    pass 5   12   FAELIGHT_STATE_DB, FAELIGHT_STATE_DIR, the FAELIGHT_ prefix, forest-stats, the
                  org.faelight.Forest D-Bus names
    pass 6   14   the forest_ tables and friday's stored nixos and forest values
    pass 7    6   zero-doctor's compatibility-link probe
    pass 9    3   the sayings: novashell main.rs, scripting.rs, engine stress
    pass 10   4   the retired-name guard tests
```

### Found for pass 4 (NO ZOMBIE CODE) -- live code on paths nothing writes

```text
    novashell   main.rs adds /run/current-system/sw/bin and the per-user profile bin to PATH;
                prompt.rs colours a cwd containing /nix; the type and which arms read
                0-core/scripts, deleted in e733287d
    engine      the nix domain and its help line wrap nixos-option; strategy factor 7 checks
                scripts/faelight-context and a memory path under scripts; friday seeds NixOS
                facts; entropy tracks two absent packages
    tools       deadwood scans pkgs/faelight/scripts; zero-release reads the /nix system profile;
                zero-update's snapshot flag, flake.lock drift and nix store cleanup; nsh-test's
                fixture names zero/packages/faelight, which is not tracked
```

### Next session, in order

```text
    1  re-run the census at HEAD with the same definitions -- the line to beat is 2,061
    2  pass 2, the identifiers: ForestDb -> StateDb first, then ForestHelper, ForestDeployIface,
       faelight_root, C_DIR_FOREST, forest_present, is_forest_pipeline and the nsh-test case
       names (repl_206_forest_home..., repl_230_absent_forest...), through the compiler, which
       finds every use. The 12 pass-2 comment lines change with their names
    then the passes in order, one kind per plan
```

### What pass 1 added to the method

```text
    comments-only  the apply strips every comment from each file, before and after; the code
                   must be identical token for token or nothing is written
    review         read where every deleted comment run ENDS: one ran into the next item's live
                   note (completion.rs, pkg-search) and was bounded by hand before the apply
    counts         merged blank lines are counted in the summary, so the diffstat is predicted
                   exactly
    head           when the commit before a plan is not made yet, the plan checks HEAD's subject
                   instead of a hash
    splitting      one plan per crate group (novashell, engine, the rest) kept each review short
```

## Success Criteria

- [x] LAYER 0 landed: the freeze is written into AGENTS.md or CONVENTIONS.md as a rule, not a
      plan. Nothing new is named faelight from that commit onward
      <!-- evidence: 2026-09-15. AGENTS.md section "LAYER 0 -- THE FREEZE. THIS IS A RULE, NOT A PLAN." -->
- [x] The NAME is decided in every register it has to survive, with the constraints measured
      rather than assumed
      <!-- evidence: 2026-09-14. `export 0_FOO=bar` -> "not a valid identifier"; cargo -> "invalid character `0` in package name". A binary named `0`, the directory `0-core` and the glob `0-*/` all work. Project 0 / 0-core / 0 / zero-* / ZERO_*. -->
- [x] Week 1 census exists as `docs/inventory.md`: every crate with keep / replace / retire, the
      four decision-test answers, last invocation, and whether it runs under DevBox
    <!-- evidence: 2026-09-30. docs/inventory.md rewritten whole from measurement at 4a203d4d: all 18 registry tools, each with a disposition, the four decision-test answers, invocations since 2026-08-26 through the name, the pre-rename name and every alias (shell_history), and the DevBox census run 2026-09-30 -- 14 passed, 0 failed, 3 undetermined (db-browse, friday-chat, zero-ade); core has no case. Libraries listed apart per the 2026-09-30 ruling. -->
- [x] The COUNT is reconciled. README, the generated catalog, tools.toml and the actual tree agree
      on how many tools exist. Today they do not
    <!-- evidence: 2026-09-30. Ruled: a tool is a binary Project 0 builds and deploys. Measured after ship: tools.toml rust+deployable+not retired 18, binary crates in the tree 18, binaries on PATH 18, the catalog (zero-docs readme-index, counting binaries) 18, README 18. zero-daemon became deployable (kept for Friday); zero-zone's retired binary target and its devbox census case were removed and the binary retired from PATH, its library kept. -->
- [x] faelight-fm is gone -- workspace, PATH, docs, teach, Friday facts, command registry, and the
      Hyprland bind. Moved to `retired/` or deleted, NOT commented out
      <!-- evidence: 2026-09-15. Crate deleted, binary retired with `ship --retire`, registry marked retired = true, aliases fm/fmd removed, census case deleted. deadwood reports registry orphans clean. faelight-glog went with it on the same evidence. 9,893 lines removed; 193/193 green after. -->
- [x] The NixOS-era crates are gone by the same standard. The machine has not been NixOS since
      2026-08-26
      <!-- evidence: 2026-09-29. The last one was the engine nix domain (INT-088's Nix Inspector over nixos-option): deleted in f92c664d with pub mod nix, Command::Nix and NixCommand, Commands::Nix and NixCommands and the dispatcher arm; ship deployed core and its --help no longer names NixOS; its only caller, alias inspect, left config.nsh. zero-vm was retired in 0d35dc3d. Swept 2026-09-29: no live file outside the intents and CHANGELOGs names core nix, nix inspect, Nix Inspector, domains::nix or NixCommand. -->
- [x] `faelight-docs` cannot resurrect a retired tool. Proven by running it after a retirement and
      confirming the catalog does not list it
      <!-- evidence: 2026-09-15. gather_all() reads rust-tools/*/Cargo.toml from DISK, so a deleted crate cannot appear. Proven by running readme-index after three retirements: faelight-fm, faelight-glog and faelight are all absent from the catalog. -->
- [x] Layer 3 is NOT started until layers 0-2 are done and a path audit lists every hardcoded
      reference. The audit is a deliverable in its own right
      <!-- the audit is DONE 2026-09-15 -- see THE PATH AUDIT above: 6 functions in paths.rs, 26 live sites outside it, 14 of them the health cache. Layers 0-2 are also done. The gate is open for 3a. -->
    <!-- evidence: 2026-09-30, from this record. THE PATH AUDIT section, written 2026-09-15, lists the six paths.rs functions and the 26 live sites outside it. Layers 0-2 were done the same day (LAYERS 1 AND 2 -- DONE 2026-09-15); Layer 3a consolidated after the audit (2026-09-15/16) and 3b's alias started 2026-09-17. -->
- [x] The state alias runs for A FULL WEEK with `core doctor` and `nsh history` green before any
      code default changes. Evidence: the dates
      <!-- evidence: 2026-09-24. Alias created 2026-09-17, seven days of ordinary use. Measured today BEFORE any code default changed: d 92%, 25/28, 0 failed, Zero Alias probe green (state and config aliases resolve to the faelight directories); nsh -c history -> 104 lines; state.db is ONE inode under both names (59:41436, 319037440 bytes); sole holder nsh pid 123517. Baseline HEAD e9eb743a, pushed, tree clean, nsh-test 200/200. -->
- [x] Whatever reads the state paths reports UNREADABLE as unreadable. A silent empty ledger is
      the failure this intent most needs to avoid, and it is INT-192's collapse in a new place
    <!-- evidence: 2026-09-30. Every state.db opener except NovaShell StateDb::open, the owner that creates the schema, now opens through zero_core::state_db, which never creates a file and names Absent, Unreadable and NoSchema (441b2d22; six tests, red 4 failed on the old call, then green). b1d22495 moved 40 openers in 10 crates; the engine (6, runtime::init included, ruled a non-owner) and zero-insightd (1) moved in the commit that ticks this gate. Door, red then green, ZERO_STATE_DB at a missing path: zero-docs log created a 20480-byte ledger and said No doc history yet, after: Cannot open state.db and no file; core version created a 32768-byte ledger and printed Friday: 0 facts, after: Failed to initialize: state.db absent at /tmp/zero-absent.db and no file. Census at commit: no live Connection::open outside the owner. -->
- [x] The `faelight` unified CLI is decided: renamed to `0` with the same subcommands, or deleted
      with `nsh` and `core` called directly. Written down either way
      <!-- evidence: ruled 2026-09-28 (ruling D, adopted as recommended): DELETED -- see LAYERS 1 AND 2, 2026-09-15, where every gate of the decision test is answered and the CLI retired. A command named 0 is its own future intent. -->
- [x] No layer was done in the same week as another. If one was, say so here and say why -- the
      pace rule is a gate and breaking it is a thing to record, not hide
      <!-- evidence: 2026-09-29. BROKEN, AND RECORDED. Layers 0, 1 and 2 all landed 2026-09-15; layer 3 began 2026-09-17 with the state alias, inside the same week; passes 2 through 5D, the reload fix, STATEDB and NO NIXOS A, B1 and B2 all landed 2026-09-29. Why: broken on purpose to finish the rename, as the 2026-09-29 START HERE records. What held: every pass went through its own plan, fingerprint, doors and pushed commit. -->

- [ ] THE FINISH LINE, ruled by Christian 2026-09-24: no LIVE file, path, identifier, table or
      command says faelight or forest. History is exempt by the standing rule -- completed intents,
      CHANGELOGs and git history are never rewritten. Each name is held out by a guard once its pass
      is finished, and the guard reads every file type the name lives in, not only .rs

- [x] STATEDB, ruled 2026-09-28: ForestDb is StateDb everywhere, and no live Rust identifier
      contains Forest or Faelight in any case
- [x] NO NIXOS, ruled 2026-09-28: no live file names NixOS, nixos-rebuild, flake.nix, build-vm or
      /nix/ -- code, comments and the WAS REMOVED HERE tombstone notes included; history exempt
    <!-- evidence: 2026-09-30. The census (git ls-files; intents and CHANGELOGs excluded; nixos, flake.nix, build-vm, /nix/ in any case) finds no line outside the exemptions. The hits left are all exempt: the font's 2 glyph names, AGENTS.md:40, and tools.toml:29, :93, :103 inside retired = true entries. The last live lines were friday/mod.rs 142-144 and 1007-1009, comments, rewritten in this step. -->
- [x] NO ZOMBIE CODE, ruled 2026-09-28: no live code spawns a retired or absent binary, reads a
      path nothing writes, sits on a path nothing reaches, or leaves a child unreaped. A guard
      reads tools.toml's retired names and fails on any Command::new naming one -- seen red first.
      The dead-path finding (execute_and_record's timing and notification) is resolved: rewired
      onto the path commands take, or deleted
    <!-- evidence: 2026-09-29: zero-gate retired-spawns gate green on the tree (seen red first on a zero-release probe, bc20fdb8); every spawned name absent from PATH is a third-party optional tool whose absence is reported or skipped: bacon, cargo-cache, flatpak, pip, pipx, rustup; discarded spawns in nsh and zero-daemon reaped on threads; execute_and_record timing and notification rewired onto the spine via record_timing (2e7c6ebf); the pass-4 zombie list resolved in b627f4d4, eceb401d, f4d74c48, bc20fdb8 and 2e7c6ebf -->
- [x] /ETC/FAELIGHT: no live file names /etc/faelight. What /etc/zero holds, and the system-run
      writer that writes it, is INT-268's
    <!-- evidence: 2026-09-29, over every tracked file with history (the five history intent dirs, CHANGELOGs) and live intents (ruling B) excluded: 0 lines name /etc/faelight; AGENTS.md's was the last -->
- [ ] THE GUARD'S EXEMPTIONS, ruled 2026-09-28, and nothing else: history (completed, decision,
      philosophy, cancelled and incident intents; CHANGELOGs; git), live intents (ruling B),
      registry entries with retired = true, zero-gen's dictionary word forest (ruling E), and --
      ruled by Christian 2026-09-30 -- font files (third-party glyph names) and AGENTS.md (his file)

## Relationship

- PRIORITY, ruled by Christian 2026-09-28: the faelight, forest and NixOS removal comes BEFORE
  every other intent, the shell included. It replaces "below the shell, always", written while
  the rename was spelling; the rename is now the finish line of the project's name.
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
