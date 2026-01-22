//! workspace-view v1.0.0 - Sway Workspace Intelligence
//! 🌲 Faelight Forest

use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;
use std::thread;
use std::time::Duration;

const VERSION: &str = "1.0.0";

// ANSI colors
const CYAN: &str = "\x1b[0;36m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[0;33m";
const BLUE: &str = "\x1b[0;34m";
const GRAY: &str = "\x1b[0;90m";
const RED: &str = "\x1b[0;31m";
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
    Watch(u64), // Watch mode with interval in seconds
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mode = parse_args(&args);
    
    // Handle watch mode specially
    if let OutputMode::Watch(interval) = mode {
        run_watch_mode(interval);
        return;
    }
    
    // Single execution for other modes
    execute_mode(&mode);
}

fn parse_args(args: &[String]) -> OutputMode {
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("workspace-view v{}", VERSION);
                std::process::exit(0);
            }
            "--health" => {
                run_health_check();
                std::process::exit(0);
            }
            "--active" | "-a" => OutputMode::Active,
            "--all" => OutputMode::All,
            "--summary" | "-s" => OutputMode::Summary,
            "--json" => OutputMode::Json,
            "--watch" | "-w" => {
                // Check for interval argument
                let interval = if args.len() > 2 {
                    args[2].parse::<u64>().unwrap_or(2)
                } else {
                    2 // Default 2 seconds
                };
                OutputMode::Watch(interval)
            }
            "--help" | "-h" => {
                show_help();
                std::process::exit(0);
            }
            _ => {
                eprintln!("{}Unknown option: {}{}", RED, args[1], NC);
                eprintln!("Try --help for usage information");
                std::process::exit(1);
            }
        }
    } else {
        OutputMode::Default
    }
}

fn run_watch_mode(interval: u64) {
    println!("{}🌲 Workspace Watch Mode{} (updating every {}s, Ctrl+C to exit)", 
             CYAN, NC, interval);
    println!();
    
    loop {
        // Clear screen
        print!("\x1b[2J\x1b[H");
        
        // Show timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        println!("{}Last updated: {} seconds since epoch{}", 
                 GRAY, now.as_secs(), NC);
        println!();
        
        // Execute in active mode for watch
        execute_mode(&OutputMode::Default);
        
        // Wait for interval
        thread::sleep(Duration::from_secs(interval));
    }
}

fn execute_mode(mode: &OutputMode) {
    // Get all workspaces
    let workspaces = match get_workspaces() {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("{}❌ Error getting workspaces: {}{}", RED, e, NC);
            std::process::exit(1);
        }
    };
    
    // Get detailed tree
    let tree = match get_tree() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}❌ Error getting tree: {}{}", RED, e, NC);
            std::process::exit(1);
        }
    };
    
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
        OutputMode::Watch(_) => unreachable!(),
    }
}

fn run_health_check() {
    println!();
    println!("{}🏥 workspace-view v{} - Health Check{}", CYAN, VERSION, NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut healthy = true;
    
    // Check swaymsg available
    print!("  Checking swaymsg... ");
    match Command::new("which").arg("swaymsg").output() {
        Ok(output) if output.status.success() => println!("{}✅{}", GREEN, NC),
        _ => {
            println!("{}❌ Not found{}", RED, NC);
            healthy = false;
        }
    }
    
    // Check if running under Sway
    print!("  Checking Sway session... ");
    match env::var("SWAYSOCK") {
        Ok(_) => println!("{}✅{}", GREEN, NC),
        Err(_) => {
            println!("{}❌ Not running in Sway{}", RED, NC);
            healthy = false;
        }
    }
    
    // Check if can get workspaces
    print!("  Checking workspace access... ");
    match get_workspaces() {
        Ok(ws) if !ws.is_empty() => {
            println!("{}✅ {} workspaces detected{}", GREEN, ws.len(), NC);
        }
        Ok(_) => {
            println!("{}⚠️  No workspaces found{}", YELLOW, NC);
        }
        Err(e) => {
            println!("{}❌ Error: {}{}", RED, e, NC);
            healthy = false;
        }
    }
    
    // Check if can get tree
    print!("  Checking tree access... ");
    match get_tree() {
        Ok(_) => println!("{}✅{}", GREEN, NC),
        Err(e) => {
            println!("{}❌ Error: {}{}", RED, e, NC);
            healthy = false;
        }
    }
    
    // Check pgrep available (for cwd detection)
    print!("  Checking pgrep (for cwd)... ");
    match Command::new("which").arg("pgrep").output() {
        Ok(output) if output.status.success() => println!("{}✅{}", GREEN, NC),
        _ => {
            println!("{}⚠️  Not found (cwd detection disabled){}", YELLOW, NC);
        }
    }
    
    println!();
    if healthy {
        println!("{}✅ All systems operational{}", GREEN, NC);
        std::process::exit(0);
    } else {
        println!("{}❌ System unhealthy{}", RED, NC);
        std::process::exit(1);
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
            println!("{}{}{} → {} window{} ({}){}",
                BLUE, ws.num, NC,
                ws.window_count,
                if ws.window_count == 1 { "" } else { "s" },
                apps.join(", "),
                status
            );
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

fn get_workspaces() -> Result<Vec<Workspace>, String> {
    let output = Command::new("swaymsg")
        .args(["-t", "get_workspaces"])
        .output()
        .map_err(|e| format!("Failed to run swaymsg: {}", e))?;
    
    if !output.status.success() {
        return Err("swaymsg failed".to_string());
    }
    
    serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse workspaces: {}", e))
}

fn get_tree() -> Result<TreeNode, String> {
    let output = Command::new("swaymsg")
        .args(["-t", "get_tree"])
        .output()
        .map_err(|e| format!("Failed to run swaymsg: {}", e))?;
    
    if !output.status.success() {
        return Err("swaymsg failed".to_string());
    }
    
    serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse tree: {}", e))
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
    println!("{}🌲 workspace-view v{}{} - Sway Workspace Intelligence", CYAN, VERSION, NC);
    println!();
    println!("USAGE:");
    println!("   workspace-view [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("   --active, -a         Show only the active workspace");
    println!("   --all                Show all workspaces including empty ones");
    println!("   --summary, -s        Compact summary view");
    println!("   --watch, -w [SEC]    Watch mode (default: 2 seconds)");
    println!("   --json               Machine-readable JSON output");
    println!("   --health             Run health check");
    println!("   --version, -v        Show version");
    println!("   --help, -h           Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("   workspace-view              # Default detailed view");
    println!("   workspace-view --active     # Quick glance at current workspace");
    println!("   workspace-view --summary    # One-line per workspace");
    println!("   workspace-view --watch      # Live updates every 2s");
    println!("   workspace-view --watch 5    # Live updates every 5s");
    println!("   workspace-view --json       # JSON for scripting");
    println!();
    println!("ALIASES:");
    println!("   alias ws='workspace-view'");
    println!("   alias wsa='workspace-view --active'");
    println!("   alias wss='workspace-view --summary'");
    println!("   alias wsw='workspace-view --watch'");
    println!();
    println!("PHILOSOPHY:");
    println!("   'Understanding over convenience'");
    println!("   See your workspace state at a glance.");
}
