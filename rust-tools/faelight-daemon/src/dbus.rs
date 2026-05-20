//! INT-294 -- Forest Event Bus v2
//! org.faelight.Forest D-Bus service
//! Exposes forest state (health, intent) as D-Bus properties and signals.
//! Any tool on the system can subscribe -- bar, FM, compositor, external scripts.

use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::{connection, interface, SignalContext};

// ── Shared state ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ForestBusState {
    pub health: Arc<Mutex<u32>>,
    pub intent_title: Arc<Mutex<String>>,
    pub intent_id: Arc<Mutex<u32>>,
}

impl ForestBusState {
    pub fn new() -> Self {
        Self {
            health: Arc::new(Mutex::new(read_health())),
            intent_title: Arc::new(Mutex::new(read_intent())),
            intent_id: Arc::new(Mutex::new(0)),
        }
    }
}

// ── Health interface -- org.faelight.Forest.Health ────────────────────────────

pub struct ForestHealthIface {
    pub health: Arc<Mutex<u32>>,
}

#[interface(name = "org.faelight.Forest.Health")]
impl ForestHealthIface {
    /// Current health percentage (0-100)
    #[zbus(property)]
    async fn health_percent(&self) -> u32 {
        *self.health.lock().await
    }

    /// Emitted when health changes
    #[zbus(signal)]
    async fn health_changed(
        ctx: &SignalContext<'_>,
        old: u32,
        new_val: u32,
    ) -> zbus::Result<()>;
}

// ── Intent interface -- org.faelight.Forest.Intent ───────────────────────────

pub struct ForestIntentIface {
    pub title: Arc<Mutex<String>>,
    pub id: Arc<Mutex<u32>>,
}

#[interface(name = "org.faelight.Forest.Intent")]
impl ForestIntentIface {
    /// Title of the currently active intent
    #[zbus(property)]
    async fn active_intent(&self) -> String {
        self.title.lock().await.clone()
    }

    /// ID of the currently active intent
    #[zbus(property)]
    async fn active_intent_id(&self) -> u32 {
        *self.id.lock().await
    }

    /// Emitted when the active intent changes
    #[zbus(signal)]
    async fn intent_changed(
        ctx: &SignalContext<'_>,
        old: String,
        new_val: String,
    ) -> zbus::Result<()>;
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn read_health() -> u32 {
    let home = std::env::var("HOME").unwrap_or_default();
    std::fs::read_to_string("/etc/faelight/HEALTH")
        .ok()
        .and_then(|s| s.trim().trim_end_matches('%').parse().ok())
        .or_else(|| {
            std::fs::read_to_string(format!("{}/.cache/faelight/health-status", home))
                .ok()
                .and_then(|s| s.trim().parse().ok())
        })
        .unwrap_or(100)
}

pub fn read_intent() -> String {
    std::fs::read_to_string("/etc/faelight/INTENT")
        .unwrap_or_default()
        .trim()
        .chars()
        .take(80)
        .collect()
}

pub fn read_intent_id() -> u32 {
    // Try to extract INT-NNN from the INTENT file content
    let content = std::fs::read_to_string("/etc/faelight/INTENT")
        .unwrap_or_default();
    // Look for pattern "INT-NNN" in content
    content
        .split_whitespace()
        .find(|w| w.starts_with("INT-"))
        .and_then(|s| s.trim_start_matches("INT-").parse().ok())
        .unwrap_or(0)
}

// ── Main D-Bus service loop ───────────────────────────────────────────────────

pub async fn run_forest_bus() {
    eprintln!("🌲 forest-bus: starting org.faelight.Forest on session D-Bus");

    let state = ForestBusState::new();

    let health_iface = ForestHealthIface {
        health: state.health.clone(),
    };
    let intent_iface = ForestIntentIface {
        title: state.intent_title.clone(),
        id: state.intent_id.clone(),
    };

    let conn = match connection::Builder::session()
        .and_then(|b| b.name("org.faelight.Forest"))
        .and_then(|b| b.serve_at("/org/faelight/Forest/Health", health_iface))
        .and_then(|b| b.serve_at("/org/faelight/Forest/Intent", intent_iface))
    {
        Ok(b) => match b.build().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ forest-bus: D-Bus connection failed: {e}");
                return;
            }
        },
        Err(e) => {
            eprintln!("❌ forest-bus: D-Bus builder failed: {e}");
            return;
        }
    };

    eprintln!("✅ forest-bus: org.faelight.Forest registered on session bus");

    let mut last_health = read_health();
    let mut last_intent = read_intent();

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // ── Health check ──────────────────────────────────────────────────────
        let h = read_health();
        if h != last_health {
            let old = last_health;
            *state.health.lock().await = h;
            last_health = h;
            eprintln!("🌲 forest-bus: health {} -> {}", old, h);
            if let Ok(iface_ref) = conn
                .object_server()
                .interface::<_, ForestHealthIface>("/org/faelight/Forest/Health")
                .await
            {
                let ctx = iface_ref.signal_context();
                let _ = ForestHealthIface::health_changed(ctx, old, h).await;
            }
        }

        // ── Intent check ─────────────────────────────────────────────────────
        let i = read_intent();
        if i != last_intent {
            let old = last_intent.clone();
            let id = read_intent_id();
            *state.intent_title.lock().await = i.clone();
            *state.intent_id.lock().await = id;
            last_intent = i.clone();
            eprintln!("🌲 forest-bus: intent changed -> {}", i);
            if let Ok(iface_ref) = conn
                .object_server()
                .interface::<_, ForestIntentIface>("/org/faelight/Forest/Intent")
                .await
            {
                let ctx = iface_ref.signal_context();
                let _ = ForestIntentIface::intent_changed(ctx, old, i).await;
            }
        }
    }
}
