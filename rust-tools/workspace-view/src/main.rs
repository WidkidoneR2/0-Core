//! workspace-view v0.1 - Sway Workspace Intelligence
//! 🌲 Faelight Forest

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::process::Command;

// ANSI colors
const CYAN: &str = "\x1b[0;36m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[0;33m";
const BLUE: &str = "\x1b[0;34m";
const GRAY: &str = "\x1b[0;90m";
const NC: &str = "\x1b[0m";

#[derive(Deserialize, Debug)]
struct Workspace {
    num: i32,
    visible: bool,
    focused: bool,
}

#[derive(Deserialize, Debug)]
struct TreeNode {
    #[serde(rename = "type")]
    node_type: String,
    name: Option<String>,
    num: Option<i32>,
    focused: Option<bool>,
    nodes: Vec<TreeNode>,
    app_id: Option<String>,
    pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
struct Window {
    app_id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pid: Option<u32>,
    focused: bool,
}

#[derive(Serialize)]
struct WorkspaceInfo {
    num: i32,
    visible: bool,
    focused: bool,
    window_count: usize,
    windows: Vec<Window>,
}

enum OutputMode {
    Default,
    Active,
    All,
    Summary,
    Json,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mode = if args.len() > 1 {
        match args[1].as_str() {
            "--active" | "-a" => OutputMode::Active,
            "--all" => OutputMode::All,
            "--summary" | "-s" => OutputMode::Summary,
            "--json" => OutputMode::Json,
            "--help" | "-h" => {
                show_help();
                return;
            }
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                eprintln!("Try --help for usage information");
                std::process::exit(1);
            }
        }
    } else {
        OutputMode::Default
    };

    // Get all workspaces
    let workspaces = get_workspaces();
    
    // Get detailed tree
    let tree = get_tree();
    
    // Build workspace map
    let mut workspace_data: Vec<WorkspaceInfo> = Vec::new();
    
    for workspace in &workspaces {
        let windows = get_windows_for_workspace(&tree, workspace.num);
        workspace_data.push(WorkspaceInfo {
            num: workspace.num,
            visible: workspace.visible,
            focused: workspace.focused,
            window_count: windows.len(),
            windows,
        });
    }
    
    match mode {
        OutputMode::Json => output_json(&workspace_data),
        OutputMode::Summary => output_summary(&workspace_data),
        OutputMode::Active => output_active(&workspace_data),
        OutputMode::All => output_detailed(&workspace_data, true),
        OutputMode::Default => output_detailed(&workspace_data, false),
    }
}

fn output_json(data: &[WorkspaceInfo]) {
    println!("{}", serde_json::to_string_pretty(data).unwrap());
}

