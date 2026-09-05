#!/usr/bin/env python3
"""
INT-230 G2 -- the search-root sites REFUSE instead of searching the wrong tree.

### ⚠️ FIRST, A DEFECT CLAUDE INTRODUCED TWO PASSES AGO

`commands/mod.rs:4294` is `search_root = tools_root().unwrap_or_default()`,
written during the rust_tools_dir pass. Read the surrounding code and it is
worse than a degraded answer:

    4279:  let mut search_root = std::path::PathBuf::from(core_root);
    4294:  search_root = tools_root().unwrap_or_default();       <- OVERWRITES IT
    4352:  cmd.arg(&search_root);                                 <- handed to fd

So `find pattern @rust` on a machine without 0-Core replaces a GOOD DEFAULT with
an EMPTY PATH and runs `fd` against "". That is not degradation, it is a wrong
answer produced confidently -- the exact class this intent keeps finding, and
this one is mine.

⭐ THE GENERAL LESSON, and it is worth stating once: `unwrap_or_default()` on a
PathBuf is never a safe wrap. An empty path is not "no path", it is the current
directory, and every filesystem call accepts it.

### THE FOUR SITES, AND WHY THEY ALL REFUSE

  4294 @rust      overwrites core_root and feeds fd
  4298 @intents   same shape, same function
  4490 --intent   ⚠️ assigns into an Option whose None ALREADY MEANS SOMETHING
                  ELSE: 4547 does `search_root.unwrap_or_else(current_dir)`, so
                  None means "the user did not pick a root". Assigning None for
                  "the root does not exist" makes `fsearch --intent` silently
                  search the working directory instead.
  7946 --intent   a String in a tuple, handed to rg at 7958. No absence path
                  exists there at all.

None of these can degrade quietly, because every one of them ends in a tool
being pointed at a directory. So all four REFUSE with a reason, which is what
INT-227's invariant asks for: an unavailable capability must not become a
successful-looking result.

📍 NOT TOUCHED: 4302/4306 (@scripts, @docs) build paths by hand and @scripts
points at ~/0-core/scripts, WHICH DOES NOT EXIST -- 12 sites in this shell
reference that deleted directory. That is INT-240's subject (hand-built state
paths), measured today at THIRTY sites in novashell alone, and recorded in
INT-230's body rather than folded in here.
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
addition = '''/// The 0-Core intents tree, when 0-Core is present.
///
/// ⚠️ CALLERS MUST REFUSE ON `None`, NOT FALL BACK. Every consumer of this path
/// hands it to a tool -- `fd`, `rg`, `read_dir` -- and
/// `unwrap_or_default()` on a PathBuf yields the EMPTY path, which is the
/// current directory to every filesystem call. Claude shipped exactly that bug
/// at `commands/mod.rs:4294` two passes ago: it overwrote a good `core_root`
/// default with an empty path and ran `fd` against it.
pub fn intents_root() -> Option<PathBuf> {
    let dir = faelight_core::paths::intents_dir();
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

'''
t = swap(t, p, anchor, addition + anchor, 1, "insert intents_root")
edits.append((p, t))

# --------------------------------------------------------------- commands/mod.rs
p = os.path.join(SRC, "commands/mod.rs")
t = read(p)

# 4294 -- Claude's own unwrap_or_default, in the most dangerous position.
t = swap(
    t,
    p,
    """                    "@rust" => {
                        search_root = crate::core_integration::tools_root().unwrap_or_default();
                        i += 1;
                    }""",
    """                    "@rust" => {
                        // INT-230: was unwrap_or_default(), which REPLACED the
                        // core_root default with an EMPTY path and handed that
                        // to fd. Refuses now.
                        match crate::core_integration::tools_root() {
                            Some(d) => search_root = d,
                            None => {
                                return CommandResult::Error(
                                    "  find: @rust needs 0-Core, which is not present"
                                        .to_string()
                                        .into(),
                                    1,
                                );
                            }
                        }
                        i += 1;
                    }""",
    1,
    "4294 at-rust refuse",
)

# 4298 -- same function, same shape.
t = swap(
    t,
    p,
    """                    "@intents" => {
                        search_root = faelight_core::paths::intents_dir();
                        i += 1;
                    }""",
    """                    "@intents" => {
                        match crate::core_integration::intents_root() {
                            Some(d) => search_root = d,
                            None => {
                                return CommandResult::Error(
                                    "  find: @intents needs 0-Core, which is not present"
                                        .to_string()
                                        .into(),
                                    1,
                                );
                            }
                        }
                        i += 1;
                    }""",
    1,
    "4298 at-intents refuse",
)

# 4490 -- the Option whose None already means "user did not choose".
t = swap(
    t,
    p,
    """                    "--intent" | "--intents" => {
                        let _home = std::env::var("HOME").unwrap_or_default();
                        search_root = Some(faelight_core::paths::intents_dir());
                        i += 1;
                    }""",
    """                    "--intent" | "--intents" => {
                        // INT-230: None here already means "the user did not pick
                        // a root" -- 4547 falls back to the cwd. Assigning None
                        // for "0-Core is absent" would make this silently search
                        // the working directory instead of refusing.
                        match crate::core_integration::intents_root() {
                            Some(d) => search_root = Some(d),
                            None => {
                                return CommandResult::Error(
                                    "  fsearch: --intent needs 0-Core, which is not present"
                                        .to_string()
                                        .into(),
                                    1,
                                );
                            }
                        }
                        i += 1;
                    }""",
    1,
    "4490 fsearch intent refuse",
)

# 7946 -- a String in a tuple handed to rg, with no absence path at all.
t = swap(
    t,
    p,
    """            "--intent" => (
                None,
                faelight_core::paths::intents_dir()
                    .to_string_lossy()
                    .to_string(),
            ),""",
    """            "--intent" => (
                None,
                match crate::core_integration::intents_root() {
                    Some(d) => d.to_string_lossy().to_string(),
                    None => {
                        return CommandResult::Error(
                            "  --intent needs 0-Core, which is not present"
                                .to_string()
                                .into(),
                            1,
                        );
                    }
                },
            ),""",
    1,
    "7946 rg intent refuse",
)
edits.append((p, t))

for path, text in edits:
    write(path, text)
    print("patched " + path)

print("")
print("Next: cargo build -p novashell --message-format=short")
