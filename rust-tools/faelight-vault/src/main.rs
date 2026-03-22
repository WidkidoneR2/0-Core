// faelight-vault v1.0.0 — Forest-Native Credential Manager
// INT-132 — "faelight-gen with memory"
// "Trust, but verify. Store, but protect. Generate, but remember."

mod crypto;
mod store;
mod health;
mod display;

use colored::*;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    match cmd {
        "--health" | "health" => { println!("faelight-vault v1.0.0 — healthy"); process::exit(0); }
        "init"     => cmd_init(),
        "add"      => cmd_add(&args),
        "get"      => cmd_get(&args),
        "list"     => cmd_list(&args),
        "rotate"   => cmd_rotate(&args),
        "audit"    => cmd_audit(),
        "generate" => cmd_generate(&args),
        "unlock"   => cmd_unlock(&args),
        "lock"     => cmd_lock(),
        "export"   => cmd_export(&args),
        "import"   => cmd_import(&args),
        "remove"   => cmd_remove(&args),
        _          => cmd_help(),
    }
}

fn require_name(args: &[String], usage: &str) -> String {
    match args.get(2) {
        Some(n) => n.clone(),
        None => { eprintln!("  {} {}", "✗".bright_red(), usage); process::exit(1); }
    }
}

fn get_type(args: &[String]) -> &str {
    args.windows(2).find(|w| w[0] == "--type").map(|w| w[1].as_str()).unwrap_or("password")
}

fn cmd_init() {
    display::print_banner();
    let vault_path = store::vault_path();
    if vault_path.exists() {
        println!("  {} Vault already exists at {}", "⚠".yellow(), vault_path.display().to_string().dimmed());
        return;
    }
    let master = display::prompt_master("Create master password");
    let confirm = display::prompt_master("Confirm master password");
    if master != confirm { eprintln!("  {} Passwords do not match", "✗".bright_red()); process::exit(1); }
    match store::init_vault(&master) {
        Ok(_) => {
            println!();
            println!("  {} Vault initialized", "✅".normal());
            println!("  {} Encrypted with Argon2id", "🔒".normal());
            println!("  {} Run: faelight-vault add github", "→".bright_cyan());
        }
        Err(e) => { eprintln!("  {} {}", "✗".bright_red(), e); process::exit(1); }
    }
}

fn cmd_add(args: &[String]) {
    let name = require_name(args, "Usage: faelight-vault add <name>");
    let cred_type = get_type(args);
    display::print_banner();
    let master = display::prompt_master("Master password");
    println!("  {} Generate automatically? [Y/n]: ", "?".bright_cyan());
    use std::io::BufRead;
    let answer = std::io::stdin().lock().lines().next()
        .and_then(|l| l.ok()).unwrap_or_default().trim().to_lowercase();
    let secret = if answer == "n" {
        rpassword::prompt_password("  Enter secret: ").unwrap_or_default()
    } else { generate_secret(cred_type) };
    if secret.is_empty() { eprintln!("  {} No secret", "✗".bright_red()); process::exit(1); }
    match store::add_credential(&master, &name, cred_type, &secret) {
        Ok(_) => { let score = health::score(&secret, 0); display::print_added(&name, cred_type, score); }
        Err(e) => { eprintln!("  {} {}", "✗".bright_red(), e); process::exit(1); }
    }
}

fn cmd_get(args: &[String]) {
    let name = require_name(args, "Usage: faelight-vault get <name>");
    let master = display::prompt_master("Master password");
    match store::get_credential(&master, &name) {
        Ok(Some(e)) => {
            println!();
            println!("  {} {}", "🔓".normal(), name.bright_white().bold());
            println!("  {}  {}", "Secret:".dimmed(), e.secret.bright_green());
            println!("  {}    {}", "Type:".dimmed(), e.cred_type.dimmed());
            display::print_health_bar(health::score(&e.secret, e.age_days));
        }
        Ok(None) => { eprintln!("  {} Not found: {}", "✗".bright_red(), name); process::exit(1); }
        Err(e) => { eprintln!("  {} {}", "✗".bright_red(), e); process::exit(1); }
    }
}

fn cmd_list(args: &[String]) {
    let filter = args.get(2).map(|s| s.as_str());
    let master = display::prompt_master("Master password");
    match store::list_credentials(&master) {
        Ok(entries) => { display::print_banner(); display::print_list(&entries, filter); }
        Err(e) => { eprintln!("  {} {}", "✗".bright_red(), e); process::exit(1); }
    }
}

