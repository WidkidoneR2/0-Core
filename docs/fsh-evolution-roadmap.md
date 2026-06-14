# fsh Evolution Roadmap

Filter: a feature earns a place only if it deepens **understanding +
authorized, reproducible control**. Opaque convenience and auto-magic are cut.
Sequence: **foundation (INT-060) -> stability (INT-057) -> feature lanes**.

## Already in the shell / ecosystem (verify + keep)
- [x] Parallel execution
- [x] Session save / load / replay (= command recording & replay, persistent sessions)
- [x] Natural-language `?` prefix (NL -> command, human-confirmed)
- [x] Run scripts: `run` (.py/.sh/.fsh)
- [x] Aliases (reconciliation in flight = INT-060)
- [x] Syntax highlighting -- partial (highlight_rust_line / colorize_line); verify scope
- [x] Sandboxed execution -- faelight-sandbox (5 policies)
- [x] File browser -- faelight-fm
- [x] Split panes -- faelight-ade (fsh PTY + friday-chat)
- [x] Git status in prompt + bar
- [ ] (add anything else you spot already done)

## Lane 1 -- Declarative / Reproducible  [FOUNDATION = INT-060]
- [ ] config.fsh = single declarative source of truth (INT-060)
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
- [ ] Show current flake in prompt
- [ ] Detect dirty git + flake state
- [ ] Built-in nix command wrappers
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
- [ ] Command error diagnosis
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
- [ ] Resource usage widgets
- [ ] Network monitor

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
Sequence: 060 (foundation) -> 057 (df-crash stability) -> Lane 2 (Nix)
-> Lane 3 (Rust) -> Lane 4 (Friday). Lane 5 is a separate epic decision.
