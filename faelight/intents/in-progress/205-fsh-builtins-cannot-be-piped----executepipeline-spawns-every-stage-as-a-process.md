---
id: 205
date: 2026-08-06
type: future
title: "fsh builtins cannot be piped -- execute_pipeline spawns every stage as a process"
status: in-progress
tags: [fsh, pipeline, spawns, builtins]
---

## Vision
A builtin is a command. `spine parse x | cat` should work for the same reason `ls | cat` does, and
nothing about being implemented inside fsh should change that.

## The Problem
    spine parse echo hi | cat
    x   spine: No such file or directory (os error 2)

The router claimed the line. `spine` is a real builtin. It failed anyway, because
`commands::execute_pipeline` builds every stage with `Command::new(program)` and asks the operating
system for a binary called `spine`. There is none.

WHY IT LOOKED FINE UNTIL NOW. Most piped commands are external, so most pipelines work. `echo hi |
cat` succeeds because /bin/echo exists -- not because the builtin ran. That is the more worrying half
of this: a builtin that SHARES A NAME with a binary does not fail, it silently runs the OTHER ONE.
Whether fsh has any such builtin is a question this intent must answer rather than assume.

THE SINGLE-COMMAND PATH ALREADY GETS THIS RIGHT. execute_plan_dispatch tries the builtin first and
falls through to a spawn only when the answer is NotBuiltin. The pipeline path never asks. So the
shell has two different ideas of what a command is, depending on whether a pipe is present.

## THE DESIGN QUESTION THIS INTENT EXISTS TO ANSWER
A builtin is not a process. It returns a value; it does not have a file descriptor. So "just check
for builtins in the pipeline too" is not an implementation, it is the beginning of one. The real
question is how a builtin participates in a chain of processes:

  BUFFERED     run the builtin, capture its output, write that into the next stage's stdin. Simple
               and correct for finite output. ⚠️ But it breaks streaming: `somebuiltin | head -1`
               would run the builtin to completion before head sees a byte, and a long-running or
               infinite builtin would never hand over at all.

  THREADED     run the builtin on a thread writing into a pipe, so downstream sees output as it is
               produced. Preserves streaming and early termination. Costs a thread per builtin stage
               and needs a decision about what a builtin does when its reader goes away.

  REFUSED      detect a builtin in a pipeline and say so plainly. Not a fix, but it is honest, and it
               is strictly better than handing the name to the operating system.

⚠️ AND STDIN IS THE OTHER HALF. A builtin in a middle or final position must READ the previous stage's
output. Builtins today take a line of arguments, not a stream. Whether any builtin should consume
stdin at all is a scope decision, and answering it narrowly is legitimate -- but it must be answered,
not discovered later.

## THE DECISION (gate 3, 2026-08-07): the rule already exists -- apply it to pipes
This does not need a new rule. It needs an existing one to reach one more place.

execute_plan_dispatch already answers this question for a REDIRECT. It checks whether a real binary
exists on PATH and, if so, spawns that instead of the builtin, with this reason: fsh's builtins are
RENDERERS as much as commands. Its cat returns line-numbered, ANSI-dimmed text for source files, and
writing a rendering into a file is a category error rather than a formatting quirk. The comment calls
itself BUG-298-4's cat-with-redirect rule generalised.

FEEDING A RENDERING INTO grep IS THE SAME CATEGORY ERROR. So the rule transfers word for word, and
this intent is the next generalisation: from redirects to pipes.

THE MECHANISM IS INVERTED ON PURPOSE, and that inversion is what makes it implementable. The dispatch
comment says it plainly: try_builtin cannot answer "is this a builtin?" without running it, so the
question asked instead is "is there a real binary?" -- program_on_path, a pure predicate that already
exists. Nothing new is invented and there is no second owner of the answer.

WHAT CHANGES. Per pipeline stage: if a real binary exists, spawn it. That is exactly what happens
today, so every working pipeline keeps working. If no binary exists -- spine, intl, d -- dispatch to
the builtin through ExecutionMode::Spine, which the single-command path already uses, and buffer its
output into the next stage's stdin.

WHERE IT GOES. Peel the builtin stage ABOVE spawn_pipeline rather than teaching spawn_pipeline about
builtins. That function returns a vector of children and a builtin is not a child, and background_
pipeline shares it -- so the shape must not change.

⚠️ BUFFERED, NOT STREAMED, AND SAY SO. A builtin returns a value rather than a file descriptor, so
its output is complete before the next stage sees any of it. For a finite builtin that is invisible.
It is stated here because streaming and early termination are what a reader would otherwise assume.

⚠️ FIRST STAGE ONLY. A builtin later in a chain would have to READ its predecessor's output, and
builtins take arguments rather than a stream. That is out of scope, refused with a clear message, and
the refusal is the honest answer rather than a half-working chain.

