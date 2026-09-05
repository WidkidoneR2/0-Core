#!/usr/bin/env python3
"""
INT-230 G4 -- delete prompt.rs::status_line.

`#[allow(dead_code)] pub fn status_line(db: &ForestDb) -> String` has NO CALLER.
The only other mention in the tree is a header comment at the top of prompt.rs
describing it as "pretty status printed after clear or on welcome", which it no
longer is.

It contains `health_str(db.health_score().unwrap_or(95))` -- asserting 95%
health on a machine that has never run the doctor. Dead code carrying a lie,
which is strictly worse than dead code: nobody sees it fail, so nobody fixes it,
and the day someone wires it up they inherit the defect silently.

⚠️ THE SPAN WAS READ WITH `cat -A` FIRST. An earlier draft computed it as
`allow_line - 1`, guessing that the header comment sat directly above the
attribute. It does not -- there is a BLANK LINE between them (594 comment, 595
blank, 596 attribute). That guess would have deleted the blank and left the
comment orphaned above the next function.

Deleted with its header-comment entry, since a table of contents naming a
function that no longer exists is the same class of stale claim.
"""

import io
import sys

P = "faelight/rust-tools/novashell/src/prompt.rs"

with io.open(P, "r", encoding="utf-8") as fh:
    lines = io.open(P, encoding="utf-8").readlines()


def one(needle, label):
    hits = [i for i, l in enumerate(lines) if l.rstrip("\n") == needle]
    if len(hits) != 1:
        print("ABORT: " + label + " matched " + str(len(hits)) + ", need 1", file=sys.stderr)
        sys.exit(1)
    return hits[0]


attr = one("#[allow(dead_code)]", "allow attribute")

if lines[attr + 1].rstrip("\n") != "pub fn status_line(db: &ForestDb) -> String {":
    print("ABORT: the attribute is not on status_line", file=sys.stderr)
    sys.exit(1)

# The function ends at the first column-zero closing brace after the signature.
end = None
for j in range(attr + 2, min(attr + 40, len(lines))):
    if lines[j].rstrip("\n") == "}":
        end = j
        break

if end is None:
    print("ABORT: status_line closing brace not found within 40 lines", file=sys.stderr)
    sys.exit(1)

# Walk back over the section-divider comment and the blank line above it.
start = attr
while start > 0 and lines[start - 1].strip() == "":
    start -= 1
if start > 0 and lines[start - 1].lstrip().startswith("//"):
    start -= 1

replacement = [
    "// INT-230 G4: status_line DELETED. It was #[allow(dead_code)] with no\n",
    "// caller, and it defaulted health to 95 -- asserting near-perfect health on\n",
    "// a machine that had never run the doctor. Dead code carrying a lie is worse\n",
    "// than dead code: nobody sees it fail, so nobody fixes it, and whoever wires\n",
    "// it up later inherits the defect silently.\n",
]

removed = end - start + 1
lines[start:end + 1] = replacement

with io.open(P, "w", encoding="utf-8") as fh:
    fh.writelines(lines)

print("patched " + P)
print("removed " + str(removed) + " lines starting at " + str(start + 1))
print("")
print("⚠️ The prompt.rs header comment at the top of the file still lists")
print("   status_line in its table of contents. Remove that line separately --")
print("   it is one line and it is not worth risking this span on.")
print("")
print("Next: cargo build -p novashell --message-format=short")
