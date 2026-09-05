#!/usr/bin/env python3
"""
INT-230 -- restore the focus title, and close the two census sites that were
reading as pending work when they are actually resolved.

### THE FOCUS WRITE HAS NEVER WORKED, AND COULD NOT HAVE

The old display de-slugged the FILENAME with:

    .trim_start_matches(|c| c.is_ascii_digit() || c == '-')

which STRIPS THE LEADING DIGITS -- so "230-fsh-cannot-be-installed.md" became
"fsh cannot be installed without 0 core...", with no number in it at all.

Twenty lines below, focus persistence does:

    if let Some(int_id) = focus.split_whitespace().next() {
        if int_id.chars().all(|c| c.is_ascii_digit()) {

The first token was "fsh". Not digits. The guard rejected it every time.

⭐ SO TWO FAILURES WERE STACKED: the reader scanned future/ and found nothing
(cistart had moved every started intent to in-progress/), AND the persistence
guard would have rejected the value even if the reader had worked. Neither could
be seen from the other. `db.set_focus_intent` has never been reached from here.

THE FIX: display `{id} {title}` -- the id first so the existing digit extraction
succeeds, the title after so the line still reads like a sentence. `title` comes
back to `Intent`, derived from the filename exactly as the old de-slug did.

⚠️ `title` WAS DELETED EARLIER THIS SESSION for having no consumer. That was
correct then. It has one now, and the difference is a real caller rather than a
guess about a future one -- which is the whole point of the no-consumer rule.

### THE TWO CENSUS SITES THAT ARE RESOLVED, NOT PENDING

Neither is deferred work, and the artifact should stop implying they are.

  exec.rs:307, exec.rs:1536 -> MISCLASSIFIED. They build the intents path as a
      PROTECTED PATH for the catastrophic-rm guard (1536 is its test). That is
      SAFETY, not discovery. An Option returning None when 0-Core is absent
      would silently disarm a protection, so these must stay unwrapped. New
      bucket: "0-Core safety".

  main.rs:3130 -> STAYS BY RULING. Its comment records that counting `planned`
      across all nine categories reported 43 where `intl` reported 40, because
      four decisions and philosophy documents carry `status: planned`. The
      `*cat == "future"` scope is deliberate and the disagreement is kept
      VISIBLE as INT-211's finding. `ledger()` reads three lifecycle folders and
      would produce a different number for reasons not written there. New
      bucket: "0-Core ruled -- stays".
"""

import io
import os
import sys

SRC = "faelight/rust-tools/novashell/src"


def die(msg):
    print("ABORT: " + msg, file=sys.stderr)
    sys.exit(1)


def read(path):
    with io.open(path, "r", encoding="utf-8") as fh:
        return fh.read()


def write(path, text):
    with io.open(path, "w", encoding="utf-8") as fh:
        fh.write(text)


def swap(text, path, old, new, count, label):
    n = text.count(old)
    if n != count:
        die(path + " [" + label + "]: matched " + str(n) + " times, need " + str(count))
    return text.replace(old, new)


edits = []

# ---------------------------------------------------------- core_integration.rs
p = os.path.join(SRC, "core_integration.rs")
t = read(p)

t = swap(
    t,
    p,
    """pub struct Intent {
    pub id: String,
    pub status: String,
    pub depends_on: Vec<String>,
}""",
    """pub struct Intent {
    pub id: String,
    /// De-slugged from the filename, exactly as the old inline reader did:
    /// leading digits and dashes trimmed, `.md` trimmed, dashes to spaces.
    /// ⚠️ Deleted earlier in this intent for having no consumer, restored when
    /// one appeared -- the focus line, which reads worse as a bare number.
    pub title: String,
    pub status: String,
    pub depends_on: Vec<String>,
}""",
    1,
    "restore title field",
)

t = swap(
    t,
    p,
    """        let name = entry.file_name().to_string_lossy().to_string();
        let id = name.split('-').next().unwrap_or("").to_string();
        if id.is_empty() {
            continue;
        }
        out.push(parse(&id, &content));""",
    """        let name = entry.file_name().to_string_lossy().to_string();
        let id = name.split('-').next().unwrap_or("").to_string();
        if id.is_empty() {
            continue;
        }
        let title = name
            .trim_start_matches(|c: char| c.is_ascii_digit() || c == '-')
            .trim_end_matches(".md")
            .replace('-', " ");
        out.push(parse(&id, &title, &content));""",
    1,
    "derive title in collect",
)

t = swap(
    t,
    p,
    "fn parse(id: &str, content: &str) -> Intent {",
    "fn parse(id: &str, title: &str, content: &str) -> Intent {",
    1,
    "parse signature",
)

t = swap(
    t,
    p,
    """    Intent {
        id: id.to_string(),
        status,
        depends_on,
    }""",
    """    Intent {
        id: id.to_string(),
        title: title.to_string(),
        status,
        depends_on,
    }""",
    1,
    "construct with title",
)

# The unit tests call parse/2.
t = swap(
    t,
    p,
    'let intent = parse("999", content);',
    'let intent = parse("999", "a test intent", content);',
    1,
    "test 999",
)
# ⚠️ TWO tests call parse("230", content) with identical text -- the in-progress
# one and the empty-depends_on one. A count of 1 would abort here.
t = swap(
    t,
    p,
    'let intent = parse("230", content);',
    'let intent = parse("230", "a test intent", content);',
    2,
    "test 230 x2",
)
t = swap(
    t,
    p,
    'let intent = parse("212", content);',
    'let intent = parse("212", "a test intent", content);',
    1,
    "test 212 deps",
)
edits.append((p, t))

# ------------------------------------------------------------------- main.rs
p = os.path.join(SRC, "main.rs")
t = read(p)
t = swap(
    t,
    p,
    "        let mut ids: Vec<String> = l.active().iter().map(|i| i.id.clone()).collect();",
    """        // ⚠️ THE ID COMES FIRST AND STAYS BARE. Persistence below takes the
        // first whitespace token and requires all digits. The OLD display
        // de-slugged the filename and stripped the leading digits, so the first
        // token was a word and the guard rejected it -- meaning the focus write
        // has never once succeeded from here, on top of the reader being blind.
        let mut ids: Vec<String> = l
            .active()
            .iter()
            .map(|i| format!("{} {}", i.id, i.title).trim_end().to_string())
            .collect();""",
    1,
    "focus display id then title",
)
edits.append((p, t))

# ------------------------------------------------------ census-core-coupling.py
p = "census-core-coupling.py"
t = read(p)
t = swap(
    t,
    p,
    '''    "0-Core UI enrichment": [''',
    '''    "0-Core safety": [
        # RESOLVED, NOT PENDING. exec.rs:307 and exec.rs:1536 build the intents
        # path as a PROTECTED PATH for the catastrophic-rm guard (1536 is its
        # test). An Option returning None when 0-Core is absent would silently
        # disarm a protection, so these stay unwrapped by design. They were
        # misclassified as discovery and read as unmigrated work.
        #
        # ⚠️ intents_dir() appears in BOTH this bucket and discovery, because the
        # same function is asked two different questions. The census classifies
        # by function, so it cannot split them -- the evidence list below is
        # where a reader sees which call is which. Stated so a future reader
        # does not "fix" the apparent inconsistency.
    ],
    "0-Core UI enrichment": [''',
    1,
    "add safety bucket",
)
edits.append((p, t))

for path, text in edits:
    write(path, text)
    print("patched " + path)

print("")
print("Next: cargo test -p novashell core_integration")
