# Docs inventory

Measured 2026-11-08 against commit `f43cf128cbe2d5d7071cbd72360f81275dcb0c0f`.

Produced by listing every tracked markdown file outside `zero/intents/` with `git ls-files` of `*.md`, then reading each file. That directory is one row, the ledger. `docs/public/` is one row. The per-crate scaffolds under `zero/tools/` are one row. This file is the inventory; it was not part of the listing it records.

Status is `current`, `stale`, or `history`. A history row is a record that is not updated in place: changelogs, `zero/meta/releases/`, `zero/meta/INCIDENTS.md`, and `docs/rfc-169-parser-spine.md`. A stale row is drift measured against the tree on this date, with the evidence in the note. Everything else is current.

| path | purpose | status | note |
|---|---|---|---|
| `.githooks/README.md` | Why the git hooks are tracked, and what zero-gate runs | current | |
| `AGENTS.md` | The operating contract for anyone changing this tree | current | |
| `README.md` | Front page: Omarchy is the base, and the work is the shell | current | |
| `docs/ALIASES.md` | Where aliases live, and why this file does not list them | stale | Header says nsh 3.9.0. `zero/shell/novashell/Cargo.toml` is 5.3.10. The newest section of `zero/shell/novashell/CHANGELOG.md` is also 3.9.0, so the header follows that changelog, not the crate. |
| `docs/ARCHITECTURE.md` | How the layers, directories, and state paths meet | current | |
| `docs/COMMAND-GUIDE.md` | Daily commands, written to be checked against core and the shell config | current | |
| `docs/CONVENTIONS.md` | Small rules, including the evidence comment on a closed gate | current | |
| `docs/FINGERPRINT.md` | How the fingerprint is collected, compared, and consumed | current | |
| `docs/NOVASHELL.md` | What nsh is, including pipelines and job control | current | |
| `docs/NSH-COMPATIBILITY.md` | What a version number is allowed to break | current | |
| `docs/NSH-PHILOSOPHY.md` | Why the shell exists, and the invariants that do not move | current | |
| `docs/PHILOSOPHY.md` | The position on how the machine should behave | current | |
| `docs/TREE.md` | One purpose line for every repo-root entry and every entry in `zero/` | current | |
| `docs/WORKING-FROM-ANOTHER-MACHINE.md` | Clone, hooks, and build on a second Arch machine | current | The file says it has not been run on a second machine yet. That is a stated limit. |
| `docs/history-inventory.md` | Generated census of who reads and writes shell_history | stale | Header says 4 writers and 8 excluded. The generator's own classifier on this tree counts 6 writers, 6 excluded, 80 readers, and 24 untyped. It still skips the old doctor marker; the probe now writes a different marker in `zero/shell/novashell/src/commands/mod.rs`. The cited line 331 of `zero/shell/novashell/src/db.rs` is an aliases insert. Re-run `zero/shell/novashell/generate-history-inventory.py`. |
| `docs/inventory.md` | The 2026-09-30 measurement of which tools to keep | stale | It says `docs/NOVASHELL.md` says fg is not job control. The Job Control section of that file documents fg, bg, and jobs as job control. |
| `docs/public/` | Generated public copies of the canonical docs, plus the contents index | current | Six tracked files. `zero/tools/zero-docs/src/main.rs` copies five docs over the same names here and writes the index. On this date, four copies differ from `docs/` by a missing final newline only. The shell philosophy copy also drops lines that contain an intent id, which is that generator's filter. Ruled: `docs/` is canonical (INT-305 G3). |
| `docs/rfc-169-parser-spine.md` | The parser-spine RFC | history | Kept as history by the INT-305 ruling. The header still says active build. |
| `zero/engine/CHANGELOG.md` | Changelog of the core engine | history | Changelog. Not updated in place. |
| `zero/engine/README.md` | What the core binary is, and how to build and deploy it | current | |
| `zero/intents/` | The intent ledger | current | Not expanded. 452 tracked markdown files on this listing. |
| `zero/meta/CHANGELOG.md` | Project changelog since the Omarchy install | history | Changelog. Not updated in place. |
| `zero/meta/INCIDENTS.md` | Incident writeups | history | Incident record. Not updated in place. |
| `zero/meta/releases/10.4.0/intents-shipped.md` | Intents recorded for the v10.4.0 release | history | The only tracked file under `zero/meta/releases/`. |
| `zero/policy/security/README.md` | Notes for the sysctl and fail2ban files beside it | stale | States a current hardening index of 71/100, arch-audit, and four weekly commands (security-check, vuln-check, security-score, jail-status). None of those names are in `zero/registry/`. arch-audit is not in the live Rust sources. The two conf files next to this README are real. |
| `zero/schema/README.md` | Which schema is supposed to validate which registry file | stale | Names 01-registry and 04-schema. Neither directory exists. Schemas are in `zero/schema/` and registry files are in `zero/registry/`. |
| `zero/shell/novashell/CHANGELOG.md` | Changelog of nsh | history | Changelog. Not updated in place. Newest section is 3.9.0; the crate is 5.3.10. |
| `zero/shell/novashell/CORE-COUPLING.md` | Census of how the shell calls the core path helpers | history | Marked history in 7d322730 (2026-11-07). Its generator is not a tracked file. |
| `zero/shell/novashell/README.md` | How to build, ship, and test nsh | current | |
| `zero/shell/nsh-test/CHANGELOG.md` | Changelog of the shell suite | history | Changelog. Not updated in place. |
| `zero/shell/nsh-test/README.md` | What the suite is, and how to build and deploy it | current | |
| `zero/tools/README.md` | Generated index of tool name, version, and description | current | Each of the 17 version cells matches that crate's Cargo.toml. The "0 retired" figure is retired crates the generator still sees, not every retired row in `zero/registry/tools.toml`. |
| `zero/tools/*/README.md` | Scaffold README for each tool crate | current | 16 tracked files, one per crate. Same shape: title, description, changelog, build, deploy. Eight descriptions are empty because the Cargo.toml has no description key: `zero/tools/db-browse/README.md`, `zero/tools/friday-chat/README.md`, `zero/tools/zero-ade/README.md`, `zero/tools/zero-context/README.md`, `zero/tools/zero-core/README.md`, `zero/tools/zero-deadwood/README.md`, `zero/tools/zero-gate/README.md`, `zero/tools/zero-gen/README.md`. Six have a description and still say there is no changelog: `zero/tools/ship/README.md`, `zero/tools/teach/README.md`, `zero/tools/zero-daemon/README.md`, `zero/tools/zero-docs/README.md`, `zero/tools/zero-doctor/README.md`, `zero/tools/zero-insightd/README.md`. Two link a real changelog: `zero/tools/zero-git/README.md`, `zero/tools/zero-sandbox/README.md`. `zero/tools/zero-core/README.md` and `zero/tools/zero-doctor/README.md` are libraries, and the scaffold still says ship deploys them. |
| `zero/tools/zero-git/CHANGELOG.md` | Changelog of zero-git | history | Changelog. Not updated in place. |
| `zero/tools/zero-sandbox/CHANGELOG.md` | Changelog of zero-sandbox | history | Changelog. Not updated in place. |
