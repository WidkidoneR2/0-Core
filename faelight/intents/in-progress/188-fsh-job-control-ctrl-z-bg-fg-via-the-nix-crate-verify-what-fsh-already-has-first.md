---
id: 188
date: 2026-07-21
type: feature
title: "fsh has no job control layer: no process groups, no terminal foreground ownership, and one signal handler"
status: in-progress
tags: [fsh, job-control, signals, process-groups, terminal, int-207]
---

## Vision
fsh owns its signals, its process groups and the terminal foreground -- so a job can be suspended,
resumed, backgrounded and reported on, and the shell's idea of what its children are doing matches
what they are actually doing.

## ✅ VERIFY-FIRST: DONE 2026-08-23, and it CHANGED THE SCOPE
The intent's own first gate demanded this before any build, and it was right to. Measured across
`faelight-shell/src/`, non-comment lines. **THREE signal touchpoints in the entire shell:**

    libc::signal(SIGPIPE, SIG_DFL)   mod.rs:9253    one reset at startup
    ctrlc::set_handler               mod.rs:6708    ONE handler, ONE signal
    status.signal()                  main.rs:729    READING a dead child's signal as 128 + s

**ZERO references to SIGCHLD, SIGTSTP, SIGCONT, SIGWINCH, SIGTTIN, SIGTTOU, SIGHUP.
ZERO setpgid. ZERO tcsetpgrp. ZERO tcgetpgrp. ZERO waitpid.**

★ SO GATE ZERO IS ANSWERED, AND NOT THE WAY THE INTENT EXPECTED. Job control is not partial. There
is no process-group handling and no terminal-foreground control anywhere in fsh. Ctrl+Z cannot work,
a stopped job cannot be resumed, and a terminal resize reaches nothing.

⚠️ THIS IS NOT A DEPENDENCY TASK. The original title says "via the nix crate" and the tags name
tokio -- a crate chosen before the shape. **THE MISSING ABSTRACTION IS THE DEFECT, NOT THE MISSING
DEPENDENCY.** Adding a crate that exposes the right signals would leave every hard question
unanswered.

## The Problem
A shell is a process manager. fsh currently spawns children and reads their exit status, and that is
the whole of it. It does not know when a child stops, does not place children in process groups, and
never negotiates which process group owns the terminal. Every capability below follows from those
three absences, not from a missing library.

## The Solution: ONE dispatcher, then everything else
    OS signals
        |
    signal dispatcher            <- ONE owner. ctrlc is SUBSUMED, not kept beside it.
        |
    +---------+---------+
    |         |         |
  SIGCHLD  SIGWINCH  SIGINT/SIGTSTP
    |         |         |
  job      terminal   foreground
  table     state      control

★ SIGCHLD DRIVES RECONCILIATION, NOT NOTIFICATION. A signal that flips a boolean or prints a line is
the shape this ledger deleted four times on 2026-08-22 -- consumers that reported confidently from a
source that could not support it. SIGCHLD must cause the job table to ASK what actually happened and
record it.

    RUNNING --(SIGTSTP/SIGSTOP)--> STOPPED --(SIGCONT)--> RUNNING
    RUNNING --(SIGCHLD + exit)---> EXITED
    RUNNING --(SIGCHLD + signal)-> SIGNALED

⚠️ SIGWINCH IS NOT JOB STATE. It belongs to terminal and editor state, and mixing it into the job
table would be the same category error as `TIMING:` rows living in shell_history.

## Success Criteria
- [x] VERIFY-FIRST: document what fsh's current signal/job handling actually does. Scope only what is
      genuinely missing
