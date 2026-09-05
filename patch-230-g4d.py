#!/usr/bin/env python3
"""
INT-230 G4 -- the STORED health value and its three readers.

⚠️ FIFTH ANCHOR COLLISION TODAY, AND THE MOST DANGEROUS SO FAR.
`r.get::<_, i64>(3)?,` appears FIVE times in this file and
`r.get::<_, i64>(1)?,` appears five more. Worse: inside the `fetch` closure
alone, line 15211 is HEALTH and 15213 is PROCESSES -- both matching the same
generic pattern. Anchoring on the binding would have converted the wrong column
AND STILL COMPILED, because both are i64.

⭐ THE DURABLE FORM: anchor on the SQL string, which is unique, and index the
column bindings from it. Generic-looking text in a 16,000-line file usually is
not unique; a query is.

### WHY THIS IS SEPARATE FROM THE DISPLAY FIXES

`capture_snapshot` fires BEFORE DESTRUCTIVE COMMANDS, so a snapshot on a machine
with no doctor run recorded `health = 0` AS A FACT -- the record read back when
something has gone wrong. A display lie lasts a render; a stored lie is history.

The column is nullable (`PRAGMA table_info` -> notnull=0), so the truth was
always representable. Three readers pull it as non-null i64 and a NULL fails at
RUNTIME, so the write cannot move alone.

### ⚠️ THE DIFF IS A DESIGN DECISION

`snapshot diff` computes `s2.1 - s1.1` and prints "unchanged" on zero. With
`unwrap_or(0)`, two UNMEASURED snapshots compare equal and report "unchanged" --
a measurement claim about two things never measured, in a command whose whole
purpose is comparing measurements.

⭐ A DIFF AGAINST AN UNKNOWN BASELINE IS NOT A DIFF. Three-way match.

And today's behaviour is already wrong the other way: a NULL makes `fetch`'s `?`
return None and the command says "Snapshot #N not found" about a snapshot that
plainly exists -- a failure disguised as absence.

📍 INT-244 (filed): the live table has 13 columns, the CREATE defines 9. Four
are ALTER-only, so a source-built database cannot run the 14953 query.
📍 NOT FIXED: 15129 also binds commits, processes and load_avg as non-null, and
all three are nullable. Any NULL there already breaks `snapshot list`.
"""

import io
import sys

P = "faelight/rust-tools/novashell/src/commands/mod.rs"

with io.open(P, "r", encoding="utf-8") as fh:
    lines = fh.readlines()


def one(needle, label):
    hits = [i for i, l in enumerate(lines) if l.rstrip("\n") == needle]
    if len(hits) != 1:
        print("ABORT: " + label + " matched " + str(len(hits)) + ", need 1", file=sys.stderr)
        sys.exit(1)
    return hits[0]


def contains_one(fragment, label):
    hits = [i for i, l in enumerate(lines) if fragment in l]
    if len(hits) != 1:
        print("ABORT: " + label + " matched " + str(len(hits)) + ", need 1", file=sys.stderr)
        sys.exit(1)
    return hits[0]


def expect(idx, needle, label):
    if lines[idx].rstrip("\n") != needle:
        print("ABORT: " + label + " -- line " + str(idx + 1) + " is:", file=sys.stderr)
        print("  " + lines[idx].rstrip("\n"), file=sys.stderr)
        sys.exit(1)


edits = []

# ---- the WRITE ---------------------------------------------------------------
c = one("    // Capture health", "capture comment")
expect(c + 1, "    let health = db.health_score().unwrap_or(0);", "capture score")
edits.append((c + 1, c + 1, [
    "    // INT-230 G4: was unwrap_or(0). capture_snapshot fires BEFORE\n",
    "    // DESTRUCTIVE COMMANDS, so a snapshot on a doctorless machine recorded\n",
    "    // health 0 AS A FACT -- the record read back when something went wrong.\n",
    "    // The column is nullable, so the truth was always representable.\n",
    "    let health = db.health_score();\n",
]))

# ---- reader: snapshot list. Anchored on its SQL. -----------------------------
sql = contains_one(
    "SELECT id, name, timestamp, health, commits, processes, load_avg FROM shell_snapshots",
    "list SQL",
)
# health is column 3; find its binding within the closure below the SQL.
h = None
for j in range(sql, sql + 20):
    if lines[j].rstrip("\n") == "                r.get::<_, i64>(3)?,":
        h = j
        break
if h is None:
    print("ABORT: list health binding not found below its SQL", file=sys.stderr)
    sys.exit(1)
edits.append((h, h, [
    "                // INT-230 G4: health is nullable and may now be NULL.\n",
    "                r.get::<_, Option<i64>>(3)?,\n",
]))

