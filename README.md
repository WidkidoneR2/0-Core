<!-- HEADER - hand-written; zero-docs never touches above the END marker -->

# Project 0

![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)

> **Omarchy is the base. Project 0 is only what is needed on top of it.**

## Where it stands

On 2026-08-26 the machine was wiped and reinstalled on Omarchy. What the move found, in the
open: checks that could not fail, counts that disagreed with each other, tools whose purpose
left with the old platform, and a project name that no longer fit. Tools were retired with
recorded reasons, the old name is being removed from every live file, and the commit history
is the record.

The attention goes to the shell.

_Release notes and everything since live in the [changelog](zero/meta/CHANGELOG.md)._

## Project 0 DNA

| | |
|---|---|
| 🧱 **Base** | Omarchy -- Arch, Hyprland, systemd |
| 🐚 **Shell** | NovaShell (`nsh`), with `bash` as the login shell |
| 🛠 **Tools** | each one listed in [docs/inventory.md](docs/inventory.md) |
| ⚡ **Stack** | Rust · SQLite · ratatui |
| 🌍 **Philosophy** | Understanding over convenience · No mystery packages |

> Every tool written or fully understood. Nothing runs blindly.

[Full Changelog →](zero/meta/CHANGELOG.md)

---

<!-- END DYNAMIC SECTION -->

<!-- STATIC SECTION -->

## The shell

The work is the shell now. **NovaShell** -- `nsh` -- is a structured shell: commands return
tables rather than text, and the pipeline filters values instead of re-parsing strings.

```
ps | where pgid == 1

  name     cpu  memory  pgid  pid  status  user
  systemd  0.0  0.0     1     1    Ss      root
```

### Job control

As of September 2026, `nsh` owns process groups and the terminal:

```
sleep 300
^Z
  ☸ [1] sleep -- suspended (fg 1 to resume)

jobs
  [1] stopped   sleep  (pgid 46310, 8s elapsed)

bg 1
  ▲ [1] sleep -- continued in the background
```

A pipeline is ONE job in one process group. A suspended job is registered and resumable, not
announced and lost. Job state is OBSERVED per process, never inferred -- a job the shell cannot
account for says `UNKNOWN` with its reason rather than vanishing from the table.

### What it is not

`nsh` is **not POSIX** and does not pretend to be. It is **not a login shell**: `bash` is still
what `/etc/passwd` names, deliberately, so a shell that cannot start costs a terminal tab rather
than a session. And where it has not modelled a construct, it hands the line to `sh` rather than
guessing.

The principle the shell keeps relearning: **a tool that cannot answer must say so**, rather than
reporting an answer it never established.

---

## What is Project 0?

Only what is needed, and nothing carried over out of habit.

Omarchy is the base -- someone else's good work, kept. Project 0 is the layer on top: a shell
being made good, a sandbox that tests it, and the tools that survived asking "what breaks
tomorrow if I delete this?"

The tools are counted in [docs/inventory.md](docs/inventory.md), not here: a count written into
a README is wrong the day after, and this one once said thirty, then thirty-eight, while the
tree held twenty-six. Most have had little attention since the Omarchy move. The shell needed
it, so the shell got it -- a statement of where the work went, not a claim that the rest is
finished.

Rust not for its own sake -- Rust because understanding every line is the point.

    POSIX shells   text  -> text   -> text
    Nu shell       table -> filter -> transform
    nsh            table -> filter -> judgment, and a refusal when it cannot answer

The third line is the only one this project had to earn rather than adopt.

## Origin

Project 0 began in August 2026, when the machine moved to Omarchy and the real question turned
out to be what to carry over.

The answer was: less than expected.

**What came before was not a failure.** It was more projects than one person could keep honest at
the pace they were arriving -- eight or nine months of ideas, each worth building, none with
enough attention left over. Nothing was broken. Everything was half-tended, which is a different
problem and a harder one to see.

Project 0 is the same person with a shorter list.

The principle underneath has not moved: build it from parts you understand, or do not run it at
all. What changed is the admission that understanding has a budget, and that a system which
claims to understand itself has to be tested by taking the ground out from under it.

## Philosophy

Four principles govern everything:

1. **Understanding over convenience** -- if you don't understand it, it doesn't run.
2. **Manual control over automation** -- nothing happens without explicit authorization.
3. **Intentional design** -- every tool has a purpose; every decision has a record.
4. **Project 0 remembers** -- every commit, decision, and intent is documented and learned from.

This is stewardship, not consumption: every part is known and tended on purpose.

## The thesis

A shell can know what it did.

Not "logged it" -- KNOWN it: which session, which command, what it exited with, what you ran
next. `nsh` has 200,000 commands of that, and the tool inventory in `docs/inventory.md` was
decided from it rather than from memory. Counting only tool names said twelve crates were dead;
counting the ALIASES that actually reach them said nine, and three working tools were three
weeks from a wrong deletion.

No other shell on this machine could have answered that question about itself.

## Architecture

Three pieces carry the weight:

- **nsh (NovaShell)** -- the shell. Structured values, its own job control, and a habit of
  refusing rather than guessing.
- **core** -- one Rust engine of native domains: health, the intent ledger, integrity.
- **Friday** -- the intelligence layer. Partly built, deliberately quiet, and not claimed here
  as finished.

Real commands, each copied from a working terminal:

```sh
ps | where pgid == 1              # structured: filter processes by process group
fsearch "pub fn" --type rs | count   # search returns rows, not lines
signals | where handling != "default"  # what this process does with each signal
? show health                     # natural language -- PROPOSES, then asks before running
```

The last one matters more than it looks. `?` translates and shows you the pipeline with a
confidence level; it does not run anything until you say yes. When it has no pattern, it says so
instead of guessing.

Around these sit the remaining tools -- git governance, the sandbox, the doc engine.
`docs/inventory.md` says which are used, which are kept for a stated reason, and which are
neither.

**See the full, always-current tool catalog:** [rust-tools/](zero/rust-tools/)

## Going deeper

This README is the front door. The depth lives here:

- [Architecture](docs/ARCHITECTURE.md) -- how the pieces fit
- [Philosophy](docs/PHILOSOPHY.md) -- why it is built this way
- [Shell Philosophy](docs/NSH-PHILOSOPHY.md) -- the case for a human-first shell
- [Tool Catalog](zero/rust-tools/) -- every active tool, generated from source
- [Inventory](docs/inventory.md) -- what is used, what is kept, and what the numbers say
- [Changelog](zero/meta/CHANGELOG.md) -- the full history, Arch era to Omarchy

## Security

Nothing runs without explicit authorization.

- Firewall on and sshd key-only -- both checked on every `d` run
- zero-sandbox -- policy engine with namespace isolation
- Health and integrity monitoring -- `d` names what it could not determine rather than passing it
- cargo-audit -- findings surfaced, triaged, and documented, never silent

## The decision record

Every intent is documented -- not just what was built, but why, when, what the health score
was, what risk was accepted, and what happened next. Project 0 remembers. The human decides.

## License

MIT -- see [LICENSE](LICENSE). Use it, learn from it, build on it.

---

*Every tool written or fully understood. Nothing runs blindly.*
*Auto-generated by zero-docs v2.0.0 — last sync: 2026-09-15 18:55*
