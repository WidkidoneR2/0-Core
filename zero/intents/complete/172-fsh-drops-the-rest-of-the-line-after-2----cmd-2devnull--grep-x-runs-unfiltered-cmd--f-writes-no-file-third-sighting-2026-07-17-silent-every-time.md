---
id: 172
date: 2026-07-17
type: fix
title: "fsh drops the rest of the line after 2> -- `cmd 2>/dev/null | grep x` runs UNFILTERED, `cmd > f` writes no file. Third sighting 2026-07-17, silent every time"
status: complete
tags: [fsh, parser, redirect, pipes, 171, 143, regression]
---

## Vision
`cmd 2>/dev/null | grep x` must FILTER. `cmd > f 2>&1` must WRITE THE FILE. Both silently do neither.

## MEASURED 2026-07-17 -- three failures, all silent, all in one session
    ls /etc/systemd/system/multi-user.target.wants/ 2>/dev/null | grep -i ssh
      -> printed the ENTIRE directory. The pipe was dropped. Looked like a plausible answer.
    echo hello 2>/dev/null | grep -c hello
      -> printed `hello`. Should print `1`. The simplest possible case.
    sudo nix-env -p /nix/var/nix/profiles/system --list-generations > /tmp/gens.txt 2>&1
      -> NO FILE CREATED. Output went to the terminal. Only caught because the next command
         threw FileNotFoundError.
CONTROLS, same session, same minute:
    ls DIR | grep -c ssh                       -> 2      pipes work fine WITHOUT 2>
    /run/current-system/sw/bin/ls DIR 2>/dev/null | grep -c ssh
                                               -> BROKEN TOO. Not a builtin problem. The PARSER.
THE SHAPE IS INT-143's, EXACTLY: not a crash -- a plausible wrong answer. fsh silently does something
other than what was typed. The third failure corrupted a command handed over during a SECURITY intent
(INT-164), which is the precise scenario where a silently-unfiltered grep is worst.

## THE CAUSE IS NOT A MISSING CASE. IT IS A MISSING PARSER.
    expand.rs:386   // Match 2>/dev/null and 2>file FIRST
    expand.rs:387   if line.contains(" 2>/dev/null")
    expand.rs:388      || line.contains(" 2>&1")
THAT IS A LITERAL STRING MATCH ON TWO EXACT SPELLINGS. It is not parsing, it is recognition.
    `2>/tmp/log`     does not match -- different text
    `2> /dev/null`   does not match -- one space
    `cmd 2>&1 >f`    order swapped, nothing matches
And main.rs does string SURGERY on top of it:
    main.rs:2376   if working_line.contains(" 2>&1")
    main.rs:2377   let cleaned = working_line.replace(" 2>&1", "").trim().to_string();
    main.rs:2379   let (c2, _) = detect_redirect(&cleaned);
    main.rs:2381   } else if let Some(idx) = working_line.find(" 2>/dev/null")
    main.rs:1417   || lcmd_trim.contains("2>")          <- a bail-out in the chain logic
Nothing here knows that `|` exists. So once `2>` is recognised, the remainder of the line -- INCLUDING
THE PIPE -- is discarded with it.

## WHY IT KEEPS COMING BACK -- and this is the whole point of the intent
Christian, 2026-07-17: "i am just tired of these bugs because i fixed them several times, first time
when i was in arch and early when i came to nix."
HE IS RIGHT, AND THE FIXES WERE REAL. INT-291 fixed pipes (via sh fallback). INT-109 fixed
pipeline-on-left-of-&&. Each worked, in the path it touched.
BUT `2>/dev/null | grep` GOES THROUGH THE REDIRECT PATH, WHICH HAS ITS OWN PARSER -- and that parser
never learned what a pipe is. This is not a missed case. It is the SAME BUG RE-MANUFACTURED IN A
DIFFERENT LOCATION, because there is no single place where a line is understood.
Every previous fix taught one parser. The others were never in the room.

