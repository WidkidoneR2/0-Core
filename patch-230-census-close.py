#!/usr/bin/env python3
"""
INT-230 -- close the census: daemon_socket is core shell state, not 0-Core.

### MEASURED, NOT ASSUMED

    pub fn daemon_socket() -> PathBuf {
        runtime_dir().join("daemon.sock")
    }

`runtime_dir()` has been XDG state since the 2026-08-21 move, so it resolves on
any machine with or without a forest. Same family as `state_db`, `bin_dir` and
`runtime_dir` itself.

### AND THE CALLERS WERE ALREADY RIGHT -- THE FIRST BUCKET THAT WAS

All three (`engine.rs:275`, `engine.rs:540`, `main.rs:3629`) do:

    if Path::new(&sock_path).exists() {
        if let Ok(mut stream) = UnixStream::connect(&sock_path) {

Two guards, neither fabricating availability.

⭐ AND THE REASON NOT TO WRAP THEM IS SUBSTANTIVE, NOT COSMETIC: a socket's
absence is NOT 0-Core's absence. The daemon can be stopped on a machine with the
full forest. An accessor gated on `present()` would conflate "no forest" with
"daemon not running" -- and the callers correctly care about neither, only about
whether the socket is connectable. Adding the check would answer a question no
caller asks, which is how a boundary starts collecting ceremony.

So the execution bucket is EMPTY: those three move to core shell state, where
the function actually belongs.

### THE CENSUS NOW CARRIES ITS OWN LIMITS

Three things it cannot see or cannot express, written into the file so a clean
census is never read as a clean shell:

  - it matches TEXT, not syntax, so `nl.rs:460` -- a COMMENT mentioning
    `paths::registry_dir()` -- counts as a call
  - it only finds `paths::` calls, so THIRTY hand-built `0-core/` paths in this
    shell are invisible to it, TWELVE of them pointing at `~/0-core/scripts`
    which does not exist. That is INT-240's subject
  - it classifies by FUNCTION, so one function asked two different questions
    (`intents_dir` for discovery AND for the rm guard's protected paths) cannot
    be split. The evidence list is where a reader sees which call is which
"""

import io
import sys

P = "census-core-coupling.py"

with io.open(P, "r", encoding="utf-8") as fh:
    t = fh.read()


def swap(text, old, new, count, label):
    n = text.count(old)
    if n != count:
        print("ABORT: " + label + " matched " + str(n) + " times, need " + str(count),
              file=sys.stderr)
        sys.exit(1)
    return text.replace(old, new)


# daemon_socket moves to core shell state.
t = swap(
    t,
    '''    "0-Core execution": [
        "daemon_socket",
    ],''',
    '''    "0-Core execution": [
        # EMPTY. daemon_socket moved to core shell state: it derives from
        # runtime_dir(), which has been XDG state since 2026-08-21, so it
        # resolves on any machine. And its three callers were already correct --
        # Path::exists() then UnixStream::connect(), two guards, no fabricated
        # availability. Deliberately NOT routed through the adapter: a socket's
        # absence is not 0-Core's absence (the daemon can be stopped on a
        # machine with the full forest), so gating it on present() would answer
        # a question no caller asks.
    ],''',
    1,
    "empty the execution bucket",
)

t = swap(
    t,
    '''        "bin_dir",
        "state_home",''',
    '''        "bin_dir",
        "state_home",
        "daemon_socket",''',
    1,
    "add daemon_socket to core shell state",
)

# Record the artifact's limits in the generated file itself.
t = swap(
    t,
    '''    out.append("A single line can do more than one of these. The unit G1 counts is the")
    out.append("classified call.")
    out.append("")''',
    '''    out.append("A single line can do more than one of these. The unit G1 counts is the")
    out.append("classified call.")
    out.append("")
    out.append("## What this census CANNOT see")
    out.append("")
    out.append("Recorded so a clean census is never read as a clean shell.")
    out.append("")
    out.append("- It matches **text, not syntax**, so a comment mentioning a")
    out.append("  `paths::` function counts as a call (`nl.rs:460`).")
    out.append("- It only finds **`paths::` calls**. Thirty hand-built `0-core/`")
    out.append("  paths exist in this shell and are invisible here, twelve of them")
    out.append("  pointing at `~/0-core/scripts`, which does not exist. INT-240.")
    out.append("- It classifies by **function**, so one function asked two different")
    out.append("  questions cannot be split -- `intents_dir` serves both discovery")
    out.append("  and the catastrophic-rm guard. The evidence list below is where a")
    out.append("  reader sees which call is which.")
    out.append("- It measures **path coupling, not the defect class**. A fabricated")
    out.append("  default read from the database rather than a path is invisible to")
    out.append("  it (`commands/mod.rs:6776`, `db.health_score().unwrap_or(0)`).")
    out.append("")''',
    1,
    "document census limits",
)

with io.open(P, "w", encoding="utf-8") as fh:
    fh.write(t)

print("patched " + P)
print("")
print("Next: python3 census-core-coupling.py > faelight/rust-tools/novashell/CORE-COUPLING.md")
