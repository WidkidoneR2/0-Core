"""Guarded source patching. Invoked as ONE argv word, from anywhere:

    python3 -c 'import base64,sys; exec(base64.b64decode(sys.argv[1]).decode("utf-8"))' <b64>

    ...where the decoded payload begins:

    import sys
    sys.path.insert(0, "/home/christian/0-core/zero/scripts/dev")
    from fpatch import patch, patch_between, Plan
    patch("path/to.rs", old, new)             # expects exactly one match
    patch("path/to.rs", old, new, count=2)    # or state the count
    patch("path/to.rs", old, new, dry=True)   # show it, write nothing

SEVERAL EDITS, ALL OR NOTHING (INT-278):

    p = Plan()
    p.patch("a.rs", old, new)
    p.between("b.md", "## Start", None, new_lines)   # None = to end of file
    p.show()          # prints every edit and a Seal; writes nothing
    p.apply(SEAL)     # refuses unless the Seal matches what show printed
    p.text()          # the exact text the Seal covers: keep it under refs/notes/seals

Every check in a plan runs before the first write. Files are staged beside their
targets (mode bits kept, symlinks followed, fsynced), renamed into place, then read
back and compared byte for byte. A cached payload can call p.run(argv): no
arguments shows, "apply SEAL" applies. In the -c form, pass sys.argv[2:].

THE PATH IS ABSOLUTE (INT-258), AND THERE IS DELIBERATELY NO CLI: a command line would
put old and new back into shell syntax. Inside a python payload they stay Python
string literals.

Set FPATCH_COLOR=0 when the output is captured rather than read in a terminal.

Exit codes: 1 = refused, nothing written. 2 = fpatch defect, or a write that did not
verify; check the file. INT-199: a safe abort and a crash must not look the same.
"""

import difflib
import hashlib
import os
import sys
import tempfile
from pathlib import Path

DEBUG = "FPATCH_DEBUG" in os.environ

# INT-215: colour is the differentiator. Four codes and a reset, defined once.
_ANSI = {
    "red": "\033[31m",
    "green": "\033[32m",
    "yellow": "\033[33m",
    "reset": "\033[0m",
}


def _color(stream):
    """Whether to colour THIS stream. isatty by default; FPATCH_COLOR overrides both ways."""
    env = os.environ.get("FPATCH_COLOR")
    if env is not None:
        return env.strip().lower() not in ("0", "no", "off", "false", "")
    try:
        return stream.isatty()
    except Exception:
        return False


def _paint(text, name, on):
    """Wrap text in a colour, or return it UNCHANGED when colour is off."""
    if not on:
        return text
    return _ANSI[name] + text + _ANSI["reset"]


class PatchRefused(Exception):
    """A SAFE ABORT: the operation could not proceed, and nothing was changed."""


class InternalError(Exception):
    """A DEFECT IN THIS TOOL, or a write whose result could not be verified."""


