---
id: 068
date: 2026-06-18
type: future
title: "fsh cache commands: cache status + cache push"
status: in-progress
tags: [fsh, faelight-shell, cachix, cache, command-dispatch, rust, nixos]
---

## Why
INT-043 wired the Cachix backend (pull + push, deps in cache), but pushing and
inspecting are still manual: `nix build .#faelight-deps --no-link --print-out-paths`
then `cachix push faelight-forest <path>`. Two fsh commands -- `cache status` and
`cache push` -- make this first-class forest vocabulary. These were gates 132/133 of
INT-043, spun out because exposing them as fsh commands first requires mapping fsh's
command-dispatch internals -- its own piece of work.

## What Already Exists
- INT-043: cache live (faelight-forest.cachix.org), substituter + key declarative in
  framework16 config, auth token configured, deps pushed (614 paths).
- flake output `faelight-deps` exposes the crane deps derivation for `nix build`.
- cachix CLI installed system-wide (1.11.1).
- fsh is a compiled Rust shell with builtins + a completion system (completion.rs).
  The dispatch path -- where a typed command matches a builtin before PATH fallthrough
  -- is not yet mapped. That is Phase 0.

## Honest Scope
`cache status` does NOT track a historical hit/miss RATE -- that needs per-build
instrumentation and is a separate future intent. It reports the useful state:
substituters configured, the current deps derivation path, and whether that path is
present in the remote cache right now (narinfo check). "Is my current build cached?"
answered honestly -- no fabricated metrics.

## Approach
Logic lives in scripts (pkgs/faelight/scripts/cache-push, cache-status): testable,
iterable without a full rebuild, keeps fsh thin (DEC-005). fsh gains a `cache` command
that dispatches `push` / `status` to the scripts.

## Phases
Phase 0 -- locate the dispatch
  Find and document where faelight-shell matches builtins/commands. Identify file +
  function + the pattern a new command must follow.
  Gate: fsh command-dispatch path identified and recorded in this charter

Phase 1 -- scripts
  cache-status: substituters (confirm faelight-forest) + current deps path + remote
  present/absent (narinfo 200/404). cache-push: build deps, cachix push.
  Gate: cache-status script reports config + present/absent
  Gate: cache-push script builds and pushes the deps unit

Phase 2 -- fsh command
  Add `cache` dispatching status/push to the scripts; tab-completion for the two
  subcommands.
  Gate: `cache status` works in fsh
  Gate: `cache push` works in fsh

## Phase 0 Findings (2026-06-18)
Dispatch lives in `rust-tools/faelight-shell/src/commands/mod.rs`:
- `pub enum CommandResult` (line 17): `Output(String)`, `Error(String)`, `Empty`.
- `pub fn execute(line, db, core_root)` (line 173) -> `execute_impl(...)` (line 177)
  holds the master `match` on the command name. Arms map a command-name `&str` to a
  `CommandResult`. `db: &ForestDb` and `core_root` are in scope.
- Handlers are `fn h(args: &[&str]) -> CommandResult` (e.g. `fn cd`, line 6159) or
  inline blocks. `args` is `&[&str]`.
- Templates: the `"d"` arm shells out via `std::process::Command...output()` and folds
  stdout+stderr into `CommandResult::Output` (use for `cache status`). The `"edit"` arm
  uses `.status()` with inherited stdio to stream live (use for `cache push`).
- To add: a `"cache" => cache(args),` arm near `"cd"`, plus `fn cache(args: &[&str])`
  branching on `args[0]` (status/push) -> shells out to `cache-status` / `cache-push`
  on PATH. Completion: add `cache` + subcommands in `completion.rs`.

## Gates
- [x] Phase 0: dispatch identified -- commands/mod.rs execute_impl master match (CommandResult; handlers fn(args:&[&str]); d/edit arms as templates); documented above, 2026-06-18
- [ ] cache-status script: substituter config + current-deps present/absent in remote cache
- [ ] cache-push script: builds .#faelight-deps and pushes to faelight-forest
- [ ] fsh `cache status` command works
- [ ] fsh `cache push` command works

## Depends On
- INT-043 (Cachix backend): provides cache, substituter, token, faelight-deps output.
  043 stays open on its own clean-VM gate; this intent does not block on that.

## The Rule
"The forest should name what it does. Build once, cache everywhere -- and let the shell
speak it." 🌲
