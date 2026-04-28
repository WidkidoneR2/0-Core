---
id: 245
title: "faelight-shell v9 -- The Shell That Thinks"
status: in-progress
date: 2026-04-20
tags: [shell, fsh, v9, parallel, intelligence, execution, linux, innovation, friday]
---
Every other shell executes one command at a time.
fsh v9 executes what needs to be executed,
when it needs to be executed,
in the order that makes sense,
at the speed the hardware allows.
This is not a feature addition.
This is a rethinking of what a shell is.
Linux has always been powerful.
The shell has always been the bottleneck.
fsh v9 removes the bottleneck.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
THE VISION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Other shells: you type a command, you wait, you type another.
fsh v9: you type what you want to happen, fsh figures out how.
You should never wait for something that could run in parallel.
You should never type more than you need to.
You should never repeat yourself.
You should never have to think about the order of independent tasks.
fsh v9 is the shell that:
  - Runs independent commands simultaneously without you asking
  - Detects when you are stuck and surfaces the fix
  - Remembers what worked and what did not
  - Understands your intent, not just your syntax
  - Makes Linux feel like it was designed for humans
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 1: PARALLEL EXECUTION ENGINE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
The core innovation. What no other shell does natively.
EXPLICIT PARALLEL:
  parallel {
      deploy core
      deploy faelight-shell
      deploy faelight-term
  }
  All three build simultaneously. Output streamed live, labeled by tool.
  Waits for all to complete. Reports success/failure per command.
  If one fails: others continue, failure reported clearly.
PIPE PARALLEL (||| operator):
  deploy core ||| deploy faelight-shell
  Both run at the same time. Natural syntax.
AUTO-PARALLEL DETECTION:
  fsh analyzes a sequence of commands for dependencies.
  Independent commands are automatically parallelized.
  deploy core
  deploy faelight-shell    <- fsh detects: no dependency on above
  gc "message"             <- fsh detects: depends on both deploys
  Result: first two run in parallel, gc waits for both.
SMART PARALLEL OUTPUT:
  Each parallel job gets a labeled output stream:
  [core]          Building...
  [faelight-shell] Building...
  [core]          ✅ deployed in 14.8s
  [faelight-shell] ✅ deployed in 6.6s
  Clean. No interleaving. No confusion.
JOB CONTROL (UPGRADED):
  jobs          -- show all running parallel jobs
  wait <job>    -- wait for specific job
  cancel <job>  -- cancel a running job
  job-log <job> -- show full output of a completed job
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 2: INTELLIGENT EXECUTION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
fsh watches what you do and learns from it.
STUCK DETECTION:
  Same command fails 3 times in a row.
  fsh interrupts: "You've hit this 3 times.
                  Last time this happened, the fix was: <solution>
                  Want me to try it? (y/n)"
  Not after the fact. In the moment.
ERROR PATTERN RECOGNITION:
  fsh knows the forest's error patterns from friday_knowledge.
  When a build fails: fsh surfaces the relevant knowledge entry inline.
  No need to run friday ask. It happens automatically.
  "E0597: this is the rusqlite lifetime pattern.
   Fix: let x = stmt.query_map()?; x"
COMMAND PREDICTION:
  After cicomplete, fsh knows deploy comes next.
  After deploy, fsh knows gc comes next.
  fsh surfaces these as inline suggestions with confidence.
  You press Tab to accept or just keep typing to ignore.
SMART RETRY:
  retry <n> <command>   -- run command up to n times until success
  retry-on-fail {       -- retry block if any command fails
      cargo build
      deploy core
  }
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 3: DIRECT LINUX CONTROL
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Linux is powerful. The shell should expose that power directly.
PROCESS INTELLIGENCE:
  ps-tree           -- show process tree with resource usage
  kill-tree <pid>   -- kill process and all children
  watch-proc <pid>  -- live resource monitoring for a process
  top-fsh           -- fsh-native process viewer, forest-aware
RESOURCE AWARENESS:
  fsh knows when the system is under load.
  Before running a heavy build: checks CPU/memory.
  "System is at 90% CPU from another build. Wait? (y/n)"
FILE OPERATIONS (UPGRADED):
  move-safe    -- move with conflict detection and rollback
  copy-delta   -- copy only changed files (rsync-style, no rsync needed)
  find-smart   -- natural language file search via Friday
