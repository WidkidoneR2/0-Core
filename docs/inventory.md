# Tool inventory

Measured 2026-09-30 at 4a203d4d, for INT-247. A tool is a binary Project 0 builds and deploys;
libraries are listed apart (Christian's ruling, 2026-09-30). This file is a MEASUREMENT: a number
here is only as good as the day it was taken, so rerun the three signals before trusting one that
is more than a month old.

## How it was measured

    used      rows in shell_history since 2026-08-26 (the Omarchy install), counting the tool's
              own name, its pre-rename name, and every alias in ~/.config/nsh/config.nsh whose
              first word is the tool
    callers   who runs it: Command::new sites and string literals in .rs files outside its own
              crate; git hooks and scripts; Hyprland, systemd user units, desktop entries
    clean     zero-sandbox verify devbox/census --allow-undetermined, run 2026-09-30:
              14 passed, 0 failed, 3 undetermined, 37.9s

Limits, stated rather than hidden: a command matches when the tool or alias is its first word, so
a tool run mid-pipeline is not counted. "Undetermined" means the case did not finish within the
census's 10 seconds -- a tool waiting for input, not a failure.

## The decision test

    1  Did I run this after 2026-08-26, on Omarchy?
    2  Does NovaShell or DevBox depend on it?
    3  Does Omarchy, or Flea/Yazi, already do the job better?
    4  If I deleted the crate tonight, what breaks tomorrow morning?

If (1) is no and (4) is nothing, the tool retires that week -- unless Christian rules otherwise,
and then the ruling is written here beside it.

## The eighteen tools

| tool | disposition | used since 08-26 | total | last | reached by | DevBox |
|---|---|---:|---:|---|---|---|
| `core` | keep | 946 | 5,870 | 2026-09-30 | `d`, `ints`, `inta`, `cistart`, `cicomplete` and 74 more | no case |
| `ship` | keep | 568 | 568 | 2026-09-30 | -- | passed |
| `nsh-test` | keep | 211 | 211 | 2026-09-30 | `nt` | passed |
| `nsh` | keep | 94 | 94 | 2026-09-26 | -- (the shell itself) | passed |
| `zero-deadwood` | keep | 68 | 117 | 2026-09-25 | `dw` | passed |
| `zero-docs` | keep | 30 | 102 | 2026-09-30 | `docs-check`, `docs-status`, `docs-sync`, `zdocs` | passed |
| `zero-gate` | keep | 30 | 33 | 2026-09-30 | -- (the pre-commit hook) | passed |
| `zero-daemon` | keep, for Friday | 23 | 36 | 2026-09-25 | `z-daemon` | passed |
| `teach` | keep | 19 | 24 | 2026-09-20 | `t` | passed |
| `zero-git` | keep | 18 | 1,170 | 2026-09-30 | `zg`, `zga`, `zgc`, `zgp`, `zgs` | passed |
| `zero-sandbox` | keep | 18 | 19 | 2026-09-25 | `sb`, `sb-clear`, `sb-diff`, `sb-restore`, `sb-snap`, `sb-snaps`, `sb-status` | passed |
| `zero-update` | keep for now | 9 | 118 | 2026-09-30 | `zu`, `zudr`, `zui`, `zuup`, `update` | passed |
| `zero-ade` | keep, for Friday | 5 | 47 | 2026-09-25 | `ade` | undetermined |
| `zero-gen` | keep | 3 | 3 | 2026-09-25 | `gen` | passed |
| `db-browse` | keep for now | 2 | 29 | 2026-09-15 | `db` | undetermined |
| `friday-chat` | keep, for Friday | 2 | 38 | 2026-09-15 | `fc` | undetermined |
| `zero-insightd` | keep, for Friday | 2 | 2 | 2026-09-02 | -- | passed |
| `zero-context` | keep by ruling | 0 | 2 | 2026-05-26 | `ctx` | passed |

No tool is marked replace or retire today. Twenty-three registry entries are already
`retired = true` in zero/registry/tools.toml; each retirement is recorded there and in INT-247.

## The four answers, per tool

| tool | 1 run since 08-26 | 2 NovaShell or DevBox depends | 3 done better elsewhere | 4 what breaks if deleted |
|---|---|---|---|---|
| `core` | yes | yes -- nsh spawns it (commands/mod.rs, engine.rs:369, health_tui.rs:218, main.rs:100) | no -- Project 0's own engine | `d`, the intent verbs, 79 aliases; hyprland.lua:10 names it |
| `ship` | yes | yes -- it deploys nsh | no -- builds this repo | the only deploy path |
| `nsh-test` | yes | yes -- it is NovaShell's suite | no | the pre-push gate (.githooks/lib/nsh-test-gate.sh), devshell |
| `nsh` | yes | yes -- it is NovaShell | no | the interactive shell, the pre-push gate, devshell |
| `zero-deadwood` | yes | yes -- an nsh-test case reads it | no | a `d` check: zero-doctor spawns it (probes/internals.rs:160) |
| `zero-docs` | yes | no | no | the catalog and docs/public; core integrity spawns it (integrity/mod.rs:234) |
| `zero-gate` | yes | no | no | every commit loses its gate (.githooks/pre-commit) |
| `zero-daemon` | yes | no | no | nothing spawns it; named in core deploy and doctor, nsh, zero-doctor's tools probe |
| `teach` | yes | no | no -- Christian's own learning tool | `t` |
| `zero-git` | yes | no | no -- it wraps git | core's git domain spawns it (git/mod.rs:262); five aliases |
| `zero-sandbox` | yes | yes -- it IS the DevBox census runner | no | the census; core's sandbox domain spawns it (sandbox/mod.rs:214, 225) |
| `zero-update` | yes | no | UNANSWERED -- omarchy-update overlaps; not compared | five aliases; nothing spawns it |
| `zero-ade` | yes | yes -- nsh spawns it (commands/mod.rs:16851) | no -- Friday's terminal | `ade` and that spawn |
| `zero-gen` | yes | no | no -- a generator Christian built | `gen` |
| `db-browse` | yes | yes -- nsh spawns it (engine.rs:179) | no | `db`; core dispatcher.rs:767 and nsh engine.rs:179 |
| `friday-chat` | yes | yes -- nsh spawns it (commands/mod.rs, engine.rs:403) | no | `fc` and nsh's Friday entry points |
| `zero-insightd` | yes | no | no | nothing measured; named in core's engines and strategy |
| `zero-context` | NO | no | no | nothing measured; named in core's strategy |

## Rulings on record

    2026-09-23   zero-gen kept: a self-built password generator that may go to Omarchy users
    2026-09-25   zero-insightd and zero-context kept, for Friday
    2026-09-28   teach kept -- Christian's own learning tool
    2026-09-30   a tool is a binary Project 0 builds and deploys; libraries are not tools
    2026-09-30   zero-daemon kept and deployable, to be connected with Friday
    2026-09-30   db-browse and zero-update kept for now

One line matters most: zero-context meets the retire rule -- (1) is no
and (4) is nothing -- and is KEPT by the 2026-09-25 ruling. A stated purpose is a good reason to
keep a tool and a bad reason to stop measuring it. If it is still at zero at the next measurement,
this file says so.

## Libraries -- not tools

| crate | linked by |
|---|---|
| `zero-core` | 18 crates: core, db-browse, friday-chat, novashell, nsh-test, ship, teach and every zero-* crate |
| `zero-doctor` | core |
| `zero-zone` | core -- its binary retired 2026-09-30; the library stays |

## Findings, each with its owner

    core has no census case        the most-used tool is the one the clean room does not check
    three tools cannot answer      db-browse, friday-chat, zero-ade wait for input; unchanged
    in a clean room                since 2026-09-15
    zero-update vs omarchy-update  question 3 is unanswered for it
    the z-prefix aliases           zg zga zgc zgp zgs, zu zudr zui zuup, zdocs, z-daemon --
                                   renamed from the f- prefix 2026-09-30; the counts above were measured
                                   under the old names. fg is job control only
    zero/registry/aliases.toml     read by ship (ship/src/main.rs:309) and stale: it lists tools
                                   that no longer exist and maps core to a cd. INT-265
    docs/NOVASHELL.md:216          says fg is not job control; nsh main.rs:250 already treats a
                                   bare fg and fg N as job control. The docs pass