## THIS IS INT-171's FIFTH PARSER
INT-143 counted four:
    1. commands/mod.rs tokenize_args   CORRECT, quote-aware
    2. exec.rs tokenize                CORRECT, quote-aware, a BYTE-FOR-BYTE DUPLICATE of #1
    3. main.rs:2411 redirect branch    WRONG -- split_whitespace (fixed bfe25bc9)
    4. main.rs:1956 inline-VAR loop    WRONG -- split_whitespace (fixed d5a52c1c)
NOW FIVE:
    5. expand.rs:373 detect_redirect   WRONG -- contains() on two literal spellings
FSH ALREADY HAS A CORRECT TOKENIZER. TWICE. Neither is called here.

## The decision this intent must make, and it is not obvious
OPTION A -- PATCH IT. Teach detect_redirect about pipes. Probably small, probably the same one-guard
shape as INT-143's six. Fixes the pain tonight.
    AGAINST: it is the FIFTH patch to a parser that needs consolidating, and every patch makes
    INT-171's job bigger. It also teaches parser #5 what parser #1 already knows -- which is how #5
    came to exist.
OPTION B -- LET INT-171 KILL THE CLASS. One parsing entry point, one place to understand a line, and
this bug becomes unrepresentable rather than fixed.
    AGAINST: 171 is not started, October is ~10 weeks out, and `2>/dev/null | grep` is typed daily.
RECOMMENDATION: A as a HOLDING PATCH with an explicit comment pointing at 171, or B if 171 starts
first. What must NOT happen is a sixth spelling added to line 387 -- that is the fix that has failed
twice already, in Arch and in early Nix.

## Success Criteria
- [x] `git log -S 'detect_redirect'` and `git log -S '2>/dev/null'` -- HOW MANY TIMES has this been
      fixed before, and what did each fix do? Christian says several, across two distros. Archaeology
      settles it, the way `git log -S` proved python3 was born broken (e18b4d62) and was never a
      NixOS regression. This is gate ZERO: the history IS the argument
      <!-- evidence: 2026-07-17, gate ZERO answered. `git log --oneline --all -S 'detect_redirect'`
      returns 21 commits; FIVE touch the function:
        f09e56d1             INT-146 Phase 13 -- redirection > and >> BORN
        91f8f65f + 33c5e3cc  (DUPLICATE PAIR) "native stderr redirect (2>/dev/null), || and && --
                             no more sh fallback"   <-- THE BIRTH COMMIT OF THIS BUG
        ba52feb7             fix echo redirect -- builtin output writes correctly to files
        8ddb564c + bbcf6fa5  (DUPLICATE PAIR) INT-245 #10 -- bare > with no target = parse error
        273c414c             INT-246/INT-299 Phase 2 -- detect_redirect MOVED into expand.rs
                             (a move, not a fix)
      VERDICT: five touches, and the 2>-swallows-the-pipe defect has NEVER been repaired once. It has
      been live since its birth commit. The memory of fixing it "several times, first in arch" is
      REAL -- but it is about OTHER redirect bugs: echo-to-file (ba52feb7) and bare-> (8ddb564c).
      Those fixes were real and they held. This one was never in the room.
      SAME SHAPE AS INT-143's python3: born broken at e18b4d62, never a NixOS regression. Twice now
      the archaeology has returned "born broken" rather than "fixed and regressed" -- so part of what
      feels like backsliding is FIRST SIGHTINGS, not repeats.
      The history was the argument. It argued something other than this intent expected -- see the
      CORRECTION section at the foot of this file. -->
