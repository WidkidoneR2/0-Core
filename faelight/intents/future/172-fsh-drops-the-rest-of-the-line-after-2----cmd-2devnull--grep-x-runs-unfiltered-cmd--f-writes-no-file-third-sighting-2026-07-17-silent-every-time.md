---
id: 172
date: 2026-07-17
type: fix
title: "fsh drops the rest of the line after 2> -- `cmd 2>/dev/null | grep x` runs UNFILTERED, `cmd > f` writes no file. Third sighting 2026-07-17, silent every time"
status: planned
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
- [ ] `git log -S 'detect_redirect'` and `git log -S '2>/dev/null'` -- HOW MANY TIMES has this been
      fixed before, and what did each fix do? Christian says several, across two distros. Archaeology
      settles it, the way `git log -S` proved python3 was born broken (e18b4d62) and was never a
      NixOS regression. This is gate ZERO: the history IS the argument
- [ ] Reproduce all three failures on the DEPLOYED binary before touching anything
- [ ] Whatever the fix: `echo hello 2>/dev/null | grep -c hello` -> 1, on the DEPLOYED binary
      (INT-110: a cargo build alone shows green while the live command still fails)
- [ ] `cmd > f 2>&1` writes the file, with BOTH streams in it
- [ ] `cmd 2>/tmp/err | grep x` works -- a spelling that has NEVER been in the contains() list. If
      the fix only handles the two known spellings, it is the same fix that already failed twice
- [ ] `cmd 2>&1 | grep x` works (order swapped)
- [ ] Regression tests that FAIL on today's fsh, per INT-158 and INT-143's discipline
- [ ] The fix names its relationship to INT-171 in a comment: holding patch, or the consolidation
      itself. A fifth parser patched without saying so is how there came to be five

## The Rule
"Every previous fix was real. Each taught ONE parser. The others were never in the room." 🌲
