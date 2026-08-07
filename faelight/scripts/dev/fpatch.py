"""Guarded source patching. Import from a python heredoc:

    import sys; sys.path.insert(0, "faelight/scripts/dev")
    from fpatch import patch
    patch("path/to.rs", old, new)          # expects exactly one match
    patch("path/to.rs", old, new, count=2) # or state the count

Every guard here exists because a real edit failed without it.

INT-199: a refusal REPORTS rather than raising. Six aborts in one session printed a bare
AssertionError, and the tool was correct every time -- it declined a patch whose anchor no longer
matched, and wrote nothing. But the fact that mattered, NOTHING WAS WRITTEN, appeared nowhere, and
twice a safe refusal was read as a broken tool. A safe abort and a crash must not look the same.

The message carries the diagnostic. There is no error code to look up: on a miss it shows the
anchor with whitespace visible and the nearest lines in the file, because every anchor failure so
far has been a trailing space, a wrapped line, or a brace eaten in transmission -- all invisible
until printed with repr.
"""

import difflib
import sys
from pathlib import Path

DEBUG = "FPATCH_DEBUG" in __import__("os").environ


class PatchRefused(Exception):
    """A SAFE ABORT: the operation could not proceed, and nothing was changed.

    Pattern not found, several matches where one was required, a file already patched, a checksum
    that does not agree. The program behaved correctly and protected the data.
    """


class InternalError(Exception):
    """A DEFECT IN THIS TOOL: an impossible state, a violated invariant, a bug here.

    INT-199 keeps these apart because they ask different things of the reader. A safe abort says
    "fix your anchor"; an internal error says "fix fpatch". Presenting them the same way is what made
    six correct refusals read as a broken tool on 2026-07-29.

    ⚠️ ONLY TWO SEVERITIES EXIST HERE, AND ON PURPOSE. The taxonomy also names Info and Warning; this
    tool cannot currently produce either, and inventing producers so the set looks complete would be
    the error-code catalogue by another name. They arrive when behaviour requires them.
    """


def _refuse(path, reason, causes=(), recovery=(), detail=()):
    """Print the refusal and exit non-zero. Never leaves the reader guessing about writes."""
    bar = "=" * 66
    out = [
        "",
        bar,
        "  PATCH REFUSED -- safe abort",
        bar,
        "",
        "Status",
        "  Safe abort. The operation stopped to avoid an unsafe change.",
        "",
        "Result",
        f"  No changes written to {path}",
        "",
        "Reason",
    ]
    out += [f"  {line}" for line in reason.split("\n")]
    if detail:
        out += ["", "What was compared"] + [f"  {d}" for d in detail]
    if causes:
        out += ["", "Likely cause"] + [f"  - {c}" for c in causes]
    if recovery:
        out += ["", "Recovery"] + [f"  - {r}" for r in recovery]
    out += ["", bar, ""]
    print("\n".join(out), file=sys.stderr)
    if DEBUG:
        raise PatchRefused(reason)
    sys.exit(1)


def _internal(path, exc):
    """Present a DEFECT IN THIS TOOL, and be honest that the file's state is unknown.

    ⚠️ THIS CANNOT PROMISE WHAT A SAFE ABORT PROMISES. A refusal happens before any write, so it can
    say nothing was written and mean it. An internal error can happen anywhere, including mid-write,
    so the honest line is that the file may or may not have changed. Saying "no changes written" here
    would be the comforting answer rather than the true one.
    """
    bar = "=" * 66
    out = [
        "",
        bar,
        "  FPATCH INTERNAL ERROR",
        bar,
        "",
        "Status",
        "  Internal error. This is a defect in fpatch, not in the patch you asked for.",
        "",
        "Result",
        f"  The operation did not complete. {path} may or may not have changed -- check it before",
        "  retrying, because unlike a refusal this did not necessarily stop before writing.",
        "",
        "Reason",
        f"  {type(exc).__name__}: {exc}",
        "",
        "Recovery",
        "  - Do not work around this at the call site; the fault is here.",
        "  - Re-run with FPATCH_DEBUG=1 to get the traceback.",
        "  - Report it with that traceback and the call that produced it.",
        "",
        bar,
        "",
    ]
    print("\n".join(out), file=sys.stderr)
    if DEBUG:
        raise InternalError(str(exc)) from exc
    sys.exit(2)


def _guard(fn):
    """Route an UNEXPECTED exception through the internal presenter instead of a bare traceback.

    This is what gives InternalError a producer. Without it the class would be decoration: every
    existing failure in this file is operational, so nothing would ever raise it. An uncaught
    exception escaping as a raw traceback IS the case INT-199 was filed about -- six safe aborts
    printing bare AssertionErrors on 2026-07-29 -- and this closes it from the other side.

    A refusal and a SystemExit pass through untouched: they are already the presented form.
    """
    import functools

    @functools.wraps(fn)
    def wrapper(*args, **kwargs):
        try:
            return fn(*args, **kwargs)
        except (PatchRefused, SystemExit):
            raise
        except Exception as exc:
            _internal(args[0] if args else "<unknown path>", exc)

    return wrapper


def _nearest(lines, needle, n=3):
    """The lines most similar to the anchor's first line, shown with whitespace visible."""
    probe = needle.split("\n")[0].strip()
    scored = sorted(
        ((difflib.SequenceMatcher(None, probe, l.strip()).ratio(), i, l) for i, l in enumerate(lines)),
        reverse=True,
    )
    return [f"line {i + 1}: {l!r}" for ratio, i, l in scored[:n] if ratio > 0.5]


