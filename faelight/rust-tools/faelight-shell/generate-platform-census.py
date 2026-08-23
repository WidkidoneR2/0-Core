#!/usr/bin/env python3
"""INT-227 G1 census + a platform error-swallow LINT.

    python3 faelight/rust-tools/faelight-shell/generate-platform-census.py          # write the doc
    python3 faelight/rust-tools/faelight-shell/generate-platform-census.py --check  # G6, exit 1 on drift

WHY MECHANICAL, AND WHY IT CLASSIFIES RATHER THAN COUNTS. The intent's own headline said
"forty-two places", and reading them found: two real defects, twelve TEST FIXTURES where a literal
home is correct, five word-list entries, three candidate lists that already degrade properly, and
ONE genuine platform question. A count is not a census -- the categories are the finding.

THE CATEGORIES:
  A  wrong on EVERY system, not only off NixOS      -- fixed under G2
  B  a real platform capability that must DEGRADE   -- systemctl, journalctl, nix-store, ...
  C  noise: word lists, completions, comments       -- not calls at all
  D  already correct: candidate lists, PATH additions, test fixtures
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SRC = ROOT / "faelight" / "rust-tools" / "faelight-shell" / "src"

PATTERNS = [
    "systemctl", "journalctl", "nixos-rebuild", "nix-store", "nix-env",
    "/nix/store", "/run/current-system", "/home/christian", "pacman",
]

# Judged individually, each with the reason. A pattern-based exclusion would be the thing this
# census exists to replace.
def classify(path: str, text: str, in_test_fn: bool = False):
    name = Path(path).name
    # ⚠️ A #[test] ATTRIBUTE IS RARELY ON THE MATCHED LINE. exec.rs:1524 sat four lines below its
    # attribute and the first version of this classifier called it unread. Test membership is a
    # property of the enclosing function, not of the line, so the caller determines it.
    if in_test_fn:
        return "D", "inside a #[test] function -- a literal path is correct in a fixture"
    # Direct store access is a real capability, not a message about one.
    if "read_dir(\"/nix/store" in text or "starts_with(\"/nix/store/\")" in text:
        return "B", "reads the Nix store directly -- a capability that must degrade"
    if "/tests/" in path or name in {"plan.rs", "compare.rs", "migrate.rs", "classifier.rs"}:
        return "D", "test fixture -- a literal path is correct in an assertion"
    if "#[test]" in text or "assert" in text or "cold_fix(" in text or "classify(" in text:
        return "D", "test assertion or fixture"
    # A path inside a MESSAGE is not a call. Judged individually: each of these formats text for a
    # human, and the platform word in it is describing what failed, not invoking anything.
    if text.lstrip().startswith(('"', 'format!("', '.filter(', 'cold_fix(')) and "Command::new" not in text:
        return "C", "message or predicate text, not a call"
    if "Some(" in text and "NixOS command" in text:
        return "C", "typo-correction advice, not a call"
    if re.search(r'^\s*"[a-z-]+",\s*$', text):
        return "C", "word list entry (completion / typo correction), not a call"
    if "aliases:" in text:
        return "C", "alias name, not a call"
    if name == "platform.rs":
        return "OWNER", "the platform module itself"
    # ⭐ THE GUARD AND ITS MESSAGE ARE THE FIX, not a finding. A site asking has_tool, and the
    # diagnostic it returns when the answer is no, are what this census exists to produce.
    if "has_tool(" in text:
        return "GUARD", "asks the platform before assuming the capability"
    if any(k in text for k in ("no systemctl on this system", "no journalctl on this system",
                               "this machine has no nix-store", "cannot query the store here")):
        return "GUARD", "the message a guard prints when the capability is absent"
    if "candidates" in text or "format!(" in text and "/bin/faelight-shell" in text:
        return "D", "candidate list -- probes and falls through when absent"
    if "nix_system" in text:
        return "D", "PATH augmentation -- harmless when the directory is absent"
    if "Command::new" in text or "nix_query_lines" in text:
        return "B", "real process spawn -- a capability that must degrade"
    return "B?", "needs reading"


def _inside_test(path: str, line: int) -> bool:
    """Is this line inside a #[test] function? Scans back for the nearest attribute or fn."""
    lines = Path(path).read_text().splitlines()
    for i in range(line - 1, max(line - 40, 0), -1):
        t = lines[i].strip()
        if t.startswith("#[test]") or t.startswith("#[cfg(test)]"):
            return True
        if t.startswith("pub fn ") or t.startswith("fn "):
            # reached a function header without seeing #[test] first
            return lines[i - 1].strip().startswith("#[test]") if i else False
    return False


def collect():
    rows = []
    for pat in PATTERNS:
        out = subprocess.run(
            ["grep", "-rn", "--include=*.rs", pat, str(SRC)],
            capture_output=True, text=True,
        ).stdout
        for line in out.splitlines():
            if not line:
                continue
            path, num, text = line.split(":", 2)
            stripped = text.strip()
            if stripped.startswith("//"):
                continue
            cat, why = classify(path, stripped, in_test_fn=_inside_test(path, int(num)))
            rows.append({
                "pattern": pat,
                "file": str(Path(path).relative_to(SRC)),
                "line": int(num),
                "cat": cat,
                "why": why,
                "text": stripped[:90],
            })
    return rows