NETWORK DIRECT:
  http-get <url>    -- direct HTTP without curl/wget
  http-post <url>   -- direct HTTP POST
  check-port <port> -- is this port open?
  ssh-quick <host>  -- SSH with forest key management
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 4: SESSION INTELLIGENCE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
The shell remembers. Every session. Every command. Every outcome.
NAMED SESSIONS:
  session save "building-term-v2"   -- snapshot current context
  session load "building-term-v2"   -- restore directory, env, history
  session list                      -- show saved sessions
COMMAND HISTORY (UPGRADED):
  history-search <term>   -- fuzzy search with context
  history-replay <n>      -- replay last n commands
  history-diff            -- what changed between sessions
ENVIRONMENT SNAPSHOTS:
  env-save <name>    -- save current env vars
  env-load <name>    -- restore env vars
  env-diff           -- show what changed
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PILLAR 5: FRIDAY DEEP INTEGRATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
fsh v9 is the first shell with a built-in AI partner.
INLINE INTELLIGENCE:
  After every command that fails: Friday surfaces the fix automatically.
  After every deploy: Friday confirms health trajectory.
  After every commit: Friday notes what changed and why it matters.
NATURAL LANGUAGE COMMANDS:
  ? build and deploy core        -> fsh translates to: cargo build && deploy core
  ? show me what changed today   -> fsh translates to: git log --since=today
  ? find the rust file with E0597 -> fsh translates to: grep -r "E0597" --type rs
FRIDAY INTERRUPT LEVELS IN SHELL:
  CHALLENGE: fsh stops you before executing.
             "That command will overwrite state.db. Are you sure?"
  RECOMMEND: fsh suggests before you finish typing.
             "You usually run d before this. Want to?"
  SUGGEST:   fsh mentions after execution.
             "That took 45s -- longer than usual. Friday noted it."
  SILENT:    Friday watches but says nothing.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
WHAT v9 IS NOT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
v9 is NOT:
  - A rewrite from scratch (fsh v8 is the foundation)
  - A different language or syntax (backwards compatible)
  - An internet-connected AI assistant
  - A replacement for understanding your system
