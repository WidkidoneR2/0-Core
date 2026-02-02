use std::process::Command;

pub fn check_firmware_updates() -> Vec<String> {
    let output = Command::new("fwupdmgr")
        .args(["get-updates"])
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout
                .lines()
                .filter(|line| line.contains("Device:") || line.contains("Update"))
                .map(|s| s.trim().to_string())
                .collect();
        }
    }
    
    Vec::new()
}
