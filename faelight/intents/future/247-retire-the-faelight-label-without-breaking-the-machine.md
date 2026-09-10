---
id: 247
date: 2026-09-09
type: future
title: "retire the Faelight label without breaking the machine"
status: planned
tags: [naming, migration, infrastructure, zero-core]
---

## Vision

The project is Project 0. `faelight` is a name it used to have, and one day nothing in the tree
says it except history. Getting there costs nothing if it is done slowly and could cost the
machine if it is done fast.

⭐ THIS IS DELIBERATELY NOT URGENT, AND THAT IS THE MOST IMPORTANT LINE IN THIS FILE. The label
costs nothing to keep. The shell is the priority, job control is the real deadline, and the
failure mode of rushing a rename is a machine that does not boot or a ledger that silently reads
empty. Slow is not a compromise here -- it is the requirement.

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

README, ROADMAP, AGENTS.md, the catalog generator. Project name becomes Project 0; a subtitle may
honestly say *formerly Faelight Forest*.

⚠️ AND STOP GENERATING FICTION. `faelight-docs` currently produces a catalog listing
`faelight-shell` when the crate has been `novashell` since 2026-09-01. A generator that
resurrects old names is a machine for undoing this intent.

### LAYER 2 -- BINARIES AND CRATES YOU STILL RUN

⭐ DO NOT RENAME A CRATE FOR SPELLING. `faelight-git` does not become `zero-git` because the
prefix is ugly. When a crate is REWRITTEN OR SPLIT, the new crate gets the new name. That is
exactly how faelight-shell became novashell without anyone doing a rename pass.

Old crate directories stay until they are DELETED, not until they are renamed.

### LAYER 3 -- STATE PATHS, LAST AND MOST CAREFULLY

⚠️ THIS IS THE ONE THAT CAN BREAK THE MACHINE. Alias first, flip later:

    ~/.local/state/zero  ->  ~/.local/state/faelight
    ~/.config/zero       ->  ~/.config/faelight

Only flip the code default after `core doctor` and `nsh history` both work through the alias for
A FULL WEEK.

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

## THE DECISION TEST -- ask these in order, every time

    1. Did I run this after 2026-08-26, on Omarchy?
    2. Does NovaShell or DevBox depend on it?
    3. Does Omarchy, or Flea/Yazi, already do the job better?
    4. If I deleted the crate tonight, what breaks tomorrow morning?

If (1) is NO and (4) is "nothing", it goes to `retired/` that week.
If (3) is YES, the replacement lands FIRST and the delete follows in the SAME week -- not after
the old one is polished.

## WHAT NOT TO DO -- each with the reason, so it survives being re-argued

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

## Success Criteria

- [ ] LAYER 0 landed: the freeze is written into AGENTS.md or CONVENTIONS.md as a rule, not a
      plan. Nothing new is named faelight from that commit onward
- [ ] Week 1 census exists as `docs/inventory.md`: every crate with keep / replace / retire, the
      four decision-test answers, last invocation, and whether it runs under DevBox
- [ ] The COUNT is reconciled. README, the generated catalog, tools.toml and the actual tree agree
      on how many tools exist. Today they do not
- [ ] faelight-fm is gone -- workspace, PATH, docs, teach, Friday facts, command registry, and the
      Hyprland bind. Moved to `retired/` or deleted, NOT commented out
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
- [ ] The `faelight` unified CLI is decided: renamed to `zero` / `0` with the same subcommands, or
      deleted with `nsh` and `core` called directly. Written down either way
- [ ] No layer was done in the same week as another. If one was, say so here and say why -- the
      pace rule is a gate and breaking it is a thing to record, not hide

## Relationship

- ⏭ PRIORITY: BELOW INT-245 AND INT-246, and far below job control. Both of those are correctness
  in the shell and the sandbox; this is spelling. If a session has to choose, this loses. Recorded
  so the ordering survives the enthusiasm of whoever picks it up
- INT-245 and INT-246 share this intent's central failure mode: something unreadable or
  unenforceable answering as though it were empty or applied. Layer 3's silent-empty-ledger risk
  is the same defect, which is why it is last
- DevBox (INT-167 lineage) is the instrument, per the section above
- The naming policy in the existing identity notes stands: Faelight is not renamed WHOLESALE. This
  intent is how it is retired gradually instead
