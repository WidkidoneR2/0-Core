use std::path::Path;
use std::process::Command;

/// Check for Git repository updates
pub fn check_git_updates() -> Vec<String> {
    println!("   Checking git repositories...");
    Vec::new()
}

/// Update Git repositories in ~/repos
pub fn update_git_repos() -> anyhow::Result<()> {
    let repos_dir = std::env::var("HOME").unwrap() + "/repos";

    if !Path::new(&repos_dir).exists() {
        println!("   ⚠️  ~/repos directory not found");
        return Ok(());
    }

    println!("   Updating repositories in ~/repos...");

    // Find all git repos
    let output = Command::new("find")
        .arg(&repos_dir)
        .arg("-maxdepth")
        .arg("2")
        .arg("-name")
        .arg(".git")
        .arg("-type")
        .arg("d")
        .output()?;

    // Fix borrow issue by converting to String immediately
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();
    let repos: Vec<_> = output_str
        .lines()
        .map(|line| line.trim_end_matches("/.git"))
        .collect();

    if repos.is_empty() {
        println!("   ℹ️  No git repositories found");
        return Ok(());
    }

    for repo in repos {
        if let Some(name) = Path::new(repo).file_name() {
            print!("     Pulling {}...", name.to_string_lossy());

            let status = Command::new("git")
                .arg("-C")
                .arg(repo)
                .arg("pull")
                .arg("--ff-only")
                .output();

            match status {
                Ok(out) if out.status.success() => println!(" ✓"),
                _ => println!(" ✗"),
            }
        }
    }

    println!("   ✅  Git repositories updated");
    Ok(())
}
