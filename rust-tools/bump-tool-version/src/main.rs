use anyhow::{anyhow, Context, Result};
use clap::Parser;
use colored::*;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use faelight_core::paths;

#[derive(Parser)]
#[command(name = "bump-tool-version")]
#[command(version = "2.0.0")]
#[command(about = "Bump individual tool versions with auto-increment support", long_about = None)]
struct Cli {
    /// Tool name (e.g., faelight-link, faelight-fm)
    tool: String,

    /// New version (e.g., 1.2.0) or omit to use increment flags
    new_version: Option<String>,

    /// Increment minor version (X.Y.0 -> X.Y+1.0)
    #[arg(long, conflicts_with = "new_version")]
    minor: bool,

    /// Increment patch version (X.Y.Z -> X.Y.Z+1)
    #[arg(long, conflicts_with = "new_version")]
    patch: bool,

    /// Increment major version (X.Y.Z -> X+1.0.0)
    #[arg(long, conflicts_with = "new_version")]
    major: bool,

    /// Skip confirmation prompts
    #[arg(long)]
    yes: bool,
}

#[derive(Debug)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid version format: {}", s));
        }
        
        Ok(Version {
            major: parts[0].parse()?,
            minor: parts[1].parse()?,
            patch: parts[2].parse()?,
        })
    }

    fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    fn bump_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }

    fn bump_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    fn bump_patch(&mut self) {
        self.patch += 1;
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Find tool directory
    let tool_dir = paths::core_dir().join("rust-tools").join(&cli.tool);
    if !tool_dir.exists() {
        return Err(anyhow!("Tool '{}' not found in rust-tools/", cli.tool));
    }

    let cargo_toml = tool_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(anyhow!("Cargo.toml not found for tool '{}'", cli.tool));
    }

    // Read current version
    let current_version = read_tool_version(&cargo_toml)?;
    let mut version = Version::parse(&current_version)?;

    // Calculate new version
    let new_version = if let Some(v) = cli.new_version {
        Version::parse(&v)?
    } else if cli.major {
        version.bump_major();
        version
    } else if cli.minor {
        version.bump_minor();
        version
    } else if cli.patch {
        version.bump_patch();
        version
    } else {
        return Err(anyhow!("Must specify new version or use --major, --minor, or --patch"));
    };

    let increment_type = if cli.major {
        "Major version (breaking changes)"
    } else if cli.minor {
        "Minor version (new features, backwards compatible)"
    } else if cli.patch {
        "Patch version (bug fixes)"
    } else {
        "Manual version"
    };

    // Show pre-flight
    show_preflight(&cli.tool, &current_version, &new_version.to_string(), increment_type)?;

    if !cli.yes {
        println!("\n{}", "Ready to proceed? (y/n): ".cyan());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "❌ Cancelled".yellow());
            return Ok(());
        }
    }

    // Update files
    println!("\n{}", "📝 Updating files...".cyan().bold());
    update_tool_cargo(&cargo_toml, &new_version.to_string())?;
    println!("  ✅ Updated {}/Cargo.toml", cli.tool);

    if let Some(readme) = find_tool_readme(&tool_dir) {
        update_tool_readme(&readme, &current_version, &new_version.to_string())?;
        println!("  ✅ Updated {}/README.md", cli.tool);
    }

    // Git operations
    println!("\n{}", "🔧 Git operations...".cyan().bold());
    
    let tag = format!("{}-v{}", cli.tool, new_version.to_string());
    let commit_msg = format!(
        "feat({}): bump to v{}\n\nIncrement type: {}",
        cli.tool, new_version.to_string(), increment_type
    );

    git_commit_and_tag(&cli.tool, &commit_msg, &tag)?;
    
    println!("\n{}", "━".repeat(60));
    println!("{}", format!("🎉 {} bumped: {} → {}", cli.tool, current_version, new_version.to_string()).green().bold());
    println!("{}", format!("📦 Tag created: {}", tag).cyan());
    println!("{}", "━".repeat(60));

    Ok(())
}

