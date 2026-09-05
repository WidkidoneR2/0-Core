#!/usr/bin/env python3
"""
INT-230 -- the last four unwrap_or_default() calls, and one of them is serious.

### ⚠️⚠️ exec.rs:302 BLOCKS EVERY rm -rf ON A FORESTLESS MACHINE

The catastrophic-rm guard builds a protected-path list and tests:

    if expanded.contains(protected)

`core_src` was `tools_root().unwrap_or_default().to_string_lossy()`, which on a
machine without 0-Core is the EMPTY STRING -- and **every string contains the
empty string**. So the guard matches EVERY `rm -rf` and refuses it with
`Blocked: rm -rf on forest source ''`.

⭐ That is worse than the `fd ""` bug fixed in the last commit. That one searched
the wrong directory; this one takes away a user's ability to delete anything
recursively, on exactly the machine this intent exists to support, while naming
an empty string as the thing it is protecting. Introduced by Claude two passes
ago, wrapping a Some/None in the laziest way available.

⭐ THE FIX IS A FILTER, NOT A WRAP. A protected list should contain paths that
exist. An absent one is not a path to compare against -- it is an entry that
does not belong in the list.

### THE OTHER THREE

  13937 geiger  -> fed straight to `cargo --manifest-path`. An empty root gives
  13956 check      `/faelight-shell/Cargo.toml`, an absolute path into the
                   filesystem root. ⚠️ AND THESE ARE THE SAME PATTERN
                   `tool_manifest()` ALREADY OWNS -- four of the six manifest
                   builds were migrated to it in the rust_tools_dir pass and
                   these two were wrapped instead. Inconsistent inside one pass;
                   they use the accessor now, which also gives them the
                   existence check for free.

  3778 @rust    -> substituted into a user-facing path expansion. An empty
                   string silently expands `@rust` to nothing; returning the
                   shortcut unexpanded at least shows what happened.

📍 `13936` also defaults `tool` to "faelight-shell" -- the pre-rename crate name.
Still correct today because the PACKAGE is still faelight-shell, and it breaks
the day the crate is renamed. Noted, not changed.
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

# ------------------------------------------------------------------- exec.rs
p = os.path.join(SRC, "exec.rs")
t = read(p)
t = swap(
    t,
    p,
    """            let core_src = crate::core_integration::tools_root()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let core_engine = format!("{}/engine", core_root);
            let core_intents = faelight_core::paths::intents_dir()
                .to_string_lossy()
                .to_string();
            for protected in &[
                core_src.as_str(),
                core_engine.as_str(),
                core_intents.as_str(),
            ] {
                if expanded.contains(protected) {""",
    """            // ⚠️ INT-230: `core_src` was unwrap_or_default(), so on a machine
            // without 0-Core it was the EMPTY STRING -- and every string
            // contains the empty string, so this guard BLOCKED EVERY rm -rf and
            // named '' as the thing it was protecting. A protected list must
            // hold paths that EXIST; an absent one is not an entry.
            let core_src = crate::core_integration::tools_root()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let core_engine = format!("{}/engine", core_root);
            let core_intents = faelight_core::paths::intents_dir()
                .to_string_lossy()
                .to_string();
            for protected in [
                core_src.as_str(),
                core_engine.as_str(),
                core_intents.as_str(),
            ]
            .iter()
            .filter(|p| !p.is_empty())
            {
                if expanded.contains(protected) {""",
    1,
    "exec.rs protected list filter",
)
edits.append((p, t))

# --------------------------------------------------------------- commands/mod.rs
p = os.path.join(SRC, "commands/mod.rs")
t = read(p)

# geiger -- use the accessor that already owns this pattern.
t = swap(
    t,
    p,
    """            let tool = args.get(1).copied().unwrap_or("faelight-shell");
            let manifest = crate::core_integration::tools_root()
                .unwrap_or_default()
                .join(tool)
                .join("Cargo.toml")
                .to_string_lossy()
                .to_string();
            println!("  {} scanning unsafe code in {}", "☢".normal(), tool);""",
    """            let tool = args.get(1).copied().unwrap_or("faelight-shell");
            // INT-230: was unwrap_or_default(), which fed cargo an absolute path
            // into the filesystem root. tool_manifest already owns this pattern
            // and carries the existence check.
            let manifest = match crate::core_integration::tool_manifest(tool) {
                Some(m) => m.to_string_lossy().to_string(),
                None => {
                    return CommandResult::Error(
                        format!("  dev geiger: no Cargo.toml found for '{}'", tool).into(),
                        1,
                    );
                }
            };
            println!("  {} scanning unsafe code in {}", "☢".normal(), tool);""",
    1,
    "geiger manifest",
)

# check/bacon -- same accessor.
t = swap(
    t,
    p,
    """                let manifest = crate::core_integration::tools_root()
                    .unwrap_or_default()
                    .join(tool)
                    .join("Cargo.toml")
                    .to_string_lossy()
                    .to_string();
                println!("  {} starting bacon for {}", "🥓".normal(), tool);""",
    """                let manifest = match crate::core_integration::tool_manifest(tool) {
                    Some(m) => m.to_string_lossy().to_string(),
                    None => {
                        return CommandResult::Error(
                            format!("  dev check: no Cargo.toml found for '{}'", tool).into(),
                            1,
                        );
                    }
                };
                println!("  {} starting bacon for {}", "🥓".normal(), tool);""",
    1,
    "bacon manifest",
)

# @rust in expand_path -- show the shortcut rather than expanding to nothing.
t = swap(
    t,
    p,
    """                    "@rust" => crate::core_integration::tools_root()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),""",
    """                    // INT-230: an absent 0-Core expanded this to the EMPTY
                    // string, silently. Returning the shortcut unexpanded at
                    // least shows what happened.
                    "@rust" => crate::core_integration::tools_root()
                        .map(|d| d.to_string_lossy().to_string())
                        .unwrap_or_else(|| "@rust".to_string()),""",
    1,
    "expand_path at-rust",
)
edits.append((p, t))

for path, text in edits:
    write(path, text)
    print("patched " + path)

print("")
print("Next: cargo build -p novashell --message-format=short")
