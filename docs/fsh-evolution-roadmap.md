# fsh Evolution Roadmap

Filter: a feature earns a place only if it deepens **understanding +
authorized, reproducible control**. Opaque convenience and auto-magic are cut.
Sequence: **foundation (INT-060) -> stability (INT-057) -> feature lanes**.

**Lane order (authoritative):** INT-060 (foundation, DONE) -> Lane 0 (stability;
builtin-shadowing first, DONE) -> INT-057 (df-crash, DONE) -> Lane 2 (Nix-native) ->
Lane 3 (Rust-native) -> Lane 4 (Friday/AI). Lane 5 (structured-data pipelines) is a
separate epic, decided on its own. Lanes 1/UX/Productivity/Security/Async/Experimental
are opportunistic -- pulled from as items become relevant, not strictly sequenced.

**KEEP/CUT filter:** applied at lane-placement time -- every item sits in a lane because it
passed the filter (deepens understanding + authorized, reproducible control); the "Cut" section
records what failed it. Unchecked items are KEEP-with-lane; no item-by-item re-litigation needed
unless the filter itself changes.

**How this stays true (reconciliation):** the roadmap is reconciled by Christian at each fsh
version bump -- when fsh releases, done items are re-verified against the shell, newly-shipped
features get checked off with their intent number, and lane order is re-confirmed. Drift between
a bump is expected; the bump is the checkpoint.

## Lane 0 -- Stability / Correctness  [known papercuts, evidence-dated]
Bugs that cost real time in sessions. Fix before polishing features. Highest priority:
builtin shadowing (caused a disk-corruption risk, 2026-06-23).

- [x] **Builtin shadowing of process tools** -- FIXED (INT-095, 2026-06-26): kill split
      three ways -- `kill %N` -> job table, `kill <PID>`/`-SIG` -> real kill, `terminate <pat>`
      -> pgrep pattern. All gates proven live. Corruption risk removed. (vm /proc workaround
      left in place, harmless; retire separately.)
