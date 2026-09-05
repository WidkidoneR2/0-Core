#!/usr/bin/env python3
"""
INT-230 -- Today: renders the FOCUS, not the active list.

### WHAT THE EVIDENCE SETTLED

`focus.toml` is the source of truth. It was written by `cistart 230` at
23:51:17 and holds:

    id = "230"
    title = "fsh cannot be installed without 0-Core, ..."
    started = "2026-09-04T23:51:17"
    workflow = "in-progress"

The engine says so in its own comment (`friday/attention.rs:98`): *"read
focus.toml (written by cistart, source of truth)"*.

### THREE THINGS RESOLVED HERE

1. **`Today:` was a SECOND RENDERER of the active list.** Fixing the blind
   reader made it print four full titles across three wrapped lines, directly
   above `Working on` saying the same thing in ids. Its own comment always said
   "today's focus" -- singular. Four active intents is not a focus. It now
   renders the focused intent and prints NOTHING when none is set.

2. **The auto-persist block is DELETED.** It wrote `set_focus_intent`, which
   stores `shell_state.focus_intent` -- a key NO READER READS, because
   `get_focus_intent` reads focus.toml. It was also unreachable: the old display
   stripped leading digits, so its all-digits guard rejected every value. A
   write nobody reads, guarded by a test that never passed, deriving a "focus"
   from "the first of four active intents" -- which invents a fact. All three
   reasons point the same way.

3. **A 0-CORE PATH THE CENSUS CANNOT SEE.** `db.rs:495` hand-builds
   `$HOME/.local/state/0-core/intent/focus.toml` from `env::var("HOME")` and
   never asks `paths.rs`. The census only finds `paths::` calls, so this
   coupling was invisible to G1. It is INT-240's subject sitting inside the
   mechanism. The adapter now owns the path.

### NOT FIXED HERE, AND FILED AS INT-242

`flow focus INT-231` writes the database key, prints "focus set" in green, and
changes nothing any reader observes. `flow clear` clears a key nobody reads, so
focus survives being cleared. And the formats differ -- `flow` stores "INT-231"
while focus.toml stores "230" bare. Three defects in one mechanism, bigger than
this boundary, and not folded in silently.
"""

import io
import os
import sys

SRC = "faelight/rust-tools/novashell/src"


def die(msg):
    print("ABORT: " + msg, file=sys.stderr)
    sys.exit(1)


def read(path):
    with io.open(path, "r", encoding="utf-8") as fh:
        return fh.read()


def write(path, text):
    with io.open(path, "w", encoding="utf-8") as fh:
        fh.write(text)


def swap(text, path, old, new, count, label):
    n = text.count(old)
    if n != count:
        die(path + " [" + label + "]: matched " + str(n) + " times, need " + str(count))
    return text.replace(old, new)


edits = []

# ---------------------------------------------------------- core_integration.rs
p = os.path.join(SRC, "core_integration.rs")
t = read(p)

anchor = "/// The status a lifecycle folder carries in frontmatter."
addition = '''/// The intent `cistart` last focused, when 0-Core is present.
///
/// ⚠️ `focus.toml` IS THE SOURCE OF TRUTH and the engine says so
/// (`friday/attention.rs:98`: "written by cistart, source of truth"). It holds
/// the id AND the title, which is what a focus line wants.
///
/// ⚠️ THE OTHER OWNER IS BROKEN AND IS NOT USED HERE: `db::set_focus_intent`
/// writes `shell_state.focus_intent`, a key no reader reads, while
/// `db::get_focus_intent` reads this file. A setter and a getter sharing a name
/// and not a storage. Filed as INT-242, deliberately not fixed inside this
/// boundary.
///
/// 📍 The path was hand-built from `env::var("HOME")` at `db.rs:495` and never
/// asked `paths.rs` -- so the census, which only finds `paths::` calls, could
/// not see this coupling at all. INT-240's subject, inside the mechanism.
pub fn focus() -> Option<(String, String)> {
    let path = faelight_core::paths::state_home()
        .join("0-core/intent/focus.toml");
    let content = std::fs::read_to_string(path).ok()?;
    let mut id = String::new();
    let mut title = String::new();
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("id = ") {
            id = rest.trim().trim_matches('"').to_string();
        } else if let Some(rest) = line.strip_prefix("title = ") {
            title = rest.trim().trim_matches('"').to_string();
        }
    }
    if id.is_empty() {
        None
    } else {
        Some((id, title))
    }
}

'''
t = swap(t, p, anchor, addition + anchor, 1, "insert focus accessor")
edits.append((p, t))

# ------------------------------------------------------------------- main.rs
p = os.path.join(SRC, "main.rs")
t = read(p)

old_block = '''    // ⚠️ THE ID COMES FIRST AND STAYS BARE. Persistence below takes the
        // first whitespace token and requires all digits. The OLD display
        // de-slugged the filename and stripped the leading digits, so the first
        // token was a word and the guard rejected it -- meaning the focus write
        // has never once succeeded from here, on top of the reader being blind.
        let mut ids: Vec<String> = l
            .active()
            .iter()
            .map(|i| format!("{} {}", i.id, i.title).trim_end().to_string())
            .collect();
        ids.sort();
        if ids.is_empty() {
            None
        } else {
            Some(ids.join(", "))
        }
    });'''

if t.count(old_block) != 1:
    die("main.rs: focus_intent block matched " + str(t.count(old_block)) + " times, need 1")

# Find the whole construct from `let focus_intent` through the persist block.
start = "    let focus_intent: Option<String> = crate::core_integration::ledger().and_then(|l| {"
end = """                }
            }
        }
    }
    println!();"""

if t.count(start) != 1:
    die("main.rs: focus_intent start matched " + str(t.count(start)) + " times, need 1")
if t.count(end) != 1:
    die("main.rs: persist-block end matched " + str(t.count(end)) + " times, need 1")

i = t.index(start)
j = t.find(end, i + len(start))
if j == -1:
    die("main.rs: persist-block end not found after start")

replacement = '''    // INT-230: THE FOCUS, NOT THE ACTIVE LIST. This line's own comment always
    // said "today's focus" -- singular -- but it rendered every in-progress
    // intent, duplicating the "Working on" line below it and wrapping three
    // lines with four active intents. focus.toml is what cistart writes and
    // what every reader reads.
    //
    // The auto-persist block that sat here is DELETED: it called
    // set_focus_intent, which writes a shell_state key NO READER READS, guarded
    // by an all-digits test the old display could never satisfy, deriving a
    // "focus" from the first of N active intents -- a fact nobody stated.
    // See INT-242.
    if let Some((id, title)) = crate::core_integration::focus() {
        let shown = if title.is_empty() {
            id
        } else {
            format!("{} {}", id, title)
        };
        println!(
            "  {}  {}",
            fc_dim(255, 180, 50, "Today:"),
            fc_bold(255, 230, 100, &shown)
        );
    }
    println!();'''

t = t[:i] + replacement + t[j + len(end):]
edits.append((p, t))

for path, text in edits:
    write(path, text)
    print("patched " + path)

print("")
print("⚠️ focus() calls paths::state_home(). If that helper does not exist the")
print("   build fails and the real name needs looking up -- do not guess it.")
print("")
print("Next: cargo build -p novashell --message-format=short")
