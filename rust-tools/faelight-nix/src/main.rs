//! faelight-nix -- Nix package search TUI (INT-076)
//! Phase 1 (data layer demo): run a search, print results. TUI comes next.
//! "Find it, then let the config own it."

mod search;

fn main() -> anyhow::Result<()> {
    let query = std::env::args().nth(1).unwrap_or_else(|| "ripgrep".to_string());
    println!("\u{1f332} faelight-nix -- searching nixpkgs for: {query}\n");

    let results = search::search(&query)?;
    if results.is_empty() {
        println!("  (no matches)");
        return Ok(());
    }

    for p in &results {
        let name_note = if p.pname != p.attr && !p.pname.is_empty() {
            format!("  ({})", p.pname)
        } else {
            String::new()
        };
        println!("  {:<28} {:<14} {}{}", p.attr, p.version, p.description, name_note);
    }
    println!("\n  {} result(s)", results.len());
    Ok(())
}