- [x] Reproduce all three failures on the DEPLOYED binary before touching anything
      <!-- gate 1 evidence: 2026-07-17, reproduced on the DEPLOYED binary (gen 395), /tmp only,
      no code touched. Every prediction was made FROM SOURCE first, then observed:

        rm -f /tmp/g1-out.txt
        echo hello 2>/dev/null | grep -c hello        -> `hello`   (POSIX: 1)  PIPE AMPUTATED
        echo REDIRECT_TEST > /tmp/g1-out.txt 2>&1     -> `REDIRECT_TEST` on the terminal,
                                                         and /tmp/g1-out.txt WAS NEVER CREATED
        echo hello 2>/tmp/g1-err | grep -c hello      -> `hello`   (POSIX: 1)  PIPE AMPUTATED

      Then, listing /tmp with python and printing repr() of each name:

        'g1-err | grep -c hello'  |  0 bytes

      A FILE WHOSE NAME IS A COMMAND. working_line[idx+3..] = `/tmp/g1-err | grep -c hello` went
      straight into File::create() as a path, because that is a legal Linux filename. Predicted from
      reading main.rs:2366-2400 BEFORE the run, then observed character for character.

      All three original failures reproduce. No hole in the trace. The mechanism in the CORRECTION
      section at the foot of this file is confirmed, not inferred. -->
- [x] Whatever the fix: `echo hello 2>/dev/null | grep -c hello` -> 1, on the DEPLOYED binary
      (INT-110: a cargo build alone shows green while the live command still fails)
      <!-- evidence: 2026-07-17, DEPLOYED binary gen 396
      /nix/store/xcn9bfr4glpcaj4izrkyg4radkpn2arf-faelight-forest-9.2.0/bin/faelight-shell
      driven through a PTY, so the INTERACTIVE REPL is what answered. This matters: `fsh -c`
      NEVER had this bug, and fsh-test only speaks `-c` -- see gate 7. Fix: commit 976862c6.
        echo hello 2>/dev/null | grep -c hello   -> `1`   (gate 1, same day: `hello`)
        echo hello | grep -c hello               -> `1`   control, unchanged
      -->
- [x] `cmd > f 2>&1` writes the file, with BOTH streams in it
      <!-- evidence: 2026-07-17, DEPLOYED binary gen 396
      /nix/store/xcn9bfr4glpcaj4izrkyg4radkpn2arf-faelight-forest-9.2.0/bin/faelight-shell
      driven through a PTY, so the INTERACTIVE REPL is what answered. This matters: `fsh -c`
      NEVER had this bug, and fsh-test only speaks `-c` -- see gate 7. Fix: commit 976862c6.
        ls /tmp/definitely-not-here /tmp > /tmp/g4-out.txt 2>&1
        -> FILE CREATED, 15902 bytes, and it CONTAINS "No such file" -- so BOTH streams
           landed, not merely the file appearing. At gate 1 this created NO FILE AT ALL.
      The 2>&1-to-file code always existed and was UNREACHABLE; deleting the arms that
      intercepted the line is what let sh do the job instead.
      -->
- [x] `cmd 2>/tmp/err | grep x` works  <!-- PREMISE CORRECTED 2026-07-17: this gate STANDS -- it is a good
      test -- but the reason given for it below is WRONG. `2>/tmp/err` DOES match today, via a third
      catch-all clause this intent's text did not know about. It breaks for a different reason. See
      the CORRECTION section at the foot of this file. --> -- a spelling that has NEVER been in the contains() list. If
      the fix only handles the two known spellings, it is the same fix that already failed twice
      <!-- evidence: 2026-07-17, DEPLOYED binary gen 396
      /nix/store/xcn9bfr4glpcaj4izrkyg4radkpn2arf-faelight-forest-9.2.0/bin/faelight-shell
      driven through a PTY, so the INTERACTIVE REPL is what answered. This matters: `fsh -c`
      NEVER had this bug, and fsh-test only speaks `-c` -- see gate 7. Fix: commit 976862c6.
        echo hello 2>/tmp/g4-err | grep -c hello  -> `1`
      AND THE TELL: no file named 'g4-err | grep -c hello' was created. `g4-err` was a fresh
      name. At gate 1 the same shape left a file literally named 'g1-err | grep -c hello'.
      The pipeline no longer becomes a path.
      -->
