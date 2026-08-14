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
- [ ] KEEP (Lane 1, after the current fsh intents) -- Per-project isolated command namespaces -- SPLIT OUT to INT-144 (full scope management: scoped aliases + management UI + project manifests). Checked off when INT-144 completes.
- [ ] KEEP (Lane 1, with the item above) -- Project-specific shell configuration -- FOLDED into INT-144 (fsh per-project scope system, Layer 3). Same foundation as project namespaces: scope-keyed state loaded on fsh enter from a committable manifest. Checked off when INT-144 completes.
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
- [x] Crate search completion -- SHIPPED (INT-134 Lane 3, 2026-07-12): `dev search <query>` -- crates.io keyword search via cargo search (analogue of pkg-search's nixpkgs search). Text-parses cargo's name="ver" # desc format, char-safe truncation, caches to /tmp/fsh-crate-search.json for a future completion feature. Scoped as a SEARCH command (not tab-completion). commands/mod.rs dev_cmd.
- [ ] Native Rust scripting support
- [ ] Compile shell scripts to binaries
- [x] Dependency graph visualization -- SHIPPED (INT-134 Lane 3, 2026-07-12): `dev graph [crate] [--full]` -- FORWARD cargo tree (what a crate depends ON), complement of `dev deps` (--invert). Depth-2 default since forward trees explode, --full for all depths. Reuses dev deps' ambiguous-version prompt. commands/mod.rs dev_cmd.
- [x] Benchmark commands with hyperfine -- SHIPPED/VERIFIED (INT-134, 2026-07-12): `dev bench` uses hyperfine (retitled from Criterion -- hyperfine is the actual tool wired). commands/mod.rs dev_cmd.

## Lane 4 -- Friday / AI  (always human-authorized)
- [~] Explain command before execution -- PARTIAL (verified INT-134, 2026-08-06): an explicit `explain <cmd>` builtin exists (commands/mod.rs:9980) alongside semantic_explain_cmd and semantic_dryrun_cmd. What is NOT built is the automatic half -- nothing explains a command before it runs unless you ask.
- [x] Command error diagnosis -- partial (Friday knowledge hints)
- [ ] KEEP (Lane 4, after the current fsh intents) -- Interactive troubleshooting mode. Measured
      absent: `troubleshoot` and `diagnose` both report command not found. The PARTS exist and are
      recorded above -- last-error recall, error-history, explain_exit_code and Friday failure hints
      -- but each answers a question you ask afterwards. An interactive MODE is a guided session,
      which none of them is.
- [ ] CUT -- Shell script generation. Reason: measured absent (`generate` and `scaffold` both report
      command not found), and the nearest thing is scripting_run_cmd, which RUNS scripts rather than
      writing them. ⚠️ AND THE PREMISE HAS AGED: generating shell script text is what an assistant
      does at the prompt, and fsh is moving AWAY from text toward structured plans -- the spine
      lowers to argv rather than to a line. Building a text generator inside the shell now would run
      against the direction of the rebuild.
- [~] Extend NL -> commands (the `?` prefix) -- BASE BUILT (verified INT-134, 2026-08-06): translate_natural_language (main.rs:411, INT-268) is a real rule table of pattern-words plus command plus confidence -- pattern-based, no LLM, forest-specific. The `?` prefix is already VERIFIED above. EXTENDING the rule set is the open work, not building it.
- [x] Autocomplete from command-history patterns -- ALREADY BUILT (verified INT-134, 2026-08-14),
      and it is the SAME feature as fish-style autosuggestions above rather than a second one:
      `impl Hinter for ForestHelper` (completion.rs 1207-1231) prefix-matches `shell_history`,
      then falls back to high-confidence `friday_patterns` rows. Cross-referenced rather than
      counted twice.

## Lane 5 -- Structured-data pipelines  [EPIC -- own decision]
- [x] Structured data pipelines (objects, not plain text) -- ALREADY BUILT (verified INT-134, 2026-08-06): value::PipeOp and apply_pipeline carry a typed Value between stages (22 uses incl. tests/pipeline.rs); Engine::try_query_executor runs them. `ps | where cpu > 0.5 | sort cpu desc` returns ROWS, not text. This is the shell's differentiator and it was sitting unchecked.
- [ ] KEEP (Lane 5, with the EPIC decision record) -- Native JSON / YAML / TOML. Measured absent as
      a pipeline source: `json` and `from-json` both report command not found. serde_json appears
      throughout the tree and toml in a few places, but those are INTERNAL parsing -- config files,
      checkpoints, manifests -- not sources a pipeline can read from. ⚠️ AND THERE IS NO serde_yaml
      at all, so the three formats in this line are not equally close.
- [~] Interactive tables -- HALF BUILT (verified INT-134, 2026-08-14). The RENDERING is real:
      `ps | first 2` prints a typed table with named columns, a separator rule and aligned rows, via
      format_table over the Value pipeline. What is absent is INTERACTIVITY -- no `table` command,
      no navigation, no sorting by clicking a column. So the data model and the renderer are done
      and only the input layer is missing, which is a much smaller item than the line implies.
- [~] Charts in terminal -- CUT (INT-134, 2026-07-12): low utility-to-effort. `the bar` already shows live metrics (CPU/RAM/battery/wifi) and the health panel covers status; a general terminal-charting engine is a lot of build for occasional sparklines. No specific recurring visualization need identified. Filter-appropriate cut.

## UX / Editing  (evaluate per item)
- [x] Multi-line editing -- BUILT 2026-08-06: the Validator holds the prompt open for an unterminated quote, an unterminated command substitution, and a trailing backslash. It asks the spine lexer rather than counting quotes itself, so there is one owner of that knowledge. Heredocs and multi-line shell constructs (if/then/fi) are NOT covered -- both need the shell to model constructs it currently hands to sh.
- [x] Vim mode -- BUILT 2026-08-06: `set edit_mode = vi` in config.fsh, emacs remaining the default. Both vi and vim are accepted, and an unrecognised value warns and falls back rather than silently doing nothing. NOTE for anyone switching: after Esc you are in normal mode, so you need i/a/o before typing again -- the extra keystroke is vi, not a bug.
- [x] Emacs mode -- ALREADY BUILT (verified INT-134, 2026-08-06): .edit_mode(EditMode::Emacs) at main.rs:885, an explicit choice rather than a rustyline default. This means READLINE KEYBINDINGS (Ctrl+A/E/K/W), not the Emacs editor -- nothing is installed and nothing is required.
- [ ] Undo / redo command editing
- [x] Fish-style autosuggestions -- ALREADY BUILT, and better than the item asks (verified INT-134, 2026-08-06): impl Hinter for ForestHelper (completion.rs:1207-1231) hints only at end of line, matches history by prefix, then falls back to high-confidence friday_patterns rows. Fish suggests from history; this also suggests from what Friday has learned.
- [ ] Fuzzy command completion
- [~] Command history with semantic search -- PARTIAL (INT-134, 2026-07-12): literal search SHIPPED (`hs`/`history-search`/`hsearch` -> history_search_cmd, SQLite LIKE-substring, dedup + frequency/recency ranking + timestamps, commands/mod.rs:4643). SEMANTIC (meaning-based/embedding) half DEFERRED -- would need embeddings for a shell-history feature (high cost, marginal benefit over literal); possible future pairing with Friday's fact infrastructure, or a later CUT. Not owned by a current intent.
- [ ] Popup command palettes
- [ ] Command previews before execution
- [~] Interactive file picker -- ADJACENT, not the item (verified INT-134, 2026-08-06): pick_cmd (commands/mod.rs:7427) pipes candidates through an external selector, but its subcommands pick INTENTS. A general file picker is unbuilt; yazi covers file browsing (INT-063).
- [~] Directory jumping / bookmark directories -- PARTIAL (INT-134, 2026-07-12): JUMPING shipped (`z`/`zi` zoxide frecency, commands/mod.rs:667; plus `dev workspace <name>` authoritative crate-jump). Named BOOKMARKS half unbuilt (no mark/bm command -- only session-save exists, which is L129 Session workspaces, a different feature). Bookmarks deferred; not owned by a current intent.
- [x] Notifications when long tasks finish -- ALREADY BUILT (verified INT-134, 2026-07-13): commands >30s fire faelight-notify on completion (main.rs:3001-3006). Long-command notification hook is live.

## Productivity
- [x] Session workspaces -- ALREADY BUILT (verified INT-134, 2026-07-13): full env-snapshot cycle -- `env-save <name>` (snapshot), `env-load` (restore named), `env-rollback` (restore most recent), `env-diff` (compare current vs snapshot). SQLite fsh_env_snapshots table (INT-269). commands/mod.rs:1221-1359.
- [ ] Named command collections
- [ ] CUT -- Macro system. Reason: three mechanisms already cover this ground and a fourth would be
      a fourth owner of one idea. Aliases do text substitution (and append arguments, as bash does),
      `scripting_run_cmd` runs sequences, and the `on` trigger DSL handles event-driven repetition.
      Measured absent: `macro` reports command not found. INT-193 exists because two owners of one
      rule caused real bugs; this would be the same shape by choice.
- [x] Aliases with arguments -- ALREADY BUILT (verified INT-134, 2026-08-14): arguments APPEND to
      the expansion exactly as in bash. Measured: `alias zzgreet=echo hello` then `zzgreet world`
      prints `hello world`. ⚠️ POSITIONAL PARAMETERS ARE NOT AN ALIAS FEATURE IN ANY SHELL --
      `alias zzp=echo $1` then `zzp WORLD` prints `$1 WORLD`, which is what bash does too. What
      that half of the item wants is SHELL FUNCTIONS, which this roadmap does not list at all and
      which is a genuine gap worth its own line.
- [~] Scheduled commands -- ADJACENT, not the item (verified INT-134, 2026-08-06): on_cmd (commands/mod.rs:12565) is the EVENT trigger DSL over crate::triggers -- on list, on remove, on <event> => <action> -- and watch_cmd polls. Time-based scheduling is genuinely absent.
- [ ] Built-in task runner
- [ ] CUT -- Quick notes / todos. Reason: the intent ledger IS this, with gates, evidence, history
      and a TUI (`it`). A second note store would compete with it and the two would drift.
      Measured absent as a shell feature: `note`, `notes` and `todo` all report command not found.

## Terminal UI  (some covered by bar / ade / fm)
- [x] Dashboard mode -- ALREADY BUILT (verified INT-134, 2026-07-13): `dashboard`/`dash` -> dashboard_cmd (commands/mod.rs:12824). `dashboard` = full overview, `dashboard system` = CPU/memory/network/top processes, `dashboard forest` = forest state.
- [x] Built-in process monitor -- ALREADY BUILT (verified INT-134, 2026-07-13): `dashboard system` shows CPU, memory, network, and top processes (commands/mod.rs:12822-12827). Covered by the dashboard subsystem.
- [x] Resource usage widgets -- the bar (CPU / RAM / battery / wifi)
- [x] Network monitor -- partial (wifi up/down in bar)

## Security
- [x] Command risk scoring -- ALREADY BUILT (INT-246, verified live 2026-07-13): safety_guard::check() classifies commands by danger -- rm -rf on non-temp paths, sqlite3 DROP TABLE/DATABASE, direct state.db DELETE, plus DB-backed user allow/deny lists (deny wins). First-word matched (never args/paths); safe-command fast-path. Wired BEFORE execution at main.rs:1165/1207.
- [x] Dangerous command confirmation -- ALREADY BUILT (INT-246, verified live 2026-07-13): safety_guard::challenge_gate() prompts 'Type yes to proceed, anything else to abort' at CHALLENGE tier, blocks by default. LIVE TEST: rm -rf /nonexistent-test challenged with 'Destructive remove' + prompt; safe commands (ls) passed with no prompt.
- [x] Secret management -- ALREADY BUILT (verified INT-134, 2026-08-14) as `faelight-vault`, a
      separate crate at faelight/rust-tools/faelight-vault, reachable from the shell. Subcommands:
      init (master password), add, get, list with health scores, rotate, generate, audit for weak
      or old credentials, unlock with a TTL cache, lock, remove, export and import encrypted
      backups. That is MORE than this line asks for.
      ⚠️ FOUND ONLY BY RUNNING IT. A grep of commands/mod.rs found nothing, because this is not a
      builtin -- it is its own tool. The sweep rule "a symbol existing is not the feature existing"
      has a mirror: a symbol being ABSENT is not the feature being absent.
- [ ] Environment variable permissions

## Async / Jobs
- [ ] Async jobs with futures

## Experimental / Research  (later, eyes open)
- [ ] KEEP (Experimental, after the shell is fully migrated) -- Transactional filesystem operations.
      Measured absent: `transaction` and `tx` both report command not found. ⚠️ AND IT IS NOT THE
      SAME AS `undo` ABOVE. undo REVERSES individual mv/cp/rm after the fact; a transaction is
      all-or-nothing across several operations with a rollback boundary, so a failure halfway leaves
      nothing applied. Different and much harder, which is why it stays Experimental rather than
      folding into the undo line.
- [x] Reversible commands (undo for file ops) -- ALREADY BUILT (verified INT-134, 2026-08-14): the
      `undo` builtin tracks mv, cp and rm and reverses them. Measured on a clean session: it reports
      "Nothing to undo -- use mv/cp/rm to track operations", which is the empty-stack message of a
      real feature rather than a missing command.
      ⚠️ Listed under Experimental/Research "later, eyes open" and shipped anyway -- the third item
      this sweep found built in a section that assumed it was not.
- [ ] KEEP (Experimental, and cheaper than it looks now) -- Pipe execution visualizer. Measured
      absent: `visualize`, `viz` and `pipeline` all report command not found. ★ BUT THE DATA NOW
      EXISTS where it did not when this was written: the spine lowers a pipeline into one
      ExecutionPlan PER STAGE, each carrying argv, cwd, env and io, and FSH_SPINE_TRACE already
      prints what the router claimed. A trace is not a visualizer, but the structured facts a
      visualizer would render are already produced rather than needing to be reconstructed from text.
- [ ] CUT -- Command dependency graphs. Reason: WRONG OWNER. Measured absent in the shell (`graph`
      reports command not found), and `core deps` already provides dependency intelligence at the
      SYSTEM level, which is where the dependency facts actually live. A second graph inside the
      shell would either duplicate that or answer a narrower question nobody asked.
- [x] Event-driven shell hooks -- ALREADY BUILT (verified INT-134, 2026-08-14) as the `on` trigger
      DSL over crate::triggers: `on list`, `on remove <id>`, `on <event> => <action>`. Live and in
      daily use -- two triggers enabled, one fired 2081 times and the other 1111. The word `hooks`
      is not a command; `on` is the feature.
- [ ] KEEP (owner: INT-170, before any runtime is chosen) -- WASM plugins / Lua-Rhai plugins /
      hot-reload extensions. A plugin MECHANISM already exists: the `plugins` builtin loads .fsh
      files from ~/.config/faelight-shell/plugins, which is TEXT EXPANSION -- exactly what INT-170
      says today. INT-170 defines initialize/execute/shutdown/metadata BEFORE picking a runtime, so
      this line is that intent rather than a separate one, and picking WASM or Lua here would be
      choosing a runtime before the contract exists.

## Cut -- fails the filter
- Smart cd with typo correction (erodes explicitness)
- Silent auto-magic / opaque AI that runs commands without authorization
- Distributed shell across machines (CUT INT-134, 2026-07-13: a distributed-systems project, not a shell feature -- out of scope for a personal single-machine daily driver; complexity vastly exceeds value)
- Time-travel shell state snapshots (CUT INT-134, 2026-07-13: the grand 'rewind the whole shell to any past moment' version is over-engineered for the need. The useful SUBSET already exists elsewhere: env-save/env-load/env-rollback session snapshots + cistart/cicomplete auto-checkpoints. Cutting the cathedral, keeping the chapel.)

---
**Status:** roadmap finalized at fsh v3.1.0 (2026-07-11, INT-134). Reconciled at each version bump.
Sequence: 060 (foundation, DONE) -> Lane 0 (stability: builtin-shadowing first, DONE) -> 057 (df-crash, DONE)
-> Lane 2 (Nix) -> Lane 3 (Rust) -> Lane 4 (Friday). Lane 5 is a separate epic decision.
