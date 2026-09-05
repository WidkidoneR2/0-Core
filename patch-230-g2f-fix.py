#!/usr/bin/env python3
"""
INT-230 -- repair the focus_intent cut from patch-230-g2f.

TWO ERRORS, BOTH CLAUDE'S:

  1. The span cut ended at `} else {` and left three lines referencing
     `in_progress`, which no longer exists. The compiler caught it.

  2. WORSE, AND THE COMPILER COULD NOT CATCH IT: the replacement changed the
     display string from a de-slugged filename starting with a BARE NUMBER
     ("230 fsh cannot be installed...") to "INT-230". Twenty lines below,
     focus persistence does:

         if let Some(int_id) = focus.split_whitespace().next() {
             if int_id.chars().all(|c| c.is_ascii_digit()) {
                 let intent_key = format!("INT-{}", int_id);

     With "INT-230" the first token is not all digits, the guard fails, and
     FOCUS PERSISTENCE SILENTLY STOPS -- inside an `if`, with no error. The
     prompt reads focus from that write, so the failure would have shown up as
     "the prompt stopped tracking focus" days later, with no obvious cause.

THE FIX KEEPS THE ID BARE so the existing extraction still holds. The title is
dropped from the display because the adapter no longer parses it -- so the line
reads "230" where it used to read "230 fsh cannot be installed without 0 core".
That is a real narrowing and it is stated rather than hidden.
"""

import io
import os
import sys

SRC = "faelight/rust-tools/novashell/src"


def die(msg):
    print("ABORT: " + msg, file=sys.stderr)
    sys.exit(1)


p = os.path.join(SRC, "main.rs")
with io.open(p, "r", encoding="utf-8") as fh:
    t = fh.read()

old = '''    let focus_intent: Option<String> = crate::core_integration::ledger().and_then(|l| {
        let mut names: Vec<String> = l
            .active()
            .iter()
            .map(|i| format!("INT-{}", i.id))
            .collect();
        names.sort();
        if names.is_empty() {
            None
        } else {
                    Some(in_progress.join(", "))
                }
            });'''

new = '''    // ⚠️ THE ID STAYS BARE. Twenty lines below, focus persistence takes the
    // first whitespace token and requires it to be all ASCII digits before
    // writing `INT-{id}` to the database. Formatting the id as "INT-230" here
    // makes that guard fail silently -- inside an `if`, with no error -- and the
    // prompt reads focus from that write. The display and the extraction are
    // coupled, so the coupling is written down rather than rediscovered.
    let focus_intent: Option<String> = crate::core_integration::ledger().and_then(|l| {
        let mut ids: Vec<String> = l.active().iter().map(|i| i.id.clone()).collect();
        ids.sort();
        if ids.is_empty() {
            None
        } else {
            Some(ids.join(", "))
        }
    });'''

n = t.count(old)
if n != 1:
    die("focus_intent repair: matched " + str(n) + " times, need 1")

with io.open(p, "w", encoding="utf-8") as fh:
    fh.write(t.replace(old, new))

print("patched " + p)
print("")
print("⚠️ DISPLAY NARROWING, stated: the Today line now reads the bare id (230)")
print("   where it read a de-slugged title (230 fsh cannot be installed...).")
print("   The adapter does not parse titles. If the title is wanted back, that is")
print("   a title field on Intent -- which was DELETED earlier this session for")
print("   having no consumer. It would now have one.")
print("")
print("Next: cargo build -p novashell --message-format=short")
