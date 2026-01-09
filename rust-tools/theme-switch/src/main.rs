use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// ANSI colors
const GREEN: &str = "\x1b[0;32m";
const NC: &str = "\x1b[0m";

fn main() {
    let args: Vec<String> = env::args().collect();
    let home = env::var("HOME").expect("HOME not set");
    let themes_dir = PathBuf::from(&home).join("0-core/themes");
    let current_file = themes_dir.join("current.txt");
    
    let action = args.get(1).map(|s| s.as_str()).unwrap_or("toggle");
    
    match action {
        "dark" => apply_theme(&home, &themes_dir, &current_file, "dark"),
        "light" => apply_theme(&home, &themes_dir, &current_file, "light"),
        "toggle" => {
            let current = fs::read_to_string(&current_file)
                .unwrap_or_else(|_| "dark".to_string())
                .trim()
                .to_string();
            let new_theme = if current == "dark" { "light" } else { "dark" };
            apply_theme(&home, &themes_dir, &current_file, new_theme);
        }
        "status" => {
            let current = fs::read_to_string(&current_file)
                .unwrap_or_else(|_| "dark".to_string());
            println!("{}", current.trim());
        }
        _ => {
            println!("Usage: theme-switch [dark|light|toggle|status]");
        }
    }
}

fn get_color(themes_dir: &PathBuf, theme: &str, key: &str) -> String {
    let json_path = themes_dir.join(format!("faelight-{}/theme.json", theme));
    
    if let Ok(content) = fs::read_to_string(&json_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(color) = json.get("colors").and_then(|c| c.get(key)).and_then(|v| v.as_str()) {
                return color.to_string();
            }
        }
    }
    String::new()
}

fn apply_theme(home: &str, themes_dir: &PathBuf, current_file: &PathBuf, theme: &str) {
    println!("🌲 Switching to Faelight Forest {}...", theme);
    
    // Get colors from theme JSON
    let border = get_color(themes_dir, theme, "border");
    let border_active = get_color(themes_dir, theme, "border_active");
    
    // 1. Hyprland borders
    if !border_active.is_empty() {
        let color = border_active.trim_start_matches('#');
        Command::new("swaymsg")
            .args(["keyword", "general:col.active_border", &format!("rgb({})", color)])
            .output()
            .ok();
    }
    if !border.is_empty() {
        let color = border.trim_start_matches('#');
        Command::new("swaymsg")
            .args(["keyword", "general:col.inactive_border", &format!("rgb({})", color)])
            .output()
            .ok();
    }
    
    let core_dir = PathBuf::from(home).join("0-core");
    
    // 2. Foot terminal - switch theme
    Command::new("stow")
        .args(["-D", "foot-theme-dark", "foot-theme-light"])
        .current_dir(&core_dir)
        .output()
        .ok();
    Command::new("stow")
        .arg(format!("foot-theme-{}", theme))
        .current_dir(&core_dir)
        .output()
        .ok();
    
    // 3. Ghostty terminal - switch theme
    Command::new("stow")
        .args(["-D", "ghostty-theme-dark", "ghostty-theme-light"])
        .current_dir(&core_dir)
        .output()
        .ok();
    Command::new("stow")
        .arg(format!("ghostty-theme-{}", theme))
        .current_dir(&core_dir)
        .output()
        .ok();
    
    // 4. Fuzzel - switch theme
    Command::new("stow")
        .args(["-D", "fuzzel-theme-dark", "fuzzel-theme-light"])
        .current_dir(&core_dir)
        .output()
        .ok();
    Command::new("stow")
        .arg(format!("fuzzel-theme-{}", theme))
        .current_dir(&core_dir)
        .output()
        .ok();
    
    // 5. GTK apps
    let color_scheme = if theme == "dark" { "prefer-dark" } else { "prefer-light" };
    Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", "color-scheme", color_scheme])
        .output()
        .ok();
    
    // 6. Mako notifications
    let mako_config = PathBuf::from(home).join(".config/mako/config");
    if let Ok(content) = fs::read_to_string(&mako_config) {
        let (bg, text, border_color) = if theme == "dark" {
            ("#2e6146E6", "#e8f5d5", "#5bb7a5")
        } else {
            ("#e8f0edE6", "#0a150f", "#1a6b54")
        };
        
        let mut new_content = String::new();
        for line in content.lines() {
            if line.starts_with("background-color=") {
                new_content.push_str(&format!("background-color={}\n", bg));
            } else if line.starts_with("text-color=") {
                new_content.push_str(&format!("text-color={}\n", text));
            } else if line.starts_with("border-color=") {
                new_content.push_str(&format!("border-color={}\n", border_color));
            } else {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }
        fs::write(&mako_config, new_content).ok();
    }
    
    // Restart mako
    Command::new("killall").arg("mako").output().ok();
    Command::new("mako").spawn().ok();
    
    // Save current theme
    fs::write(current_file, theme).ok();
    
    // Output summary
    println!("{}✅ Theme switched to {}!{}", GREEN, theme, NC);
    println!("   - Hyprland borders: updated");
    println!("   - Foot colors: updated");
    println!("   - Ghostty theme: {}", theme);
    println!("   - Fuzzel theme: {}", theme);
    println!("   - GTK theme: updated");
    println!("   - Mako notifications: updated");
    
    // Notification
    Command::new("notify-send")
        .args(["🌲 Faelight Forest", &format!("Theme: {}", theme), "-t", "2000"])
        .output()
        .ok();
}