def _refuse(path, reason, causes=(), recovery=(), detail=(), result=None):
    """Print the refusal and exit 1. Never leaves the reader guessing about writes."""
    on = _color(sys.stderr)
    bar = _paint("=" * 66, "red", on)
    out = [
        "",
        bar,
        _paint("  PATCH REFUSED -- safe abort", "red", on),
        bar,
        "",
        "Status",
        "  Safe abort. The operation stopped to avoid an unsafe change.",
        "",
        "Result",
        f"  {result or f'No changes written to {path}'}",
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


def _yellow_block(title, status, result, reason, recovery):
    on = _color(sys.stderr)
    bar = _paint("=" * 66, "yellow", on)
    out = ["", bar, _paint(f"  {title}", "yellow", on), bar, "", "Status", f"  {status}",
           "", "Result"] + [f"  {r}" for r in result] + ["", "Reason", f"  {reason}", "",
           "Recovery"] + [f"  - {r}" for r in recovery] + ["", bar, ""]
    print("\n".join(out), file=sys.stderr)


def _internal(path, exc):
    """A DEFECT IN THIS TOOL. It cannot promise what a refusal promises, so it does not."""
    _yellow_block(
        "FPATCH INTERNAL ERROR",
        "Internal error. This is a defect in fpatch, not in the patch you asked for.",
        [f"The operation did not complete. {path} may or may not have changed -- check it before",
         "retrying, because unlike a refusal this did not necessarily stop before writing."],
        f"{type(exc).__name__}: {exc}",
        ["Do not work around this at the call site; the fault is here.",
         "Re-run with FPATCH_DEBUG=1 to get the traceback.",
         "Report it with that traceback and the call that produced it."],
    )
    if DEBUG:
        raise InternalError(str(exc)) from exc
    sys.exit(2)


def _unverified(path, expected, got):
    """The write happened and the file does not hold what was computed. Not a safe abort."""
    first = next((i for i, (a, b) in enumerate(zip(expected, got)) if a != b), min(len(expected), len(got)))
    line = expected.count("\n", 0, first) + 1
    _yellow_block(
        "WRITE NOT VERIFIED",
        "The file WAS written, and it does not hold the computed text.",
        [f"{path} changed. Inspect it before anything else.",
         f"Expected {len(expected)} chars, found {len(got)}; first difference near line {line}."],
        "Something altered the file between the write and the read-back.",
        ["Restore it with git checkout -- <file> if the result is wrong.",
         "Check for another process writing the same file.",
         "Re-run the edit once the file is known-good."],
    )
    if DEBUG:
        raise InternalError(f"write not verified: {path}")
    sys.exit(2)


def _guard(fn):
    """Route an UNEXPECTED exception through the internal presenter, not a bare traceback."""
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


# ---- disk: exact bytes in, exact bytes out ---------------------------------------------

def _read(path):
    # newline="" keeps CRLF as CRLF: byte for byte means no translation on the way in.
    with open(path, encoding="utf-8", newline="") as f:
        return f.read()


def _unlink(path):
    try:
        os.unlink(path)
    except FileNotFoundError:
        pass


def _stage(target, text):
    """Write text to a temp file beside target, mode bits kept, fsynced. Returns its path."""
    fd, tmp = tempfile.mkstemp(dir=os.path.dirname(target), prefix="." + os.path.basename(target) + ".fpatch-")
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="") as f:
            f.write(text)
            f.flush()
            os.fsync(f.fileno())
        os.chmod(tmp, os.stat(target).st_mode & 0o7777)
    except BaseException:
        _unlink(tmp)
        raise
    return tmp


# ---- edits: compute only, never write --------------------------------------------------

def _compute_patch(path, text, old, new, count, context, result):
    if not old:
        _refuse(path, "The anchor is empty, so it matches everywhere.", result=result)
    # An em dash becomes -- in transmission, so a non-ASCII anchor silently never matches.
    if not old.isascii():
        bad = [c for c in old if not c.isascii()]
        _refuse(
            path,
            "The anchor contains non-ASCII characters, which do not survive transmission intact.",
            detail=[f"characters: {bad!r}"],
            causes=["An em dash, an arrow, or a box-drawing character copied from the file."],
            recovery=["Anchor on an ASCII-only line.", "Use patch_between and match by index."],
            result=result,
        )
    n = text.count(old)
    lines = text.split("\n")
    if n != count:
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
            result=result,
        )
    # A replacement identical to the anchor passes every existence check and does nothing.
    if new == old:
        _refuse(
            path,
            "The replacement is identical to the anchor, so the patch would be a silent no-op.",
            causes=["The change was already applied.", "The wrong text was passed as new."],
            recovery=["Re-read the region -- it may already be correct."],
            result=result,
        )
    # Every site is shown, not just the first.
    shown, pos = [], 0
    for _ in range(n):
        i = text.index(old, pos)
        pos = i + len(old)
        first = text.count("\n", 0, i)
        last = first + old.count("\n")
        shown.append((None, f"--- {path}: replacing lines {first + 1}..{last + 1} ---"))
        for j in range(max(0, first - context), min(len(lines), last + context + 1)):
            if first <= j <= last:
                shown.append(("red", f">> {j + 1}: {lines[j]}"))
            else:
                shown.append((None, f"   {j + 1}: {lines[j]}"))
        for nl in new.split("\n"):
            shown.append(("green", f"++ {nl}"))
    return text.replace(old, new), shown, f"{n} replaced"


