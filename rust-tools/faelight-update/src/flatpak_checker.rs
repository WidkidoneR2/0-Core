use std::process::Command;

pub fn check_flatpak_updates() -> Vec<String> {
    let output = Command::new("flatpak")
        .args(["remote-ls", "--updates"])
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|s| s.to_string())
                .collect();
        }
    }
    
    Vec::new()
}
