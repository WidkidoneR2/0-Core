use anyhow::Result;
use colored::Colorize;
use regex::Regex;

pub fn validate_commit_msg(msg: &str) -> Result<bool> {
    println!("{}", "💬 Validating commit message...".cyan());

    // Conventional commit format: type(scope): subject
    let conventional_regex = Regex::new(
        r"^(feat|fix|docs|style|refactor|perf|test|chore|build|ci|revert)(\(.+\))?: .{1,}"
    ).unwrap();

    let lines: Vec<&str> = msg.lines().collect();
    
    if lines.is_empty() {
        println!("{}", "❌ Empty commit message!".red().bold());
        return Ok(false);
    }

    let subject = lines[0];

    // Check length (recommended: ≤72 chars)
    if subject.len() > 72 {
        println!("{}", "⚠️  Subject line too long".yellow());
        println!("   Length: {} chars (recommended: ≤72)", subject.len());
        println!();
    }

    // Check if it follows conventional commits
    if !conventional_regex.is_match(subject) {
        println!("{}", "⚠️  Not following conventional commit format".yellow());
        println!();
        println!("Expected format:");
        println!("  {}({}): {}", 
            "type".green(), 
            "scope".cyan(), 
            "description".dimmed()
        );
        println!();
        println!("Valid types: {}", 
            "feat, fix, docs, style, refactor, perf, test, chore, build, ci, revert".dimmed()
        );
        println!();
        println!("Example:");
        println!("  {}", "feat(hooks): Add commit message validation".green());
        println!();
        println!("{}", "💡 This is a recommendation, not enforced.".yellow());
        println!();
    } else {
        println!("{}", "✅ Follows conventional commit format".green());
    }

    // Check for minimum length
    if subject.len() < 10 {
        println!("{}", "⚠️  Subject line very short".yellow());
        println!("   Consider adding more detail");
        println!();
    }

    // Always pass - these are warnings, not errors
    Ok(true)
}
