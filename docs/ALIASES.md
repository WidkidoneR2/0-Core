# Alias Reference -- Zero Core

**Updated:** 2026-09-01
**System:** Omarchy (Arch) + nsh 3.8.4

---

## There is no list here, and that is the point

This file used to carry a table of fifty aliases, generated once in June 2026
and never again. By September the config held **270** and the table still said
fifty -- listing `faelight-bar`, which no longer runs, and `bump`, marked
disabled pending an intent that has since closed.

A snapshot of a live file is wrong the moment the file changes, and nothing
here noticed for three months.

## Where the aliases actually are

    ~/.config/faelight-shell/config.nsh

That file IS the source of truth. INT-060 made it so: nsh seeds its alias table
from the config at every startup and PRUNES anything absent from it, so a
runtime `alias` is ephemeral by design and permanence lives in the file.

## How to read them

    cheat                 the cheatsheet TUI -- reads config.nsh live
    alias                 what this session has loaded
    grep '^alias' ~/.config/faelight-shell/config.nsh

`cheat` is the one to reach for. It parses the deployed config directly rather
than the alias table, because the table is only refreshed at startup and can be
stale at deploy time.

## The rules, which have not changed

- Every alias must be used regularly or it gets removed
- Aliases document intent, not just shortcuts
- Forest tools get short aliases (`d`, `fm`, `nt`)

`faelight-deadwood` flags any alias whose target no longer resolves. It caught
two during the nsh rename: `ft` pointing at `fsh-test` and `fs` at
`faelight-shell`, both hours after their targets were deleted.
