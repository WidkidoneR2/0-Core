# The tree

Where a file goes. Every entry at the repo root and in `zero/` is named here, with one line of
purpose. nsh-test keeps the map true in both directions: the_tree_map_names_every_entry fails on
an entry this map does not name, and every_backticked_repo_path_in_the_docs_exists fails on a
name here that does not exist. A new entry is added to this map in the same commit (INT-267).

This map describes the tree as it is. The target tree INT-267 designed lands with its move, and
that commit changes the tree and this map together.

| Entry | Purpose |
|---|---|
| `zero/` | Project 0's platform: the engine, the tools, the ledger, the registry |
| `zero/RISK.toml` | the risk tier for zero/ (user) |
| `zero/engine/` | core, the engine binary |
| `zero/tools/` | every other crate: NovaShell, nsh-test, ship, zero-core and the rest |
| `zero/intents/` | the intent ledger: future, in-progress, complete and the other states |
| `zero/meta/` | CHANGELOG, INCIDENTS, VERSION, plugins.toml and the release records |
| `zero/registry/` | declared tools, aliases, profiles, zones, sandbox policies, shell patterns, doctor checks, packages |
| `zero/schema/` | JSON schemas for policies, profiles, tools and zones |
| `zero/policy/` | policy files: hooks and security |
| `zero/scripts/` | fpatch (dev/) and the devshell scripts |
| `devbox/` | DevBox cases and census, read by zero-sandbox verify |
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