<!-- DONE 2026-08-23, measured not assumed: three touchpoints (SIGPIPE reset, one ctrlc handler,
     one read of a dead child's signal), and zero of every primitive job control needs. The gate
     earned its place -- it changed the intent from "add a crate" to "build the layer". -->
- [x] Gate zero: is job control absent, partial, or adequate? If adequate, CANCEL
<!-- ABSENT. Not partial. No process groups, no terminal foreground control, no SIGCHLD. -->
- [x] G1 THE DISPATCHER SHAPE IS DECIDED BEFORE ANY DEPENDENCY IS CHOSEN, and the decision is
      written here: who receives signals, how a signal becomes a shell event, how SIGCHLD drives
      AUTHORITATIVE job-state reconciliation, and how process groups and terminal foreground
      ownership interact.
      ⚠️ A DEPENDENCY IS NOT SELECTED BECAUSE IT EXPOSES THE REQUIRED SIGNALS. signal-hook, rustix,
      nix and raw libc are all capable; capability is not the criterion
- [ ] G2 ONE SIGNAL OWNER. `ctrlc` is removed or subsumed -- two mechanisms answering one question
      is the shape INT-207 and INT-221 both existed to end
- [ ] G3 THE REQUIRED SET IS HANDLED AND EACH ONE'S PURPOSE IS STATED: SIGCHLD, SIGTSTP, SIGCONT,
      SIGINT, SIGWINCH, SIGTTIN, SIGTTOU, and SIGHUP
- [x] G4 PROCESS GROUPS: children are placed in groups deliberately, with the requirement stated --
      what gets its own group, what shares one, and what happens to a pipeline
- [x] G5 TERMINAL FOREGROUND OWNERSHIP: the shell-versus-job foreground model is defined and
      implemented, including what happens when a background job reads from the terminal
- [x] G6 ONLY THEN: the implementation is chosen -- signal-hook, rustix, nix or libc -- with the
      reason recorded and what it gives up
- [x] G7 DEMONSTRATED ON A REAL COMMAND: Ctrl+Z suspends, `bg` resumes in background, `fg` returns
      it to the foreground, and `jobs` reports state that matches reality
- [x] G8 NO REGRESSION: the line editor, Ctrl+C, login, and deploy all still work. fsh-test green
- [ ] G9 each gate carries evidence per INT-158

---

## G1 RULED 2026-09-13 — the shape, before the dependency

Measured first, against the code as it stands today, not against the 2026-08-23 census:

    setpgid / process_group      0 occurrences
    tcsetpgrp / tcgetpgrp        0
    waitpid / WUNTRACED          0
    SIGTSTP / SIGCONT / SIGCHLD  0
    ctrlc::set_handler           1   (commands/mod.rs:6946)

Gate zero still holds. Nothing has drifted. What HAS been found is that the seam this intent
needs already exists and is already correct.

### THE SEAM IS BUILT. THIS INTENT ATTACHES A PGID TO IT.

    engine.rs:1087   BackgroundAttempt::Single(c, l) => jobs.register(c, &l)
    engine.rs:1089   BackgroundAttempt::Chain(ch, l) => jobs.register_chain(ch, &l)

So `sleep 30 &` ALREADY produces a Job. There is no second background path to build and none
should be built. The invariant this intent implements:

    spawn  ->  establish process group  ->  register Job { child(ren), pgid, tmodes }

And the boundary that makes it possible is one exec.rs already declares (exec.rs:1157):
*exec.rs SPAWNS BUT STILL NEVER LEARNS WHAT A JobTable IS.* Groups are established where the
spawn happens; the pgid travels to the table as data. Nothing about job control leaks back into
the lowering path.

⭐ AND THE TWO SHAPES ARE NOT SYMMETRIC, WHICH DECIDES THE BUILD ORDER:

    Single(Command, _)   UNSPAWNED. jobs.rs:126 calls .spawn(). ONE line establishes the group --
                         process_group(0) before spawn -- and the pgid is then child.id(),
                         because a group leader's pgid equals its pid.
    Chain(Vec<Child>, _) ALREADY SPAWNED by background_pipeline. The shared group must be set
                         THERE: first stage leads with process_group(0), later stages join with
                         process_group(first_pid). Only that function knows stage order.
    foreground           NO SEAM AT ALL. execute_pipeline waits on every stage inline. This is
                         the genuinely new code, and it is why background comes first.

### RULED: NO SIGNAL DISPATCHER IN v1

`check_completed` (jobs.rs:160) already runs before every prompt render. That is the
reconciliation point, and it is already in the right place. What changes is the QUESTION it asks:

    today      job.child.try_wait()                    -- waitpid(WNOHANG). Cannot see a stop.
    v1         waitpid(-pgid, WNOHANG|WUNTRACED|WCONTINUED)

⚠️ `try_wait` IS STRUCTURALLY BLIND TO A STOPPED JOB, not merely incomplete. Even with Ctrl+Z
working, a stopped job would report as still running forever, because WUNTRACED is the flag that
makes a stop observable and `Child::try_wait` never passes it. This is the wrong syscall for the
question, not a missing feature.

SIGCHLD is DEFERRED, deliberately. The intent's own diagram makes SIGCHLD drive reconciliation --
that principle is right and is honoured by the prompt loop asking. A handler that sets a flag the
prompt loop then checks adds a SECOND source of truth for zero new capability, and this ledger
deleted four consumers on 2026-08-22 for exactly that shape. SIGCHLD earns its place when
something needs to know BETWEEN prompts. Nothing does yet.

### RULED: WHO GETS A GROUP

    background single      its own group, leader = itself
    background pipeline    ONE group for all stages, leader = first stage
    foreground external    its own group, leader = itself
    builtins               NO group. A builtin is not a job.
    command substitution   NO group. Its output is a value, not a process the user manages.
    nsh -c and scripts     NO job control at all. Stays in nsh's own group.

Interactive-only is the same split fish makes, and it is not a limitation: job control is a
property of a terminal session, not of a shell binary. A script has no terminal to hand over.

### RULED: TERMINAL OWNERSHIP

    give     tcsetpgrp(stdin, job_pgid)   before waiting
    take     tcsetpgrp(stdin, getpgrp())  after the wait returns, ALWAYS -- including on a stop
    ignore   SIGTTOU, once at init, or the shell suspends itself calling tcsetpgrp
    save     Termios on the Job when it stops; restore on resume

⚠️ THE ORDER IS NOT COSMETIC. Take the terminal back on EVERY exit from the wait, not only the
happy one. A job that stops leaves the shell without a terminal if the restore sits after a
success branch -- the same class as the cgroup cleanup that had to move into Drop in INT-246.

⚠️ AND `ctrlc` BECOMES WRONG AT EXACTLY THIS POINT, not before. Once a foreground job owns the
terminal, the kernel delivers SIGINT to ITS group and a process-wide handler steals it from
sleep, vim and every pipeline. It is untouched in the background step and removed in the step
that gives the terminal away -- G2 closes there, not earlier, because removing it before there
is anything to hand over would break Ctrl+C for no gain.

### RULED: rustix, WITH WHAT IT GIVES UP

    rustix, features: process, termios, stdio

One crate covers setpgid, waitpid, tcsetpgrp, tcgetpgrp and Termios, with I/O-safe AsFd rather
than a bare STDIN_FILENO, and it REFUSES pid 0 from tcgetpgrp (returns OPNOTSUPP on a pty) rather
than handing back an invalid Pid to be used as a group.

What it gives up: it is one more dependency in a crate stack INT-198 keeps deliberately small,
and its error type is another to translate at the boundary. Accepted, because the alternative is
growing `unsafe` inside the module that can least afford a wrong fd.

REJECTED, each with the reason:

    nix              same calls, weaker fd model, and we are not already on it. Being able to
                     paste Nushell examples verbatim is not worth a second syscall crate.
    raw libc         already present, and that is the problem: it invites a c_int in the wrong
                     slot and silent EINTR. jobs.rs is where that costs most.
    ctrlc            process-wide handler. Actively wrong once a foreground group exists.
    crossterm        cursor, colour, raw mode. Raw mode is "make stdin a byte stream", not "this
                     group owns the terminal". Using it here would fight rustyline.
    portable-pty     creates a NEW pty and usually a new session. Correct for nsh-test driving
                     nsh; the exact opposite of operating on the EXISTING controlling terminal.
                     It stays in the test harness and never touches the interactive path.
    termios crate    a fourth tty crate for calls rustix already exposes.

⚠️ rustyline OWNS THE LINE DISCIPLINE while the prompt is up. It must release before tcsetpgrp
and reacquire after. Do not teach rustyline about process groups; teach the exec path to bracket
the handover.

### RULED: fg, bg AND jobs ARE RESERVED BUILTIN NAMES

`fg` is currently an alias for faelight-git (measured: `which fg` -> `fg -> faelight-git`).

Every shell in existence spells the builtin `fg`. A shell that ships with `fg` meaning "git" is
the first bug an Omarchy user files. The alias is the squatter and the alias moves -- inside this
intent, because the demo cannot otherwise be typed.

This also sits under INT-247 Layer 0: nothing new is named faelight, and a git alias holding a
POSIX builtin name is exactly the kind of thing that retirement is for.

YSH-style verbs (`foreground`, `resume`, `fork`) may be added later as aliases pointing at the
same primitives. They are not the v1 interface, and `&` is not being replaced.

### THE ORDER, AND WHY IT IS AN ORDER

    1. background single gets a real pgid, and jobs shows it        smallest possible change
    2. background pipeline shares one pgid                          same idea, one function over
    3. check_completed asks waitpid(-pgid, WUNTRACED) instead       stops become OBSERVABLE
    4. foreground handover: give tty, wait, take tty back           Ctrl+Z now reaches the JOB
    5. fg: tcsetpgrp + SIGCONT + wait                               the loop closes
    6. bg                                                           only after 5 works

Steps 1-3 are invisible to a user and break nothing. Step 4 is where behaviour changes and where
`ctrlc` goes. `bg` prints an honest "not implemented" until step 6 -- OSH shipped exactly that
way, and a surface that refuses is better than one that lies.

---

## STEPS 1-4 BUILT AND DEMONSTRATED, 2026-09-13

Four commits, each proven by watching the kernel rather than reading the code.

### Step 1 -- a background job gets its own group (1b188dab)

    sleep 300 &
    jobs                  [1] sleep  (pgid 71379, 4s elapsed)
    ps -o pid,pgid        71379   71379 sleep      <- pgid == pid, it leads its own group
    nsh pgid              3259                     <- and it is NOT the shell's

One line does it: `command.process_group(0)` before spawn in `register`, and the pgid is then
`child.id()` because a group leader's pgid IS its pid.

### Step 2 -- a pipeline is ONE group (351872ff)

    sleep 300 | cat &
    ps -eo pid,pgid       86267    3259 sleep   <- OLD binary: shares the shell's group
                          86268    3259 cat
                          86430   86430 sleep   <- NEW: stage 0 leads
                          86431   86430 cat     <- and stage 1 joined it

Both shapes visible in one listing, which is as clear a before/after as this work produced.
`register_chain` was DELETED in the same commit -- its only caller now knows its group, so the
group-less variant had no reason to exist.

### Step 3 -- states are OBSERVED, never assumed (95fdcca3, 44faa9e5)

    kill -STOP -119597    ♸ [1] sleep -- stopped (44.2s)
    jobs                  [1] stopped   sleep  (pgid 119597, ...)
    jobs                  [1] stopped   sleep  (...)        <- HELD across polls
    kill -CONT -119597    ▶ [1] sleep -- continued          <- fired ONCE, on the real resume

⚠️ AND THE BUG THAT FOUND ITSELF HERE. The first version reported "stopped" and then "continued"
on the very next prompt, with nobody resuming anything. waitpid reports a stop ONCE; every poll
after returns Ok(None). The fold read only fresh events, so it computed Running and the change
detector faithfully announced a resume that never happened.

    A stop is an EVENT the kernel delivers once.
    "Still stopped" is a STATE only the shell remembers.

Code that reads only fresh events forgets every fact the instant it arrives.

### Step 4 -- Ctrl+Z reaches the JOB (15e400b1)

    sleep 300
    ^Z
      ♸ suspended -- not yet resumable: job registration is INT-188 step 5 (pid 20622)
    echo still alive      still alive        <- the shell survived and can read the keyboard
    ps                    17613 17613 T sleep <- genuinely stopped, own group

## ⭐ THE FINDING THAT COST THREE WRONG THEORIES: SIG_IGN SURVIVES exec

Step 4 was built correctly and did not work. Everything measured clean:

    child pgid         15061          its own group
    tpgid              15061          the TERMINAL's foreground group IS the child's
    ps state           S+             running, in the foreground group
    stty isig          on             Ctrl+Z generates a signal
    stty susp          ^Z             and that signal is SIGTSTP
    gave_terminal      true           the handover was accepted

Ctrl+Z was delivered perfectly, to a process that had been told to ignore it:

    /proc/<child>/status    SigIgn: 0000000000380000
                                     bits 19/20/21 = SIGTSTP, SIGTTIN, SIGTTOU

**A disposition of SIG_IGN is INHERITED ACROSS exec.** The shell ignores those three so it cannot
suspend ITSELF -- correct for the shell, fatal for a child. Every foreground job must be born with
DEFAULT dispositions regardless of what the shell did to itself, and the reset belongs in
`pre_exec`, between fork and exec:

    libc::signal(SIGTSTP, SIG_DFL);   libc::signal(SIGINT,  SIG_DFL);
    libc::signal(SIGTTIN, SIG_DFL);   libc::signal(SIGQUIT, SIG_DFL);
    libc::signal(SIGTTOU, SIG_DFL);   libc::signal(SIGCHLD, SIG_DFL);

⚠️ THREE THEORIES WERE WRONG BEFORE THE MEASUREMENT WAS RIGHT: that rustix's WUNTRACED was
failing, that the terminal was in raw mode with ISIG off, and that a placeholder exit code was
triggering a shell exit. Each was plausible and each was disproved by one command. The fact that
ended it -- `SigIgn` in /proc -- was available from the first minute and was the last thing
looked at.

⭐ AND THE ONE FACT NEVER CHECKED UNTIL LATE: `ps` was run four times with `pid,pgid,comm` and no
STAT column. The question the whole session turned on -- WAS THE CHILD STOPPED -- was invisible in
every one of those listings. Ask for the column that answers the question.

## G7 IS NOT CLOSED, AND HERE IS WHAT IS MISSING

Ctrl+Z suspends. `bg`, `fg` and `jobs` do not see the suspended job, because `spawn_with_tee`
cannot reach a JobTable -- exec.rs deliberately never learns what one is (exec.rs:1157).

The suspended job currently prints its pid and is otherwise lost. That is honest, and it is
temporary: step 5 builds the seam the way `BackgroundAttempt` already does it -- exec returns
something, the caller registers it -- rather than smuggling a table across the boundary.

---

## STEPS 5-6: THE LOOP CLOSES, 2026-09-13

### Step 5 -- a suspended job is REGISTERED, not lost (09ddfcff)

Step 4 left the job alive, stopped, and unreachable: the shell printed its pid and forgot it.
The gap was structural -- `spawn_with_tee` discovers the stop and cannot reach a JobTable,
because exec deliberately never learns what one is (exec.rs:1157).

RULED: the event travels back as DATA and the engine registers it.

    spawn_with_tee_jc  ->  Suspension { pgid, label }
                       ->  CommandResult::Empty { suspension }
                       ->  absorb_result_with_jobs
                       ->  JobTable::register_suspended

⭐ `Empty` WAS WIDENED RATHER THAN A VARIANT ADDED -- the rule this enum already states twice in
its own doc. 18 of its match sites carry a `_` arm and would have swallowed a new variant
silently; widening made all 65 construction sites a COMPILE ERROR. The compiler was the
checklist, and one mechanical pass over its own JSON diagnostics fixed every one.

⚠️ AND THE FIELD IS THE EXTENSION POINT, NOT THE VARIANT. `Empty` still means "nothing was
produced", which is true of a suspended command. Other execution-side events can cross the same
boundary later without this variant becoming about suspension.

⭐ Suspension CARRIES A pgid, NOT A pid. For a single foreground command the two are equal and
that is COINCIDENCE: `fg` resumes a GROUP, and a pipeline makes them differ while the
job-control identity stays the group. Encoding today's equality would have bought one step and
cost an architectural correction at the next.

### The type change that forced two latent bugs into the open

`Job.child` became `Option<Child>` -- a suspended job has no Child, because the wait that saw it
stop consumed the handle. That broke exactly two call sites, and both were already wrong:

    fg()        waited on ONE Child instead of resuming a GROUP, never sent SIGCONT (so a
                stopped job would have hung the shell forever), and never handed over the
                terminal (so the resumed job could not read the keyboard).
    kill_job()  killed one pid and removed the row WITHOUT reaping -- the `sleep <defunct>`
                observed during step 4. It also needed SIGCONT first: SIGTERM to a stopped
                process is queued, not delivered.

Making the field optional is what stopped them compiling unchanged. Verified after the fix: zero
`sleep <defunct>` in the process table.

### ⭐ RESERVED NAMES BEAT ALIASES, AND THAT IS A PROPERTY OF THE SHELL

`fg 1` reached faelight-git, not job control:

    raw "fg 1"  ->  expand_aliases (main.rs:1520)  ->  "faelight-git 1"  ->  try_fg asks
                    command_word, gets "faelight-git", declines

The builtin was unreachable BY NAME. Renaming the git alias would have removed one collision and
left the shape intact -- any user aliasing `fg`, `bg` or `jobs` would make the shell's own job
control disappear.

RULED: classify against the RAW line before expanding. Once expansion has run the evidence is
gone, which is why the question is asked first. Scoped to what `is_repl_state_command` actually
claims, so `fg commit` and `fg status` still reach git -- verified.

### Step 6 -- bg (65bf9f79)

The whole difference from `fg` is two things MISSING, not anything added: no tcsetpgrp, no wait.

    sleep 60
    ^Z            suspended (fg 1 to resume)
    bg 1          continued in the background
    jobs          [1] running   sleep  (pgid 46310, 8s elapsed)
    bg 1          x [1] sleep is not stopped -- nothing to resume
    (waits)       done (66.4s)

⭐ THE LAST LINE IS THE ONE THAT MATTERS. Nobody asked for it. The job was suspended from the
terminal, resumed by a builtin, finished on its own, and was reported by step 3's reconciler --
the first time these parts composed without being individually driven.

## G7 CLOSED. WHAT REMAINS

    G2  ctrlc is still installed, and the ruling says it becomes wrong at exactly the point
        that is now true: a foreground group owns the terminal, so the kernel delivers SIGINT
        to IT, and a process-wide handler steals it from sleep, vim and every pipeline.
    G3  each signal's purpose stated -- mostly written across these sections, needs collecting.
    G8  no regression: nsh-test green, line editor, Ctrl+C, login, deploy.
    G9  evidence per gate.

⚠️ G2 IS A REAL BEHAVIOUR CHANGE, not a cleanup. Removing the handler means Ctrl+C stops being
the shell's and starts being the job's. Worth its own careful pass rather than a tidy-up at the
end of a session.

---

## G2 RULED 2026-09-14 -- THE GATE'S PREMISE DID NOT SURVIVE MEASUREMENT

The gate says: *ONE SIGNAL OWNER. `ctrlc` is removed or subsumed -- two mechanisms answering one
question.* There are not two mechanisms. Measured, not assumed:

    fresh nsh            SigCgt: 0x100000440    SIGINT NOT caught
    during `watch`       SigCgt: 0x100000442    SIGINT caught      (bit 1 appears)
    after `watch` exits  SigCgt: 0x100000442    still caught

A fresh shell installs NO SIGINT handler. `ctrlc` has exactly one caller in the crate --
`watch_cmd` (commands/mod.rs:6979) -- and it does a job that needs doing: without it, Ctrl+C
during a polling loop would terminate the shell by default disposition. Demonstrated working:
`^C` during `watch health 2` prints `watch stopped` and returns the prompt.

⭐ THE DEFECT IS THE HANDLER'S LIFETIME, NOT ITS EXISTENCE. `ctrlc::set_handler` changes a
PROCESS-WIDE disposition and offers no scoped restoration. After `watch` returns, SIGINT remains
caught by the handler that command installed; its state is no longer observed by anything.

### ⚠️ AND THE CONSEQUENCE IS SMALLER THAN THIS GATE ASSUMED. SAY SO.

The tempting claim -- "a stale handler steals Ctrl+C from sleep, vim and every pipeline" -- is
FALSE, and the fix from step 4 is why. With job control working, the kernel delivers SIGINT to
the FOREGROUND PROCESS GROUP, and a foreground child is in its own group with a clean slate:

    child of `nsh -c sleep 20`    SigIgn: 0    SigCgt: 0

So the shell's disposition is irrelevant to a foreground job. The stale handler can only matter
when the SHELL ITSELF is the foreground group -- at the prompt, or inside a builtin.

And there, measured interactively: typing `abc` then Ctrl+C clears the line and returns a fresh
prompt, IDENTICALLY before and after `watch` has run. rustyline handles the keystroke itself.

**THE LEAK IS LATENT, NOT OBSERVABLE.** No symptom was found. It is still wrong -- a disposition
outliving the command that set it is a fact waiting to become a bug when something else starts
caring about SIGINT at the prompt -- but this ledger does not record symptoms it did not see.

### RULED

`watch` keeps its interrupt mechanism. Its SIGINT disposition is SCOPED to the lifetime of the
command: save, install, restore on exit. The `ctrlc` crate cannot express that -- it installs and
never restores -- so the dependency goes and the disposition is managed directly.

⭐ AND THE GATE IS NOT TICKED ON A PREMISE THAT WAS WRONG. What is recorded is the measured
behaviour and the narrower defect actually found. Signal behaviour in a shell is subtle and easy
to misattribute; a gate closed on the wrong reasoning is worse than one left open, because the
next reader inherits the wrong model.

---

## G1 DISPOSITION 2026-09-14: PREMISE OVER-BROAD, NO CORRECTIVE CHANGE WARRANTED

G1 ruled *"nsh -c and scripts: NO job control at all."* Measured, that is not what the code does:

    stdin = tty         SigIgn 0x301000   TTIN/TTOU ignored -- job control ENGAGED
    stdin = /dev/null   SigIgn 0x001000   not ignored       -- job control OFF

`nsh -c` enables job control whenever it is invoked from a terminal, because initialization is
gated by TERMINAL OWNERSHIP rather than by shell mode. `is_interactive()` asks "is there a
terminal and do we own it" -- the right question for a shell, the wrong question for deciding
which door we came in through.

### ⭐ AND THE FEARED FAILURE DOES NOT OCCUR. MEASURED, NOT ASSUMED.

The argument for stripping job control from `-c` was that Ctrl+Z would strand a stopped child
nobody could resume. Tested under a bare pty with no job-control parent above it:

    BEFORE   15489 15489 Ssl  nsh        15490 15490 S+  sleep
    stop the sleep's group, as the terminal would
    AFTER    (nothing left)

Both gone. The reason is POSIX rather than anything this shell does: **when a process group
becomes orphaned while it contains stopped members, the kernel sends it SIGHUP followed by
SIGCONT.** The `-c` shell exits, the group is orphaned, the kernel cleans up.

Commands needing the terminal (`vim`, `less`, `top`) work either way -- not having shell job
control never meant children could not use the tty.

### RULED

Premise over-broad; no corrective change warranted. No concrete correctness or resource-lifetime
failure has been demonstrated, and working behaviour is not changed merely because an earlier
statement was too broad.

⭐ THE DISTINCTION REMAINS ARCHITECTURALLY VALUABLE AND IS WORTH STATING FOR LATER:

    TTY PRESENCE tells you what the process is CAPABLE of doing.
    SHELL MODE tells you what the shell is SUPPOSED to do.

Job-control initialization should eventually depend on a declared mode rather than on
`stdin.isatty()`. Defer that redesign until a concrete requirement or failure establishes one --
and do NOT fix it by redefining `is_interactive()` until it returns the wanted answer, which
would hide the conflation rather than remove it.

## G2: THE RULING ABOVE NEEDS ONE CORRECTION BEFORE IT IS TICKED

The G2 ruling says *"a fresh shell installs NO SIGINT handler"*. That is true of `nsh -c` and
FALSE of the interactive shell. Measured side by side:

    nsh -c        SigIgn  PIPE, TTIN, TTOU        SigCgt  7, 11, 33
    interactive   SigIgn  PIPE, 25                SigCgt  INT, 7, 11, WINCH, 33

**rustyline catches SIGINT and SIGWINCH.** So there ARE two SIGINT mechanisms in an interactive
shell -- rustyline's and, until today, `watch`'s. The gate was right about the shape and wrong
about the members, and the ruling as written overclaims.

The `watch` fix itself is verified: SigCgt is IDENTICAL before and after running and interrupting
`watch` in one live process (0x108000442 both times). The disposition is restored.

⚠️ NOT TICKED. The fix is done and demonstrated; the ruling's supporting claim is wrong and must
be corrected first. A gate closed on a wrong premise is worse than one left open.

### Would reedline solve this? No -- and the measurement says why.

The mode conflation is in `tty.rs`, in this project's own code, on the exec path. No line editor
participates in that decision. Reedline WOULD change the SIGINT/SIGWINCH half of the picture,
because those handlers are rustyline's -- but swapping line editors to fix a mode-detection
problem is treating a symptom two layers from the cause, and INT-168 already owns that swap with
preconditions of its own.

---

## G3: THE SIGNAL INVENTORY, 2026-09-14

Every signal in the gate's required set, what it is for here, and where that lives. Measured by
census plus the /proc masks taken today, not by intention.

    SIGNAL     DISPOSITION                    PURPOSE IN THIS SHELL
    --------   ----------------------------   ------------------------------------------------
    SIGCHLD    default; SIG_DFL in pre_exec   NOT handled, DELIBERATELY (G1). check_completed
                                              asks waitpid before every prompt, so a handler
                                              would add a second source of truth for no new
                                              capability. Earns its place when something needs
                                              to know BETWEEN prompts. Nothing does yet.
    SIGTSTP    default; SIG_DFL in pre_exec   The stop. Delivered by the tty to the FOREGROUND
                                              GROUP, which after step 4 is the job rather than
                                              the shell. The pre_exec reset is what makes it
                                              reach the child at all -- see the SIG_IGN finding.
    SIGCONT    sent, never caught             `fg` and `bg` resume a GROUP with
                                              kill(-pgid, SIGCONT). kill_job sends it too,
                                              because SIGTERM to a stopped process is queued
                                              rather than delivered.
    SIGINT     rustyline catches it;          Ctrl+C at the prompt is rustyline's. `watch` now
               `watch` scopes its own         installs and RESTORES its own for the life of the
                                              command. A foreground job gets SIGINT from the
                                              kernel directly -- its group, its clean slate.
    SIGTTOU    SIG_IGN at job-control init    Without it the shell SUSPENDS ITSELF: handing the
               SIG_DFL in pre_exec            terminal to a child makes the shell a background
                                              process, and taking it back then raises SIGTTOU.
    SIGTTIN    SIG_IGN at job-control init    Same reason, for a background READ of the terminal.
               SIG_DFL in pre_exec            A backgrounded job that reads the tty stops with
                                              this -- correct Unix behaviour, not a defect.
    SIGPIPE    SIG_DFL at startup (INT-299)   The shell needs EPIPE rather than death on a broken
               SIG_DFL in pre_exec            pipe; a child needs the opposite, or `yes | head`
                                              spins forever.
    SIGQUIT    SIG_DFL in pre_exec            Reset for children with the rest. The shell itself
                                              takes the default.
    SIGTERM    sent, never caught             kill_job, after SIGCONT.

### ⚠️ TWO IN THE REQUIRED SET ARE NOT HANDLED BY THIS SHELL AT ALL. SAY SO.

    SIGWINCH   ZERO references in the crate. The interactive mask shows it CAUGHT -- by
               rustyline, for redraw on resize. The intent's own non-goals already say
               "SIGWINCH-driven redraw belongs to the editor; this intent only routes the
               signal", and in fact it does not route it either: the editor owns it end to end.
               That is a working arrangement, not a gap, but the gate cannot claim nsh handles
               it.

    SIGHUP     ZERO references. Nothing in this shell has a story for the terminal going away.
               Today's orphan test showed the KERNEL doing the right thing -- an orphaned group
               with stopped members gets SIGHUP then SIGCONT -- but that is the kernel cleaning
               up after us, not us deciding anything. What should happen to REGISTERED BACKGROUND
               JOBS when the shell's terminal closes is undefined. A real shell either hups them
               or disowns them; nsh does neither on purpose.

⭐ SIGHUP IS THE ONE GENUINE GAP THIS INVENTORY FOUND, and it is out of scope here: it is about
job LIFETIME across a session ending, not about the suspend/resume loop INT-188 exists to build.
Worth its own intent rather than a rushed arm in this one.

G3 is therefore ANSWERED but NOT TICKED: nine of eleven have a stated purpose and a site, one is
owned by the editor, and one is an admitted absence with no design behind it.

---

## G8 CLOSED 2026-09-14

The gate asks that the line editor, Ctrl+C, login and deploy all still work, and that the suite
is green. Taken one at a time, after a session that widened an enum across 65 sites, added a
module, and rewrote `fg` and `kill_job`:

    nsh-test          193 / 193 passed, 2 skipped, exit 0, run in the DevBox sandbox.
                      The 2 skips are absent preconditions, not failures -- both say so.
                      Three cases cover today's path directly:
                          repl_jobs_lists_a_running_job
                          repl_background_job_honours_its_redirect
                          repl_background_job_keeps_quoted_arguments
                      No file changes detected by the sandbox after the run.

    line editor       Exercised by hand throughout: typing, history, and `abc` + Ctrl+C
                      clearing the line and returning a fresh prompt.

    Ctrl+C            Measured before and after the `watch` fix, in one live process.
                      SigCgt identical (0x108000442), prompt behaviour identical.

    deploy            `ship` run repeatedly across the session -- 1-2 crates each time,
                      0 failed, and the shipped binary re-exec'd and used immediately after.

    login             ⭐ NOT AT RISK BY ARCHITECTURE, not by luck.

                          /etc/passwd: christian ... /usr/bin/bash

                      bash is the login shell. nsh is started FROM a terminal and is never
                      invoked by `login`, which is the interactive-not-login split the Omarchy
                      port chose deliberately. A shell that cannot start therefore costs a
                      terminal tab, not a session -- and `exec bash` is the recovery, which is
                      what this session's riskiest tests relied on.

⚠️ THE ONE THING THIS GATE CANNOT CLAIM: that job control is exercised by the suite in the way it
is exercised by hand. `nsh-test` has three background-job cases and NO suspend/resume case --
Ctrl+Z needs a pty and a signal, and the harness drives the REPL rather than a terminal. Every
proof in this intent for steps 4, 5 and 6 is a hand-run measurement recorded above.

⭐ THAT IS A REGRESSION RISK WITH A NAME: the suspend/resume loop could break tomorrow and
nothing would catch it. A pty-driven job-control case belongs in nsh-test, and INT-202's capture
window already solved the hard half of that problem. Worth its own work rather than a claim here.

## Sequencing
- INT-168 (reedline) owns keystroke handling and the same terminal territory. Do not build job
  control on rustyline immediately before swapping the editor -- coordinate or sequence after.
- ⭐ THIS INTENT IS THE PREREQUISITE FOR THE IDENTIFIER SCHEME (Crockford Base32 job ids): job
  control is what makes an identifier visible and gives it a first real consumer.
- INT-207's observability has a `Jobs` target already defined and unused. Every state transition
  above should emit through it rather than growing a private log.

## Non-goals
- Choosing a crate before G1. That is the failure this rewrite exists to prevent.
- Reimplementing what the OS provides. The question is which layer owns the call, not whether to
  write a scheduler.
- SIGWINCH-driven redraw. It belongs to the editor; this intent only routes the signal.
