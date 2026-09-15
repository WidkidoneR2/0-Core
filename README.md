<!-- DYNAMIC SECTION - Updated by bump-system-version -->

# 🌲 Faelight Forest 1.0.0

![Version](https://img.shields.io/badge/version-1.0.0-green?style=flat-square)
![Rust](https://img.shields.io/badge/Rust-96.5%25-dea584?style=flat-square)
![Lines](https://img.shields.io/badge/lines-125k-blue?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)

> **A self-aware personal computing environment built from first principles. Pure Rust. No Electron. No telemetry.**

## 🍂 Migrating to Omarchy (Arch), August 2026

The 1.0.0 release ran on NixOS. It does not any more.

On 2026-08-26 the machine was wiped and reinstalled on Omarchy, and the forest moved
with it. What that migration is finding, in the open: checks that could not fail,
counts that disagreed with each other, tools whose purpose left with the platform, and
a test suite that reported a 2% shell because its harness had lost the binary it was
meant to test.

Ten tools retired, seventeen intents cancelled with recorded reasons, and the health
panel stopped measuring a distribution it does not own. The work is ongoing and the
commit history is the record.

The next chapter is the shell. Forty-one binaries built at the migration; thirty do
now, and that is the number one person can keep honest. The shell is where the
attention goes.

_Release notes for 1.0.0 and everything since live in the [changelog](faelight/meta/CHANGELOG.md)._

## 🌲 Forest DNA

| | |
|---|---|
| 🛠 **Tools** | 30 custom Rust tools |
| 📋 **Codebase** | ~125k lines of Rust |
| ⚡ **Stack** | Rust · Wayland · ratatui · SQLite |
| 🌍 **Philosophy** | Understanding over convenience · No mystery packages |

> Every tool written or fully understood. Nothing runs blindly.

[Full Changelog →](faelight/meta/CHANGELOG.md)

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

**Twenty-six crates**, and honestly: most have not been touched since the Omarchy migration in
August 2026. The shell needed the attention, so the shell got it. That is a statement of where
the work went, not a claim that the rest is finished.

**~141,000 lines of Rust across 284 files**, with small amounts of Lua and shell where they
serve best. Rust not for its own sake -- Rust because understanding every line is the point.

    POSIX shells   text  -> text   -> text
    Nu shell       table -> filter -> transform
    nsh            table -> filter -> judgment, and a refusal when it cannot answer

The third line is the only one this project had to earn rather than adopt.

## Origin

Project 0 began in August 2026, when the machine moved to Omarchy and the real question turned
out to be what to carry over.

The answer was: less than expected.

**Faelight Forest was not a failure.** It was more projects than one person could keep honest at
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
4. **The forest remembers** -- every commit, decision, and intent is documented and learned from.

This is stewardship, not consumption: the forest is tended intentionally, every part known.

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

Around these sit the remaining crates -- git governance, a release manager, a credential vault,
the sandbox. `docs/inventory.md` says which are used, which are kept for a stated reason, and
which are neither.

**See the full, always-current tool catalog:** [rust-tools/](faelight/rust-tools/)

## Going deeper

This README is the front door. The depth lives here:

- [Theory of Operation](docs/THEORY_OF_OPERATION.md) -- how the forest thinks
- [Architecture](docs/ARCHITECTURE.md) -- how the pieces fit
- [Philosophy](docs/PHILOSOPHY.md) -- why it is built this way
- [Shell Philosophy](docs/NSH-PHILOSOPHY.md) -- the case for a human-first shell
- [Release Process](docs/RELEASE.md) -- how the forest publishes itself
- [Tool Catalog](faelight/rust-tools/) -- every active tool, generated from source
- [Inventory](docs/inventory.md) -- what is used, what is kept, and what the numbers say
- [Changelog](faelight/meta/CHANGELOG.md) -- the full history, Arch era through NixOS to Omarchy

## Security

Nothing runs without explicit authorization.

- UFW firewall + fail2ban active
- faelight-vault -- encrypted credential manager
- faelight-sandbox -- policy engine with namespace isolation
- Health + integrity monitoring -- continuous verification
- cargo-audit on every deploy -- findings surfaced, triaged, and documented, never silent

## The decision record

Every intent is documented -- not just what was built, but why, when, what the health score
was, what risk was accepted, and what happened next. The forest does not forget.

## License

MIT -- see [LICENSE](LICENSE). Use it, learn from it, build on it.

---

*Every tool written or fully understood. Nothing runs blindly.*
🌲
*Auto-generated by faelight-docs v2.0.0 — last sync: 2026-09-15 18:55*