fn cmd_audit() {
    let master = display::prompt_master("Master password");
    match store::list_credentials(&master) {
        Ok(entries) => { display::print_banner(); display::print_audit(&entries); }
        Err(e) => { eprintln!("  {} {}", "✗".bright_red(), e); process::exit(1); }
    }
}

fn cmd_rotate(args: &[String]) {
    let name = require_name(args, "Usage: faelight-vault rotate <name>");
    let master = display::prompt_master("Master password");
    match store::get_credential(&master, &name) {
        Ok(Some(e)) => {
            let new_secret = generate_secret(&e.cred_type);
            match store::update_credential(&master, &name, &new_secret) {
                Ok(_) => { println!("  {} {} rotated", "🔄".normal(), name.bright_white()); display::print_health_bar(health::score(&new_secret, 0)); }
                Err(e) => eprintln!("  {} {}", "✗".bright_red(), e),
            }
        }
        Ok(None) => eprintln!("  {} Not found: {}", "✗".bright_red(), name),
        Err(e) => eprintln!("  {} {}", "✗".bright_red(), e),
    }
}

fn cmd_generate(args: &[String]) {
    let name = require_name(args, "Usage: faelight-vault generate <name>");
    let cred_type = get_type(args);
    let master = display::prompt_master("Master password");
    let secret = generate_secret(cred_type);
    match store::add_credential(&master, &name, cred_type, &secret) {
        Ok(_) => display::print_added(&name, cred_type, health::score(&secret, 0)),
        Err(e) => eprintln!("  {} {}", "✗".bright_red(), e),
    }
}

fn cmd_unlock(args: &[String]) {
    let ttl = args.windows(2).find(|w| w[0] == "--ttl").map(|w| w[1].as_str()).unwrap_or("15m");
    let master = display::prompt_master("Master password");
    match store::validate_master(&master) {
        Ok(true) => { store::write_session_cache(&master, ttl); println!("  {} Vault unlocked for {}", "🔓".normal(), ttl.bright_green()); }
        Ok(false) => { eprintln!("  {} Invalid master password", "✗".bright_red()); process::exit(1); }
        Err(e) => eprintln!("  {} {}", "✗".bright_red(), e),
    }
}

fn cmd_lock() { store::clear_session_cache(); println!("  {} Vault locked", "🔒".normal()); }

fn cmd_remove(args: &[String]) {
    let name = require_name(args, "Usage: faelight-vault remove <name>");
    let master = display::prompt_master("Master password");
    match store::remove_credential(&master, &name) {
        Ok(_) => println!("  {} {} removed", "✅".normal(), name.bright_white()),
        Err(e) => eprintln!("  {} {}", "✗".bright_red(), e),
    }
}

fn cmd_export(args: &[String]) {
    let path = args.get(2).map(|s| s.as_str()).unwrap_or("vault-backup.db");
    let master = display::prompt_master("Master password");
    match store::export_vault(&master, path) {
        Ok(_) => println!("  {} Exported to {}", "✅".normal(), path.bright_green()),
        Err(e) => eprintln!("  {} {}", "✗".bright_red(), e),
    }
}

fn cmd_import(args: &[String]) {
    let path = require_name(args, "Usage: faelight-vault import <file>");
    let master = display::prompt_master("Master password");
    match store::import_vault(&master, &path) {
        Ok(n) => println!("  {} Imported {} credentials", "✅".normal(), n),
        Err(e) => eprintln!("  {} {}", "✗".bright_red(), e),
    }
}

fn cmd_help() {
    display::print_banner();
    println!();
    let cmds = [
        ("init",           "Initialize vault with master password"),
        ("add <name>",     "Add a credential"),
        ("get <name>",     "Retrieve a credential"),
        ("list",           "List all credentials with health scores"),
        ("rotate <name>",  "Regenerate and update a credential"),
        ("generate <name>","Generate and store in one step"),
        ("audit",          "Find weak or old credentials"),
        ("unlock",         "Cache master password (--ttl 15m)"),
        ("lock",           "Clear session cache"),
        ("remove <name>",  "Remove a credential"),
        ("export [file]",  "Export encrypted backup"),
        ("import <file>",  "Import from backup"),
    ];
    for (c, d) in &cmds { println!("  {:26} {}", c.bright_cyan(), d.dimmed()); }
    println!();
}

fn generate_secret(cred_type: &str) -> String {
    let args: &[&str] = match cred_type {
        "passphrase" => &["passphrase", "--words", "5"],
        "apikey"     => &["apikey"],
        "token"      => &["base64", "--length", "32"],
        "pin"        => &["pin", "--digits", "6"],
        _            => &["random", "--length", "32"],
    };
    std::process::Command::new("faelight-gen").args(args).output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}
