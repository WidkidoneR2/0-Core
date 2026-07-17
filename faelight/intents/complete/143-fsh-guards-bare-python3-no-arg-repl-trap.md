---
id: 143
date: 2026-07-11
type: future
title: "fsh guards bare python3 (no-arg REPL trap)"
status: complete
tags: [fsh, papercut, python, lane-0]
---

## Vision
fsh builtins that shadow real binaries must never SILENTLY do something other than what the command
says. Either do the right thing, or say clearly that you cannot. Never succeed at the wrong thing.

## The Problem -- much bigger than the original "bare python3" papercut
Filed 2026-07-11 for one case: bare `python3` drops into a REPL. The 2026-07-15 session (INT-027 /
INT-159 / INT-059) hit FOUR MORE, and they cost hours. The shape is always the same: A BUILTIN
SHADOWS A REAL BINARY AND SWALLOWS ITS ARGUMENTS.

1. `bash script.sh`  -> drops into INTERACTIVE BASH. The script never runs. Returns "successfully"
   in ~7s having done nothing. Hit twice on 2026-07-15; the second time the missing output was
   misread as a qemu failure and sent the session chasing a ghost.
   Workaround: chmod +x /tmp/x.sh && /tmp/x.sh  (shebang), never `bash /tmp/x.sh`.
2. `env VAR=x cmd`   -> prints fsh's Shell Environment table. The command never runs.
3. `time cmd`        -> shells out to sh, exit 127.
4. `VAR="a b" cmd`   -> THE WORST ONE. fsh WORD-SPLITS the value, errors on the remainder
   ("command not found: q35,smm=on\""), and SILENTLY LEAVES THE TRUNCATED FRAGMENT IN THE SESSION
   ENVIRONMENT.
   THE INCIDENT (2026-07-15): `QEMU_OPTS="-machine q35,smm=on" vm up` left QEMU_OPTS="-machine" in
   the session. An hour later the vm script's ${QEMU_OPTS:-} prepended that fragment, producing
   `-machine -machine q35,smm=on`, and qemu died with `unsupported machine type: "-machine"`.
   FOUR consecutive VM boots failed. The blame landed on the firmware, the launcher, and the
   Secure Boot config in turn -- none of which were at fault. `unset QEMU_OPTS` fixed it instantly.
   THAT is the class of bug worth the intent: not "it failed" but "it succeeded at the wrong thing,
   poisoned durable state, and misattributed the blame to a different tool an hour later."

Also observed, lower priority:
- `nix eval --raw` prints no trailing newline -> output glues to the next prompt (cosmetic).
- Multi-part pasted blocks occasionally truncate or swallow. python3 heredocs were reliable ALL
  session and are the workaround for everything above.

## The Solution
Detect the shadowing cases and REFUSE LOUDLY rather than silently substituting behaviour.
- Bare `python3` / `bash` / `sh` with NO arguments -> interactive is correct, keep it.
- The SAME builtins WITH arguments -> either exec the real binary with those arguments, or refuse
  with a message naming the workaround. Silently entering an interactive shell is the bug.
- `env`, `time` with arguments -> same rule.
- Inline `VAR=value cmd` -> at minimum, do not word-split; at minimum, do not leave a partial value
  in the session env after erroring. Silent state mutation on a FAILED command is indefensible.
- Open design question: which is right -- pass through to the real binary, or refuse and instruct?
  Passing through is friendlier; refusing is more honest about what fsh is. Decide before building.

## Success Criteria
- [x] `bash /tmp/x.sh` either RUNS the script or refuses with a message naming the workaround --
      it never silently drops into an interactive shell
<!-- evidence: commit 5cba096d. RUNS it. Reproduced first: `bash /tmp/t2.sh` -> "Stepping out of the
     forest... You are entering bash" -> a prompt -> SCRIPT_RAN never printed. Cause:
     shell_handoff_cmd kept only the first word (split_whitespace().next()) and never called .args().
     Fix: one guard, `"zsh" | "bash" if args.is_empty()`, mirroring `"git" if args.is_empty()` four
     lines below it. With args it falls through to run_external -> `sh -c "bash script.sh"`.
     Verified on the DEPLOYED binary (gen 389): bash /tmp/t2.sh -> SCRIPT_RAN; bash -c "echo X" ->
     INLINE_WORKS; bare `bash` -> banner + interactive shell (the good part kept). -->
- [x] `env VAR=x cmd` either runs cmd or refuses -- it never silently prints the environment instead
<!-- evidence: commit 56aa0798. RUNS it. Reproduced: `env FOO=1 echo real_env_would_print_this` ->
     fsh's Shell Environment table, no echo. Real env was at coreutils-9.11/bin/env the whole time.
     Fix: no args -> fsh's curated table (kept); any args -> the real binary. Guarded on
     args.is_empty() rather than sniffing for '=' -- a sniff is a second parser and second parsers
     drift. TWO arms, because env sits before the fallthrough and needs its own allow_external
     handling or `env FOO=1 cmd > file` runs twice.
     Verified DEPLOYED (gen 390): env FOO=1 echo real_env_runs_this -> real_env_runs_this;
     env -u PATH echo flags_work -> flags_work; bare env -> the table;
     env FOO143=xyz env > /tmp/e.txt -> FOO143=xyz, exactly 1 match (one execution). -->