- [x] `cmd 2>&1 | grep x` works (order swapped)
      <!-- evidence: 2026-07-17, DEPLOYED binary gen 396
      /nix/store/xcn9bfr4glpcaj4izrkyg4radkpn2arf-faelight-forest-9.2.0/bin/faelight-shell
      driven through a PTY, so the INTERACTIVE REPL is what answered. This matters: `fsh -c`
      NEVER had this bug, and fsh-test only speaks `-c` -- see gate 7. Fix: commit 976862c6.
        echo hello 2>&1 | grep -c hello   -> `1`
      -->
- [x] Regression tests that FAIL on today's fsh, per INT-158 and INT-143's discipline
      <!-- evidence: 2026-07-17. THE GATE AS WRITTEN WAS IMPOSSIBLE, and finding out why is the most
      important thing this intent produced.
      fsh-test's run_fsh() invokes `fsh -c`. MEASURED: `fsh -c` NEVER HAD THIS BUG. Only the
      interactive REPL did. Same binary, same day, opposite results:
          fsh -c 'echo hello 2>/dev/null | grep -c hello'  -> 1       CORRECT
          the same line typed at the prompt                -> hello   WRONG
      So no fsh-test case could EVER have failed on it. 83/83 green never meant "fsh works" -- it
      meant "the -c path works". fsh has two front doors and only one was ever tested.
      BUILT INSTEAD: faelight/rust-tools/fsh-test/src/repl.rs -- a pty driver. fsh asks isatty(); with
      a plain pipe it answers "no terminal" and the REPL never exists. A pty is how you make it believe
      a human is there. nix 0.31.1 (`features = ["term"]`) was ALREADY in Cargo.lock four times over,
      because rustyline pulls it -- so this added ZERO new crates. Five tests, new Category::Repl:
        repl_pipe_control_no_redirect        repl_stdout_redirect_with_2to1
        repl_stderr_null_then_pipe           repl_pipeline_never_becomes_a_filename
        repl_2to1_then_pipe
      WATCHED IT FAIL FIRST, per INT-158. Built a shell from 976862c6^ (the commit before the fix),
      saved it as /tmp/fsh-broken, restored the tree, then ran the suite against each:
          RED   /tmp/fsh-broken            85 / 88   -- ALL 83 ORIGINAL TESTS PASSED.
                                                        Only the new REPL tests failed:
                                                          repl_stderr_null_then_pipe
                                                            expected "1" got "hello"
                                                          repl_stdout_redirect_with_2to1
                                                            no file created (os error 2)
                                                          repl_pipeline_never_becomes_a_filename
                                                            ["fsh_test_g7err | grep -c hello"]
          GREEN target/debug/faelight-shell 88 / 88
      The old suite is 100% green on a demonstrably broken shell. That is the receipt.
      repl_pipeline_never_becomes_a_filename is the July 12 fossil turned into an assertion. A file
      named 'pi.err | python3 -c "..."' sat in /tmp for five days, holding 201 bytes of swallowed
      stderr from real work that never ran. This test is the reason that cannot happen unnoticed again.
      ONE TEST BUG, found by running red then green: the filename check did not clear leftovers, so it
      failed against a FIXED shell because the RED run had just created the file it looks for. Fixed --
      it now clears /tmp/fsh_test_g7err* first. The test caught its own flaw by being run twice.
      KNOWN COST: ~5s per REPL test (fsh's banner spawns `nixos-rebuild list-generations --json` on
      every start), so the suite goes from ~1.5s to ~26s. Recorded, not hidden. -->
- [x] The fix names its relationship to INT-171 in a comment: holding patch, or the consolidation
      itself. A fifth parser patched without saying so is how there came to be five
      <!-- evidence: commit 976862c6. The comment is IN THE CODE, and it says this is NEITHER:
      "RELATION TO INT-171: this is neither a holding patch nor the consolidation. The `2>`
       handling stops PARSING and becomes a ROUTER -- one boolean saying 'this line has a 2>,
       give it to sh whole'. 171's inventory goes from five parsers to four parsers and a
       router. A deletion makes 171's job SMALLER."
      That inverts this intent's own argument against Option A, which was that every patch
      makes 171 bigger. True of a patch. False of a deletion: 73 lines out, 50 in, and most
      of the 50 is this comment. -->

