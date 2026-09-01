#!/usr/bin/env python3
"""INT-191 G1: the shell_history consumer inventory, produced mechanically.

The gate asks for an ARTIFACT, not a paragraph -- something that can be re-run and
diffed rather than re-argued. Run it from the repo root:

    python3 faelight/rust-tools/novashell/generate-history-inventory.py

It writes docs/history-inventory.md.

WHY MECHANICAL. The intent's July enumeration counted seven writers. A first
line-counting pass reported twelve, and reading each one showed the truth was FOUR --
the difference being trigger machinery, a doctor test row written and deleted by one
owner, and two pruning statements. An inventory that disagrees with its own intent is
not evidence until the disagreement is explained, so this script separates what it can
classify from what needs a human, rather than presenting a confident total.
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SRC = ROOT / "faelight"

# Sites that MATCH shell_history but are not consumers of it. Each is here because it
# was read and judged, not because a pattern excluded it.
NOT_A_CONSUMER = {
    "AFTER INSERT ON shell_history": "trigger definition (shell_history_audit)",
    "INSERT INTO shell_history_audit": "writes the AUDIT table, not history",
    "BEFORE UPDATE ON shell_history_audit": "immutability guard on the audit table",
    "BEFORE DELETE ON shell_history_audit": "immutability guard on the audit table",
    "__fsh_doctor_test__": "fsh doctor writability probe -- inserts and deletes its own row",
    "LIKE 'SUGGEST:%' AND timestamp <": "retention pruning, not recording",
    "WHERE timestamp < ?1 AND command IN": "retention pruning, not recording",
}


def classify(text):
    if re.search(r"INSERT INTO shell_history\b|UPDATE shell_history\b|DELETE FROM shell_history\b", text):
        return "WRITE"
    if "SELECT" in text:
        return "READ"
    return "UNTYPED"


def main():
    out = subprocess.run(
        ["grep", "-rn", "--include=*.rs", "shell_history", str(SRC)],
        capture_output=True, text=True,
    ).stdout

    rows = []
    for line in out.splitlines():
        if not line:
            continue
        path, num, text = line.split(":", 2)
        stripped = text.strip()
        if stripped.startswith("//"):
            continue
        skip = next((why for pat, why in NOT_A_CONSUMER.items() if pat in stripped), None)
        rows.append({
            "file": str(Path(path).relative_to(SRC)),
            "line": int(num),
            "kind": classify(stripped),
            "skip": skip,
            "text": stripped[:100],
        })

    live = [r for r in rows if not r["skip"]]
    excluded = [r for r in rows if r["skip"]]
    writes = [r for r in live if r["kind"] == "WRITE"]
    reads = [r for r in live if r["kind"] == "READ"]
    untyped = [r for r in live if r["kind"] == "UNTYPED"]

    doc = []
    doc.append("# shell_history inventory (INT-191 G1)")
    doc.append("")
    doc.append("GENERATED. Do not edit by hand -- run")
    doc.append("`python3 faelight/rust-tools/novashell/generate-history-inventory.py`.")
    doc.append("")
    doc.append(f"- writers: **{len(writes)}**")
    doc.append(f"- readers: **{len(reads)}**")
    doc.append(f"- untyped (multi-line statements a single line cannot classify): **{len(untyped)}**")
    doc.append(f"- matched but NOT consumers: **{len(excluded)}**")
    doc.append("")
    doc.append("## Writers")
    doc.append("")
    doc.append("The gate asks whether every history write has a single, well-defined owner.")
    doc.append("")
    for r in sorted(writes, key=lambda r: (r["file"], r["line"])):
        doc.append(f"- `{r['file']}:{r['line']}` -- {r['text']}")
    doc.append("")
    doc.append("## Matched but not consumers")
    doc.append("")
    doc.append("Each was read and judged rather than excluded by a pattern.")
    doc.append("")
    for r in sorted(excluded, key=lambda r: (r["file"], r["line"])):
        doc.append(f"- `{r['file']}:{r['line']}` -- {r['skip']}")
    doc.append("")
    doc.append("## Readers, by file")
    doc.append("")
    by_file = {}
    for r in reads + untyped:
        by_file.setdefault(r["file"], []).append(r)
    for f in sorted(by_file):
        doc.append(f"### {f} ({len(by_file[f])})")
        doc.append("")
        for r in sorted(by_file[f], key=lambda r: r["line"]):
            doc.append(f"- `{r['line']}` [{r['kind']}] {r['text']}")
        doc.append("")

    target = ROOT / "docs" / "history-inventory.md"
    target.write_text("\n".join(doc) + "\n")
    print(f"wrote {target.relative_to(ROOT)}: {len(writes)} writers, {len(reads)} readers, "
          f"{len(untyped)} untyped, {len(excluded)} excluded")


if __name__ == "__main__":
    sys.exit(main())
