#!/usr/bin/env python3
"""
INT-230 G4 -- the three consumers the Option change exposed.

The compiler found four errors from three causes. Each is a place the stored
health value is consumed, and none of them was visible from the write site --
which is the argument for the type change rather than a comment.

  15114  the capture confirmation renders `health.to_string()` with a HARDCODED
         percent sign. Same treatment as the displays: the % moves inside so
         absence does not print as "unknown%".

  15161  `Value::Int(health)` in a structured row -- and `Value` HAS a Nothing
         variant, so the pipeline has had a way to express absence since it was
         built and this code reached for Int(0) instead.

         ⭐ CHOOSING Text("unknown") OVER Nothing, deliberately. `as_text()`
         renders Nothing as "", which is honest for `awk -F'\\t'` (an empty
         field is unambiguously not a number) but SILENT for a human scanning
         the table. Health is the column this whole gate is about. Nothing fits
         a field with no value; health has a value that was never measured, and
         that difference is worth reading as different.

  15638  the `snap` binding is EXPLICITLY annotated `Option<(String, i64, i64)>`,
         so the middle slot needs Option<i64>. Fixing this also fixes the fourth
         error at 15659 ("i64 is not an iterator"), which was only a symptom of
         `sh` still being typed i64.
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


edits = []

# ---- 15114: the capture confirmation ----------------------------------------
i = one(
    "        \"  {} Snapshot '{}' captured — health: {}%  commits: {}  procs: {}  load: {}\",",
    "capture format string",
)
edits.append((i, i, [
    "        \"  {} Snapshot '{}' captured — health: {}  commits: {}  procs: {}  load: {}\",\n",
]))

i = one("        health.to_string().bright_green(),", "capture health arg")
edits.append((i, i, [
    "        // INT-230 G4: the % moved into the format above so absence does not\n",
    "        // render as \"unknown%\".\n",
    "        health\n",
    "            .map(|h| format!(\"{}%\", h))\n",
    "            .unwrap_or_else(|| \"unknown\".to_string())\n",
    "            .bright_green(),\n",
]))

# ---- 15161: the structured row ----------------------------------------------
# ⚠️ `Value::Int(health)` appears TWICE -- 6596 is inside the events query, with
# its OWN i64 binding parsed from a JSON payload, and must NOT be converted.
# Anchored on the `time` insert directly above the one we want.
t = one("                    row.insert(\"time\".to_string(), Value::Text(time));", "row time insert")
i = t + 1
if lines[i].rstrip("\n") != "                    row.insert(\"health\".to_string(), Value::Int(health));":
    print("ABORT: row health not on the line after row time", file=sys.stderr)
    sys.exit(1)
edits.append((i, i, [
    "                    // INT-230 G4: Value has a Nothing variant, but as_text()\n",
    "                    // renders it as \"\" -- honest for awk, silent for a human\n",
    "                    // reading the table. Health is the column this gate is\n",
    "                    // about, so absence says so.\n",
    "                    row.insert(\n",
    "                        \"health\".to_string(),\n",
    "                        match health {\n",
    "                            Some(h) => Value::Int(h),\n",
    "                            None => Value::Text(\"unknown\".to_string()),\n",
    "                        },\n",
    "                    );\n",
]))

# ---- 15638: the declared tuple type -----------------------------------------
i = one("    let snap: Option<(String, i64, i64)> = db", "snap annotation")
edits.append((i, i, [
    "    // INT-230 G4: health is nullable. This annotation is what made `sh` an\n",
    "    // i64 below, so fixing it here also fixes the render at the println.\n",
    "    let snap: Option<(String, Option<i64>, i64)> = db\n",
]))

for start, end, repl in sorted(edits, key=lambda e: e[0], reverse=True):
    lines[start:end + 1] = repl

with io.open(P, "w", encoding="utf-8") as fh:
    fh.writelines(lines)

print("patched " + P + " (" + str(len(edits)) + " spans)")
print("")
print("⚠️ The capture format string contains an em-dash (U+2014). If the anchor")
print("   misses, that is why -- read it with cat -A rather than retyping it.")
print("")
print("Next: cargo build -p novashell --message-format=short")