## The Rule
"Every previous fix was real. Each taught ONE parser. The others were never in the room." 🌲

## CORRECTION (2026-07-17): the cause above is WRONG -- it is not expand.rs:387

Gate zero's archaeology, plus a read of the deployed source, corrected this intent's diagnosis in
three places. Per INT-027's convention the original text is left ABOVE verbatim -- the wrong turn is
part of the record.

### WRONG #1 -- "A LITERAL STRING MATCH ON TWO EXACT SPELLINGS"

There are THREE clauses, not two. The deployed source, expand.rs detect_redirect:

    if line.contains(" 2>/dev/null")
        || line.contains(" 2>&1")
        || (line.contains(" 2>") && !line.contains(" 2>="))

The third is a CATCH-ALL. So `2>/tmp/log` DOES match. `2> /dev/null` (one space) DOES match.
`cmd 2>&1 >f` DOES match. The claim that these "do not match -- different text" is false.

### WRONG #2 -- gate 5's premise

Gate 5 demands `cmd 2>/tmp/err | grep x` work as "a spelling that has NEVER been in the contains()
list". That spelling matches today, via the catch-all, and still breaks. The gate STANDS; it tests
something other than what it claims. And the danger this intent named -- "what must NOT happen is a
sixth spelling added to line 387" -- was never the risk. Widening the match fixes nothing, because
matching was never the failure.

### WRONG #3 -- the cause is PREFIX TRUNCATION in main.rs, not recognition in expand.rs

`__stderr__` has exactly THREE occurrences in the whole tree:

    expand.rs:393   creation
    main.rs:2370    consumer
    main.rs:2400    consumer

expand.rs does not parse `2>` at all. All three clauses return the ENTIRE UNMODIFIED LINE plus a
sentinel, and its own comment says so: "The caller will handle 2> patterns natively." The line is
destroyed by the CALLER. main.rs:2366-2400:

    let working_line = if redirect_target == "__stderr__" { line } else { line_stripped.as_str() };
    let (cmd_part, stderr_to_stdout, stderr_file) =
        if working_line.contains(" 2>&1") {
            let cleaned = working_line.replace(" 2>&1", "").trim().to_string();
            let (c2, _) = detect_redirect(&cleaned);          // <-- DISCARDS the stdout target
            (c2, true, None)
        } else if let Some(idx) = working_line.find(" 2>/dev/null") {
            (working_line[..idx].trim().to_string(), false, Some("/dev/null".to_string()))
        } else if let Some(idx) = working_line.find(" 2>") {
            let after = working_line[idx + 3..].trim().to_string();
            (working_line[..idx].trim().to_string(), false, Some(after))
        } else { (line_stripped.clone(), false, None) };

THE DEFECT IS `working_line[..idx]`. PREFIX TRUNCATION. The two `find(" 2>...")` arms take the text
LEFT of the `2>` token as the whole command and throw away everything to its right -- including the
pipe. Nothing there parses past the redirect target.

CORRECTION TO THE CORRECTION (2026-07-17, later the same day): this paragraph first said "EVERY arm".
That was WRONG, and gate 7's red run disproved it. The `2>&1` arm does NOT truncate -- it does
`working_line.replace(" 2>&1", "")`, which removes the TOKEN and keeps the REST, so the pipe survives;
`detect_redirect(&cleaned)` then finds no `>` and hands back the whole line, so `c2` keeps it too.
PROVEN: on a binary built from 976862c6^, `repl_2to1_then_pipe` (`echo hello 2>&1 | grep -c hello`)
PASSED. That shape was never broken. So gate 6 was testing something that already worked, and the
three real failures were TWO truncations plus ONE no-file -- and the no-file case dies from the
`(c2, _)` stdout-target discard plus the hardcoded `.stdout(inherit)`, not from truncation at all.
The fix is unaffected; the description of the defect was over-general. Written down rather than
quietly edited, because a correction that hides its own correction is the disease this file is about. The intent's "IT IS A MISSING PARSER" reading is right in
SPIRIT and points at the WRONG LINE.

