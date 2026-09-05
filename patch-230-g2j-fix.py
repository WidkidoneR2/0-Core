#!/usr/bin/env python3
"""
INT-230 -- fold the two intents_root definitions into one.

⚠️ TEN ERRORS, ONE CAUSE, AND IT IS THE SAME MISS AS EVERY OTHER ONE TODAY.

`core_integration.rs` ALREADY had a private `fn intents_root() -> PathBuf`,
written in the module's first version to feed `present()`. Claude added a
`pub fn intents_root() -> Option<PathBuf>` beside it without looking. The
compiler reported one redefinition and then resolved all three call sites to
the OLD private function -- producing nine type errors that all pointed
somewhere other than the actual mistake.

⭐ The census was checked for `paths::` calls. The MODULE was not checked for a
function of that name. Measuring one thing and assuming another, which is the
error this whole intent keeps finding in the code and which I keep repeating in
the process.

THE FIX IS A DELETION, NOT A RENAME. The private helper existed only so
`present()` could ask whether the directory is there. The public accessor
answers a strictly stronger question, so `present()` is expressed in terms of it
and the helper goes -- one owner, which is the entire point of this module.
"""

import io
import os
import sys

SRC = "faelight/rust-tools/novashell/src"
p = os.path.join(SRC, "core_integration.rs")


def die(msg):
    print("ABORT: " + msg, file=sys.stderr)
    sys.exit(1)


with io.open(p, "r", encoding="utf-8") as fh:
    t = fh.read()


def swap(text, old, new, count, label):
    n = text.count(old)
    if n != count:
        die(label + ": matched " + str(n) + " times, need " + str(count))
    return text.replace(old, new)


# Delete the private helper and re-express present() through the public accessor.
t = swap(
    t,
    """pub fn present() -> bool {
    intents_root().is_dir()
}

fn intents_root() -> PathBuf {
    faelight_core::paths::intents_dir()
}""",
    """pub fn present() -> bool {
    // INT-230: was `intents_root().is_dir()` against a PRIVATE helper returning
    // a bare PathBuf. That helper was a second definition of the same name as
    // the public accessor below, and the compiler resolved three call sites to
    // the wrong one. One owner: presence IS the accessor answering Some.
    intents_root().is_some()
}""",
    1,
    "fold present into the accessor",
)

with io.open(p, "w", encoding="utf-8") as fh:
    fh.write(t)

print("patched " + p)
print("")
print("Next: cargo build -p novashell --message-format=short")
