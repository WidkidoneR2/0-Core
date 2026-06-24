//! nix domain -- inspect NixOS option resolution (INT-088, Nix Inspector)
//! Answers "why did this value win?": value, type, where declared, where defined.
//! Wraps `nixos-option --flake <repo>#<host>`, translates store-paths to repo-paths,
//! handles freeform-submodule leaves, flags redundant-vs-default. Built as a core
//! capability so the future friday-daemon can consume option resolution programmatically.
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;

struct OptionInfo {
    value: Option<String>,
    default: Option<String>,
    type_: Option<String>,
    description: Option<String>,
    declared_by: Vec<String>,
    defined_by: Vec<String>,
    submodule_note: Option<String>,
}

fn repo_path(p: &str) -> String {
    if let Some(idx) = p.find("-source/") {
        return p[idx + "-source/".len()..].to_string();
    }
    if let Some(rest) = p.strip_prefix("/nix/store/") {
        if let Some(slash) = rest.find('/') {
            return format!("<nixpkgs>/{}", &rest[slash + 1..]);
        }
    }
    p.to_string()
}

fn parse_nixos_option(out: &str) -> OptionInfo {
    let mut info = OptionInfo {
        value: None, default: None, type_: None, description: None,
        declared_by: vec![], defined_by: vec![], submodule_note: None,
    };
    let lines: Vec<&str> = out.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let key = lines[i].trim();
        match key {
            "Value:" => { if i+1 < lines.len() { info.value = Some(lines[i+1].trim().to_string()); } i+=1; }
            "Default:" => { if i+1 < lines.len() { info.default = Some(lines[i+1].trim().to_string()); } i+=1; }
            "Type:" => { if i+1 < lines.len() { info.type_ = Some(lines[i+1].trim().to_string()); } i+=1; }
            "Description:" => { if i+1 < lines.len() { info.description = Some(lines[i+1].trim().to_string()); } i+=1; }
            "Declared by:" => {
                let mut j=i+1;
                while j<lines.len() && lines[j].starts_with("  ") && !lines[j].trim().ends_with(':') {
                    let v=lines[j].trim(); if !v.is_empty() { info.declared_by.push(repo_path(v)); } j+=1;
                }
                i=j-1;
            }
            "Defined by:" => {
                let mut j=i+1;
                while j<lines.len() && lines[j].starts_with("  ") && !lines[j].trim().ends_with(':') {
                    let v=lines[j].trim(); if !v.is_empty() { info.defined_by.push(repo_path(v)); } j+=1;
                }
                i=j-1;
            }
            _ => {}
        }
        i+=1;
    }
    info
}