v9 IS:
  - The evolution of fsh into something no other shell has been
  - Parallel by default, sequential when needed
  - Intelligent about what you are trying to do
  - Honest about what it does not know
  - Still your shell -- you are still in control
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
HARD DEPENDENCIES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ fsh v8 (current) -- foundation to build on
✅ Friday Phase 2 (INT-219) -- pattern detection and knowledge engine
✅ Core v20 -- temporal models and prediction
⬜ Friday Knowledge Engine fully seeded -- error patterns loaded
⬜ fsh v9 architecture audit -- identify extension points in current source
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
GATES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Pillar 1 -- Parallel Execution:
⬜ parallel { } block syntax -- multiple commands run simultaneously
⬜ ||| operator -- inline parallel pipe
⬜ Auto-parallel detection -- fsh analyzes command independence
⬜ Labeled parallel output -- each job stream identified
⬜ jobs / wait / cancel / job-log commands live
⬜ Parallel error handling -- one fails, others continue, all reported
Pillar 2 -- Intelligent Execution:
⬜ Stuck detection -- 3 failures triggers Friday interrupt
⬜ Error pattern recognition -- build failures surface knowledge inline
⬜ Command prediction -- Tab accepts Friday suggestion
⬜ retry <n> and retry-on-fail { } syntax
⬜ Demonstrated: fsh surfaces fix without being asked
Pillar 3 -- Direct Linux Control:
⬜ ps-tree -- process tree with resources
⬜ kill-tree -- kill process family
⬜ Resource awareness -- warns before heavy build on loaded system
⬜ move-safe and copy-delta builtins
⬜ http-get / http-post builtins (no curl dependency)
⬜ check-port builtin
Pillar 4 -- Session Intelligence:
⬜ session save / load / list
⬜ history-search fuzzy with context
⬜ history-replay <n>
⬜ env-save / env-load / env-diff
Pillar 5 -- Friday Integration:
⬜ Inline Friday interrupt after failed commands
⬜ Natural language ? prefix translates to real commands
⬜ CHALLENGE level stops execution before dangerous commands
⬜ Friday notes commit content automatically after gc
⬜ Demonstrated: Friday interrupts and is right
Final:
⬜ All existing fsh commands work unchanged -- backwards compatible
⬜ fsh v9 deployed and used as daily driver for 7 days without regression
⬜ Parallel execution demonstrated with 3+ simultaneous deploys

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
FRICTION FIXES (from INT-232 session 2026-04-22)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Issues discovered during INT-232 development that must be fixed in fsh v9:
1. fsh-patch argument confusion -- fsh-patch takes filepath as arg2 not string. Add clear usage error.
2. python3 -c multiline fails in fsh -- document in COMMAND-GUIDE.md: always write to /tmp/script.py
3. heredoc RSEOF contamination -- warn when RSEOF appears literally in heredoc output
4. Caret red on deploy warnings -- deploy with warnings should exit 0 not 1
5. grep | in pattern -- unquoted | in grep treated as pipe; fsh should detect grep context
6. COMMAND-GUIDE.md needs: binary mode rule for Python writing Rust files
7. >> append redirect -- fsh does not support `cat file >> target`; errors with "No such file or directory". Blocks normal shell idioms.
8. python3 -c "complex arg" -- fails with "File name too long (os error 36)" when the -c argument contains escaped quotes or exceeds some length. Workaround: write script to /tmp/ and python3 /tmp/script.py. Document in COMMAND-GUIDE.
9. fsh silent DB connection failure -- `save_history_entry` in rust-tools/faelight-shell/src/db.rs:155 uses `.ok()`, dropping all INSERT errors. When connection goes stale (e.g. after schema migrations on state.db), shell_history writes silently stop for the entire fsh lifetime. Discovered during INT-234 gate 8: rows stopped at id 19748 for ~1 hour with no error indication. Recovery requires fsh restart.
10. fsh redirect failures create junk files -- when `>` or `>>` fails to parse, fsh creates a file named after the broken argument instead of erroring. Discovered files in repo root: `=68`, `=69`, `=257`, `(strftime('%s','now') - 86400) GROUP BY ...`. Cleanup required Python os.remove() for special-char filenames. Should emit parse error to stderr, not touch filesystem.
11. fsh multi-line paste splits wrong -- multi-line commands pasted into fsh often fail or split incorrectly. Example: `echo "X" > /tmp/q.sql\nsqlite3 db < /tmp/q.sql` pasted together fails second command with redirect error. Workaround: one command at a time, or write script to /tmp/.
12. fsh backslash-escape in double quotes -- `"\$?"` should print literal `$?` (POSIX backslash escapes `$` inside double quotes). fsh ignores the backslash and expands `$?` anyway, producing `\0` style output. Affects: any command line where you want a literal `$` inside double quotes. Workaround: use single quotes. Discovered: 2026-04-28 while crafting commit message for INT-245
13. fsh multi-line accumulator hangs on long quoted strings -- multi-line input containing a long quoted commit message (especially with em-dashes, parentheses, or special chars) sometimes leaves fsh in `...` continuation prompt requiring Ctrl+C. is_complete_command parser thinks the input is unclosed. Workaround: open editor with `git commit` (no -m) and write message in nvim. Discovered: 2026-04-28 trying to commit single-quoted multi-clause message about INT-245
14. faelight-notify D-Bus collision on git_commit signal -- when fsh emits git_commit signal, faelight-notify tries to start a new instance ("name already taken on the bus") even though one is already running. Resolves itself, but emits noisy error. Likely missing "is already running" check in faelight-notify startup. Discovered: 2026-04-28.
15. fsh git-commit -m with special chars -- combining factors (single quotes, em-dashes, $ chars, parentheses) routinely break fsh's parser when crafting `git commit -m '...'` lines. Use `git commit` (no -m, opens editor) until INT-245 #12-13 are fixed. Track separately even though it overlaps #12 and #13 because git-commit is the most common workflow that exercises these gaps.
Pillar 6 -- Friction Fixes:
⬜ fsh-patch: clear usage error when args are wrong type
⬜ COMMAND-GUIDE.md updated: python3 multiline, binary mode, heredoc rules, python3 -c arg-size workaround
⬜ deploy: exit 0 when successful with warnings only
⬜ grep pattern: | inside grep arguments not treated as pipe
⬜ heredoc: warn when literal RSEOF/PYEOF appears in output (likely missing delimiter)
⬜ >> append redirect supported natively in fsh
⬜ python3 -c handles complex arguments without "File name too long" error
⬜ fsh DB writer logs INSERT errors instead of silent .ok() -- stale connections should surface, not hide
⬜ fsh redirect parse failures emit error to stderr without creating filesystem artifacts
⬜ fsh handles multi-line paste as single input when appropriate (heuristic: detect clipboard origin or bracketed paste mode)
"Every shell before fsh asked:
'What command do you want to run?'
fsh v9 asks:
'What do you want to happen?'
And then makes it happen --
in parallel,
with intelligence,
with Friday watching,
at the speed Linux was always capable of
but no shell ever unlocked.
This is not a better shell.
This is what a shell should have always been." 🌲
