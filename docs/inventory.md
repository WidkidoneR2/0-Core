# Tool inventory -- INT-247 Week 1 census

Generated 2026-09-14 from three INDEPENDENT signals. Not one of them is sufficient alone,
and the disagreements between them are the reason this document exists.

    used      invocations since the Omarchy migration, through the tool's own name
              AND every alias that reaches it -- from shell_history (201,442 rows)
    commits   edits to the crate since 2026-08-26 -- from git
    clean     does it answer in a clean room -- from `faelight-sandbox verify`

## The correction that made this worth doing

Counting only each tool's own name, TWELVE crates showed zero use. Counting the aliases
that actually reach them, three of those were in daily use:

    faelight-vm     0 -> 8    reached by `vm`
    faelight-git    3 -> 9    reached by `fg`, `fga`, `fgc`, `fgp`, `fgs`
    faelight-ade    0 -> 2    reached by `ade`

Three wrong retirements, avoided by asking the question correctly. A census that measures
what the author TYPES rather than what the author RUNS is measuring a habit, not a tool.

## The table

| crate | used | commits | reached by |
|---|---:|---:|---|
| `ship` | 367 | 5 | -- |
| `nsh-test` | 75 | 20 | `nt` |
| `novashell` | 73 | 66 | -- |
| `faelight-deadwood` | 66 | 7 | `dw` |
| `zero-gate` | 21 | 4 | -- |
| `faelight-docs` | 21 | 4 | `docs-check`, `docs-status`, `docs-sync`, `fdocs` |
| `faelight-daemon` | 20 | 5 | `f-daemon` |
| `teach` | 15 | 2 | `t` |
| `faelight-sandbox` | 12 | 15 | `sb`, `sb-clear`, `sb-diff`, `sb-restore`, `sb-snap`, `sb-snaps`, `sb-status` |
| `faelight-git` | 9 | 0 | `fg`, `fga`, `fgc`, `fgp`, `fgs` |
| `faelight-vm` | 8 | 0 | `vm` |
| `faelight-update` | 6 | 6 | `fu`, `fudr`, `fui`, `fuup`, `update` |
| `faelight` | 4 | 3 | `f` |
| `faelight-ade` | 2 | 1 | `ade` |
| `faelight-insightd` | 2 | 0 | -- |
| `faelight-gen` | 2 | 0 | `gen` |
| `intent-guard` | 1 | 0 | `guard` |
| `faelight-core` | 0 | 7 | -- |
| `friday-chat` | 0 | 1 | `fc` |
| `faelight-release` | 0 | 1 | `bump`, `fr-history`, `fr-preview`, `fr-status`, `release` |
| `faelight-zone` | 0 | 0 | `zone` |
| `faelight-wallpaper` | 0 | 0 | `wallpaper` |
| `faelight-vault` | 0 | 0 | `fva`, `fvg`, `fvl`, `vault` |
| `faelight-glog` | 0 | 0 | `fgl` |
| `faelight-fm` | 0 | 0 | `fm`, `fmd` |
| `faelight-context` | 0 | 0 | `ctx` |
| `faelight-clipboard` | 0 | 0 | `cbh`, `cbp`, `clip` |
| `db-browse` | 0 | 0 | `db` |

## Zero on every signal

These answer nothing, were edited by nobody, and were run by no one -- through any name:

- `faelight-zone` -- reached by `faelight-zone`, `zone`
- `faelight-wallpaper` -- reached by `faelight-wallpaper`, `wallpaper`
- `faelight-vault` -- reached by `faelight-vault`, `fva`, `fvg`, `fvl`, `vault`
- `faelight-glog` -- reached by `faelight-glog`, `fgl`
- `faelight-fm` -- reached by `faelight-fm`, `fm`, `fmd`
- `faelight-context` -- reached by `ctx`, `faelight-context`
- `faelight-clipboard` -- reached by `cbh`, `cbp`, `clip`, `faelight-clipboard`
- `db-browse` -- reached by `db`, `db-browse`

⚠️ THIS LIST IS EVIDENCE, NOT A DECISION. INT-247's decision test has a fourth question
this document cannot answer -- *if I deleted the crate tonight, what breaks tomorrow
morning?* -- and a desktop bind, a systemd unit or a script is not in shell history.
Layer 4 says grep the whole HOME before deleting anything, and that still stands.

## Cannot be asked anything

Five tools block waiting for input instead of answering `--version` or `help`:

    db-browse  faelight-ade  faelight-fm  faelight-glog  friday-chat

⭐ TWO INDEPENDENT MEASUREMENTS AGREE HERE. Every one of these is also at or near zero
use. A tool that cannot be scripted tends not to be run, and a tool nobody runs is not
noticed when it stops being scriptable. That is the same fact from both ends.

`faelight-ade` is the exception worth naming: 2 invocations, so it IS used -- interactively,
by hand, which is exactly what a blocking tool is good for. Blocking is a finding about
AUTOMATABILITY, not about worth.

## Still red in the clean room

- `ship` -- 367 invocations, the most-used tool here, and it fails `ship help` under devbox
- `zero-gate` -- exits 0 on this machine and 1 in a clean room

Both are real. `ship` matters most: it is how everything else is deployed.
