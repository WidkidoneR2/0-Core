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

## What is Faelight Forest?

A self-aware personal computing environment, built from first principles. Every piece a
modern desktop needs -- a shell, an intelligence layer, and 30 custom Rust tools --
written or fully understood. No mystery packages. No magic. No convenience at the
cost of comprehension.

**~97% Rust** (125k lines across 239 files), with small amounts of Lua and shell where
they serve best. The forest is not Rust
for its own sake -- it is Rust because understanding every line is the point.
POSIX shells:      text -> text -> text
Nu shell:          table -> filter -> transform
Faelight Forest:   forest_data -> judgment -> wisdom -> anticipation -> alignment

## Origin

Faelight Forest began in a failure. A catastrophic update broke a working system, and the
rebuild that followed asked a harder question than "how do I fix this?" -- it asked "why
don't I understand my own machine?" The answer became a principle: build it from parts you
understand, or don't run it at all.

That rebuild started on Arch Linux. In June 2026, after another Arch failure, the forest
migrated to **NixOS 26.05** -- a deliberate move toward declarative, reproducible,
rollback-safe computing.

In August 2026 it moved again, to **Omarchy**. The reasons are in the commit history
rather than in a manifesto. What the move made plain is that a system which claims to
understand itself has to be tested by taking the ground out from under it.

## Philosophy

Four principles govern everything:

1. **Understanding over convenience** -- if you don't understand it, it doesn't run.
2. **Manual control over automation** -- nothing happens without explicit authorization.
3. **Intentional design** -- every tool has a purpose; every decision has a record.
4. **The forest remembers** -- every commit, decision, and intent is documented and learned from.

This is stewardship, not consumption: the forest is tended intentionally, every part known.

## The thesis

A computing environment can be coherent, self-documenting, and self-aware -- grown one
intent at a time, with understanding rather than assembly at its core. Faelight Forest is
that proof, in daily use: a shell that speaks human, an engine that reasons about its own
health, and an intelligence layer that learns. Not text streams. Not configuration.
Structured wisdom.

## Architecture

The forest rests on three pillars, plus an ecosystem of tools:

- **fsh (faelight-shell)** -- the forest's own shell. Speaks human first, UNIX as fallback.
- **core** -- a single Rust engine of native domains: health, intent ledger, integrity,
  prediction, decisions, strategy.
- **Friday** -- an intelligence layer that watches, learns, and speaks only when confident.
  Persistent memory across sessions; confidence-gated voice.

```sh
? show health                  # natural language -> health dashboard
deploy core                    # intelligent deploy with audit
build ||| test                 # true parallel execution
friday where risk > medium     # Friday intelligence query
```

Around these sit 0 custom Rust tools -- compositor helpers, a GPU terminal, a file
manager, git governance, a release manager, a credential vault, a sandbox, and more.

**See the full, always-current tool catalog:** [rust-tools/](faelight/rust-tools/)

## Going deeper

This README is the front door. The depth lives here:

- [Theory of Operation](docs/THEORY_OF_OPERATION.md) -- how the forest thinks
- [Architecture](docs/ARCHITECTURE.md) -- how the pieces fit
- [Philosophy](docs/PHILOSOPHY.md) -- why it is built this way
- [Shell Philosophy](docs/FSH-PHILOSOPHY.md) -- the case for a human-first shell
- [Release Process](docs/RELEASE.md) -- how the forest publishes itself
- [Tool Catalog](faelight/rust-tools/) -- every active tool, generated from source
- [Changelog](faelight/meta/CHANGELOG.md) -- the full history, Arch era through NixOS and back

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
*Auto-generated by faelight-docs v2.0.0 — last sync: 2026-07-10 09:06*
