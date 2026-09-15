# Tool inventory -- INT-247 Week 1 census

Generated from three INDEPENDENT signals. No one of them is sufficient alone, and the
disagreements between them are the reason this document exists.

    used      invocations since the Omarchy migration, through the tool's own name AND
              every alias that reaches it -- from shell_history
    commits   edits to the crate since 2026-08-26 -- from git
    clean     does it answer in a clean room -- from `faelight-sandbox verify devbox/census`

## The correction that made this worth doing

Counting only each tool's own name, TWELVE crates showed zero use. Counting the aliases that
actually reach them, three of those were in daily use:

    faelight-vm     0 -> 8    reached by `vm`
    faelight-git    3 -> 9    reached by `fg`, `fga`, `fgc`, `fgp`, `fgs`
    faelight-ade    0 -> 2    reached by `ade`

Three wrong retirements avoided by asking the question in the right vocabulary. A census that
measures what the author TYPES rather than what the author RUNS is measuring a habit.

⭐ AND IT CUT THE OTHER WAY TOO. `faelight-glog` looked alive because `glog` is typed often --
but `glog` is `git log --oneline -10` and never pointed at that tool at all. It had been
replaced without being retired, and only the alias map showed which was which.

## The table

| crate | used | commits | reached by |
|---|---:|---:|---|
| `ship` | 367 | 6 | -- |
| `nsh-test` | 75 | 20 | `nt` |
| `novashell` | 73 | 69 | -- |
| `faelight-deadwood` | 66 | 7 | `dw` |
| `zero-gate` | 22 | 5 | -- |
| `faelight-docs` | 21 | 4 | `docs-check`, `docs-status`, `docs-sync`, `fdocs` |
| `faelight-daemon` | 20 | 5 | `f-daemon` |
| `teach` | 15 | 2 | `t` |
| `faelight-sandbox` | 14 | 15 | `sb`, `sb-clear`, `sb-diff`, `sb-restore`, `sb-snap`, `sb-snaps`, `sb-status` |
| `faelight-git` | 11 | 0 | `fg`, `fga`, `fgc`, `fgp`, `fgs` |
| `faelight-vm` | 8 | 0 | `vm` |
| `faelight-update` | 6 | 6 | `fu`, `fudr`, `fui`, `fuup`, `update` |
| `faelight` | 4 | 3 | `f` |
| `faelight-ade` | 3 | 1 | `ade` |
| `friday-chat` | 2 | 1 | `fc` |
| `faelight-insightd` | 2 | 0 | -- |
| `faelight-gen` | 2 | 0 | `gen` |
| `db-browse` | 2 | 0 | `db` |
| `intent-guard` | 1 | 0 | `guard` |
| `faelight-core` | 0 | 7 | -- |
| `faelight-release` | 0 | 1 | `bump`, `fr-history`, `fr-preview`, `fr-status`, `release` |
| `faelight-zone` | 0 | 0 | `zone` |
| `faelight-wallpaper` | 0 | 0 | `wallpaper` |
| `faelight-vault` | 0 | 0 | `fva`, `fvg`, `fvl`, `vault` |
| `faelight-context` | 0 | 0 | `ctx` |
| `faelight-clipboard` | 0 | 0 | `cbh`, `cbp`, `clip` |

## Retired 2026-09-15 -- INT-247 Layer 2

| crate | replaced by | evidence |
|---|---|---|
| `faelight-fm` | yazi, Flea | 0 invocations via `fm`/`fmd`; ROADMAP already recorded yazi |
| `faelight-glog` | `git log --oneline -10` | 0 invocations via `fgl`; `glog` was never this tool |

Crates deleted, binaries retired with `ship --retire`, registry entries marked `retired = true`,
aliases removed. `faelight-deadwood` reports registry orphans clean. Documentation still mentions
both in ~12 files -- that is Layer 1 and waits its turn, per the intent's pace rule.

## Kept for a stated reason, not yet used since the migration

Three tools sit at zero invocations and are kept deliberately. All three belong to Project
Friday, which is a thing to return to rather than a thing running now:

| crate | what it is |
|---|---|
| `faelight-ade` | terminal plus a conversation with Friday |
| `friday-chat` | Friday alone, no terminal |
| `db-browse` | audit scores, logs, Friday's attention, and more |

⚠️ THE HONEST LINE IS "KEPT FOR A REASON", NOT "IN USE". A stated purpose is a good reason
to keep a tool and a bad reason to stop measuring it. If these are still at zero in three
months, this document should be able to say SO rather than letting the reason harden into
permanence unexamined. The number is the check on the intention.

⭐ db-browse's own entry names a job it is not doing: the audit scores it shows need updating
against the current tool list. A viewer of stale data is worth less than its zero suggests.

## Blocking is a finding about AUTOMATABILITY, not about worth

Three tools wait for input instead of answering `--version` or `help`:

    db-browse   faelight-ade   friday-chat

All three are interactive by design -- a chat UI that blocks is a chat UI working. But a tool
that cannot say its own name cannot be installed by automation, checked by a census, or
diagnosed on a machine that is misbehaving.

⭐ `zero-gate` SHOWS THE FIX AND IT IS SMALL. It answered `--version` only AFTER demanding a
git repository, so it passed here and failed in a clean room. Moving the version check ahead of
the environment check was four lines. Identity questions must be answerable on a broken system,
because a broken system is exactly when they get asked.

## Census results

Run: `faelight-sandbox verify devbox/census --allow-undetermined`

Two crates were repaired rather than recorded as failures, and both were message defects:

  - `ship` said "cargo did not run" while cargo sat at /usr/bin/cargo. The missing thing was
    the WORKING DIRECTORY -- it builds a repo the clean room hides on purpose. It now names the
    directory and where the path came from. `ship` cannot build without a repo, correctly.
  - `zero-gate` exited 0 here and 1 in the clean room. See above.
