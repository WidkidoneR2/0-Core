//! config_edit.rs -- the declarative-add engine for INT-076 Phase 2.
//! Pure logic: given home.nix content + a package attr, produce the modified
//! content (package inserted into home.packages) + a unified-ish diff, OR an
//! error (already present / anchor not found). Writing to disk is the caller's
//! job and is always gated behind a reviewable diff + explicit confirmation.
//! "Find it, then let the config own it." Never imperative, never silent.

use anyhow::{bail, Result};

pub struct AddPlan {
    pub new_content: String,
    pub diff: String, // human-readable preview (old/new context)
    pub pkg: String,
}

/// The anchor line that opens the home.packages list.
const ANCHOR: &str = "home.packages = with pkgs; [";

/// Insert `pkg` as the first entry of home.packages.
/// Returns an AddPlan (content + diff) or an error.
pub fn plan_add(content: &str, pkg: &str) -> Result<AddPlan> {
    let pkg = pkg.trim();
    if pkg.is_empty() {
        bail!("no package selected");
    }
    // Only single-attr leaf packages are valid Nix identifiers here.
    // Dotted attrs (e.g. nerd-fonts.hack) ARE valid in a with-pkgs list,
    // so we allow letters, digits, -, _, and .
    if !pkg
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        bail!("'{pkg}' is not a plain package attr (won't insert)");
    }

    let lines: Vec<&str> = content.lines().collect();

    // Find the anchor line (the `home.packages = with pkgs; [`).
    let anchor_idx = lines.iter().position(|l| l.contains(ANCHOR));
    let anchor_idx = match anchor_idx {
        Some(i) => i,
        None => bail!("anchor `{ANCHOR}` not found in config"),
    };

    // Determine the list's extent (anchor+1 .. closing `];`).
    let mut close_idx = None;
    for (off, l) in lines[anchor_idx + 1..].iter().enumerate() {
        if l.trim() == "];" {
            close_idx = Some(anchor_idx + 1 + off);
            break;
        }
    }
    let close_idx = match close_idx {
        Some(i) => i,
        None => bail!("could not find closing `];` for home.packages"),
    };

    // Duplicate check: is pkg already a bare entry in the list?
    for l in &lines[anchor_idx + 1..close_idx] {
        let entry = l.trim().split_whitespace().next().unwrap_or("");
        if entry == pkg {
            bail!("'{pkg}' is already in home.packages");
        }
    }

    // Match the indentation of an existing entry (4 spaces here), fall back to 4.
    let indent = lines
        .get(anchor_idx + 1)
        .map(|l| l.len() - l.trim_start().len())
        .filter(|n| *n > 0)
        .unwrap_or(4);
    let pad = " ".repeat(indent);
    let new_line = format!("{pad}{pkg}");

    // Build new content: insert new_line right after the anchor.
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + 1);
    for (i, l) in lines.iter().enumerate() {
        out.push((*l).to_string());
        if i == anchor_idx {
            out.push(new_line.clone());
        }
    }
    // Preserve trailing newline if the original had one.
    let mut new_content = out.join("\n");
    if content.ends_with('\n') {
        new_content.push('\n');
    }

    // Build a small context diff for review.
    let ctx_start = anchor_idx;
    let ctx_end = (anchor_idx + 3).min(lines.len());
    let mut diff = String::new();
    diff.push_str(&format!(
        "  users/christian/home.nix  (insert into home.packages)\n\n"
    ));
    for l in &lines[ctx_start..ctx_end] {
        diff.push_str(&format!("    {l}\n"));
        if l.contains(ANCHOR) {
            diff.push_str(&format!("  + {new_line}\n"));
        }
    }

    Ok(AddPlan {
        new_content,
        diff,
        pkg: pkg.to_string(),
    })
}
