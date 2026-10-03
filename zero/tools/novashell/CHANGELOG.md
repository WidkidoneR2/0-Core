# Changelog -- NovaShell (nsh)

## 3.9.0 -- 2026-09-02

- ✨ The safety guard is on BOTH doors. `-c` was ungated: `nsh -c` ran a
  challenge-tier command with no prompt, because the guard lived in the REPL
  only. It refuses on `-c` rather than asking, since a pipe has nobody to answer.
- ✨ The guard judges every executing segment, not just the first. `true && zap`
  where `zap` expands to something destructive was outside the gate --
  `guard_command_word` answered `true` and no rule matched. `split_into_segments`
  stays the sole segmenter; the guard consumes its output.
- ✨ The guard judges the EXPANDED line (INT-197). An alias body reached
  execution without ever being checked.
- ⚡ Interactive startup 400ms to 55ms, from two missing indexes. `shell_history`
  had none, so every terminal scanned 187,630 rows and built a temp b-tree to
  take the newest 10,000: 277ms. `nsh_test_results` had none: 60ms more.
- 🗑️ The plugin expander is deleted. Zero users, a quoting bug, and a third
  owner of expansion. `plugins` now explains why rather than listing a loader
  that does not exist.
- ♻️ The crate is `novashell`, the binary is `nsh`. The rename completed.
- 🐛 `terminate` built a `pgrep` pattern through `sh -c`, so a quote in the
  pattern escaped into shell. It passes the pattern as one argument now.
- 🐛 Ten `core` shortcuts -- doctor, predict, react, stress, goals, evolution,
  security, capabilities, genealogy, autonomy -- had answered "command not
  found" since the spine flip.
- 🐛 An unreadable deny list is Unknown, not empty. It was failing open.

## 3.1 - 3.8 -- The Spine

- ✨ The parse/plan/execute spine (INT-169): a real lexer, a recursive-descent
  parser, an AST, and an `ExecutionPlan` the executor consumes rather than
  re-deriving from text. It executes by default; `NSH_SPINE=0` is the escape
  hatch.
- ✨ `spine migrate` replays real history against both parsers and reports where
  they disagree -- 43,469 rows, grouped by shape.
- ✨ `spine conform` compares behaviour against bash, three verdicts.
- ✨ Observability: `NSH_OBSERVE` emits structured events with a correlation id
  and a process clock. Silent unless asked.
- ✨ nsh-test drives the real REPL through a pty, not just `-c` -- the two doors
  behave differently and only one was being tested.
- ♻️ One tokenizer, one alias expander, one command-word owner. Each had been
  two.

## 🎉 3.0.0 -- The Nix Crossing

- 🌲 The shell crossed from Arch to NixOS -- fully native, no package-manager assumptions, no distro lock-in.
- 🗑️ Removed the Arch package commands (`pkg`, `pkgs`, `sys_packages`) -- a breaking change; the forest no longer speaks pacman.
- ✨ Candy-neon powerline prompt: a two-line prompt where the directory segment's color tells you what kind of place you're in.
- ✨ Multi-command paste blocks -- paste many lines, each runs as its own command, heredocs and quotes stay intact.
- ♻️ Command-snapshot schema split cleanly from health snapshots -- one purpose per table.
- ♻️ Relocated into the unified `faelight/` tree.

## 2.x -- The Shell Grows Up
- ✨ Tab completion, structured output, session save/load/replay.
- ✨ Natural-language `?` prefix -- ask the shell in plain words.
- ✨ Parallel execution (`|||`) -- true concurrent commands.
- ⚡ Startup and rendering improvements.

## Earlier
- 🌲 Born as the forest's own shell: speaks human first, UNIX as fallback. The daily driver.
