#!/usr/bin/env python3
"""
INT-230 -- remove Intent.title. Its consumer lasted twenty minutes.

THE SEQUENCE, recorded because it is the no-consumer rule working rather than
churn:

  1. `title` was written speculatively with the adapter. DELETED for having no
     consumer.
  2. The Today: line needed a title, so it came BACK with a real caller.
  3. Today: was then found to be a second renderer of the active list, and the
     correct source is focus.toml -- which carries its OWN title. The caller
     went away again.

So `title` is deleted a second time, and the deletion is right for the same
reason both times: the ledger's titles are not what the shell needs. The one
place a title is displayed reads it from the focus file, which is the file that
already holds it.

⚠️ THE LESSON IS NOT "do not add fields". It is that a field added for a caller
one is about to write is a guess until that caller exists and survives. Twenty
minutes is short enough to be funny; the same mistake at three months is a dead
field nobody dares remove.
"""

import io
import os
import sys

SRC = "faelight/rust-tools/novashell/src"


def die(msg):
    print("ABORT: " + msg, file=sys.stderr)
    sys.exit(1)


p = os.path.join(SRC, "core_integration.rs")
with io.open(p, "r", encoding="utf-8") as fh:
    t = fh.read()


def swap(text, old, new, count, label):
    n = text.count(old)
    if n != count:
        die(label + ": matched " + str(n) + " times, need " + str(count))
    return text.replace(old, new)


t = swap(
    t,
    """    /// De-slugged from the filename, exactly as the old inline reader did:
    /// leading digits and dashes trimmed, `.md` trimmed, dashes to spaces.
    /// ⚠️ Deleted earlier in this intent for having no consumer, restored when
    /// one appeared -- the focus line, which reads worse as a bare number.
    pub title: String,
""",
    "",
    1,
    "drop title field",
)

t = swap(
    t,
    """        let title = name
            .trim_start_matches(|c: char| c.is_ascii_digit() || c == '-')
            .trim_end_matches(".md")
            .replace('-', " ");
        out.push(parse(&id, &title, &content));""",
    "        out.push(parse(&id, &content));",
    1,
    "drop title derivation",
)

t = swap(
    t,
    "fn parse(id: &str, title: &str, content: &str) -> Intent {",
    "fn parse(id: &str, content: &str) -> Intent {",
    1,
    "parse signature",
)

t = swap(
    t,
    """    Intent {
        id: id.to_string(),
        title: title.to_string(),
        status,
        depends_on,
    }""",
    """    Intent {
        id: id.to_string(),
        status,
        depends_on,
    }""",
    1,
    "construct without title",
)

t = swap(
    t,
    'let intent = parse("230", "a test intent", content);',
    'let intent = parse("230", content);',
    2,
    "tests 230",
)
t = swap(
    t,
    'let intent = parse("999", "a test intent", content);',
    'let intent = parse("999", content);',
    1,
    "test 999",
)
t = swap(
    t,
    'let intent = parse("212", "a test intent", content);',
    'let intent = parse("212", content);',
    1,
    "test 212",
)

with io.open(p, "w", encoding="utf-8") as fh:
    fh.write(t)

print("patched " + p)
print("")
print("Next: cargo build -p novashell --message-format=short")
