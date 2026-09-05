#!/usr/bin/env python3
"""
INT-230 G4 -- three health displays and one dead function.

Every anchor below was read with `cat -A` first. Three attempts at an earlier
G4 patch failed on reconstructed whitespace; the blank lines here are known, not
assumed, and they differ between the two health functions:

    6816 arm:   blank line between `};` and `println!(`
    11479 fn:   NO blank line between the score and `let status`, blank after `};`
    15558:      no blank lines in the span at all
    prompt.rs:  header comment at 594, blank at 595, `#[allow(dead_code)]` at 596

### THE THREE DISPLAYS

All three do `db.health_score().unwrap_or(0)` and immediately branch:

    >= 95 HEALTHY / >= 80 ADVISORY / else DEGRADED

so a machine that has never run the doctor reports **0% DEGRADED** -- number and
verdict both invented. PROVEN LIVE with a genuinely clean room
(`HOME=/tmp/g3home FAELIGHT_STATE_DB=/tmp/g3home/clean.db nsh -c health`).

⚠️ THE CLEAN ROOM NEEDED BOTH VARIABLES. With `HOME` alone the same command
printed `88% ADVISORY` -- this machine's REAL health, shown as though it
belonged to a machine with no forest, because `health_score()` reads the
database and `HOME` does not redirect it. Six turns of G4 testing used the
incomplete form.

### THE DELETION

`prompt.rs::status_line` is `#[allow(dead_code)]` with no caller -- the only
other mention is a header comment calling it "printed after clear or on
welcome", which it no longer is. It defaulted health to 95. Dead code carrying a
lie; deleted with its comment.

### NOT IN THIS PATCH, AND WHY

`commands/mod.rs:15043` writes health into `shell_snapshots`. The column is
nullable, so the WRITE is one line -- but FIVE readers (14943, 15119, 15198,
15615) pull it positionally with `r.get::<_, i64>(3)`, which fails at RUNTIME on
a NULL. Writing Option without converting all five would break `snapshot list`
and `rewind` on any machine that snapshotted before its first doctor run. Six
edits, its own commit.

📍 FOUND WHILE READING, FILED NOT FIXED: the query at 14943 selects
`command, git_hash, cwd, intent_id` FROM `shell_snapshots` -- columns that
table's CREATE at 13585 does not define. They belong to `command_snapshots`,
which db.rs:108 says was deliberately split out. And `Err(_) => "No snapshots
yet."` swallows the failure either way, so a broken query reads as an empty
result. INT-192's exact shape.
"""

import io
import sys


def load(p):
    with io.open(p, "r", encoding="utf-8") as fh:
        return fh.readlines()


def save(p, lines):
    with io.open(p, "w", encoding="utf-8") as fh:
        fh.writelines(lines)


def one(lines, needle, label):
    hits = [i for i, l in enumerate(lines) if l.rstrip("\n") == needle]
    if len(hits) != 1:
        print("ABORT: " + label + " matched " + str(len(hits)) + ", need 1", file=sys.stderr)
        sys.exit(1)
    return hits[0]


def after(lines, needle, start, label, window=25):
    for j in range(start + 1, min(start + window, len(lines))):
        if lines[j].rstrip("\n") == needle:
            return j
    print("ABORT: " + label + " not found in " + str(window) + " lines", file=sys.stderr)
    sys.exit(1)


def apply(path, specs):
    lines = load(path)
    resolved = [s(lines) for s in specs]
    for start, end, repl in sorted(resolved, key=lambda e: e[0], reverse=True):
        lines[start:end + 1] = repl
    save(path, lines)
    print("patched " + path + " (" + str(len(resolved)) + " spans)")


# ==================================================== commands/mod.rs
def arm_score(lines):
    i = one(lines, "                let health = db.health_score().unwrap_or(0);", "arm 6816 score")
    return (i, i, [
        "                // INT-230 G4: was unwrap_or(0) -- a machine that had\n",
        "                // never run the doctor reported 0% DEGRADED, number and\n",
        "                // verdict both invented.\n",
        "                let health = db.health_score();\n",
    ])


