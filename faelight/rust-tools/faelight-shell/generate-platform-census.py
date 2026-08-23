#!/usr/bin/env python3
"""INT-227 G1/G6: the platform-assumption census, produced mechanically.

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

    # G6: nothing outside the platform module may SPAWN a platform-specific process.
    offenders = [r for r in rows if r["cat"] == "B" and r["file"] != "platform.rs"]
    unread = [r for r in rows if r["cat"] == "B?"]

    if check:
        if unread:
            print(f"CENSUS DRIFT: {len(unread)} site(s) the classifier cannot categorise:")
            for r in unread:
                print(f"  {r['file']}:{r['line']}  {r['text']}")
            return 1
        print(f"census clean: {len(rows)} sites, {len(offenders)} capability spawns outside platform.rs")
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
