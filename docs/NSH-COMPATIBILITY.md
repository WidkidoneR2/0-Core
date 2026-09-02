# NovaShell Compatibility Contract

**Status:** first written 2026-09-02. This is a contract, not a description --
it says what nsh promises, and a release that violates it is a defect or a
declared break, never an accident.

---

## Why this exists

Version numbers were being chosen by weight. A one-line index change took
interactive startup from 400ms to 55ms and a dozen-line guard change altered
what a command does; the first is enormous and compatible, the second is small
and not. Size is the wrong axis and it gave the wrong answer twice in one day.

The right axis is obligation. This document defines what is owed, so a version
can describe a relationship to it rather than an opinion about effort.

---

## Who the caller is

For a shell with one operator, "the public interface" is concrete rather than
hypothetical:

- the commands typed at the prompt, and their history
- `~/.config/faelight-shell/config.nsh` -- aliases, settings, and the file itself
- `.nsh` scripts run through `run`
- anything invoking `nsh -c`
- exit codes and stream behaviour that a script branches on

That is the caller. It is small enough to enumerate, which is what makes a
contract possible here and difficult for a general-purpose shell.

---

## The promise

> A line that worked in a previous nsh works in this one, unless the change was
> declared.

Concretely, across a release:

- a command that executed still executes
- its arguments are interpreted the same way
- its exit status stays compatible
- variables behave the same way
- aliases resolve to the same thing
- `config.nsh` still loads, and its settings still mean what they meant
- existing `.nsh` scripts still run
- history entries still replay
- output and error behaviour a script depends on is preserved
- existing options keep their meaning

---

## Three kinds of behaviour, and only one is owed

This is the distinction that makes the promise keepable. Without it, every
historical accident becomes sacred and the shell can never be corrected.

**CONTRACTUAL** -- what nsh intentionally supports. Documented, tested, or
plainly implied by a feature existing. This is what the promise covers.

**INCIDENTAL** -- what happens to work but was never offered. Undocumented
output formatting, the precise wording of a message, the order of unordered
things. Not guaranteed, but changing it deserves a note.

**ERRONEOUS** -- what violates nsh's own stated semantics. A guard that claimed
to cover every execution path and did not. A rule that could never fire. A
message asserting something untrue. Correcting these is not a break, even when
the correction is observable.

⚠️ THE TEST FOR THE THIRD CATEGORY: did the user have a reasonable basis for
depending on the old behaviour? Nobody reasonably depended on `nsh -c 'rm -rf
/etc'` running ungated while the same line was challenged at the prompt. That
was erroneous, and fixing it is a patch with documentation -- not a minor.

---

## What the version number means

nsh uses semver, and the level is determined by contract impact alone. Not by
lines changed, files touched, hours spent, or how the change feels.

**PATCH** -- the contract is unchanged.
Internal refactoring. Performance. Crash fixes. Correcting a documented feature
that malfunctioned. Correcting ERRONEOUS behaviour. Improved diagnostics whose
contractual meaning is the same.

**MINOR** -- the contract grows and nothing in it is invalidated.
A new builtin, option, setting, syntax or scripting facility, where every
existing script behaves exactly as before.

**MAJOR** -- something contractual is intentionally invalidated.
A command removed. Syntax no longer accepted. An argument's meaning changed. A
config key that means something else now.

A one-line change can be major. A fifty-thousand-line rewrite can be a patch.

---

## The decision, as a question sequence

1. Did observable behaviour change? No -> **PATCH**.
2. Was the old behaviour CONTRACTUAL? No -> **PATCH**, and document it.
3. Was the change intentional, rather than a correction? No -> **PATCH**.
4. Otherwise, was capability added while existing behaviour held? -> **MINOR**.
5. Otherwise -> **MAJOR**, and it is declared in the release, not discovered.

---

## The suite is the enforcement, eventually

The strongest form of this contract is not prose. It is a compatibility suite
that replays nsh's own past:

    old config.nsh loads
    historical commands still parse
    aliases resolve as they did
    .nsh scripts still run
    exit codes hold
    history replays

Then "is this release compatible" stops being a judgement and becomes a run.
nsh-test already asserts 193 behaviours and `spine migrate` already replays
43,469 history entries against two parsers -- the machinery exists and is not
yet pointed at this question.

⚠️ UNTIL THAT EXISTS, THE CONTRACT IS ENFORCED BY READING. Say so rather than
implying more rigour than there is.

---

## The alternative that was considered and not taken

Sequential versioning plus an unconditional promise -- the Linux model, where
the number counts releases and compatibility is guaranteed absolutely rather
than encoded.

It is coherent and it was rejected for one reason: Torvalds can hold that line
because he never corrects erroneous userspace behaviour. The bug becomes the
contract, permanently. nsh does the opposite -- two days of this week were spent
correcting things that were wrong, several of them observably.

So the contractual/incidental/erroneous judgement has to be made either way.
Semver at least reports the answer.