def _compute_between(path, text, start_marker, end_marker, new_lines, context, result):
    if isinstance(new_lines, str):
        _refuse(path, "new_lines must be a list of lines, not one string.",
                recovery=["Pass text.split(chr(10)) instead of text."], result=result)
    new_lines = list(new_lines)
    for name, m in (("start", start_marker), ("end", end_marker)):
        if m is not None and not m.isascii():
            _refuse(path, f"The {name} marker contains non-ASCII characters.",
                    detail=[f"characters: {[c for c in m if not c.isascii()]!r}"],
                    recovery=["Pick a short ASCII-only marker on a neighbouring line."], result=result)
    lines = text.split("\n")
    starts = [n for n, l in enumerate(lines) if start_marker in l]
    if len(starts) != 1:
        _refuse(
            path,
            f"The start marker matched {len(starts)} lines. It must match exactly 1.",
            detail=[f"marker: {start_marker!r}"] + [f"line {n + 1}: {lines[n]!r}" for n in starts[:4]],
            causes=["The marker is not unique.", "An earlier edit duplicated or removed it."],
            recovery=["Lengthen the marker until it is unique.", "Re-read the region first."],
            result=result,
        )
    lo = starts[0]
    if end_marker is None:
        # To end of file. A trailing newline stays a trailing newline.
        hi = len(lines) - 1 if len(lines) > 1 and lines[-1] == "" else len(lines)
    else:
        ends = [n for n, l in enumerate(lines) if l.startswith(end_marker)]
        if len(ends) != 1:
            _refuse(
                path,
                f"The end marker matched {len(ends)} lines. It must match exactly 1.",
                detail=[f"marker: {end_marker!r}"] + [f"line {n + 1}: {lines[n]!r}" for n in ends[:4]],
                causes=["The marker is not unique, or is a prefix of several lines."],
                recovery=["Lengthen the marker.", "Pass None as the end marker to replace to end of file."],
                result=result,
            )
        hi = ends[0]
    if hi <= lo:
        _refuse(
            path,
            "The end marker is at or above the start marker, so the span is empty or inverted.",
            detail=[f"start: line {lo + 1}", f"end:   line {hi + 1}"],
            recovery=["Check the two markers are the right way round."],
            result=result,
        )
    if lines[lo:hi] == new_lines:
        _refuse(path, "The replacement is identical to the span, so the patch would be a silent no-op.",
                causes=["The change was already applied."],
                recovery=["Re-read the region -- it may already be correct."], result=result)
    shown = [(None, f"--- {path}: replacing lines {lo + 1}..{hi} ---")]
    for i in range(max(0, lo - context), min(len(lines), hi + context)):
        if lo <= i < hi:
            shown.append(("red", f">> {i + 1}: {lines[i]}"))
        else:
            shown.append((None, f"   {i + 1}: {lines[i]}"))
    for nl in new_lines:
        shown.append(("green", f"++ {nl}"))
    out = lines[:lo] + new_lines + lines[hi:]
    return "\n".join(out), shown, f"{hi - lo} line(s) replaced by {len(new_lines)}"