pub fn inspect(ctx: &AppContext, option: String, why: bool) -> CoreResult<()> {
    let host = std::process::Command::new("hostname")
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "framework16".to_string());
    let flake = format!("{}#{}", ctx.core_root, host);

    let run_opt = |opt: &str| -> (bool, String) {
        match std::process::Command::new("nixos-option")
            .args(["--flake", &flake, opt]).output() {
            Ok(o) => (o.status.success(), format!("{}{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr))),
            Err(e) => (false, format!("failed to run nixos-option: {}", e)),
        }
    };

    println!();
    println!("  {} {}", "🔍".bright_cyan(), option.bright_white().bold());
    println!("  {}", "─".repeat(50).bright_black());

    let (ok, raw) = run_opt(&option);
    let mut info = parse_nixos_option(&raw);

    if !ok && raw.contains("inside submodule option while traversing") {
        if let Some(dot) = option.rfind('.') {
            let parent = &option[..dot];
            let leaf = &option[dot+1..];
            let (pok, praw) = run_opt(parent);
            if pok {
                info = parse_nixos_option(&praw);
                info.submodule_note = Some(format!(
                    "'{}' is a freeform submodule key; showing parent '{}' (resolved set includes '{}')",
                    option, parent, leaf));
            } else {
                println!("  {} could not resolve this option.", "⚠️ ".yellow());
                println!();
                return Ok(());
            }
        }
    } else if !ok {
        println!("  {} {}", "⚠️ ".yellow(), "could not resolve option".yellow());
        let errline = raw.lines().rev().find(|l| l.contains("error") || l.contains("Couldn't"))
            .unwrap_or(raw.lines().last().unwrap_or(""));
        println!("  {}", errline.trim().bright_black());
        println!();
        return Ok(());
    }

    if let Some(note) = &info.submodule_note {
        println!("  {} {}", "ℹ".bright_blue(), note.bright_black());
        println!();
    }
    if let Some(v) = &info.value {
        println!("  {}  {}", "Value: ".bright_black(), v.bright_green().bold());
    }
    if let Some(t) = &info.type_ {
        println!("  {}  {}", "Type:  ".bright_black(), t.cyan());
    }
    if let (Some(v), Some(d)) = (&info.value, &info.default) {
        println!("  {}  {}", "Default:".bright_black(), d.bright_black());
        if v == d {
            println!("  {} {}", "⚠".yellow(),
                "value equals the default -- this definition is redundant".yellow());
        }
    }
    if !info.defined_by.is_empty() {
        println!();
        println!("  {} {}", "Defined by".bright_white().bold(),
            "(where the value is set -- what won):".bright_black());
        for d in &info.defined_by {
            println!("    {} {}", "→".bright_green(), d.green());
        }
    }
    // Phase 2: priority / merge analysis -- escalate to the slow nix eval only when
    // there are multiple definitions (or --why forces it). Single-def options skip this.
    if info.defined_by.len() > 1 || why {
        if let Some(w) = query_why(&ctx.core_root, &host, &option) {
            let merges = is_merge_type(&info.type_);
            println!();
            if merges {
                println!("  {} {} sources {} into this value:",
                    "🔀".bright_magenta(), w.defs.len().to_string().bright_white().bold(),
                    "merge".bright_magenta());
            } else {
                let word = if w.defs.len() == 1 { "definition" } else { "definitions" };
                println!("  {} {} {} -- {} won:",
                    "⚖".bright_yellow(), w.defs.len().to_string().bright_white().bold(),
                    word, prio_label(w.highest_prio).bright_yellow());
            }
            for (file, val) in &w.defs {
                let shown = if val.len() > 60 { format!("{}...", &val[..60]) } else { val.clone() };
                println!("    {} {} {} {}",
                    "·".bright_black(), file.green(),
                    "=".bright_black(), shown.cyan());
            }
        }
    }
    if !info.declared_by.is_empty() {
        println!();
        println!("  {} {}", "Declared by".bright_white().bold(),
            "(where the option is defined):".bright_black());
        for d in &info.declared_by {
            println!("    {} {}", "·".bright_black(), d.bright_black());
        }
    }
    if let Some(desc) = &info.description {
        println!();
        println!("  {}", desc.bright_black().italic());
    }
    println!();
    Ok(())
}

// ── Phase 2: the "why won" priority analysis (INT-088) ──
// Slow path: nix eval of definitionsWithLocations + highestPrio. Only run when an
// option has multiple definitions (escalation from the fast nixos-option count) or --why.

struct WhyInfo {
    highest_prio: i64,
    defs: Vec<(String, String)>, // (repo_path, value_repr)
}

/// NixOS priority numbers -> human label. Lower number wins.
fn prio_label(n: i64) -> String {
    match n {
        1500 => "option default (mkOptionDefault)".to_string(),
        1000 => "default (mkDefault)".to_string(),
        100 => "normal".to_string(),
        50 => "forced (mkForce)".to_string(),
        other => format!("override (mkOverride {})", other),
    }
}

/// Heuristic: does this option type MERGE (list/attrset/submodule) or OVERRIDE (scalar)?
fn is_merge_type(type_: &Option<String>) -> bool {
    match type_ {
        Some(t) => {
            let t = t.to_lowercase();
            t.contains("list of") || t.contains("attribute set")
                || t.contains("submodule") || t.contains("list or")
        }
        None => false,
    }
}

/// Run the slow nix eval to get per-definition files+values and the winning priority.
fn query_why(core_root: &str, host: &str, option: &str) -> Option<WhyInfo> {
    // highestPrio
    let prio_expr = format!(
        "((builtins.getFlake \"{}\").nixosConfigurations.{}).options.{}.highestPrio",
        core_root, host, option);
    let prio_out = std::process::Command::new("nix")
        .args(["eval", "--impure", "--expr", &prio_expr])
        .output().ok()?;
    let prio_str = String::from_utf8_lossy(&prio_out.stdout);
    let highest_prio: i64 = prio_str.trim().parse().unwrap_or(100);

    // definitionsWithLocations as a list of "file\tvalue" via toJSON for safe parsing
    let defs_expr = format!(
        "builtins.toJSON (map (d: {{ file = d.file; value = (let v = d.value; in if builtins.isAttrs v then \"<set>\" else if builtins.isList v then \"<list>\" else if builtins.isBool v then (if v then \"true\" else \"false\") else builtins.toString v); }}) (((builtins.getFlake \"{}\").nixosConfigurations.{}).options.{}.definitionsWithLocations))",
        core_root, host, option);
    let defs_out = std::process::Command::new("nix")
        .args(["eval", "--impure", "--raw", "--expr", &defs_expr])
        .output().ok()?;
    let raw = String::from_utf8_lossy(&defs_out.stdout).to_string();

    // raw is a JSON string (because toJSON). Minimal parse: it's [{"file":"..","value":".."},...]
    let mut defs = Vec::new();
    // crude but safe: split on "},{" boundaries
    for chunk in raw.trim_start_matches('[').trim_end_matches(']').split("},{") {
        let file = extract_json_str(chunk, "file");
        let value = extract_json_str(chunk, "value");
        if let Some(f) = file {
            defs.push((repo_path(&f), value.unwrap_or_default()));
        }
    }
    Some(WhyInfo { highest_prio, defs })
}

/// Extract a string field value from a JSON-ish chunk (no external json dep).
fn extract_json_str(chunk: &str, key: &str) -> Option<String> {
    let pat = format!("\"{}\":\"", key);
    let start = chunk.find(&pat)? + pat.len();
    let rest = &chunk[start..];
    // find the closing unescaped quote
    let mut out = String::new();
    let mut chars = rest.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' { if let Some(n) = chars.next() { out.push(n); } continue; }
        if c == '"' { break; }
        out.push(c);
    }
    Some(out)
}
