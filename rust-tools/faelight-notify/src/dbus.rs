// faelight-notify v4 — D-Bus server
// Implements org.freedesktop.Notifications spec

use crate::{NotifQueue, Notification, Urgency};
use std::sync::atomic::{AtomicU32, Ordering};
use zbus::{connection, interface};

static NOTIF_ID: AtomicU32 = AtomicU32::new(1);

struct NotificationsServer {
    queue: NotifQueue,
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationsServer {
    fn get_capabilities(&self) -> Vec<String> {
        vec![
            "body".to_string(),
            "body-markup".to_string(),
            "persistence".to_string(),
        ]
    }

    #[allow(clippy::too_many_arguments)]
    fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        _app_icon: &str,
        summary: &str,
        body: &str,
        _actions: Vec<String>,
        hints: std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
        expire_timeout: i32,
    ) -> u32 {
        let id = if replaces_id != 0 {
            replaces_id
        } else {
            NOTIF_ID.fetch_add(1, Ordering::SeqCst)
        };

        let urgency = Urgency::from_hints(&hints);
        let notif = Notification {
            id,
            app_name: app_name.to_string(),
            summary: summary.to_string(),
            body: body.to_string(),
            urgency,
            timeout: expire_timeout,
            created: std::time::Instant::now(),
        };

        let mut q = self.queue.lock().unwrap();
        // Replace if same id
        if let Some(pos) = q.iter().position(|n| n.id == id) {
            q[pos] = notif;
        } else {
            q.push(notif);
        }
        id
    }

    fn close_notification(&self, id: u32) {
        let mut q = self.queue.lock().unwrap();
        q.retain(|n| n.id != id);
    }

    fn get_server_information(&self) -> (&str, &str, &str, &str) {
        ("faelight-notify", "faelight-forest", "4.0.0", "1.2")
    }
}

pub async fn run(queue: NotifQueue) -> Result<(), Box<dyn std::error::Error>> {
    let server = NotificationsServer { queue };
    let _conn = connection::Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", server)?
        .build()
        .await?;

    eprintln!("🔌 D-Bus: org.freedesktop.Notifications registered");

    // Keep alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