@_guard
def patch_between(path, start_marker, end_marker, new_lines, context=2):
    """Replace a span located by two SHORT markers, without transcribing its body.

    Every anchor failure so far came from re-typing text that was already in the file -- a
    continuation line\'s indent, an em dash, an arm duplicated by an earlier edit. This mode makes
    that impossible: the markers are short and unique, the body is never quoted, and the span is
    replaced by index.

    `start_marker` matches the FIRST line of the span (substring). `end_marker` matches the first
    line AFTER it (prefix). Both must be unique in the file.
    """
    p = Path(path)
    lines = p.read_text().split("\n")

    starts = [n for n, l in enumerate(lines) if start_marker in l]
    ends = [n for n, l in enumerate(lines) if l.startswith(end_marker)]
    if len(starts) != 1:
        _refuse(
            path,
            f"The start marker matched {len(starts)} lines. It must match exactly 1.",
            detail=[f"marker: {start_marker!r}"] + [f"line {n + 1}: {lines[n]!r}" for n in starts[:4]],
            causes=["The marker is not unique.", "An earlier edit duplicated or removed it."],
            recovery=["Lengthen the marker until it is unique.", "Re-read the region first."],
        )
    if len(ends) != 1:
        _refuse(
            path,
            f"The end marker matched {len(ends)} lines. It must match exactly 1.",
            detail=[f"marker: {end_marker!r}"] + [f"line {n + 1}: {lines[n]!r}" for n in ends[:4]],
            causes=["The marker is not unique, or is a prefix of several lines."],
            recovery=["Lengthen the marker.", "Re-read the region first."],
        )
    lo, hi = starts[0], ends[0]
    if hi <= lo:
        _refuse(
            path,
            "The end marker is at or above the start marker, so the span is empty or inverted.",
            detail=[f"start: line {lo + 1}", f"end:   line {hi + 1}"],
            recovery=["Check the two markers are the right way round."],
        )

    print(f"--- {path}: replacing lines {lo + 1}..{hi} ---")
    for i in range(max(0, lo - context), min(len(lines), hi + context)):
        mark = ">>" if lo <= i < hi else "  "
        print(f"{mark} {i + 1}: {lines[i]}")

    lines[lo:hi] = new_lines
    p.write_text("\n".join(lines))
    print(f"OK {path}: {hi - lo} line(s) replaced by {len(new_lines)}")


@_guard
def patch(path, old, new, count=1, context=2):
    p = Path(path)
    s = p.read_text()

    # An em dash becomes `--` in transmission, so a non-ASCII anchor silently never matches.
    if not old.isascii():
        bad = [c for c in old if not c.isascii()]
        _refuse(
            path,
            "The anchor contains non-ASCII characters, which do not survive transmission intact.",
            detail=[f"characters: {bad!r}"],
            causes=["An em dash, an arrow, or a box-drawing character copied from the file."],
            recovery=["Anchor on an ASCII-only line.", "Use patch_between and match by index."],
        )

    n = s.count(old)
    if n != count:
        lines = s.split("\n")
        detail = [f"anchor: {old.split(chr(10))[0]!r}"]
        if n == 0:
            near = _nearest(lines, old)
            detail += ["", "nearest lines in the file:"] + near if near else ["", "nothing similar found"]
        else:
            detail += [f"found {n} occurrences, expected {count}"]
        _refuse(
            path,
            f"The anchor matched {n} time(s). It must match exactly {count}.",
            detail=detail,
            causes=[
                "Trailing whitespace, or the line wraps differently than expected.",
                "An earlier patch in this run already changed this text.",
                "Braces were consumed in transmission -- build them from pieces.",
            ]
            if n == 0
            else ["The anchor is not unique; widen it with surrounding text."],
            recovery=[
                "Print the region with repr() and copy the exact bytes.",
                "Use patch_between to replace by index instead of by text.",
            ],
        )

    # A replacement identical to the anchor passes every existence check and still does nothing.
    if new == old:
        _refuse(
            path,
            "The replacement is identical to the anchor, so the patch would be a silent no-op.",
            causes=["The change was already applied.", "The wrong text was passed as `new`."],
            recovery=["Re-read the region -- it may already be correct."],
        )

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

    # VERIFY AFTER WRITE. Every guard above runs on the string in memory, and none of them proves
    # the file on disk now holds the replacement. On 2026-08-06 three patches reported "17 replaced"
    # truthfully while the file contained none: the shell had eaten braces out of `new` before
    # python ever saw it, so the in-memory replace was correct and the result was wrong. Reading the
    # file back is the only check that spans the gap between what was sent and what was written.
    after = p.read_text()
    if after.count(new) < count:
        _refuse(
            path,
            "The write completed but the file does not contain the replacement.",
            detail=[
                f"expected at least {count} occurrence(s) of the replacement",
                f"found {after.count(new)}",
                f"replacement began: {new.split(chr(10))[0]!r}",
            ],
            causes=[
                "The replacement text was altered in transmission before python received it.",
                "Another process rewrote the file between the read and the write.",
            ],
            recovery=[
                "Print the region with repr() and compare it byte for byte.",
                "Build fragile characters from chr() so nothing can rewrite them in transit.",
            ],
        )
    print(f"OK {path}: {n} replaced, verified on disk")