- [x] `time cmd` either times cmd or refuses -- no silent exit 127
<!-- evidence: commit 968c7be5 (+ the time_cmd rewrite in the same commit).
     THE INTENT'S OWN TEXT WAS WRONG AND THE MEASUREMENT CORRECTED IT. "time cmd -> shells out to sh,
     exit 127" is not what happens. Measured 2026-07-16:
       time echo works_on_binaries -> 2ms (exit 0)     -- sh found /bin/echo
       time git --version          -> 3ms (exit 0)     -- on PATH
       time d                      -> sh: d: command not found, 127   -- an fsh ALIAS
       time hs                     -> sh: hs: command not found, 127   -- an fsh BUILTIN
     It worked for anything on PATH and failed for everything that was fsh's own -- because time_cmd
     delegated to `sh -c`, and sh has never heard of fsh's 303 command names or 285 aliases.
     Fix: time_cmd dispatches through execute(), which ALREADY resolves aliases (with INT-057's cycle
     guard), resolves plugins, runs builtins, and falls through to run_external for real binaries.
     The fix DELETED the second dispatcher rather than adding a third.
     Verified DEPLOYED (gen 392): time d -> the full dashboard, 683ms (exit 0). time hs -> the
     builtin's usage message, 1ms (exit 1). time git --version -> 10ms (exit 0). -->
- [x] `VAR="a b" cmd` NEVER leaves a partial value in the session environment after failing.
      Regression test with the exact QEMU_OPTS case that cost an hour on 2026-07-15.
