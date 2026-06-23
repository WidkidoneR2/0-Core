# fsh Evolution Roadmap

Filter: a feature earns a place only if it deepens **understanding +
authorized, reproducible control**. Opaque convenience and auto-magic are cut.
Sequence: **foundation (INT-060) -> stability (INT-057) -> feature lanes**.

## Lane 0 -- Stability / Correctness  [known papercuts, evidence-dated]
Bugs that cost real time in sessions. Fix before polishing features. Highest priority:
builtin shadowing (caused a disk-corruption risk, 2026-06-23).

- [ ] **Builtin shadowing of process tools** -- `kill` only takes job-ids (`kill %N`), not
      PIDs; `pkill`/`pgrep` fail (exit-1, no match). Broke `vm down` -> silent no-op -> two
      VMs on one qcow2 (corruption risk). Workaround used: Python /proc walkers. (2026-06-23)
- [ ] **Operators punt the whole line to bare `sh`** -- `|` `>` `;` `2>&1` and heredocs hand
      the line to `sh`, which cannot see fsh builtins (`vm` -> "command not found"). Broke
      command capture repeatedly across the session. (2026-06-23)
- [ ] **Bare `python3` -> interactive REPL trap** -- no script arg drops into the
      interpreter; looks like a hang. (2026-06-23)
- [ ] **`exec fsh` does not hot-swap** the rebuilt binary -- must close+reopen the terminal
      to pick up a new fsh after rebuild.
- [x] fsh crashes (closes terminal) on `df` -- FIXED (INT-057)

## Already in the shell / ecosystem (verify + keep)
- [x] Parallel execution
- [x] Session save / load / replay (= command recording & replay, persistent sessions)
- [x] Natural-language `?` prefix (NL -> command, human-confirmed)
- [x] Run scripts: `run` (.py/.sh/.fsh)
- [x] Aliases (reconciled; INT-060 complete)
- [x] Syntax highlighting -- partial (highlight_rust_line / colorize_line); verify scope
- [x] Sandboxed execution -- faelight-sandbox (5 policies)
- [x] File browser -- yazi (INT-063; faelight-fm -> WIP)
- [x] Split panes -- faelight-ade (fsh PTY + friday-chat)
- [x] fsh as a first-class command (`fsh` -> faelight-shell; flake.nix postFixup, 2026-06-18)
- [x] Git status in prompt + bar
- [x] Nix context in prompt -- flake + devshell, dirty git / dirty flake / rebuild-drift markers (INT-062)
- [x] Health % in prompt + live in bar (INT-033)
- [x] Active intent in bar (focus.toml) + intent ledger (cistart / cicomplete / d)
- [x] Friday AI inline -- knowledge + error hints, pattern/fact tracking, contradiction signals
- [x] Workspace indicators in bar -- i3-style, dwl-ipc via faelight-wsd (INT-053)
- [x] Notifications -- faelight-notify (INT-065)
- [x] Power menu + lock -- faelight-logout (INT-064), faelight-lock (INT-046)

## Lane 1 -- Declarative / Reproducible  [FOUNDATION = INT-060]
- [x] config.fsh = single declarative source of truth (INT-060) -- DONE 2026-06-18
- [ ] Reproducible shell sessions
- [ ] Versioned shell environments
- [ ] Rollback-able environment changes
- [ ] Environment diffs
- [ ] Immutable command history
- [ ] Shareable environment manifests
- [ ] Per-project isolated command namespaces
- [ ] Project-specific shell configuration
- [ ] Audit log
- [ ] Command allowlists / denylists

## Lane 2 -- Nix-native
- [x] Show current flake in prompt (INT-062)
- [x] Detect dirty git + flake state (INT-062)
- [x] Built-in nix command wrappers -- partial (rebuild / dep / update-flake aliases)
- [ ] Auto dev-shell activation per flake project
- [ ] Generation rollback browser
- [ ] Query installed packages from prompt
- [ ] Package search integrated into completion
- [ ] Nix store explorer
- [ ] GC statistics widget

## Lane 3 -- Rust-native
- [ ] Cargo integration commands
- [ ] Cargo workspace navigation
- [ ] Rustdoc lookup from shell
- [ ] Crate search completion
- [ ] Native Rust scripting support
- [ ] Compile shell scripts to binaries
- [ ] Dependency graph visualization
- [ ] Benchmark commands with Criterion

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
Sequence: 060 (foundation, DONE) -> Lane 0 (stability: builtin-shadowing first) -> 057 (df-crash, DONE) -> Lane 2 (Nix)
-> Lane 3 (Rust) -> Lane 4 (Friday). Lane 5 is a separate epic decision.
