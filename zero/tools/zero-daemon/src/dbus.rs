//! INT-294 -- Event Bus v2
//! org.zero.Core D-Bus service
//! Exposes state (health, intent) as D-Bus properties and signals.
//! Any tool on the system can subscribe -- bar, FM, compositor, external scripts.

use futures_util::StreamExt as _;
use rusqlite;
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::{connection, interface, SignalContext};

// ── Shared state ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct BusState {
    pub health: Arc<Mutex<Option<u32>>>,
    pub intent_title: Arc<Mutex<String>>,
    pub intent_id: Arc<Mutex<u32>>,
}

impl BusState {
    pub fn new() -> Self {
        Self {
            // INT-250: intent_id is SEEDED from read_intent_id() like its two siblings.
            //
            // ⚠️ IT WAS Mutex::new(0) -- a literal zero -- while health and intent_title both called
            // their readers. The id then only became correct if the focus CHANGED while the
            // daemon was running, because the watch loop is the only other writer.
            //
            // So INT-250's read_intent_id() fix was correct and still reported 0: the defect was
            // one layer up, in the seeding, which no amount of reading the function would show.
            // MEASURED live over D-Bus, 2026-09-17.
            health: Arc::new(Mutex::new(read_health())),
            intent_title: Arc::new(Mutex::new(read_intent())),
            intent_id: Arc::new(Mutex::new(read_intent_id())),
        }
    }
}

// ── Health interface -- org.zero.Core.Health ──────────────────────────────────

pub struct HealthIface {
    pub health: Arc<Mutex<Option<u32>>>,
}

#[interface(name = "org.zero.Core.Health")]
impl HealthIface {
    /// Current health percentage (0-100). INT-265: when health could not be read this is a
    /// D-Bus error saying so, never a number -- it answered 100.
    #[zbus(property)]
    async fn health_percent(&self) -> zbus::fdo::Result<u32> {
        (*self.health.lock().await)
            .ok_or_else(|| zbus::fdo::Error::Failed("health could not be read".to_string()))
    }

    /// Emitted when health changes
    #[zbus(signal)]
    async fn health_changed(ctx: &SignalContext<'_>, old: u32, new_val: u32) -> zbus::Result<()>;
}

// ── Intent interface -- org.zero.Core.Intent ─────────────────────────────────

pub struct IntentIface {
    pub title: Arc<Mutex<String>>,
    pub id: Arc<Mutex<u32>>,
}

#[interface(name = "org.zero.Core.Intent")]
impl IntentIface {
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

// ── Friday interface -- org.zero.Core.Friday ─────────────────────────────────

pub struct FridayIface;

#[interface(name = "org.zero.Core.Friday")]
impl FridayIface {
    /// Emitted when Friday has a suggestion ready
    #[zbus(signal)]
    async fn friday_suggested(
        ctx: &SignalContext<'_>,
        message: String,
        confidence: f64,
    ) -> zbus::Result<()>;
}

// ── Deploy interface -- org.zero.Core.Deploy ────────────────────────────────

pub struct DeployIface;

#[interface(name = "org.zero.Core.Deploy")]
impl DeployIface {
    /// Emitted when a deploy completes successfully
    #[zbus(signal)]
    async fn deploy_completed(
        ctx: &SignalContext<'_>,
        tool: String,
        version: String,
        duration_ms: u64,
    ) -> zbus::Result<()>;