- [~] **Operators punt the whole line to bare `sh`** -- CLARITY FIXED (INT-089, 2026-06-26):
      a forest word lost to the redirect->sh boundary now emits a clear message naming the
      cause + workaround. EXECUTION still routes to sh -- deeper fix (interleave fsh builtins with pipe/redirect handling) is acknowledged FUTURE work, UNFILED (changes fsh's execution model = a major bump; deliberately out of INT-134 scope). NOT owned by any current intent.
      (2026-06-23)
- [x] **Bare `python3` -> interactive REPL trap** -- FIXED (INT-143, 2026-07-11): `python3`/`python`/`py`
      with no script arg now emit a clear guard message (run a script / run a snippet / explicit `python3 -i`)
      instead of silently dropping into the REPL. commands/mod.rs:439 (dispatch) + run_python_cmd no-arg guard.
      Verified live on the debug binary. Scripts still run; `python3 -i` still gives an explicit REPL.
- [x] **`exec fsh` does not hot-swap** -- SOLVED (INT-096, `reload` builtin): fsh records
      which build a session launched from (main.rs:699) and the `reload` builtin hot-swaps to a
      freshly-rebuilt binary in place (main.rs:1012-1044) -- no terminal close+reopen needed.
      Used every session ("New fsh version detected -- reloading").
- [x] fsh crashes (closes terminal) on `df` -- FIXED (INT-057)

## Already in the shell / ecosystem (verify + keep)
<!-- Gate-45 verification (2026-07-11): claims checked against registry.rs + commands/mod.rs dispatch arrays and live behavior on the DEPLOYED binary. Method note: verify builtins on their OWN LINE, never through a pipe -- pipes route forest words to sh (INT-089), producing false 'command not found'. All claims below resolved to real source or live behavior; none required unticking. terminate/kill/jobs (INT-095) verified live: `terminate <pat>` dispatched correctly, `sleep &` populated the job table. -->
- [x] Parallel execution -- VERIFIED live 2026-07-11 (pipe chains dispatch correctly)
- [x] Session save / load / replay -- VERIFIED (commands/mod.rs:888-1032: session save/load/list/delete + history-replay; session delete = INT-269)
- [x] Natural-language `?` prefix -- VERIFIED (dispatch: `?` builtin, commands/mod.rs)
- [x] Run scripts: `run` (.py/.sh/.fsh) -- VERIFIED (dispatch: `run` builtin)
- [x] Aliases (reconciled; INT-060 complete)
- [x] Syntax highlighting -- partial, VERIFIED (highlight_rust_line + colorize_line, commands/mod.rs:91/126; applied in specific render contexts -- partial scope is accurate)
- [x] Sandboxed execution -- faelight-sandbox (5 policies) (INT-024)
- [x] File browser -- yazi (INT-063; faelight-fm -> WIP)
- [x] Split panes -- faelight-ade (fsh PTY + friday-chat) -- EVIDENCED: faelight-ade tool exists (no dedicated intent; built as ecosystem tooling)
- [x] fsh as a first-class command (`fsh` -> faelight-shell; flake.nix postFixup, 2026-06-18)
- [x] Git status in prompt + bar -- VERIFIED live (visible every prompt)
- [x] Nix context in prompt -- flake + devshell, dirty git / dirty flake / rebuild-drift markers (INT-062)
- [x] Health % in prompt + live in bar (INT-033)
- [x] Active intent in bar (focus.toml) + intent ledger (cistart / cicomplete / d) -- EVIDENCED: focus.toml + ci* mechanism, live
- [x] Friday AI inline -- VERIFIED live (Friday knowledge/contradiction signals surface every session + deploy)
- [x] Workspace indicators in bar -- i3-style, dwl-ipc via faelight-wsd (INT-053)
- [x] Notifications -- faelight-notify (INT-065)
- [x] Power menu + lock -- faelight-logout (INT-064), faelight-lock (INT-046)

## Lane 1 -- Declarative / Reproducible  [FOUNDATION = INT-060]
- [x] config.fsh = single declarative source of truth (INT-060) -- DONE 2026-06-18
- [x] Reproducible shell sessions -- FIXED (INT-134, 2026-07-11): session save/load now captures + restores the environment (PATH + FAELIGHT_/FSH_/FOREST_ vars) alongside directory/intent/commands. The env_vars column existed in the schema but was never wired -- now written on save, applied on load. Verified live: '2 env var(s) captured' -> '2 env var(s) restored'. commands/mod.rs session save/load.
- [x] Versioned shell environments -- FIXED (INT-134, 2026-07-11): the env-snapshot system (INT-269: env-save/env-load/env-diff) now RESTORES on load, not just shows. env-load was show-only ('fsh cannot set parent process env'); corrected -- fsh IS the shell, so set_var restores into its own env (children inherit it). Named versions: env-save v1/v2, env-load <ver> restores, env-diff compares. Verified live: env-save (9 vars) -> env-load ('7 var(s) restored') -> env-diff ('No differences'). commands/mod.rs env-load.
- [x] Rollback-able environment changes -- FIXED (INT-134, 2026-07-11): new `env-rollback` builtin restores the MOST RECENT env snapshot on demand (no name needed -- newest by saved_at). Reuses the env-load restore machinery. Verified live: env-save r1 -> env-rollback ('Rolled back to r1, 7 var(s) restored'), repeatable. commands/mod.rs env-rollback arm.
- [x] Environment diffs -- VERIFIED (INT-269, tested 2026-07-11): `env-diff <name>` compares current env vs a snapshot, names each differing var (saved vs current), counts diffs. Verified live: detected EDITOR hx->vim after a change (1 differ), reported 'No differences' when matched, detected again after env-load changed the live env. Already complete; no fix needed. commands/mod.rs env-diff.
- [x] Immutable command history -- FIXED (INT-134, 2026-07-11): new append-only shell_history_audit table auto-captures every real command via an AFTER INSERT trigger on shell_history (internal SUGGEST:/TIMING:/doctor-test rows excluded). UPDATE + DELETE on the audit table are blocked by BEFORE triggers (RAISE ABORT) -> DB-enforced immutability. Verified live: delete attempt blocked ('immutable: deletes not permitted', row count unchanged 17->17). The working shell_history table keeps its legitimate mutations (INT-250 exit-code backfill, marker cleanup) -- immutability lives in the audit log, not by breaking the working table. db.rs init.
- [x] Shareable environment manifests -- FIXED (INT-134, 2026-07-11): new env-export/env-import builtins. env-export <name> [path] writes a snapshot to a portable, human-readable TOML manifest (# header, name, exported_at, [vars]); env-import <path> reads it back into a snapshot. Full shareable cycle verified live: env-save -> env-export (readable TOML, PATH escaped) -> env-import -> env-load round-trip. Reuses the fsh_env_snapshots machinery; TOML is the shareable/committable transport. commands/mod.rs env-export/env-import arms.
- [ ] Per-project isolated command namespaces -- SPLIT OUT to INT-144 (full scope management: scoped aliases + management UI + project manifests). Checked off when INT-144 completes.
- [ ] Project-specific shell configuration -- FOLDED into INT-144 (fsh per-project scope system, Layer 3). Same foundation as project namespaces: scope-keyed state loaded on fsh enter from a committable manifest. Checked off when INT-144 completes.
- [x] Audit log -- FIXED (INT-134, 2026-07-11): new `audit-log [n]` builtin surfaces the immutable shell_history_audit trail (recent captured commands + audit_id + timestamps, default 20). Reads the tamper-proof append-only log built alongside 'Immutable command history'. Verified live: showed 'last 20 of 148' with DB-enforced footer, count grew live (148->150) confirming continuous capture. commands/mod.rs audit-log arm.
- [x] Command allowlists / denylists -- FIXED (INT-134, 2026-07-11): new `cmdguard` builtin manages DB-backed allow/deny lists (fsh_guard_list) that safety_guard.rs reads before execution. deny -> command hits the CHALLENGE gate (deny wins over everything); allow -> command skips the guard (vetted). cmdguard list | deny/allow add|remove <cmd>. Renamed from 'guard' (that word is aliased to the external intent-guard tool, resolved before dispatch). Verified live: deny frobnicate -> CHALLENGE 'Denylisted command'; flipped to allow -> no CHALLENGE (not-found only); existing heuristics (dd/mkfs) untouched. safety_guard.rs check() + commands/mod.rs cmdguard/guard_cmd.

## Lane 2 -- Nix-native
- [x] Show current flake in prompt (INT-062)
- [x] Detect dirty git + flake state (INT-062)
- [x] Built-in nix command wrappers -- partial (rebuild / dep / update-flake aliases)
- [~] Dev-shell activation per flake project -- PARTIAL (INT-134, 2026-07-12): manual `devshell enter [name]` SHIPPED (reproducible via `nix develop --command <fsh>`, nested fsh). commit 0af760ae. The AUTO-detect half was CUT: the session is always already in a devShell (IN_NIX_SHELL=impure, name=friday-dev-env always set) -- an edge-triggered auto hint has no reliable off-state. Filter-appropriate cut; documented.
- [x] Generation rollback browser -- SHIPPED (INT-134, 2026-07-12): `generations`/`gens` -- read-only NixOS generation browser (rollback shown, not run). commit 976d5334.
- [x] Query installed packages from prompt -- FIXED (INT-134, 2026-07-12): new `packages` (alias `pkgs`) builtin lists the current system environment, sourced from `nix-store -q --references /run/current-system/sw`, parsed to name-version (hash stripped via split_once so multi-dash names stay intact), sorted + deduped. `packages <filter>` narrows by substring (partly serves package-search too). Verified live: 211 packages listed; 'packages ripgrep' -> ripgrep-15.1.0; 'packages neovim' -> neovim-0.12.3. Reuses INT-075's nix_query_lines. commands/mod.rs packages arm.
- [x] Package search integrated into completion -- SHIPPED (INT-134, 2026-07-12): `pkg-search`/`pkgsearch <term>` (nix 2.34 `nix search nixpkgs <regex> --json`), caches to /tmp/fsh-pkg-search.json; completion.rs reads the cache for TAB (no network). commit 0af760ae.
- [x] Nix store explorer -- VERIFIED (INT-075, tested 2026-07-12): `store` command. `store why <path|name>` resolves a name to its /nix/store path and reports self size, closure size, GC roots that pin it, and direct referrers; `store reclaim` is the GC preview. Verified live: 'store why ripgrep' -> 6.2 MiB self / 54.2 MiB closure / 94 GC roots / 104 referrers. Already complete; no build needed. commands/mod.rs store_cmd.
- [x] GC statistics widget -- VERIFIED (INT-075, tested 2026-07-11): `store reclaim` is an honest read-only GC statistics preview -- computes the dead set (nix-store --gc --print-dead, deletes nothing), counts dead paths, sums true freeable size (self-sizes, not closure, to avoid double-counting shared deps). Verified live: 'dead paths: 1108, freeable: 6.24 GiB' in ~24s against the real store. Already complete; no build needed (avoided duplicating as a separate gc-stats). commands/mod.rs store_reclaim.

## Lane 3 -- Rust-native
- [x] Cargo integration commands -- SHIPPED/VERIFIED (INT-134, 2026-07-12): `dev` subcommands test/watch/check(bacon)/bench/geiger/audit-deps + `dev deps <crate>` (cargo tree --invert, 'store why for Rust'). commits 86c82588 (+ fuzzy/ambiguous handling). commands/mod.rs dev_cmd.
- [x] Cargo workspace navigation -- SHIPPED (INT-134 Lane 3, 2026-07-12): `dev workspace`/`ws` -- lists all 35 workspace crates (name/version/path, authoritative from cargo metadata, includes never-visited crates unlike zoxide); `dev workspace <name>` cd's into any crate (set_current_dir, teaches zoxide). commands/mod.rs dev_cmd.
- [x] Rustdoc lookup from shell -- SHIPPED (INT-134 Lane 3, 2026-07-12): `dev doc [crate]` -- auto-routes: no arg -> web std docs; workspace crate -> local cargo doc --open; external crate -> docs.rs. Membership resolved via serde_json parse of cargo metadata (exact match, no substring false-positives). commit 4cd4977e. commands/mod.rs dev_cmd.
- [ ] Crate search completion
- [ ] Native Rust scripting support
- [ ] Compile shell scripts to binaries
- [ ] Dependency graph visualization
- [x] Benchmark commands with hyperfine -- SHIPPED/VERIFIED (INT-134, 2026-07-12): `dev bench` uses hyperfine (retitled from Criterion -- hyperfine is the actual tool wired). commands/mod.rs dev_cmd.

## Lane 4 -- Friday / AI  (always human-authorized)
- [ ] Explain command before execution
- [x] Command error diagnosis -- partial (Friday knowledge hints)
- [ ] Interactive troubleshooting mode
- [ ] Shell script generation
- [ ] Extend NL -> commands (the `?` prefix)
- [ ] Autocomplete from command-history patterns

## Lane 5 -- Structured-data pipelines  [EPIC -- own decision]
- [ ] Structured data pipelines (objects, not plain text)
- [ ] Native JSON / YAML / TOML
- [ ] Interactive tables
- [ ] Charts in terminal

## UX / Editing  (evaluate per item)
- [ ] Multi-line editing
- [ ] Vim mode
- [ ] Emacs mode
- [ ] Undo / redo command editing
- [ ] Fish-style autosuggestions
- [ ] Fuzzy command completion
- [ ] Command history with semantic search
- [ ] Popup command palettes
- [ ] Command previews before execution
- [ ] Interactive file picker
- [ ] Directory jumping / bookmark directories
- [ ] Notifications when long tasks finish

## Productivity
- [ ] Session workspaces
- [ ] Named command collections
- [ ] Macro system
- [ ] Aliases with arguments
- [ ] Scheduled commands
- [ ] Built-in task runner
- [ ] Quick notes / todos

## Terminal UI  (some covered by bar / ade / fm)
- [ ] Dashboard mode
- [ ] Built-in process monitor
- [x] Resource usage widgets -- the bar (CPU / RAM / battery / wifi)
- [x] Network monitor -- partial (wifi up/down in bar)

## Security
- [ ] Command risk scoring
- [ ] Dangerous command confirmation
- [ ] Secret management
- [ ] Environment variable permissions

## Async / Jobs
- [ ] Async jobs with futures

## Experimental / Research  (later, eyes open)
- [ ] Time-travel shell state snapshots
- [ ] Transactional filesystem operations
- [ ] Reversible commands (undo for file ops)
- [ ] Pipe execution visualizer
- [ ] Command dependency graphs
- [ ] Event-driven shell hooks
- [ ] WASM plugins / Lua-Rhai plugins / hot-reload extensions
- [ ] Distributed shell across machines

## Cut -- fails the filter
- Smart cd with typo correction (erodes explicitness)
- Silent auto-magic / opaque AI that runs commands without authorization

---
**Status:** roadmap finalized at fsh v3.1.0 (2026-07-11, INT-134). Reconciled at each version bump.
Sequence: 060 (foundation, DONE) -> Lane 0 (stability: builtin-shadowing first, DONE) -> 057 (df-crash, DONE)
-> Lane 2 (Nix) -> Lane 3 (Rust) -> Lane 4 (Friday). Lane 5 is a separate epic decision.