fn read_tool_version(cargo_toml: &PathBuf) -> Result<String> {
    let content = fs::read_to_string(cargo_toml)?;
    
    // Check if using workspace version
    if content.contains("version.workspace = true") {
        // Read from workspace Cargo.toml
        let workspace_toml = paths::cargo_toml();
        let workspace_content = fs::read_to_string(&workspace_toml)?;
        let re = Regex::new(r#"\[workspace\.package\][\s\S]*?version\s*=\s*\"([^\"]+)\""#)?;
        
        if let Some(caps) = re.captures(&workspace_content) {
            return Ok(caps[1].to_string());
        }
        return Err(anyhow!("Could not find workspace version"));
    }
    
    // Regular version field
    let re = Regex::new(r#"version\s*=\s*"([^"]+)""#)?;
    if let Some(caps) = re.captures(&content) {
        Ok(caps[1].to_string())
    } else {
        Err(anyhow!("Could not find version in Cargo.toml"))
    }
}

fn show_preflight(tool: &str, current: &str, new: &str, increment_type: &str) -> Result<()> {
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║           🌲 TOOL VERSION BUMP - PRE-FLIGHT 🌲              ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());
    println!();
    println!("{}", "📦 Tool Information:".bold());
    println!("  Tool:     {}", tool.green());
    println!("  Current:  v{}", current.yellow());
    println!("  Target:   v{}", new.green().bold());
    println!("  Type:     {}", increment_type.cyan());
    println!();
    println!("{}", "📝 Files That Will Be Modified:".bold());
    println!("  • {}/Cargo.toml", tool);
    println!("  • {}/README.md (if exists)", tool);
    println!();
    println!("{}", "🔧 Operations That Will Execute:".bold());
    println!("  1. Update Cargo.toml version");
    println!("  2. Update README.md version badge (if exists)");
    println!("  3. Git commit with bump message");
    println!("  4. Create git tag: {}-v{}", tool, new);
    println!("{}", "━".repeat(60));
    
    Ok(())
}

fn update_tool_cargo(cargo_toml: &PathBuf, new_version: &str) -> Result<()> {
    let content = fs::read_to_string(cargo_toml)?;
    
    // If using workspace version, convert to explicit version
    let new_content = if content.contains("version.workspace = true") {
        content.replace(
            "version.workspace = true",
            &format!(r#"version = "{}""#, new_version)
        )
    } else {
        let re = Regex::new(r#"(version\s*=\s*)"[^"]+""#)?;
        re.replace(&content, format!(r#"${{1}}"{}""#, new_version).as_str()).to_string()
    };
    
    fs::write(cargo_toml, new_content)?;
    Ok(())
}

fn find_tool_readme(tool_dir: &PathBuf) -> Option<PathBuf> {
    let readme = tool_dir.join("README.md");
    if readme.exists() {
        Some(readme)
    } else {
        None
    }
}

fn update_tool_readme(readme: &PathBuf, old_version: &str, new_version: &str) -> Result<()> {
    let content = fs::read_to_string(readme)?;
    
    // Update header version (e.g., "# tool v1.0.0")
    let re1 = Regex::new(&format!(r"(#.*?v){}", regex::escape(old_version)))?;
    let content = re1.replace_all(&content, format!("${{1}}{}", new_version));
    
    // Update badge version if present
    let re2 = Regex::new(&format!(r"(version-){}(-)", regex::escape(old_version)))?;
    let content = re2.replace_all(&content, format!("${{1}}{}${{2}}", new_version));
    
    fs::write(readme, content.as_ref())?;
    Ok(())
}

fn git_commit_and_tag(tool: &str, message: &str, tag: &str) -> Result<()> {
    // Stage changes
    Command::new("git")
        .args(["add", &format!("rust-tools/{}/", tool)])
        .status()
        .context("Failed to stage changes")?;

    // Commit
    Command::new("git")
        .args(["commit", "-m", message])
        .status()
        .context("Failed to commit")?;

    // Create tag
    Command::new("git")
        .args(["tag", tag])
        .status()
        .context("Failed to create tag")?;

    println!("  ✅ Committed and tagged");

    Ok(())
}