def arm_status(lines):
    i = one(lines, "                let status = if health >= 95 {", "arm 6816 status")
    j = after(lines, "                };", i, "arm 6816 status end")
    return (i, j, [
        "                let status = match health {\n",
        "                    Some(h) if h >= 95 => \"HEALTHY\".bright_green().bold(),\n",
        "                    Some(h) if h >= 80 => \"ADVISORY\".yellow().bold(),\n",
        "                    Some(_) => \"DEGRADED\".bright_red().bold(),\n",
        "                    None => \"no doctor run recorded\".dimmed(),\n",
        "                };\n",
        "                let health_display = match health {\n",
        "                    Some(h) => format!(\"{}%\", h),\n",
        "                    None => \"unknown\".to_string(),\n",
        "                };\n",
    ])


def arm_fmt(lines):
    i = one(lines, "                    format!(\"{}%\", health).bright_white().bold(),", "arm 6816 fmt")
    return (i, i, ["                    health_display.bright_white().bold(),\n"])


def fn_score(lines):
    # ⚠️ `let health = db.health_score().unwrap_or(0);` at 4-space indent appears
    # THREE times (11479, 15043, 15558). Anchored on the preceding line, which
    # differs at each site: the fn signature here, `// Capture health` at 15043,
    # `// Health` at 15558.
    sig = one(lines, "fn health(db: &ForestDb) -> CommandResult {", "fn 11479 signature")
    i = sig + 1
    if lines[i].rstrip("\n") != "    let health = db.health_score().unwrap_or(0);":
        print("ABORT: fn 11479 score not on the line after the signature", file=sys.stderr)
        sys.exit(1)
    return (i, i, [
        "    // INT-230 G4: same collapse as the `health` target arm above.\n",
        "    let health = db.health_score();\n",
        "    let health_display = match health {\n",
        "        Some(h) => format!(\"{}%\", h),\n",
        "        None => \"unknown\".to_string(),\n",
        "    };\n",
    ])


def fn_status(lines):
    i = one(lines, "    let status = if health >= 95 {", "fn 11479 status")
    j = after(lines, "    };", i, "fn 11479 status end")
    return (i, j, [
        "    let status = match health {\n",
        "        Some(h) if h >= 95 => \"HEALTHY\".bright_green(),\n",
        "        Some(h) if h >= 80 => \"ADVISORY\".yellow(),\n",
        "        Some(_) => \"DEGRADED\".bright_red(),\n",
        "        None => \"no doctor run recorded\".dimmed(),\n",
        "    };\n",
    ])


def fn_fmt(lines):
    i = one(lines, "        format!(\"{}%\", health).bright_white().bold(),", "fn 11479 fmt")
    return (i, i, ["        health_display.bright_white().bold(),\n"])


def forest_score_line(lines):
    # Anchored on `    // Health`, which is unique and immediately precedes it.
    c = one(lines, "    // Health", "15558 comment")
    i = c + 1
    if lines[i].rstrip("\n") != "    let health = db.health_score().unwrap_or(0);":
        print("ABORT: 15558 score not on the line after // Health", file=sys.stderr)
        sys.exit(1)
    return (i, i, [
        "    // INT-230 G4: was unwrap_or(0) -- 0% rendered red as though measured.\n",
        "    let health = db.health_score();\n",
    ])


def forest_score(lines):
    i = one(lines, "    let health_color = if health >= 95 {", "15558 color")
    j = after(lines, "    };", i, "15558 color end")
    return (i, j, [
        "    let health_color = match health {\n",
        "        Some(h) if h >= 95 => format!(\"{}%\", h).bright_green(),\n",
        "        Some(h) if h >= 80 => format!(\"{}%\", h).yellow(),\n",
        "        Some(h) => format!(\"{}%\", h).bright_red(),\n",
        "        // The percent sign moves INSIDE, so absence does not render as\n",
        "        // the string \"unknown%\".\n",
        "        None => \"unknown\".dimmed(),\n",
        "    };\n",
    ])


def forest_println(lines):
    i = one(lines, "    println!(\"  {}  {}%\", \"Health:\".dimmed(), health_color);", "15558 println")
    return (i, i, ["    println!(\"  {}  {}\", \"Health:\".dimmed(), health_color);\n"])


apply("faelight/rust-tools/novashell/src/commands/mod.rs", [
    arm_score, arm_status, arm_fmt,
    fn_score, fn_status, fn_fmt,
    forest_score_line, forest_score, forest_println,
])

print("")
print("⚠️ commands/mod.rs:15043 is NOT touched -- it writes to shell_snapshots")
print("   and five readers pull that column positionally as i64, so a NULL")
print("   would fail at RUNTIME. Six edits, its own commit.")
print("")
print("Next: cargo build -p novashell --message-format=short")
