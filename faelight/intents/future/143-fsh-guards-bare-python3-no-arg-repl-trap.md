---
id: 143
date: 2026-07-11
type: future
title: "fsh guards bare python3 (no-arg REPL trap)"
status: planned
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
- [ ] `bash /tmp/x.sh` either RUNS the script or refuses with a message naming the workaround --
      it never silently drops into an interactive shell
- [ ] `env VAR=x cmd` either runs cmd or refuses -- it never silently prints the environment instead
- [ ] `time cmd` either times cmd or refuses -- no silent exit 127
- [ ] `VAR="a b" cmd` NEVER leaves a partial value in the session environment after failing.
      Regression test with the exact QEMU_OPTS case that cost an hour on 2026-07-15.
- [ ] Bare `python3` / `bash` still open a REPL -- the original papercut is fixed WITHOUT breaking
      the legitimate no-arg use
- [ ] Every case above has a test that FAILS on today's fsh, so the fix is demonstrated not declared

## Reference
- INT-027 / INT-159 / INT-059 (2026-07-15) -- where all four cases were found the hard way
- The QEMU_OPTS incident is written up in INT-159's completion record
