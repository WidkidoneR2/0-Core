#!/usr/bin/env python3
"""
INT-230 G4 -- three of the nine health readers. Verified spans only.

⚠️ CLAUDE SAID "TWO HEALTH READERS" AND WAS WRONG BY A FACTOR OF FOUR. The first
grep used `-B6 -A4 | head -30`; the truncation hid seven. Same truncation error
that hid main.rs:1880 during the Omarchy port. The line-anchor assertion caught
it -- three matches where one was expected -- and nothing was written.

`db.health_score()` is HONEST: `Option<i64>`, None when no doctor event exists.
Nine callers disagree about what None means:

    main.rs:2037          keeps the Option           <- the only correct one
    prompt.rs:311         unwrap_or(95)              <- LIVE, EVERY PROMPT
    prompt.rs:591         unwrap_or(95)              <- dead code
    db.rs:559             unwrap_or(0)               <- WRITES A FABRICATED ROW
    commands 6816         unwrap_or(0)               display
    commands 11479        unwrap_or(0)               display
    commands 15043        unwrap_or(0)               stored record
    commands 15558        unwrap_or(0)               display
    scripting.rs:341      unwrap_or(0)               exposed to scripts

⭐ THREE ANSWERS FOR ONE ABSENT MEASUREMENT IN ONE BINARY: 0, 95, and None.

### THIS PATCH DOES THREE

**prompt.rs:311** renders on EVERY PROMPT. A doctorless machine showed 95%
health hundreds of times a day. The August work fixed the FILE reader
(`read_health`, three callers, all Option now) and left this SCORE reader four
lines away still fabricating. One file, two health sources, one repaired.

**db.rs:559** writes the fabricated value. `capture_snapshot` fires BEFORE
DESTRUCTIVE COMMANDS, so every snapshot on a doctorless machine records health 0
AS A FACT -- exactly the record read back when something went wrong. ⭐ The
column is NULLABLE (db.rs:115), so the truth was always representable; the
collapse was purely in Rust.

**scripting.rs:341** hands scripts the string "0", indistinguishable from a real
zero.

### NOT IN THIS PATCH, AND SAID PLAINLY

- `prompt.rs::status_line` (dead, defaults to 95) -- deleting it needs its exact
  span read first. An earlier draft computed it as `allow_line - 1`, which is
  index arithmetic on bytes nobody has looked at, and that has failed three
  times today already.
- `commands/mod.rs` 6816, 11479, 15043, 15558 -- four sites, two with three-way
  colour branches and one a stored record. Their own pass.

⚠️ AND prompt.rs:311 BECOMES 0, NOT AN OPTION. The prompt renders a powerline
badge; threading Option through it is a display redesign, not a wrap. Zero is
not honest either -- it is LESS WRONG than 95, and it is named REMAINING G4 WORK
rather than claimed as fixed.
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


def apply(path, specs):
    lines = load(path)
    resolved = [s(lines) for s in specs]
    for start, end, repl in sorted(resolved, key=lambda e: e[0], reverse=True):
        lines[start:end + 1] = repl
    save(path, lines)
    print("patched " + path + " (" + str(len(resolved)) + " spans)")


# ============================================================ prompt.rs
def prompt_311(lines):
    i = one(lines, "    let health = db.health_score().unwrap_or(95);", "prompt 311")
    return (i, i, [
        "    // INT-230 G4: was unwrap_or(95). On a machine that has never run the\n",
        "    // doctor this asserted 95% health ON EVERY PROMPT. The August work\n",
        "    // fixed the FILE reader (read_health, Option in all three callers)\n",
        "    // and left this SCORE reader four lines away still fabricating.\n",
        "    // ⚠️ 0 is not honest either -- it is less wrong than 95. Threading\n",
        "    // Option through the powerline badge is a display redesign and is\n",
        "    // named as remaining G4 work rather than done here.\n",
        "    let health = db.health_score().unwrap_or(0);\n",
    ])


apply("faelight/rust-tools/novashell/src/prompt.rs", [prompt_311])


# ============================================================ db.rs
def db_559(lines):
    i = one(lines, "        let health = self.health_score().unwrap_or(0);", "db 559")
    return (i, i, [
        "        // INT-230 G4: was unwrap_or(0), so a snapshot taken on a machine\n",
        "        // with no doctor run recorded health 0 AS A FACT -- and\n",
        "        // capture_snapshot fires BEFORE DESTRUCTIVE COMMANDS, which is\n",
        "        // exactly the record read back when something went wrong. The\n",
        "        // column is nullable (db.rs:115), so the truth was always\n",
        "        // representable and the collapse was here.\n",
        "        let health = self.health_score();\n",
    ])


def db_insert(lines):
    needle = "            rusqlite::params![name, ts, health as i64, command, git_hash, cwd, intent_id],"
    i = one(lines, needle, "db insert params")
    return (i, i, [
        "            rusqlite::params![name, ts, health, command, git_hash, cwd, intent_id],\n",
    ])


apply("faelight/rust-tools/novashell/src/db.rs", [db_559, db_insert])


# ============================================================ scripting.rs
def scripting_341(lines):
    needle = "        \"health\" => db.health_score().unwrap_or(0).to_string(),"
    i = one(lines, needle, "scripting 341")
    return (i, i, [
        "        // INT-230 G4: a script asking for health on a doctorless machine\n",
        "        // got the string \"0\", indistinguishable from a real zero.\n",
        "        \"health\" => db\n",
        "            .health_score()\n",
        "            .map(|h| h.to_string())\n",
        "            .unwrap_or_else(|| \"unknown\".to_string()),\n",
    ])


apply("faelight/rust-tools/novashell/src/scripting.rs", [scripting_341])

print("")
print("Next: cargo build -p novashell --message-format=short")
