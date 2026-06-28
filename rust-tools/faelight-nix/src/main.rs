//! faelight-nix -- Nix package search TUI (INT-076)
//! Phase 0 scaffold: compiles, prints a banner, exits. No TUI yet.
//! "Find it, then let the config own it."

fn main() -> anyhow::Result<()> {
    println!("\u{1f332} faelight-nix 0.1.0 -- INT-076 scaffold");
    println!("   search backend: nix search nixpkgs --json");
    println!("   add target:     users/christian/home.nix (home.packages)");
    Ok(())
}
