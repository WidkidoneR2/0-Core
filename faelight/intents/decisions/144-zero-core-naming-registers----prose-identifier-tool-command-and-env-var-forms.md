---
id: 144
date: 2026-08-17
type: decision
title: "zero core naming registers -- prose, identifier, tool, command and env var forms"
status: decided
tags: [decision]
---

## Context

The project is taking a new identity: **Project 0** as codename, **Zero Core** as the eventual
public name. A logo wordmark renders it as `ZERO-CORE`, decisions 142 and 143 write it as
`Zero Core`, and the natural tool name reads `zero-shell`.

Three spellings appeared within one day. Left alone, all three propagate -- into documentation, crate
names, the boot entry, the prompt, and eventually a package someone else installs.

## Decision

These are not competing options. They are **registers**, and each has exactly one correct form.

| Register | Form | Used for |
| --- | --- | --- |
| Prose, display, wordmark | **`Zero Core`** | documentation, sentences, the logo lettering |
| Identifier, slug | **`zero-core`** | repository, crates, directories, package names |
| Tools | **`zero-shell`, `zero-vm`, `zero-fm`** | the direct successor to `faelight-*` |
| Command | **`zero`** | the CLI entry point, as `core` is today |
| Environment variables | **`ZERO_*`** | shells cannot take a hyphen |

⚠️ **`ZERO-CORE` -- uppercase with a hyphen -- fits no register.** It is not an identifier
convention and it shouts in prose.

★ It remains correct as **wordmark lettering**. Letterspaced capitals in a logo are a typographic
choice, not a name. The logo does not need to change; the documents and the code do.

### Why this shape and not another

Because it is already proven here. `faelight-shell`, `faelight-vm`, `faelight-fm` are lowercase
hyphenated today, cargo handles them, the flake's workspace glob picks them up, and nobody has ever
had to think about it.

**`zero-*` keeps the shape and changes only the word. There is no new rule to invent.**

## Migration policy

### Prefix the future. Never rename the past.

New user-facing functionality uses Zero Core terminology. Existing `faelight-*` identifiers stay.

⚠️ THIS IS NOT TIDINESS AVOIDANCE -- it is measured. A mechanical `faelight` to `zero` rename breaks
three layers at once:

- **Rust:** `faelight-core/src/paths.rs`, `faelight-deadwood/src/main.rs`, `integrity/mod.rs`,
  `doctor/checks.rs`, `cheatsheet_tui.rs`
- **Nix:** `environment.etc."faelight/VERSION"`, `xdg.configFile."faelight/profiles.toml"`
- **Persistent data:** `~/.config/faelight*`, `faelight/runtime/state.db`

★ The same lesson the ledger already paid for: two numbering eras shared one `INT-` prefix, the
Arch-era archive went missing, and 60+ code citations became unresolvable. **Renaming the past is
what broke that. Prefixing the future would not have.**

### Classify before migrating

| Category | Strategy |
| --- | --- |
| User-facing name | Rename |
| New APIs, new files | Use Zero |
| Documentation | Rename |
| Internal identifiers | Migrate gradually |
| Package and module names | Deliberate migration |
| Environment variables | Compatibility period |
| Config directories | Compatibility, then migration |
| Persistent data | Preserve; requires an explicit migration plan |
| URLs and domains | Deliberate migration |
| Existing scripts | Test before changing |
| Git history | Leave alone |

`~/.faelight/` to `~/.zero/` is **not a rename. It is a data migration.**

### Three identities coexist

- **Public** -- what a user sees: Zero Core, `zero-shell`, `zero`
- **Internal** -- what the code uses today: `faelight-shell`, `faelight_runtime`, `FAELIGHT_SHELL`
- **Compatibility** -- old names still accepted while both exist

## Deliberately not decided here

- **When the migration happens.** Not soon. Nothing is renamed as a result of this decision.
- **Whether the wordmark says LINUX.** The architecture treats Linux as substrate, not product.
  Recorded as an open question, not a correction.
- **The exact green.** The logo reads brighter than the declared `#00ff99` hero accent. Palette
  reconciliation is deferred by choice; INT-091's Stylix hybrid is the mechanism when it happens.
- **Intent renumbering or an era prefix.** Separate, and set aside.

## Consequences

- One spelling per context, decided before three of them propagated.
- No existing identifier changes, so nothing breaks and no audit is owed.
- New tools have an obvious name the moment they are created, which is the only time naming is cheap.
- The logo is already correct.
