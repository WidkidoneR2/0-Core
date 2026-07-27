"""Guarded source patching. Import from a python heredoc:

    import sys; sys.path.insert(0, "faelight/scripts/dev")
    from fpatch import patch
    patch("path/to.rs", old, new)          # expects exactly one match
    patch("path/to.rs", old, new, count=2) # or state the count

Every guard here exists because a real edit failed without it.
"""

from pathlib import Path


def patch(path, old, new, count=1, context=2):
    p = Path(path)
    s = p.read_text()

    # An em dash becomes `--` in transmission, so a non-ASCII anchor silently never matches.
    # This failed twice before the rule existed.
    assert old.isascii(), "anchor contains non-ASCII; it will not survive transmission"

    n = s.count(old)
    assert n == count, f"{path}: expected {count} match(es), found {n}"

    # A replacement identical to the anchor passes every existence check and still does nothing.
    assert new != old, "replacement is identical to the anchor; would be a silent no-op"

    # Show what is about to change, with its neighbourhood -- attributes and doc comments above an
    # item are part of that item, and stranded lines below are how two edits went wrong.
    lines = s.split("\n")
    first = s[: s.index(old)].count("\n")
    last = first + old.count("\n")
    lo, hi = max(0, first - context), min(len(lines), last + context + 1)
    print(f"--- {path}: replacing lines {first + 1}..{last + 1} ---")
    for i in range(lo, hi):
        mark = ">>" if first <= i <= last else "  "
        print(f"{mark} {i + 1}: {lines[i]}")

    p.write_text(s.replace(old, new))
    print(f"OK {path}: {n} replaced")
