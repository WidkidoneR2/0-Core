// faelight-fm v3.1 -- tree filesystem operations

use std::{fs, path::PathBuf};
use crate::types::{TreeNode, FlatNode, GitStatus};

const MAX_CHILDREN_SHOWN: usize = 6;

pub fn load_tree(path: &PathBuf, depth: usize, show_hidden: bool) -> Vec<TreeNode> {
    let git_map = crate::git::get_git_status(path);
    let mut nodes = vec![];
    let Ok(dir) = fs::read_dir(path) else { return nodes; };
    let mut entries: Vec<_> = dir.flatten().collect();
    entries.sort_by(|a, b| {
        let a_dir = a.metadata().map(|m| m.is_dir()).unwrap_or(false);
        let b_dir = b.metadata().map(|m| m.is_dir()).unwrap_or(false);
        b_dir.cmp(&a_dir).then(a.file_name().cmp(&b.file_name()))
    });
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if !show_hidden && name.starts_with('.') { continue; }
        let meta = entry.metadata().ok();
        let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let git_status = git_map.get(&name).cloned().unwrap_or(GitStatus::Clean);
        let is_symlink = entry.path().symlink_metadata()
            .map(|m| m.file_type().is_symlink()).unwrap_or(false);
        let symlink_target = if is_symlink {
            fs::read_link(entry.path()).ok()
                .map(|t| t.to_string_lossy().to_string())
        } else { None };
        nodes.push(TreeNode {
            name,
            path: entry.path(),
            is_dir,
            size,
            git_status,
            is_symlink,
            symlink_target,
            expanded: false,
            children: vec![],
            unlisted: 0,
            depth,
        });
    }
    nodes
}

pub fn expand_node(node: &mut TreeNode, show_hidden: bool) {
    if !node.is_dir { return; }
    let children = load_tree(&node.path, node.depth + 1, show_hidden);
    let total = children.len();
    let shown = MAX_CHILDREN_SHOWN.min(total);
    node.children = children.into_iter().take(shown).collect();
    node.unlisted = total.saturating_sub(shown);
    node.expanded = true;
}

pub fn collapse_node(node: &mut TreeNode) {
    node.children.clear();
    node.unlisted = 0;
    node.expanded = false;
}

/// Flatten tree into display list for rendering
pub fn flatten(nodes: &[TreeNode]) -> Vec<FlatNode> {
    let mut flat = vec![];
    for node in nodes {
        flat.push(FlatNode {
            node_path: node.path.clone(),
            name: node.name.clone(),
            is_dir: node.is_dir,
            size: node.size,
            git_status: node.git_status.clone(),
            is_symlink: node.is_symlink,
            symlink_target: node.symlink_target.clone(),
            depth: node.depth,
            unlisted: 0,
            is_unlisted_marker: false,
        });
        if node.expanded {
            let child_flat = flatten(&node.children);
            flat.extend(child_flat);
            if node.unlisted > 0 {
                flat.push(FlatNode {
                    node_path: node.path.clone(),
                    name: format!("{} unlisted", node.unlisted),
                    is_dir: false,
                    size: 0,
                    git_status: GitStatus::Clean,
                    is_symlink: false,
                    symlink_target: None,
                    depth: node.depth + 1,
                    unlisted: node.unlisted,
                    is_unlisted_marker: true,
                });
            }
        }
    }
    flat
}

/// Apply fuzzy filter -- returns indices of matching flat nodes
pub fn filter_flat(flat: &[FlatNode], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..flat.len()).collect();
    }
    let q = query.to_lowercase();
    flat.iter().enumerate()
        .filter(|(_, n)| {
            if n.is_unlisted_marker { return false; }
            fuzzy_match(&n.name.to_lowercase(), &q)
        })
        .map(|(i, _)| i)
        .collect()
}

fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    let mut h = haystack.chars();
    for c in needle.chars() {
        if !h.any(|x| x == c) { return false; }
    }
    true
}

pub fn format_size(bytes: u64) -> String {
    if bytes == 0 { return String::new(); }
    if bytes < 1024 { return format!("{}B", bytes); }
    if bytes < 1024 * 1024 { return format!("{}K", bytes / 1024); }
    if bytes < 1024 * 1024 * 1024 { return format!("{}M", bytes / (1024*1024)); }
    format!("{:.1}G", bytes as f64 / (1024.0*1024.0*1024.0))
}

pub fn load_preview(path: &PathBuf, is_dir: bool) -> String {
    if is_dir {
        let count = fs::read_dir(path).map(|d| d.count()).unwrap_or(0);
        let ctx = forest_context(path);
        let mut s = format!("📁 {} items\n{}", count, path.display());
        if !ctx.is_empty() { s.push_str(&format!("\n\n{}", ctx)); }
        return s;
    }
    if let Ok(content) = fs::read_to_string(path) {
        return content.lines().take(60).collect::<Vec<_>>().join("\n");
    }
    format!("Binary file")
}

fn forest_context(path: &PathBuf) -> String {
    let p = path.to_string_lossy();
    let mut ctx = vec![];
    if p.contains("/intents/")    { ctx.push("📋 Intent directory"); }
    if p.contains("/rust-tools/") { ctx.push("🦀 Rust tool source"); }
    if p.contains("/engine/")     { ctx.push("⚙️  Core engine"); }
    if p.contains("/pkgs/")       { ctx.push("📦 Nix derivations"); }
    if p.contains("/modules/")    { ctx.push("🔧 NixOS module"); }
    if p.contains("/config/")     { ctx.push("⚙️  Forest config"); }
    if p.contains("/nix/store/")  { ctx.push("❄️  Nix store"); }
    ctx.join("\n")
}