### The three failures, traced from source (2026-07-17)

1. `echo hello 2>/dev/null | grep -c hello`
   find(" 2>/dev/null") -> idx -> cmd_part = working_line[..idx] = `echo hello`.
   ` | grep -c hello` AMPUTATED. Runs `sh -c "echo hello"`, stdout inherited -> prints `hello`.
   That is the observed output, explained exactly.

2. `cmd 2>/tmp/log | grep x`
   third arm -> after = working_line[idx+3..] = `/tmp/log | grep x` -- and that ENTIRE STRING
   becomes the FILENAME. File::create("/tmp/log | grep x") is a LEGAL Linux filename, and it
   SUCCEEDS. CONFIRMED ON THE DEPLOYED BINARY 2026-07-17 (gate 1):
   `echo hello 2>/tmp/g1-err | grep -c hello` left a file named 'g1-err | grep -c hello' (0 bytes)
   in /tmp. Predicted from source, then observed, character for character. And .unwrap_or(Stdio::inherit())
   means any failure silently inherits instead of reporting.

3. `ls /tmp > /tmp/out.txt 2>&1`  -- THREE throw-aways in one path (and note: NOT truncation --
   this arm uses replace(), see the correction above):
   (a) detect_redirect checks 2> BEFORE the > arm ("Match 2>/dev/null and 2>file FIRST"), so
       __stderr__ wins and the > arm never runs;
   (b) `let (c2, _) = detect_redirect(&cleaned)` discards the stdout target into `_`;
   (c) is_stderr_only routes to a branch with .stdout(Stdio::inherit()) HARDCODED -- it never opens
       a stdout file at all.
   -> output to terminal, NO FILE. Also `stderr_to_stdout` is computed `true` and then read by
   NOTHING in that branch -- a dead binding.

### What is NOT broken -- recorded because the scope felt bigger than it is

detect_redirect is 3/4 CORRECT. The `>` and `>>` arms use rfind(" >> ") / rfind(" > "), split cmd
from path properly, and deliberately refuse `> 70` and `>= x` as comparisons. The INT-245 #10
bare-redirect guard works. The `2>` arm is the ONE outlier -- bolted on differently from its three
siblings and never finished. The entire mechanism is ~40 lines across three sites. This is not an
18,000-line problem. It is one arm.

### The OPTION SET HAS CHANGED -- re-decide before writing code

OPTION A as written says "teach detect_redirect about pipes." That is aimed at the WRONG FILE.
detect_redirect is not where the line dies. A holding patch means making main.rs parse the redirect
SEGMENT instead of truncating the line at an index -- which is most of what parser #5 consolidation
would have done anyway. A and B are much closer together than this intent assumed. Decide against
the traced mechanism, not the original diagnosis.

### Found alongside, no intent yet

expand.rs::expand_subshells is a hand-rolled char scanner hunting `$(` with NO quote-state variable
anywhere in the function. CONFIRMED ON THE DEPLOYED BINARY 2026-07-17:
`echo '$(date)'` -> `Fri Jul 17 03:18:17 AM CDT 2026`. `sh` prints the literal `$(date)`; fsh
executes it inside single quotes. It also drops errors silently -- .output() takes stdout only and
.unwrap_or_default() turns any failure into an empty string, so `$(nosuchcmd)` yields "". That is
INT-245's silent-drop pattern (cb298fc4), and a SIXTH `sh -c` seam.

### The method note worth keeping

Both this intent's diagnosis AND the assistant's first hypothesis (an "asymmetry" between a
3-clause raiser and a 2-spelling handler) were WRONG. main.rs has a matching third arm. Each was
killed by a lookup that took seconds, before either reached code. Recon is not ceremony.
