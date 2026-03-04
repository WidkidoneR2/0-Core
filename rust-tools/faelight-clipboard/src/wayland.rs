//! Native wlr-data-control clipboard — zero C clipboard dependencies
//! Implements zwlr-data-control-unstable-v1 directly in Rust.

use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{AsFd, FromRawFd, OwnedFd};
use wayland_client::{
    protocol::{wl_registry, wl_seat},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1::{self, ZwlrDataControlDeviceV1},
    zwlr_data_control_manager_v1::{self, ZwlrDataControlManagerV1},
    zwlr_data_control_offer_v1::{self, ZwlrDataControlOfferV1},
    zwlr_data_control_source_v1::{self, ZwlrDataControlSourceV1},
};

const MIME_UTF8: &str = "text/plain;charset=utf-8";
const MIME_PLAIN: &str = "text/plain";

// ═══════════════════════════════════════════════════════════
// 📥 PASTE STATE
// ═══════════════════════════════════════════════════════════

struct PasteState {
    seat: Option<wl_seat::WlSeat>,
    manager: Option<ZwlrDataControlManagerV1>,
    selection: Option<ZwlrDataControlOfferV1>,
    best_mime: Option<String>,
    done: bool,
}

impl PasteState {
    fn new() -> Self {
        Self { seat: None, manager: None, selection: None, best_mime: None, done: false }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for PasteState {
    fn event(state: &mut Self, registry: &wl_registry::WlRegistry,
             event: wl_registry::Event, _: &(), _: &Connection, qh: &QueueHandle<Self>) {
        let wl_registry::Event::Global { name, interface, version } = event else { return };
        match interface.as_str() {
            "wl_seat" => {
                state.seat = Some(registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ()));
            }
            "zwlr_data_control_manager_v1" => {
                state.manager = Some(
                    registry.bind::<ZwlrDataControlManagerV1, _, _>(name, version.min(2), qh, ())
                );
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for PasteState {
    fn event(_: &mut Self, _: &wl_seat::WlSeat, _: wl_seat::Event,
             _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<ZwlrDataControlManagerV1, ()> for PasteState {
    fn event(_: &mut Self, _: &ZwlrDataControlManagerV1,
             _: zwlr_data_control_manager_v1::Event,
             _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<ZwlrDataControlDeviceV1, ()> for PasteState {
    fn event(state: &mut Self, _: &ZwlrDataControlDeviceV1,
             event: zwlr_data_control_device_v1::Event,
             _: &(), _: &Connection, _: &QueueHandle<Self>) {
        match event {
            zwlr_data_control_device_v1::Event::Selection { id } => {
                state.selection = id;
                state.done = true;
            }
            zwlr_data_control_device_v1::Event::Finished => {
                state.done = true;
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwlrDataControlOfferV1, ()> for PasteState {
    fn event(state: &mut Self, _: &ZwlrDataControlOfferV1,
             event: zwlr_data_control_offer_v1::Event,
             _: &(), _: &Connection, _: &QueueHandle<Self>) {
        let zwlr_data_control_offer_v1::Event::Offer { mime_type } = event else { return };
        match mime_type.as_str() {
            MIME_UTF8 => {
                state.best_mime = Some(MIME_UTF8.to_string());
            }
            MIME_PLAIN if state.best_mime.is_none() => {
                state.best_mime = Some(MIME_PLAIN.to_string());
            }
            _ => {}
        }
    }
}

pub fn native_paste() -> Result<String> {
    let conn = Connection::connect_to_env()
        .context("cannot connect to Wayland display")?;
    let mut queue = conn.new_event_queue::<PasteState>();
    let qh = queue.handle();
    conn.display().get_registry(&qh, ());

    let mut state = PasteState::new();
    queue.roundtrip(&mut state)?;

    let seat: wl_seat::WlSeat = state.seat.take()
        .ok_or_else(|| anyhow!("no wl_seat found"))?;
    let manager: ZwlrDataControlManagerV1 = state.manager.take()
        .ok_or_else(|| anyhow!(
            "zwlr_data_control_manager_v1 not available — compositor must support wlr-data-control"
        ))?;

    let _device = manager.get_data_device(&seat, &qh, ());
    queue.roundtrip(&mut state)?;
    if !state.done {
        queue.roundtrip(&mut state)?;
    }

    let selection: ZwlrDataControlOfferV1 = match state.selection.take() {
        Some(s) => s,
        None => return Ok(String::new()),
    };

    let mime = state.best_mime.take()
        .ok_or_else(|| anyhow!("no text mime type in clipboard"))?;

    let (read_fd, write_fd) = make_pipe()?;
    selection.receive(mime, write_fd.as_fd());
    queue.flush()?;

    let mut content = String::new();
    File::from(read_fd).read_to_string(&mut content)?;
    Ok(content)
}

// ═══════════════════════════════════════════════════════════
// 📤 COPY DAEMON STATE
// ═══════════════════════════════════════════════════════════

struct CopyState {
    seat: Option<wl_seat::WlSeat>,
    manager: Option<ZwlrDataControlManagerV1>,
    content: String,
    done: bool,
}

impl CopyState {
    fn new(content: String) -> Self {
        Self { seat: None, manager: None, content, done: false }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for CopyState {
    fn event(state: &mut Self, registry: &wl_registry::WlRegistry,
             event: wl_registry::Event, _: &(), _: &Connection, qh: &QueueHandle<Self>) {
        let wl_registry::Event::Global { name, interface, version } = event else { return };
        match interface.as_str() {
            "wl_seat" => {
                state.seat = Some(registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ()));
            }
            "zwlr_data_control_manager_v1" => {
                state.manager = Some(
                    registry.bind::<ZwlrDataControlManagerV1, _, _>(name, version.min(2), qh, ())
                );
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for CopyState {
    fn event(_: &mut Self, _: &wl_seat::WlSeat, _: wl_seat::Event,
             _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<ZwlrDataControlManagerV1, ()> for CopyState {
    fn event(_: &mut Self, _: &ZwlrDataControlManagerV1,
             _: zwlr_data_control_manager_v1::Event,
             _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<ZwlrDataControlDeviceV1, ()> for CopyState {
    fn event(state: &mut Self, _: &ZwlrDataControlDeviceV1,
             event: zwlr_data_control_device_v1::Event,
             _: &(), _: &Connection, _: &QueueHandle<Self>) {
        if let zwlr_data_control_device_v1::Event::Finished = event {
            state.done = true;
        }
    }
}

impl Dispatch<ZwlrDataControlSourceV1, ()> for CopyState {
    fn event(state: &mut Self, _: &ZwlrDataControlSourceV1,
             event: zwlr_data_control_source_v1::Event,
             _: &(), _: &Connection, _: &QueueHandle<Self>) {
        match event {
            zwlr_data_control_source_v1::Event::Send { mime_type: _, fd } => {
                let mut f = File::from(fd);
                let _ = f.write_all(state.content.as_bytes());
            }
            zwlr_data_control_source_v1::Event::Cancelled => {
                state.done = true;
            }
            _ => {}
        }
    }
}

pub fn native_copy_daemon(content: String) -> Result<()> {
    let conn = Connection::connect_to_env()
        .context("cannot connect to Wayland display")?;
    let mut queue = conn.new_event_queue::<CopyState>();
    let qh = queue.handle();
    conn.display().get_registry(&qh, ());

    let mut state = CopyState::new(content);
    queue.roundtrip(&mut state)?;

    let seat: wl_seat::WlSeat = state.seat.take()
        .ok_or_else(|| anyhow!("no wl_seat found"))?;
    let manager: ZwlrDataControlManagerV1 = state.manager.take()
        .ok_or_else(|| anyhow!("zwlr_data_control_manager_v1 not available"))?;

    let source: ZwlrDataControlSourceV1 = manager.create_data_source(&qh, ());
    source.offer(MIME_UTF8.to_string());
    source.offer(MIME_PLAIN.to_string());

    let device: ZwlrDataControlDeviceV1 = manager.get_data_device(&seat, &qh, ());
    device.set_selection(Some(&source));
    queue.flush()?;

    while !state.done {
        queue.blocking_dispatch(&mut state)?;
    }
    Ok(())
}

// ─── HELPERS ────────────────────────────────────────────────────────────────

fn make_pipe() -> Result<(OwnedFd, OwnedFd)> {
    let mut fds = [0i32; 2];
    if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
        return Err(anyhow!("pipe() failed"));
    }
    Ok(unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) })
}