fn output_summary(data: &[WorkspaceInfo]) {
    println!();
    println!("{}🌲 Workspaces Summary{}", CYAN, NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    for ws in data {
        let status = if ws.focused {
            format!(" {}[ACTIVE]{}", GREEN, NC)
        } else if ws.visible {
            format!(" {}[VISIBLE]{}", YELLOW, NC)
        } else {
            String::new()
        };
        
        if ws.window_count > 0 {
            let apps: Vec<&str> = ws.windows.iter().map(|w| w.app_id.as_str()).collect();
            println!("{}{}{} → {} window{} ({})",
                BLUE, ws.num, NC,
                ws.window_count,
                if ws.window_count == 1 { "" } else { "s" },
                apps.join(", ")
            );
            println!("{}", status);
        } else {
            println!("{}{}{} → {}(empty){}",
                GRAY, ws.num, NC, GRAY, NC
            );
        }
    }
    println!();
}

fn output_active(data: &[WorkspaceInfo]) {
    if let Some(active) = data.iter().find(|ws| ws.focused) {
        println!();
        println!("{}🌲 Active Workspace{}", CYAN, NC);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        display_workspace(active);
    }
}

fn output_detailed(data: &[WorkspaceInfo], show_all: bool) {
    println!();
    println!("{}🌲 Workspace Overview{}", CYAN, NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    let mut displayed = vec![];
    
    for ws in data {
        if ws.window_count > 0 || show_all {
            display_workspace(ws);
            displayed.push(ws.num);
        }
    }
    
    if !show_all {
        // Find empty workspaces
        let empty: Vec<i32> = data.iter()
            .filter(|ws| ws.window_count == 0)
            .map(|ws| ws.num)
            .collect();
        
        if !empty.is_empty() {
            let range = if empty.len() == 1 {
                format!("Workspace {}", empty[0])
            } else {
                format!("Workspaces {}-{}", empty[0], empty[empty.len() - 1])
            };
            
            println!("{}{}{}", GRAY, range, NC);
            println!("  {}└─ (empty){}", GRAY, NC);
            println!();
        }
    }
}

fn display_workspace(ws: &WorkspaceInfo) {
    let status = if ws.visible && ws.focused {
        format!(" {}[ACTIVE] 🟢{}", GREEN, NC)
    } else if ws.visible {
        format!(" {}[VISIBLE]{}", YELLOW, NC)
    } else {
        String::new()
    };
    
    let count_info = if ws.window_count > 0 {
        format!(" {}({} window{}){}",
            GRAY,
            ws.window_count,
            if ws.window_count == 1 { "" } else { "s" },
            NC
        )
    } else {
        String::new()
    };
    
    println!("{}Workspace {}{}{}{}", BLUE, ws.num, NC, status, count_info);
    
    if ws.windows.is_empty() {
        println!("  {}└─ (empty){}", GRAY, NC);
    } else {
        for (i, window) in ws.windows.iter().enumerate() {
            let is_last = i == ws.windows.len() - 1;
            let branch = if is_last { "└─" } else { "├─" };
            
            let focus_marker = if window.focused {
                format!(" {}●{}", GREEN, NC)
            } else {
                String::new()
            };
            
            // Use cwd if available, otherwise use window name
            let detail = if let Some(cwd) = &window.cwd {
                format!(" - {}", cwd)
            } else {
                format!(" - {}", window.name)
            };
            
            println!("  {}{} {}{}{}{}", 
                GRAY, branch, NC,
                window.app_id, 
                detail,
                focus_marker
            );
        }
    }
    
    println!();
}

fn get_workspaces() -> Vec<Workspace> {
    let output = Command::new("swaymsg")
        .args(["-t", "get_workspaces"])
        .output()
        .expect("Failed to run swaymsg");
    
    serde_json::from_slice(&output.stdout).unwrap_or_else(|_| vec![])
}

fn get_tree() -> TreeNode {
    let output = Command::new("swaymsg")
        .args(["-t", "get_tree"])
        .output()
        .expect("Failed to run swaymsg");
    
    serde_json::from_slice(&output.stdout).expect("Failed to parse tree")
}

fn get_windows_for_workspace(tree: &TreeNode, workspace_num: i32) -> Vec<Window> {
    let mut windows = Vec::new();
    find_workspace_windows(tree, workspace_num, &mut windows);
    windows
}

fn find_workspace_windows(node: &TreeNode, workspace_num: i32, windows: &mut Vec<Window>) {
    if node.node_type == "workspace" && node.num == Some(workspace_num) {
        collect_windows(node, windows);
        return;
    }
    
    for child in &node.nodes {
        find_workspace_windows(child, workspace_num, windows);
    }
}

fn collect_windows(node: &TreeNode, windows: &mut Vec<Window>) {
    if let Some(app_id) = &node.app_id {
        let cwd = if app_id == "foot" {
            get_terminal_cwd(node.pid).map(shorten_path)
        } else {
            None
        };
        
        windows.push(Window {
            app_id: app_id.clone(),
            name: node.name.clone().unwrap_or_default(),
            cwd,
            pid: node.pid,
            focused: node.focused.unwrap_or(false),
        });
    }
    
    for child in &node.nodes {
        collect_windows(child, windows);
    }
}

fn get_terminal_cwd(pid: Option<u32>) -> Option<String> {
    let pid = pid?;
    
    let output = Command::new("pgrep")
        .args(["-P", &pid.to_string()])
        .output()
        .ok()?;
    
    let child_pid = String::from_utf8_lossy(&output.stdout)
        .trim()
        .lines()
        .next()?
        .parse::<u32>()
        .ok()?;
    
    let cwd_path = format!("/proc/{}/cwd", child_pid);
    std::fs::read_link(cwd_path).ok()?.to_str().map(String::from)
}

fn shorten_path(path: String) -> String {
    if let Ok(home) = std::env::var("HOME") {
        if path.starts_with(&home) {
            return path.replace(&home, "~");
        }
    }
    path
}

fn show_help() {
    println!("workspace-view - Sway Workspace Intelligence");
    println!();
    println!("USAGE:");
    println!("   workspace-view [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("   --active, -a     Show only the active workspace");
    println!("   --all            Show all workspaces including empty ones");
    println!("   --summary, -s    Compact summary view");
    println!("   --json           Machine-readable JSON output");
    println!("   --help, -h       Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("   workspace-view              # Default detailed view");
    println!("   workspace-view --active     # Quick glance at current workspace");
    println!("   workspace-view --summary    # One-line per workspace");
    println!("   workspace-view --json       # JSON for scripting");
}