    /// Emitted when a deploy fails
    #[zbus(signal)]
    async fn deploy_failed(
        ctx: &SignalContext<'_>,
        tool: String,
        error: String,
    ) -> zbus::Result<()>;
}

// ── Deploy signal channel ─────────────────────────────────────────────────────

#[derive(Debug)]
#[allow(dead_code)]
pub enum DeploySignal {
    Completed {
        tool: String,
        version: String,
        duration_ms: u64,
    },
    Failed {
        tool: String,
        error: String,
    },
}

static DEPLOY_TX: std::sync::OnceLock<tokio::sync::mpsc::UnboundedSender<DeploySignal>> =
    std::sync::OnceLock::new();

#[allow(dead_code)]
pub fn emit_deploy_completed(tool: String, version: String, duration_ms: u64) {
    if let Some(tx) = DEPLOY_TX.get() {
        let _ = tx.send(DeploySignal::Completed {
            tool,
            version,
            duration_ms,
        });
    }
}

#[allow(dead_code)]
pub fn emit_deploy_failed(tool: String, error: String) {
    if let Some(tx) = DEPLOY_TX.get() {
        let _ = tx.send(DeploySignal::Failed { tool, error });
    }
}

// ── Friday signal channel -- callable from anywhere in the daemon ─────────────

static FRIDAY_TX: std::sync::OnceLock<tokio::sync::mpsc::UnboundedSender<(String, f64)>> =
    std::sync::OnceLock::new();

/// Called from friday_record_event when a suggestion fires.
/// Non-blocking: just queues the signal for the D-Bus loop.
pub fn emit_friday_signal(message: String, confidence: f64) {
    if let Some(tx) = FRIDAY_TX.get() {
        let _ = tx.send((message, confidence));
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn read_health() -> Option<u32> {
    // INT-250: the old /etc HEALTH read that stood here is DELETED, not repointed.
    //
    // It was tried FIRST and always failed, so every call fell through to the cache -- which
    // is the real source and was already correct. Removing it changes nothing about what this
    // function returns and removes a branch that could only ever fail.
    //
    // With the second source gone, the Layer 3a note that sat here -- explaining why
    // read_health() could not be adopted -- is no longer true either, so this now uses the
    // shared accessor like every other reader. INT-265: an unreadable health is None, not 100.
    zero_core::paths::read_health().map(|h| h as u32)
}

/// INT-265: a health value for logs -- "89%" or "unknown".
fn show_health(h: Option<u32>) -> String {
    h.map(|v| format!("{}%", v))
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn read_intent() -> String {
    // INT-250: focus.toml, the source that is actually correct.
    //
    // This read an old /etc INTENT file, gone since Omarchy, and returned "" -- so the D-Bus
    // service told every caller there was NO ACTIVE INTENT while the ledger held several.
    std::fs::read_to_string(zero_core::paths::focus_file())
        .ok()
        .and_then(|c| {
            c.lines().find_map(|line| {
                line.strip_prefix("title = ")
                    .map(|r| r.trim().trim_matches('"').to_string())
            })
        })
        .map(|t| t.chars().take(80).collect())
        .unwrap_or_default()
}

pub fn read_intent_id() -> u32 {
    // INT-250: focus.toml carries the id directly -- no INT-NNN scraping required.
    std::fs::read_to_string(zero_core::paths::focus_file())
        .ok()
        .and_then(|c| {
            c.lines().find_map(|line| {
                line.strip_prefix("id = ")
                    .and_then(|r| r.trim().trim_matches('"').parse().ok())
            })
        })
        .unwrap_or(0)
}

// ── Main D-Bus service loop ───────────────────────────────────────────────────

pub async fn run_bus() {
    eprintln!("bus: starting org.zero.Core on session D-Bus");

    let state = BusState::new();

    let health_iface = HealthIface {
        health: state.health.clone(),
    };
    let intent_iface = IntentIface {
        title: state.intent_title.clone(),
        id: state.intent_id.clone(),
    };

    let conn = match connection::Builder::session()
        .and_then(|b| b.name("org.zero.Core"))
        .and_then(|b| b.serve_at("/org/zero/Core/Health", health_iface))
        .and_then(|b| b.serve_at("/org/zero/Core/Intent", intent_iface))
        .and_then(|b| b.serve_at("/org/zero/Core/Friday", FridayIface))
        .and_then(|b| b.serve_at("/org/zero/Core/Deploy", DeployIface))
    {
        Ok(b) => match b.build().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ bus: D-Bus connection failed: {e}");
                return;
            }
        },
        Err(e) => {
            eprintln!("❌ bus: D-Bus builder failed: {e}");
            return;
        }
    };

    eprintln!("✅ bus: org.zero.Core registered on session bus");

    let mut last_health = read_health();
    let mut last_intent = read_intent();
    let mut last_deploy_id: i64 = {
        let db = zero_core::paths::state_db();
        zero_core::state_db::open_at(
            std::path::Path::new(&db),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .ok()
        .and_then(|c| {
            c.query_row("SELECT COALESCE(MAX(id),0) FROM deploy_patterns", [], |r| {
                r.get(0)
            })
            .ok()
        })
        .unwrap_or(0)
    };
    let (friday_tx, mut friday_rx) = tokio::sync::mpsc::unbounded_channel::<(String, f64)>();
    let _ = FRIDAY_TX.set(friday_tx);
    let (deploy_tx, mut deploy_rx) = tokio::sync::mpsc::unbounded_channel::<DeploySignal>();
    let _ = DEPLOY_TX.set(deploy_tx);

    // ── logind power event subscription ───────────────────────────────────────
    // Spawn logind watcher as separate task
    tokio::spawn(async move {
        let Ok(sys_conn) = zbus::Connection::system().await else {
            return;
        };
        let Ok(proxy) = zbus::Proxy::new(
            &sys_conn,
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        )
        .await
        else {
            return;
        };
        let Ok(mut sigs) = proxy.receive_signal("PrepareForSleep").await else {
            return;
        };
        while let Some(sig) = sigs.next().await {
            let body: Result<bool, _> = sig.body().deserialize();
            if let Ok(sleeping) = body {
                if sleeping {
                    eprintln!("bus: system suspending");
                    // Write suspend event to state.db
                    let db = zero_core::paths::state_db();
                    if let Ok(c) = zero_core::state_db::open_at(
                        std::path::Path::new(&db),
                        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
                    ) {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        let _ = c.execute(
                            "INSERT INTO events (domain, action, payload, timestamp) VALUES ('system','suspend','logind',?1)",
                            rusqlite::params![now],
                        );
                    }
                } else {
                    eprintln!("bus: system waking");
                    let db = zero_core::paths::state_db();
                    if let Ok(c) = zero_core::state_db::open_at(
                        std::path::Path::new(&db),
                        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
                    ) {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        let _ = c.execute(
                            "INSERT INTO events (domain, action, payload, timestamp) VALUES ('system','wake','logind',?1)",
                        rusqlite::params![now],
                        );
                    }
                }
            }
        }
    });

    loop {
        tokio::select! {
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {}
        Some(sig) = deploy_rx.recv() => {
            match sig {
                DeploySignal::Completed { tool, version, duration_ms } => {
                    eprintln!("bus: deploy {} v{} ({}ms)", tool, version, duration_ms);
                    if let Ok(iface_ref) = conn.object_server()
                        .interface::<_, DeployIface>("/org/zero/Core/Deploy").await {
                        let ctx = iface_ref.signal_context();
                        let _ = DeployIface::deploy_completed(ctx, tool, version, duration_ms).await;
                    }
                }
                DeploySignal::Failed { tool, error } => {
                    eprintln!("bus: deploy failed {} -- {}", tool, error);
                    if let Ok(iface_ref) = conn.object_server()
                        .interface::<_, DeployIface>("/org/zero/Core/Deploy").await {
                        let ctx = iface_ref.signal_context();
                        let _ = DeployIface::deploy_failed(ctx, tool, error).await;
                    }
                }
            }
            continue;
        }
        Some((msg, conf)) = friday_rx.recv() => {
            eprintln!("bus: friday signal -- {} ({:.0}%)", msg, conf * 100.0);
            if let Ok(iface_ref) = conn
                .object_server()
                .interface::<_, FridayIface>("/org/zero/Core/Friday")
                .await
            {
                let ctx = iface_ref.signal_context();
                let _ = FridayIface::friday_suggested(ctx, msg, conf).await;
            }
            continue;
        }
        }

        // ── Health check ──────────────────────────────────────────────────────
        let h = read_health();
        if h != last_health {
            let old = last_health;
            *state.health.lock().await = h;
            last_health = h;
            eprintln!("bus: health {} -> {}", show_health(old), show_health(h));
            if let Ok(iface_ref) = conn
                .object_server()
                .interface::<_, HealthIface>("/org/zero/Core/Health")
                .await
            {
                let ctx = iface_ref.signal_context();
                // INT-265: the signal carries numbers, so it fires only between two known values.
                if let (Some(o), Some(n)) = (old, h) {
                    let _ = HealthIface::health_changed(ctx, o, n).await;
                }
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
            eprintln!("bus: intent changed -> {}", i);
            if let Ok(iface_ref) = conn
                .object_server()
                .interface::<_, IntentIface>("/org/zero/Core/Intent")
                .await
            {
                let ctx = iface_ref.signal_context();
                let _ = IntentIface::intent_changed(ctx, old, i).await;
            }
        }

        // ── Deploy check ──────────────────────────────────────────────────────
        {
            let db = zero_core::paths::state_db();
            if let Ok(conn_db) = zero_core::state_db::open_at(
                std::path::Path::new(&db),
                rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
            ) {
                let rows: Vec<(i64, String, String, i64)> = conn_db
                    .prepare(
                        "SELECT id, tool, version, COALESCE(duration_ms,0) FROM deploy_patterns
                     WHERE id > ?1 AND outcome = 'success' ORDER BY id ASC LIMIT 10",
                    )
                    .ok()
                    .map(|mut s| {
                        s.query_map(rusqlite::params![last_deploy_id], |r| {
                            Ok((
                                r.get(0)?,
                                r.get(1)?,
                                r.get(2).unwrap_or_default(),
                                r.get(3).unwrap_or(0),
                            ))
                        })
                        .ok()
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                        .unwrap_or_default()
                    })
                    .unwrap_or_default();
                for (id, tool, version, dur) in rows {
                    last_deploy_id = last_deploy_id.max(id);
                    eprintln!("bus: deploy {} v{} ({}ms)", tool, version, dur);
                    if let Ok(iface_ref) = conn
                        .object_server()
                        .interface::<_, DeployIface>("/org/zero/Core/Deploy")
                        .await
                    {
                        let ctx = iface_ref.signal_context();
                        let _ = DeployIface::deploy_completed(ctx, tool, version, dur as u64).await;
                    }
                }
            }
        }
    }
}
