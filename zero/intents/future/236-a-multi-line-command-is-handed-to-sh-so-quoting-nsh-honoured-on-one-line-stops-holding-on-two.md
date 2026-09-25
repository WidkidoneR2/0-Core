---
id: 236
date: 2026-08-31
type: fix
title: "A multi-line command is handed to sh, so quoting nsh honoured on one line stops holding on two"
status: planned
tags: [nsh, quoting, delegation, sh, int-203, int-195]
---

## Vision

Quoting means the same thing on line two as it does on line one.

## The Problem

MEASURED 2026-09-01, both directions, on the deployed binary:

    echo "one `echo LEAKED`"            ->  one `echo LEAKED`     LITERAL
    echo "one `echo LEAKED`
    two"                                ->  one LEAKED / two      EXECUTED

Same quotes, same substitution, opposite outcome. The only difference is a
newline inside the string.

⚠️ AND `$(...)` DOES **NOT** BEHAVE IDENTICALLY -- this was recorded as general
command substitution and the Leak suite corrected it on its first run,
2026-09-01. Nine of ten metacharacters survive the line break unchanged:
`$(...)`, `$HOME`, `${HOME}`, `;`, `&&`, `|`, `>`, `*` and `{a,b}`. Only
BACKTICKS differ.

The original claim came from running the two-line `$(...)` case, seeing it
substitute, and matching it to the backtick result -- WITHOUT running the
one-line control. `$(...)` substitutes in BOTH forms, which is consistent, and
consistent is the contract. Verified by hand after the suite disagreed.

SO THE DEFECT IS NARROWER AND STRANGER. Two syntaxes for one operation:
`$(...)` always substitutes, backticks substitute only across a line break.
They disagree with each other, and backticks disagree with themselves.

WHY IT MATTERS BEYOND THE CURIOSITY. Multi-line quoted text is PASTED text --
a commit message, a config block, a code snippet, a log extract. Anything
inside it that looks like a substitution runs. Quoting is the mechanism that
is supposed to make text inert, and it stops working at exactly the size
where the text stops being typed and starts being pasted.

It was found by accident: `git commit -m` with a message containing
`` `fg` ``, `` `let X = v` `` and `` `export X = v` `` printed three
`sh: line 1:` errors during the commit. The message stored intact, so the
substitution happened alongside the real work rather than corrupting it --
which is the quiet kind.

## WHAT IS NOT THE BUG, EACH CHECKED RATHER THAN ASSUMED

⚠️ Three plausible causes were eliminated by reading the code and running it.
Recording them because the next reader will suspect the same three.

1. `split_into_commands` (expand.rs:381) is CORRECT. The theory was that it
   splits a quoted string at the newline, losing the quoting. It does not:
   it calls `is_complete_command` per accumulated line and only splits where
   that reports complete. Proven live -- `echo "one\ntwo"` printed `one` and
   `two` from ONE command, and the session counted 2 commands including exit.

2. `is_complete_command` (expand.rs:212) is CORRECT for quotes. It walks the
   WHOLE buffer, not each line, and returns "unclosed double quote" properly.
   The per-line quote state at 216-218 is the COMMENT-STRIPPING pass and does
   reset per line -- a real defect for a `#` inside a multi-line string, and a
   DIFFERENT one from this. Worth its own look; not this.

3. The pre-commit hook does not read the message. No `$1`, no
   `COMMIT_EDITMSG`.

## The Solution

THE MECHANISM. nsh does not execute multi-line constructs -- engine.rs says
so plainly, and the spine declines them. Legacy then hands the INTACT buffer
to `sh`, and `sh` applies ITS substitution rules to text nsh had kept
literal. The `sh: line 1:` prefix on the original errors is the signature.

So nsh honours the quoting right up to the moment it gives up, and
delegation silently changes the rules underneath text that already passed
through a shell that promised not to touch it.

THE SHAPE IS FAMILIAR AND THAT IS THE ARGUMENT FOR FIXING IT PROPERLY:
  - INT-203: brace expansion ran on heredoc BODIES under a quoted delimiter
  - `nsh -c` delegated whole strings to sh, so no alias, guard or spine
    applied -- and a green conformance suite was measuring sh
  - INT-195: every stage must consume the previous stage's OUTPUT, never the
    original string

Each time, a boundary that looked like a handoff was a re-parse.

DIRECTION, NOT YET A DECISION. Either nsh learns to execute the multi-line
forms it currently delegates, or the delegation stops passing text that has
already been quote-processed. The first is INT-169's arc. The second is
smaller and might be enough. Measure which multi-line forms actually reach
sh before choosing.

## Success Criteria

- [ ] The reproduction is a test before it is a fix: a case that FAILS on
      today's binary, showing `one LEAKED` where `one \`echo LEAKED\`` is
      required
- [ ] Named: which multi-line forms reach `sh`, measured with the router
      trace rather than reasoned about
- [ ] Ruled: teach nsh the forms, or stop delegating processed text. The
      ruling is written down with what it costs
- [ ] Fixed: a multi-line double-quoted string treats substitutions exactly
      as the single-line form does
- [ ] The per-line comment-stripping defect is filed separately or fixed
      here, with the choice stated

## THE LEAK TEST -- the gate that generalises

- [x] nsh-test carries a `Leak` category that feeds shell metacharacters
      through the shell and asserts they arrive UNCHANGED. Not a test for
      this bug; a test for this CLASS.
<!-- DONE 2026-09-01. Ten metacharacters, each run inside double quotes on one
line and again split across two, asserting the two forms agree. The comparison
IS the assertion, so no table of expected strings can go stale.

IT CORRECTED THIS CHARTER ON ITS FIRST RUN. Nine of the ten agree; only
backticks differ. The claim that dollar-paren behaves identically was wrong --
it substitutes in BOTH forms, which is consistent. See the amended problem
statement above.

leak_backtick carries a DECLARED DIVERGENCE using INT-202 four-arm shape, so a
known defect does not block every push and cannot be silenced either: the case
fails if it starts AGREEING, which is the reminder to remove the declaration
when this intent is fixed. 188/188. -->

The principle: text nsh treats as literal must not come out evaluated. Every
case is the same shape -- put a metacharacter inside quoting that should make
it inert, and compare what comes back against what went in.

The matrix, each on ONE line and again spanning TWO, because that difference
is the whole finding:

    `echo X`        backtick substitution
    $(echo X)       modern substitution
    $HOME           variable expansion
    ${HOME}         braced expansion
    a;b             command separator
    a && b          logical operator
    a | b           pipe
    a > f           redirect
    *               glob
    {a,b}           brace expansion

A case fails if the output differs between the one-line and two-line form.
That comparison is the assertion -- it needs no table of expected strings,
and it stays true if the quoting rules ever change deliberately.

⚠️ AND IT MUST DRIVE THE REPL. Through `-c` the string goes to sh, which
measures sh's quoting and calls it nsh's -- the exact mistake INT-202 found
in the conformance suite.

## Notes

- Found while writing a commit message during the nsh rename, 2026-09-01.
- bash SUBSTITUTES a backtick inside double quotes through `-c`; nsh keeps it
  literal. So nsh is stricter on one line and looser on two, which is the
  worst of both.
- `is_complete_command`'s comment pass resets quote state per line, so a `#`
  inside a multi-line string is treated as a comment. Adjacent, real, and
  not measured yet.