⚠️ AND A LIMIT WORTH STATING RATHER THAN HIDING. "Prefer the real binary" is right when the two
commands do the same job -- real cat, real ps, real grep should win in a pipe. It is WRONG when an
fsh builtin merely shares a name with an unrelated program. `last` is the case: in fsh it shows the
previous command's output, and on PATH it shows login history. So `last | grep x` will silently run
the login-history tool. This intent does not fix that; naming it is the point, because the fix is a
question about the builtin namespace rather than about pipes.

⭐ AND ONE REFINEMENT DELIBERATELY NOT TAKEN. The redirect rule keys on who consumes the output, so a
strict reading says a LAST stage whose stdout reaches the terminal could legitimately use the renderer
builtin -- `git log | cat`. That would change existing behaviour, which this intent does not need to
do. It is recorded as an open question rather than built.

## Why this matters more than its size suggests
It is a daily-driver limitation with a one-line reproduction, and it is the roadmap's own Lane 0 item
resurfacing: "operators punt the whole line to bare sh -- the deeper fix, interleaving fsh builtins
with pipe and redirect handling, is acknowledged future work, unfiled." It is filed now, and it has
moved: it is no longer about sh delegation, it lives in execute_pipeline, and the pipeline executor
is code this project owns.

## Success Criteria
- [ ] RED FIRST, RECORDED: `spine parse echo hi | cat` fails today with the operating system's
      not-found error, captured before any change.
- [x] The shadowing question is MEASURED, not assumed: for every builtin whose name also exists as a
      binary on PATH, determine which one a pipeline stage currently runs. If a pipeline silently
      runs the external one, that is a second finding and belongs in this intent's body.
<!-- evidence: measured 2026-08-07. 35 builtin names also exist as binaries on PATH: cat, clear,
     core, db-browse, diff, echo, env, faelight-git, faelight-notify, faelight-release,
     faelight-shell, faelight-term, fd, file, find, friday-chat, fsh, git, grep, idle, info, last,
     make, patch, ps, pwd, python, realpath, rename, sh, test, time, watch, which, write. In a
     pipeline the EXTERNAL one runs, and per the decision above that is correct for almost all of
     them -- real cat, ps and grep should win. The exception is `last`, whose fsh meaning (the
     previous command's output) is unrelated to the binary's (login history), so `last | grep x`
     silently runs the login-history tool. Named as a limit in the decision rather than fixed. -->
      binary on PATH, determine which one a pipeline stage currently runs. If a pipeline silently
      runs the external one, that is a second finding and belongs in this intent's body.
- [x] The design question above is ANSWERED IN WRITING before implementation -- buffered, threaded,
      or refused -- with the streaming consequence stated. If the answer is REFUSED, say so plainly
      and close; a stated limitation is a legitimate outcome.
<!-- evidence: the decision section above, added 2026-08-07. Chosen: apply execute_plan_dispatch's
     existing prefer-the-real-binary-under-a-redirect rule to pipes, because its stated reason --
     builtins are renderers, and writing a rendering into a file is a category error -- transfers
     to feeding a rendering into grep unchanged. Buffered rather than threaded, and said so, since
     a builtin returns a value rather than a descriptor. First stage only; later positions refused,
     which this intent's guardrails allow explicitly. Refusing outright was rejected because the
     mechanism already exists (ExecutionMode::Spine plus program_on_path) and inventing a second
     owner of builtin-versus-external is the split-brain INT-193 exists to prevent. -->
      or refused -- with the streaming consequence stated. If the answer is REFUSED, say so plainly
      and close; a stated limitation is a legitimate outcome.
- [ ] A builtin works as the FIRST stage of a pipeline.
- [ ] A builtin's position beyond the first is either supported or explicitly out of scope, with the
      stdin question answered rather than left open.
- [ ] ⭐ IT CANNOT BREAK AGAIN: fsh-test carries a case that pipes a builtin with no binary behind it
      and asserts real output. Run it against the current build FIRST and watch it fail, so the test
      is known to be capable of catching this rather than assumed to be.
- [ ] The single-command and pipeline paths agree about what a command is -- one place decides
      builtin versus external, not two.
- [ ] Each gate carries evidence per INT-158.

## Scope guardrails
- ⚠️ DO NOT open this by adding a builtin lookup inside execute_pipeline alongside the existing one in
  execute_plan_dispatch. That would be two owners of the same question, which is the shape INT-193
  exists to prevent and the reason this bug is invisible today.
- ⚠️ DO NOT solve it by re-invoking fsh as a subprocess for builtin stages. `fsh -c` hands its string
  to sh, so the builtin would not run there either -- and INT-201 owns making `-c` real.
- A REFUSAL IS AN ACCEPTABLE ANSWER for the harder positions. Supporting a builtin as the first stage
  and refusing it elsewhere, clearly, is better than a half-working chain that sometimes reads stdin.

## Relationship
- Found while investigating whether INT-196 could start; the reproduction came from trying to pipe
  `spine migrate` into `tail`.
- INT-201 owns the executor these stages run through, and its work is what makes one dispatch answer
  possible.
- INT-193 owns the one-owner rule this violates.
- The roadmap's Lane 0 carries the original, unfiled version of this item.