# ---- reader: snapshot diff. Anchored on the closure signature. ---------------
f = one(
    "    let fetch = |id: i64| -> Option<(String, i64, i64, i64, String)> {",
    "fetch signature",
)
edits.append((f, f, [
    "    // INT-230 G4: health is Option -- a NULL used to make the `?` below\n",
    "    // return None, so this reported \"Snapshot #N not found\" about a snapshot\n",
    "    // that plainly exists. A failure disguised as absence.\n",
    "    let fetch = |id: i64| -> Option<(String, Option<i64>, i64, i64, String)> {\n",
]))

# health is column 1 INSIDE this closure. ⚠️ 15213 nearby is PROCESSES and
# matches the same generic pattern, so the search is bounded and indexed.
fh_idx = None
for j in range(f, f + 15):
    if lines[j].rstrip("\n") == "                r.get::<_, i64>(1)?,":
        fh_idx = j
        break
if fh_idx is None:
    print("ABORT: fetch health binding not found below the signature", file=sys.stderr)
    sys.exit(1)
edits.append((fh_idx, fh_idx, ["                r.get::<_, Option<i64>>(1)?,\n"]))

# The diff computation.
# ⚠️ `"unchanged".dimmed().to_string()` appears TWICE -- the health diff at
# 15248 and the commits diff at 15276, which share a rendering shape. Bounded
# search from `let health_diff` so the commits one cannot be matched.
d = one("    let health_diff = s2.1 - s1.1;", "health diff")
u = None
for j in range(d, d + 15):
    if lines[j].rstrip("\n") == "        \"unchanged\".dimmed().to_string()":
        u = j
        break
if u is None:
    print("ABORT: diff unchanged arm not found below health_diff", file=sys.stderr)
    sys.exit(1)
expect(u + 1, "    };", "diff closing brace")
edits.append((d, u + 1, [
    "    // ⭐ A DIFF AGAINST AN UNKNOWN BASELINE IS NOT A DIFF. With unwrap_or(0)\n",
    "    // two UNMEASURED snapshots would compare equal and print \"unchanged\" --\n",
    "    // a measurement claim about two things that were never measured, in a\n",
    "    // command whose whole purpose is comparing measurements.\n",
    "    let health_str = match (s1.1, s2.1) {\n",
    "        (Some(a), Some(b)) if b > a => format!(\"+{}\", b - a).bright_green().to_string(),\n",
    "        (Some(a), Some(b)) if b < a => format!(\"{}\", b - a).bright_red().to_string(),\n",
    "        (Some(_), Some(_)) => \"unchanged\".dimmed().to_string(),\n",
    "        _ => \"unknown\".dimmed().to_string(),\n",
    "    };\n",
]))

i = one("        s1.1.to_string().bright_white(),", "diff s1 render")
edits.append((i, i, [
    "        s1.1.map(|h| h.to_string())\n",
    "            .unwrap_or_else(|| \"unknown\".to_string())\n",
    "            .bright_white(),\n",
]))
i = one("        s2.1.to_string().bright_white(),", "diff s2 render")
edits.append((i, i, [
    "        s2.1.map(|h| h.to_string())\n",
    "            .unwrap_or_else(|| \"unknown\".to_string())\n",
    "            .bright_white(),\n",
]))

# ---- reader: last-snapshot line ----------------------------------------------
# ⚠️ `|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),` appears THREE times. Anchored
# on its SQL, which is unique.
sql2 = contains_one(
    "SELECT name, health, commits FROM shell_snapshots",
    "last-snapshot SQL",
)
i = None
for j in range(sql2, sql2 + 8):
    if lines[j].rstrip("\n") == "            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),":
        i = j
        break
if i is None:
    print("ABORT: last-snapshot row not found below its SQL", file=sys.stderr)
    sys.exit(1)
edits.append((i, i, [
    "            |r| {\n",
    "                Ok((\n",
    "                    r.get::<_, String>(0)?,\n",
    "                    // INT-230 G4: nullable.\n",
    "                    r.get::<_, Option<i64>>(1)?,\n",
    "                    r.get::<_, i64>(2)?,\n",
    "                ))\n",
    "            },\n",
]))

# `sh.to_string().dimmed(),` -- verified unique by the assertion below.
i = one("            sh.to_string().dimmed(),", "last-snapshot render")
edits.append((i, i, [
    "            sh.map(|h| h.to_string())\n",
    "                .unwrap_or_else(|| \"unknown\".to_string())\n",
    "                .dimmed(),\n",
]))

for start, end, repl in sorted(edits, key=lambda e: e[0], reverse=True):
    lines[start:end + 1] = repl

with io.open(P, "w", encoding="utf-8") as fh:
    fh.writelines(lines)

print("patched " + P + " (" + str(len(edits)) + " spans)")
print("")
print("Next: cargo build -p novashell --message-format=short")
