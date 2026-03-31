//! faelight-idle v1.0.0
//! 🌲 Rust idle daemon — replaces swayidle
//! Uses ext-idle-notify-v1 Wayland protocol natively.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_registry, wl_seat},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1::{self, ExtIdleNotificationV1},
    ext_idle_notifier_v1::ExtIdleNotifierV1,
};

#[derive(Parser)]
#[command(
    name = "faelight-idle",
    about = "🌲 Rust idle daemon",
    version = "1.0.0"
)]
struct Cli {
    #[arg(short, long, default_value = "300")]
    timeout: u64,
    #[arg(short, long, default_value = "600")]
    lock_timeout: u64,
    #[arg(long)]
    no_lock: bool,
    #[arg(long)]
    events_only: bool,
    /// Health check — print status and exit
    #[arg(long)]
    health: bool,
}

struct IdleState {
    #[allow(dead_code)]
    lock_timeout_ms: u32,
    no_lock: bool,
    idle_start: Option<std::time::Instant>,
    core_root: PathBuf,
    ready: bool,
}

impl IdleState {
    fn new(timeout_secs: u64, lock_timeout_secs: u64, no_lock: bool) -> Self {
        let core_root = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/home/christian"))
            .join("0-core");
        let _ = timeout_secs;
        Self {
            lock_timeout_ms: (lock_timeout_secs * 1000) as u32,
            no_lock,
            idle_start: None,
            core_root,
            ready: false,
        }
    }

    fn emit_event(&self, action: &str, detail: &str) {
        let db_path = self.core_root.join("runtime/state.db");
        if !db_path.exists() {
            return;
        }
        let conn = match rusqlite::Connection::open(&db_path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let payload = format!(
            r#"{{"actor":"faelight-idle","result":"ok","detail":{{{}}}}}"#,
            detail
        );
        let _ = conn.execute(
            "INSERT INTO events (domain, action, payload, timestamp) VALUES ('idle', ?, ?, ?)",
            rusqlite::params![action, payload, ts],
        );
    }

    fn on_idle(&mut self) {
        eprintln!("💤 Idle detected");
        self.idle_start = Some(std::time::Instant::now());
        self.emit_event("idle.start", r#""timeout_ms":0"#);
        if !self.no_lock {
            let _ = Command::new("faelight-lock").spawn();
            eprintln!("🔒 Lock triggered");
            self.emit_event("idle.lock", r#""action":"lock""#);
        }
    }

    fn on_resume(&mut self) {
        let duration_secs = self.idle_start.map(|s| s.elapsed().as_secs()).unwrap_or(0);
        eprintln!("🌿 Resumed after {}s", duration_secs);
        self.idle_start = None;
        self.emit_event("idle.end", &format!(r#""duration_secs":{}"#, duration_secs));
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for IdleState {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtIdleNotifierV1, ()> for IdleState {
    fn event(
        _: &mut Self,
        _: &ExtIdleNotifierV1,
        _: wayland_protocols::ext::idle_notify::v1::client::ext_idle_notifier_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtIdleNotificationV1, ()> for IdleState {
    fn event(
        state: &mut Self,
        _: &ExtIdleNotificationV1,
        event: ext_idle_notification_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            ext_idle_notification_v1::Event::Idled => state.on_idle(),
            ext_idle_notification_v1::Event::Resumed => state.on_resume(),
            _ => {}
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for IdleState {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.health {
        println!("faelight-idle v1.0.0 — healthy");
        return Ok(());
    }
    let timeout_ms = (cli.timeout * 1000) as u32;
    let no_lock = cli.no_lock || cli.events_only;

    let conn = Connection::connect_to_env().context("Failed to connect to Wayland")?;

    let (globals, mut queue) =
        registry_queue_init::<IdleState>(&conn).context("Failed to init registry")?;

    let qh = queue.handle();

    // Bind globals directly from the global list
    let notifier = globals.bind::<ExtIdleNotifierV1, _, _>(&qh, 1..=1, ()).ok();
    let seat = globals.bind::<wl_seat::WlSeat, _, _>(&qh, 1..=7, ()).ok();

    if notifier.is_none() {
        eprintln!("❌ ext_idle_notifier_v1 not available");
        eprintln!("   Available globals:");
        for g in globals.contents().clone_list() {
            eprintln!("     {} v{}", g.interface, g.version);
        }
        std::process::exit(1);
    }

    let notifier = notifier.unwrap();
    let seat = seat.context("wl_seat not available")?;

    // Create idle notification
    notifier.get_idle_notification(timeout_ms, &seat, &qh, ());

    eprintln!(
        "🌲 faelight-idle v1.0.0 — timeout: {}s  lock: {}",
        cli.timeout,
        if no_lock { "disabled" } else { "enabled" },
    );
    eprintln!("   Listening...");

    let mut state = IdleState::new(cli.timeout, cli.lock_timeout, no_lock);
    state.ready = true;

    loop {
        queue.blocking_dispatch(&mut state)?;
    }
}
