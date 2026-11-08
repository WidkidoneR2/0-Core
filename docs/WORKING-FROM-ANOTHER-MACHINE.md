# Working from another machine

> **Status: written from the repo files, not yet run on a second machine.** If a step here
> fails, the step is wrong and this file is what gets fixed.

Scope: a second **Arch or Omarchy** machine. Other Linux distributions are not covered.
What Project 0 is, and why it sits on Omarchy: [README](../README.md).

## 1. Prerequisites

- `git`
- `rustup`, with the stable toolchain. No version is pinned: `rustup default stable`.
- Install them with the system package manager (`pacman` on Arch and Omarchy).

## 2. Clone

```sh
git clone https://github.com/WidkidoneR2/0-core ~/0-core
cd ~/0-core
git config core.hooksPath .githooks
```

The branch is `main`. The `core.hooksPath` line is the one opt-in that makes the tracked
git hooks run on this clone. Why it is needed: [.githooks/README.md](../.githooks/README.md).

The hooks refuse every commit and push until `zero-gate` exists. Build it first with
`cargo build -p zero-gate`; the hooks look for it in `target/debug`, `target/release` or on `PATH`.

## 3. Build

The shell crate (`novashell`, binary `nsh`) is built as described in
[zero/shell/novashell/README.md](../zero/shell/novashell/README.md) under "Build".
The workspace root is `~/0-core/Cargo.toml`.

`ship` is its own crate, `zero/tools/ship`. Its README ([zero/tools/ship/README.md](../zero/tools/ship/README.md))
gives the build: `cargo build -p ship`. The novashell README then uses `ship nsh` to deploy.
**Open: not documented, not tested:** how the built `ship` itself reaches `PATH` on a machine
that has never run it. Check that `ship` is on `PATH` before relying on it.

## 4. Wire the shell

bash stays the login shell and starts nsh from `.bashrc`. The handoff is
`zero/shell/novashell/handoff/bashrc-handoff.sh`. Read it before using it; this file does
not copy it. It starts nsh only if `$HOME/.local/bin/nsh` is executable, so the deploy in
section 3 has to work first. The "What it is, and what it is not" section of the novashell
README explains why.

## 5. Check it works

- `nsh-test` runs the real REPL through a pty (see the novashell README, "Test").
- `d` is the health check: `core doctor run` on the first machine. Whether the alias and
  `nsh-test` are on `PATH` here depends on the open `ship` question in section 3.

## 6. What does not carry over

- The machine facts in [AGENTS.md](../AGENTS.md) section 1 ("This host / this tree"): the
  repo path `/home/christian/0-core`, the `~/.local/bin` deploy folder and the aliases
  (`cat` is `bat`, `ls` is `eza`) belong to the first machine. Substitute your own paths.
- Secrets and credentials. Nothing like that is in the repo and nothing here moves it.
- Anything under `~/.hermes`, or any agent setup. This file does not cover those.

## Rules that still apply

Edits go through a reviewed fpatch Plan with a Seal, and nothing is applied without
Christian's say. [AGENTS.md](../AGENTS.md) section 2 is the authority.
