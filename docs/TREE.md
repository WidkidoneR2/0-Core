# The tree

Where a file goes. Every entry at the repo root and in `zero/` is named here, with one line of
purpose. nsh-test keeps the map true in both directions: the_tree_map_names_every_entry fails on
an entry this map does not name, and every_backticked_repo_path_in_the_docs_exists fails on a
name here that does not exist. A new entry is added to this map in the same commit (INT-267).

This map describes the tree as it is. A move changes the tree and this map in the same commit
(INT-267).

| Entry | Purpose |
|---|---|
| `zero/` | Project 0's platform: the engine, the tools, the ledger, the registry |
| `zero/RISK.toml` | the risk tier for zero/ (user) |
| `zero/engine/` | core, the engine binary |
| `zero/tools/` | every crate that is neither the engine nor the shell: ship, zero-core and the zero-* tools |
| `zero/shell/` | the shell and what proves it: NovaShell (nsh), nsh-test, and devbox/ -- the DevBox cases and census zero-sandbox verify reads. The rule (L1): the engine is core; the shell is nsh and what exists only to prove or run it; every other binary is a tool, and a new crate goes in zero/tools/ unless it exists only to serve the shell |
| `zero/intents/` | the intent ledger: future, in-progress, complete and the other states |
| `zero/meta/` | CHANGELOG, INCIDENTS, VERSION, plugins.toml and the release records |
| `zero/registry/` | declared tools, aliases, profiles, zones, sandbox policies, shell patterns, doctor checks, packages |
| `zero/schema/` | JSON schemas for policies, profiles, tools and zones |
| `zero/policy/` | policy files: hooks and security |
| `zero/scripts/` | fpatch (dev/) and the devshell scripts |
| `docs/` | human documentation; nothing reads it at boot |
| `labs/` | experiments, listed by nsh's experiment command; nothing here is wired into a host |
| `assets/` | branding images and a font |
| `.githooks/` | zero-gate's pre-commit and pre-push hooks |
| `.cargo/` | audit.toml, read there by cargo-audit |
| `AGENTS.md` | the operating contract for agents |
| `README.md` | the front page |
| `LICENSE` | the license |
| `Cargo.toml` | the workspace |
| `Cargo.lock` | pinned dependency versions |
| `deny.toml` | cargo-deny's policy |
| `.envrc` | the direnv environment |
| `.gitignore` | what git ignores |
| `.gitleaks.toml` | gitleaks rules for the pre-commit secret scan |