def main():
    rows = collect()
    check = "--check" in sys.argv

    # ⚠️⚠️ A LINT, NOT A VERIFICATION -- and this was demoted after it PROVED it could not fail.
    # Disabling a live guard with `if false &&` left the guard's TEXT in place, so the scanner still
    # saw `has_tool(` and passed. A checker that reads source text cannot establish a runtime
    # property, and the PLATFORM-CHECKED marker below is a DECLARATION rather than evidence: a
    # comment can claim anything.
    #
    # ⭐ SO THE PROOF LIVES ELSEWHERE. INT-227 G6 is the runtime test that strips a tool from PATH
    # and asserts the real command paths report unavailability rather than emptiness. This scanner
    # is a cheap regression aid that surfaces NEW candidates -- undeclared swallows nobody has
    # reasoned about -- and nothing more.
    #
    # ⚠️ THE ORIGINAL GATE BANNED NAMING A TOOL, which is unachievable -- `generations` legitimately
    # runs nixos-rebuild. Naming a tool is not assuming a capability. This tests the invariant the
    # work actually found: within a few lines of a platform spawn, a failure must not be swallowed.
    # ⭐ A SITE DECLARES ITS OWN HONESTY WITH A MARKER, rather than the scanner widening its window
    # until everything passes. Three sites were honest for reasons sitting just outside a six-line
    # view -- a guard twenty lines up, an explicit unknown fourteen lines down, a precondition
    # established by an earlier query. Widening until green is how a check becomes decoration; the
    # marker makes the claim reviewable and puts the reason beside the code that needs it.
    swallow = (".ok()", "unwrap_or_default()", "if let Ok")
    offenders = []
    for r in [x for x in rows if x["cat"] == "B"]:
        lines = (SRC / r["file"]).read_text().splitlines()
        # ⚠️ THE WINDOW LOOKS BOTH WAYS, and the first version did not. A guard sits ABOVE the
        # spawn it protects, so a forward-only window reported every guarded site as a violation.
        before = "\n".join(lines[max(0, r["line"] - 14) : r["line"] - 1])
        window = "\n".join(lines[r["line"] - 1 : r["line"] + 6])
        # ⭐ AN EXPLICIT UNKNOWN IS A DEGRADE, NOT A SWALLOW, and the distinction is the whole
        # point. main.rs's generation banner ends unwrap_or_else(|| "?") -- it SHOWS that it does
        # not know, which is what this intent asks for. Only a default that READS AS AN ANSWER
        # (an empty collection, a zero, an empty string) is the defect.
        honest = (
            'unwrap_or_else(|| "?"' in window          # an explicit unknown, not a fabricated answer
            or 'unknown' in window
            or 'has_tool(' in before                    # guarded ABOVE the spawn
            or 'map_err' in window                      # the error is carried, not dropped
            or 'PLATFORM-CHECKED' in before             # the site DECLARES why it is honest
        )
        hit = None if honest else next((w for w in swallow if w in window), None)
        if hit:
            offenders.append({**r, "swallow": hit})
    unread = [r for r in rows if r["cat"] == "B?"]

    if check:
        if unread:
            print(f"CENSUS DRIFT: {len(unread)} site(s) the classifier cannot categorise:")
            for r in unread:
                print(f"  {r['file']}:{r['line']}  {r['text']}")
            return 1
        if offenders:
            print(f"LINT: {len(offenders)} platform spawn(s) with an undeclared swallow:")
            for r in offenders:
                print(f"  {r['file']}:{r['line']}  [{r['swallow']}]  {r['text'][:60]}")
            return 1
        print(f"lint clean: {len(rows)} sites, no undeclared swallow near a platform spawn")
        print("  (a lint, not a proof -- G6 is the PATH-stripping runtime test)")
        return 0

    by_cat = {}
    for r in rows:
        by_cat.setdefault(r["cat"], []).append(r)

    doc = ["# platform assumption census (INT-227 G1)", ""]
    doc.append("GENERATED. Do not edit by hand -- run")
    doc.append("`python3 faelight/rust-tools/faelight-shell/generate-platform-census.py`.")
    doc.append("")
    doc.append("⚠️ A COUNT IS NOT A CENSUS. The intent's headline said forty-two places; the")
    doc.append("categories below are what reading them actually found.")
    doc.append("")
    for cat, label in [
        ("A", "wrong on EVERY system"),
        ("B", "platform capability -- must degrade"),
        ("C", "noise -- not a call"),
        ("D", "already correct"),
        ("OWNER", "the platform module"),
        ("B?", "UNREAD -- needs a human"),
    ]:
        rs = by_cat.get(cat, [])
        doc.append(f"- **{cat}** ({label}): {len(rs)}")
    doc.append("")
    for cat in ["B?", "B", "A", "D", "C", "OWNER"]:
        rs = by_cat.get(cat, [])
        if not rs:
            continue
        doc.append(f"## {cat} ({len(rs)})")
        doc.append("")
        for r in sorted(rs, key=lambda r: (r["file"], r["line"])):
            doc.append(f"- `{r['file']}:{r['line']}` [{r['pattern']}] -- {r['why']}")
            doc.append(f"  - `{r['text']}`")
        doc.append("")

    target = ROOT / "docs" / "platform-census.md"
    target.write_text("\n".join(doc) + "\n")
    print(f"wrote {target.relative_to(ROOT)}: {len(rows)} sites")
    for cat in sorted(by_cat):
        print(f"  {cat:6} {len(by_cat[cat])}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
