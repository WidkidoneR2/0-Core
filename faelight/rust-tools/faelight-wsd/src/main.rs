use std::fs;
use std::io::Write;
use std::path::PathBuf;

use wayland_client::{
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, QueueHandle, WEnum,
};

pub mod dwl {
    use wayland_client;
    use wayland_client::protocol::*;
    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("protocols/dwl-ipc-unstable-v2.xml");
    }
    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("protocols/dwl-ipc-unstable-v2.xml");
}

use dwl::zdwl_ipc_manager_v2::{self, ZdwlIpcManagerV2};
use dwl::zdwl_ipc_output_v2::{self, ZdwlIpcOutputV2};

#[derive(Clone, Default)]
struct TagInfo {
    selected: bool,
    occupied: bool,
    urgent: bool,
    focused: bool,
}

// Single-output state (Framework 16 = one monitor). Multi-output would key
// this per wl_output and emit one file per output; deferred until needed.
struct App {
    manager: Option<ZdwlIpcManagerV2>,
    layouts: Vec<String>, // layout symbol table, in manager-announced order
    tags: Vec<TagInfo>,   // indexed by tag id
    layout_idx: u32,
    outputs: Vec<wl_output::WlOutput>,
    last_written: String,
    out_path: PathBuf,
}

impl App {
    fn ensure_tag(&mut self, idx: usize) {
        if self.tags.len() <= idx {
            self.tags.resize(idx + 1, TagInfo::default());
        }
    }

    fn to_json(&self) -> String {
        let mut s = String::from("{\"tags\":[");
        for (i, t) in self.tags.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(&format!(
                "{{\"id\":{},\"selected\":{},\"occupied\":{},\"urgent\":{},\"focused\":{}}}",
                i, t.selected, t.occupied, t.urgent, t.focused
            ));
        }
        let layout = self
            .layouts
            .get(self.layout_idx as usize)
            .cloned()
            .unwrap_or_default();
        s.push_str(&format!("],\"layout\":\"{}\"}}", layout.replace('"', "'")));
        s
    }

    fn write_if_changed(&mut self) {
        let json = self.to_json();
        if json == self.last_written {
            return;
        }
        let tmp = self.out_path.with_extension("tmp");
        if let Ok(mut f) = fs::File::create(&tmp) {
            if f.write_all(json.as_bytes()).is_ok() && f.write_all(b"\n").is_ok() {
                drop(f);
                if fs::rename(&tmp, &self.out_path).is_ok() {
                    self.last_written = json;
                }
            }
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for App {
    fn event(
        app: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<App>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "zdwl_ipc_manager_v2" => {
                    app.manager =
                        Some(registry.bind::<ZdwlIpcManagerV2, _, _>(name, version.min(2), qh, ()))
                }
                "wl_output" => app.outputs.push(registry.bind::<wl_output::WlOutput, _, _>(
                    name,
                    version.min(4),
                    qh,
                    (),
                )),
                _ => {}
            }
        }
    }
}

impl Dispatch<ZdwlIpcManagerV2, ()> for App {
    fn event(
        app: &mut Self,
        _: &ZdwlIpcManagerV2,
        event: zdwl_ipc_manager_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<App>,
    ) {
        match event {
            zdwl_ipc_manager_v2::Event::Tags { amount } => {
                app.ensure_tag(amount.saturating_sub(1) as usize)
            }
            zdwl_ipc_manager_v2::Event::Layout { name } => app.layouts.push(name),
        }
    }
}

impl Dispatch<ZdwlIpcOutputV2, ()> for App {
    fn event(
        app: &mut Self,
        _: &ZdwlIpcOutputV2,
        event: zdwl_ipc_output_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<App>,
    ) {
        use zdwl_ipc_output_v2::Event;
        match event {
            Event::Tag {
                tag,
                state,
                clients,
                focused,
            } => {
                let bits: u32 = match state {
                    WEnum::Value(v) => v.into(),
                    WEnum::Unknown(n) => n,
                };
                app.ensure_tag(tag as usize);
                let t = &mut app.tags[tag as usize];
                t.selected = bits & 1 != 0;
                t.urgent = bits & 2 != 0;
                t.occupied = clients > 0;
                t.focused = focused != 0;
            }
            Event::Layout { layout } => app.layout_idx = layout,
            Event::Frame => app.write_if_changed(),
            _ => {}
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for App {
    fn event(
        _: &mut Self,
        _: &wl_output::WlOutput,
        _: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<App>,
    ) {
    }
}

fn main() {
    let home = std::env::var("HOME").expect("HOME not set");
    let dir = PathBuf::from(&home).join(".cache/faelight");
    let _ = fs::create_dir_all(&dir);
    let out_path = dir.join("workspaces");

    let conn = Connection::connect_to_env().expect("connect to wayland");
    let mut queue = conn.new_event_queue::<App>();
    let qh = queue.handle();
    conn.display().get_registry(&qh, ());

    let mut app = App {
        manager: None,
        layouts: Vec::new(),
        tags: Vec::new(),
        layout_idx: 0,
        outputs: Vec::new(),
        last_written: String::new(),
        out_path,
    };
    queue.roundtrip(&mut app).expect("registry roundtrip");

    let mgr = match app.manager.clone() {
        Some(m) => m,
        None => {
            eprintln!("faelight-wsd: zdwl_ipc_manager_v2 not advertised -- is this mango?");
            std::process::exit(1);
        }
    };
    let outs = app.outputs.clone();
    let mut _ipc_outputs = Vec::new();
    for o in &outs {
        _ipc_outputs.push(mgr.get_output(o, &qh, ()));
    }
    eprintln!(
        "faelight-wsd: bound manager + {} output(s); writing {}",
        outs.len(),
        app.out_path.display()
    );

    loop {
        if let Err(e) = queue.blocking_dispatch(&mut app) {
            eprintln!("faelight-wsd: dispatch error: {e}; exiting for restart");
            std::process::exit(1);
        }
    }
}
