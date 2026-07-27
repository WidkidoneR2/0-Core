"""Guarded source patching. Import from a python heredoc:

    import sys; sys.path.insert(0, "faelight/scripts/dev")
    from fpatch import patch
    patch("path/to.rs", old, new)          # expects exactly one match
    patch("path/to.rs", old, new, count=2) # or state the count

Every guard here exists because a real edit failed without it.
"""

from pathlib import Path


def patch_between(path, start_marker, end_marker, new_lines, context=2):
    """Replace a span located by two SHORT markers, without transcribing its body.

    Every anchor failure so far came from re-typing text that was already in the file -- a
    continuation line's indent, an em dash, an arm duplicated by an earlier edit. This mode makes
    that impossible: the markers are short and unique, the body is never quoted, and the span is
    replaced by index.

    `start_marker` matches the FIRST line of the span (substring). `end_marker` matches the first
    line AFTER it (prefix). Both must be unique in the file.
    """
    p = Path(path)
    lines = p.read_text().split("\n")

    starts = [n for n, l in enumerate(lines) if start_marker in l]
    ends = [n for n, l in enumerate(lines) if l.startswith(end_marker)]
    assert len(starts) == 1, f"{path}: start marker matched {len(starts)} lines, need 1"
    assert len(ends) == 1, f"{path}: end marker matched {len(ends)} lines, need 1"
    lo, hi = starts[0], ends[0]
    assert hi > lo, f"{path}: end marker is at or above the start marker"

    print(f"--- {path}: replacing lines {lo + 1}..{hi} ---")
    for i in range(max(0, lo - context), min(len(lines), hi + context)):
        mark = ">>" if lo <= i < hi else "  "
        print(f"{mark} {i + 1}: {lines[i]}")

    lines[lo:hi] = new_lines
    p.write_text("\n".join(lines))
    print(f"OK {path}: {hi - lo} line(s) replaced by {len(new_lines)}")


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