<!-- evidence: commit d5a52c1c. THREE bugs in one code path, not one:
     A. WORD SPLITTING -- main.rs:1956 was `rest.split_whitespace().next()`, which does not know what
        a quote is. FOO="a b" cmd -> first token FOO="a -> value truncated to `a`, remainder `b"` RUN
        AS A COMMAND. Fixed with a quote-aware scan (the logic tokenize/tokenize_args already had).
     B. NO SCOPING, and worse because it fires on EVERY inline assignment, quoted or not. fsh set the
        vars and never unset them. Proven: `FOO143=1 echo scoping_test; echo [$FOO143]` -> [1].
        POSIX scopes VAR=x cmd to THAT COMMAND. Now the prior value is captured before the set and
        restored after -- removed if it did not exist, restored if it did. Runs on success AND
        failure: a FAILED command has even less business mutating durable state.
     C. FOUND BY TESTING THE FIX. With A and B fixed the QEMU_OPTS line STILL failed -- it never
        reached the fixed code. main.rs:1884 decided "standalone assignment?" with
        `after_eq.starts_with(quote) && after_eq.ends_with(quote)`, which is TRUE for
        `QEMU_OPTS="-machine q35,smm=on" echo "$QEMU_OPTS"` -- the LINE merely begins and ends with a
        quote. Now the opening quote's MATCHING PARTNER must be the last character.
     THE EXACT INCIDENT, verified DEPLOYED (gen 391):
       QEMU_OPTS="-machine q35,smm=on" echo "$QEMU_OPTS"  -> -machine q35,smm=on   (full string)
       echo "after: [$QEMU_OPTS]"                          -> after: []            (GONE)
     Regressions checked: A=1 B=2 echo multi -> multi then [][] (multi-var, scoped). X="quoted value"
     -> still persists (standalone is a different statement; INT-100 built it and it stays built). -->
- [x] Bare `python3` / `bash` still open a REPL -- the original papercut is fixed WITHOUT breaking
      the legitimate no-arg use
<!-- evidence: commit c5086945. THE CODE AND THIS INTENT CONTRADICTED EACH OTHER AND THE MEASUREMENT
     SETTLED IT. mod.rs:13403 REFUSED bare python3, justified by a comment: the REPL "looks like a
     hang in fsh". It does not hang. It is a REPL, >>> prompt, ran 1+1. The 2026-07-15 text here was
     right; the 2026-07-11 code was wrong.
     And the guard's own escape hatch was broken by the same function: it said "real REPL: python3 -i"
     -> NameError: name `i` is not defined. A workaround that had never once been run.
     ROOT CAUSE: run_python_cmd joined ALL args and ran `python3 -c "<args>"`, so every flag became
     Python SOURCE. Measured fsh vs bash, same machine, same minute:
       fsh: python3 --version -> NameError        bash: python3 --version -> Python 3.13.13
       fsh: python3 -c "print(6*7)" -> SyntaxError  bash: same -> 42
     FIX: remove "python3" from the dispatch arm. NO ARM AT ALL -- not a pass-through function, which
     would be code that can drift. run_external is `sh -c` with inherited stdio and is already correct.
     git log -S proves it was NEVER a NixOS regression: born broken in e18b4d62, 2026-04-04, four
     months AFTER the last pacman removal. Nobody had run `python3 --version` in fsh for three months
     because heredocs were the workaround -- the workaround hid the bug.
     Verified DEPLOYED (gen 388): python3 --version -> Python 3.13.13; python3 -c "print(6*7)" -> 42;
     python3 -> a real REPL; py "print(1+1)" -> 2 (fsh's own sugar kept under fsh's own names). -->
- [x] Every case above has a test that FAILS on today's fsh, so the fix is demonstrated not declared
<!-- evidence: every one, reproduced on demand BEFORE the fix and re-verified on the DEPLOYED binary
     after, not on target/debug (INT-110's checklist: "a cargo build alone shows green while the live
     command still fails"). The debug-binary-as-child pattern made it safe: build, run
     ./target/debug/faelight-shell as a nested shell, test, exit -- the live shell never at risk.
     THAT PATTERN CAUGHT A REGRESSION I CAUSED: guarding only the fallthrough broke
     `git status --short > f` (empty file), because `git` has its OWN arm calling run_external. Five
     run_external sites guarded, not one. It never reached metal. -->

## The bug this intent did not know it had -- and the two it never mentioned
FOUND BY RECON, not by the filed cases. Both were live in the deployed shell for as long as the code
has existed.

**1. EVERY REDIRECTED EXTERNAL COMMAND RAN TWICE.** (commit bfe25bc9)
    rm -rf /tmp/dirtest; mkdir /tmp/dirtest > /tmp/mk.txt
    -> mkdir: cannot create directory /tmp/dirtest: File exists
The directory did not exist. The FIRST execution created it; the second failed. `curl -X POST > log`
posted twice. `git push > out` pushed twice.
CAUSE: main.rs:2357 called commands::execute() to ASK whether a line was a builtin. execute() does not
test, it RUNS -- falls through to run_external -> `sh -c` -> the command runs, output goes to the
inherited terminal, returns Empty. main.rs read `_ => None` as "not a builtin" and spawned it AGAIN,
hand-split into args, into the file. `_ => None` could not tell "no arm matched" from "an arm matched
and printed instead of returning".
FIX: CommandResult::NotBuiltin -- an ANSWER, not an action. execute() keeps its exact old contract
(allow_external: true, never returns NotBuiltin) so all 13 call sites are safe BY CONSTRUCTION -- not
by the compiler, which caught only 1 of them. try_builtin() is the new question. allow_external is
threaded through the alias and plugin recursion, or a probe becomes a real run one expansion deep.
The word-splitting died with it: the second execution was the naive parser. Now `sh -c <cmd_part>`.

**2. FSH REPORTED SUCCESS FOR COMMANDS THAT NEVER RAN.** (commit 968c7be5) -- the worst of the night.
    nosuchcommand123 && echo "DANGER_THIS_RAN"
    -> command not found: nosuchcommand123
    -> DANGER_THIS_RAN                              <-- IT RAN ANYWAY
    false && echo "SHOULD_NOT_PRINT"   -> correctly silent
So `&&` honoured a REAL failure and ignored a TYPO. `mkae build && rm -rf dist` would have deleted
dist. `$?` said 0 for a command that never existed.
CAUSE: run_external printed "command not found" and returned CommandResult::Empty -- which MEANS
SUCCESS. Two sites (never-spawned, and exit-127). The message was always honest. The TYPE was the lie.
FOUND FOUR LAYERS DEEP while chasing a cosmetic complaint about `time` -- and fixing it fixed `time`
for free, which is the tell that `time` was the messenger and this was the message.

## What this intent taught -- beyond the fixes
THE SHAPE, six for six: A BUILTIN SHADOWS A REAL BINARY AND SWALLOWS ITS ARGUMENTS. This intent named
it on 2026-07-15 and every single case confirmed it.

THE CURE WAS ALMOST ALWAYS DELETION. python3's arm: removed. time_cmd's private sh path: removed. the
redirect branch's hand-rolled parser: removed. bash/env: one guard each, in a shape `git` had been
using four lines away all along. Nothing new to maintain, nothing new to drift. Christian's
instruction shaped this -- "fix this smartly so we do not have to run into this issue again."

THE INTENT WAS WRONG TWICE AND THE MEASUREMENT WON BOTH TIMES. `time` did not exit 127 across the
board. Bare python3 did not hang. An intent is a hypothesis; the machine is the referee.

FIVE CONFIDENT CLAIMS NOBODY CHECKED, all found on 2026-07-16: "unskippable" (INT-119's hook was never
installed), "exact replica" (login-mirror rendered different colours), "tracing causality"
(correlation_id: 475 rows, all empty strings), "mirrors framework16" (three false comments), and
"real REPL: python3 -i" (broken by the function that suggested it). The sixth is this intent's own
`_ => None`. A comment is a promise; only a run is a receipt.

## Reference
- INT-027 / INT-159 / INT-059 (2026-07-15) -- where all four cases were found the hard way
- The QEMU_OPTS incident is written up in INT-159's completion record
