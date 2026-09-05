#!/usr/bin/env python3
"""
INT-230 -- delete the discarded _focus binding. Line-anchored, no quotes.

⚠️ TWO FAILED ATTEMPTS, SAME CAUSE, AND IT IS THE ERROR OF THE WHOLE SESSION.
Claude wrote the anchor as `.split(SQ + backslash + Q + SQ)` -- an escaped quote.
The file contains `.split(SQ + Q + SQ)` with NO backslash. Matching against what
was expected instead of what exists, for the fourth time today, when one
`cat -A` answered it each time.

THE FIX FOR THE METHOD, not just this edit: anchor on lines that contain NO
quote characters at all. The first and last lines of the binding are unique in
the file and quote-free, so the span between them can be cut without any
character having to survive a shell, an escape level, or Claude retyping it.

### WHAT IS BEING DELETED

    let _focus = std::fs::read_to_string(paths::tools_registry())
        .map(...)          <- takes the FIRST `name = ` line in file order
        .unwrap_or_default();

TWO THINGS WRONG, COMPOUNDING:

  1. `_focus` is DISCARDED -- the underscore says so. The shell reads and parses
     the tools registry ON EVERY BANNER RENDER for a value it throws away.

  2. IT DOES NOT DO WHAT ITS COMMENT SAYS. The comment reads "lowest audit score
     tool"; the code takes the first name line in file order and reads no score
     anywhere. It would have been wrong if anything had used it.

⭐ Same family as everything this intent has found: a computation shaped like a
measurement. This one never reached a display, which is the only reason it did
no visible harm.

LAST unmigrated 0-Core discovery call.
"""

import io
import sys

P = "faelight/rust-tools/novashell/src/main.rs"

START = "    let _focus = std::fs::read_to_string(faelight_core::paths::tools_registry())"
END = "        .unwrap_or_default();"

with io.open(P, "r", encoding="utf-8") as fh:
    lines = fh.readlines()

starts = [i for i, l in enumerate(lines) if l.rstrip("\n") == START]
if len(starts) != 1:
    print("ABORT: start line matched " + str(len(starts)) + " times, need 1", file=sys.stderr)
    sys.exit(1)

i = starts[0]

# The binding ends at the first END line after the start. Bounded search so a
# runaway cannot eat the rest of the function.
end = None
for j in range(i + 1, min(i + 20, len(lines))):
    if lines[j].rstrip("\n") == END:
        end = j
        break

if end is None:
    print("ABORT: end line not found within 20 lines of start", file=sys.stderr)
    sys.exit(1)

replacement = [
    "    // INT-230: a _focus binding was computed here from the tools registry and\n",
    "    // then DISCARDED -- the underscore said so. It read and parsed the registry\n",
    "    // on every banner render to produce a value nobody used. And it did not do\n",
    "    // what its comment claimed: that said lowest audit score tool, while the\n",
    "    // code took the FIRST name line in file order and read no score at all.\n",
    "    // A computation shaped like a measurement, deleted rather than migrated.\n",
]

out = lines[:i] + replacement + lines[end + 1:]

with io.open(P, "w", encoding="utf-8") as fh:
    fh.writelines(out)

print("patched " + P)
print("removed lines " + str(i + 1) + " to " + str(end + 1))
print("")
print("Next: cargo build -p novashell --message-format=short")