def _h16(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()[:16]


class Plan:
    """A list of edits that is checked whole, shown whole, and written whole."""

    def __init__(self):
        self._edits = []

    def __str__(self):
        return ", ".join(dict.fromkeys(e[1] for e in self._edits)) or "<empty plan>"

    def patch(self, path, old, new, count=1, context=2):
        self._edits.append(("patch", str(path), old, new, count, context))
        return self

    def between(self, path, start_marker, end_marker, new_lines, context=2):
        self._edits.append(("between", str(path), start_marker, end_marker, new_lines, context))
        return self

    def _build(self):
        if not self._edits:
            _refuse("<empty plan>", "The plan has no edits.", result="No changes written to any file")
        files, shown, summary = {}, [], []
        total = len(self._edits)
        for k, e in enumerate(self._edits, 1):
            kind, path = e[0], e[1]
            if path not in files:
                orig = _read(path)
                files[path] = [orig, orig]
            result = f"No changes written to any file (edit {k} of {total} refused)" if total > 1 else None
            if kind == "patch":
                text, s, said = _compute_patch(path, files[path][1], e[2], e[3], e[4], e[5], result)
            else:
                text, s, said = _compute_between(path, files[path][1], e[2], e[3], e[4], e[5], result)
            files[path][1] = text
            shown += s
            summary.append((path, said))
        head = [f"file {p}  sha256 {_h16(o)} -> {_h16(c)}" for p, (o, c) in files.items()]
        plain = "\n".join(head + [t for _, t in shown] + [f"{p}: {s}" for p, s in summary])
        return files, head, shown, summary, _h16(plain), plain

    def show(self):
        return _plan_show(self)

    def text(self):
        """The exact text the Seal covers: everything show prints above the DRY RUN line.

        Kept as the note under refs/notes/seals, core intent trace re-checks it: both sides take
        the first 16 hex of sha256 over the same bytes (trace.rs seal_digest, INT-266).
        """
        return self._build()[5]

    def apply(self, seal):
        if not seal:
            _refuse(str(self), "apply needs the Seal that show printed.",
                    recovery=["Run show, review it, then apply with its Seal."],
                    result="No changes written to any file")
        return _plan_apply(self, seal)

    def run(self, argv=None):
        argv = sys.argv[1:] if argv is None else list(argv)
        if not argv:
            return self.show()
        if len(argv) == 2 and argv[0] == "apply":
            return self.apply(argv[1])
        _refuse(str(self), f"Unknown arguments: {argv!r}",
                recovery=["No arguments: show the plan.", "apply SEAL: apply the reviewed plan."],
                result="No changes written to any file")


def _print_shown(shown):
    on = _color(sys.stdout)
    for c, t in shown:
        print(_paint(t, c, on) if c else t)


@_guard
def _plan_show(plan):
    files, head, shown, summary, seal, plain = plan._build()
    for h in head:
        print(h)
    _print_shown(shown)
    for p, said in summary:
        print(f"{p}: {said}")
    print(f"DRY RUN -- nothing written. Seal: {seal}")
    return seal


@_guard
def _plan_apply(plan, seal, require_seal=True):
    files, head, shown, summary, actual, plain = plan._build()
    if require_seal and seal != actual:
        _refuse(
            str(plan),
            "The Seal does not match this plan.",
            detail=[f"given:  {seal!r}", f"actual: {actual}"],
            causes=["A target file changed after the plan was shown.",
                    "The edits are not the ones that were reviewed.", "The Seal was mistyped."],
            recovery=["Run show again, review it, and apply with the new Seal."],
            result="No changes written to any file",
        )
    _print_shown(shown)
    targets = {p: os.path.realpath(p) for p in files}  # follow symlinks; never replace the link
    changed = [p for p, (o, c) in files.items() if c != o]
    staged = {}
    try:
        for p in changed:
            staged[p] = _stage(targets[p], files[p][1])
    except BaseException:
        for t in staged.values():
            _unlink(t)
        raise
    landed = []
    try:
        for p in changed:
            os.replace(staged[p], targets[p])
            landed.append(p)
    except BaseException as exc:
        for p in changed:
            if p not in landed:
                _unlink(staged[p])
        rest = [p for p in changed if p not in landed]
        raise RuntimeError(f"rename failed. Written: {landed or 'none'}. Not written: {rest}. ({exc})") from exc
    for p in changed:
        got = _read(targets[p])
        if got != files[p][1]:
            _unverified(p, files[p][1], got)
    for p, said in summary:
        print(f"OK {p}: {said}, verified on disk")
    return actual


def patch(path, old, new, count=1, context=2, dry=False):
    """One edit. dry=True shows it and writes nothing."""
    p = Plan().patch(path, old, new, count, context)
    return p.show() if dry else _plan_apply(p, None, require_seal=False)


def patch_between(path, start_marker, end_marker, new_lines, context=2, dry=False):
    """Replace a span located by two SHORT markers, without transcribing its body.

    start_marker matches the FIRST line of the span (substring). end_marker matches the first
    line AFTER it (prefix), or None to replace to end of file. Both must be unique.
    """
    p = Plan().between(path, start_marker, end_marker, new_lines, context)
    return p.show() if dry else _plan_apply(p, None, require_seal=False)
