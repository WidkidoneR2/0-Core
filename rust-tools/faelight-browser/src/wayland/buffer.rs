//! Buffer management for rendering
//! Based on faelight-bar patterns

use smithay_client_toolkit::shm::{slot::SlotPool, Shm};
use wayland_client::protocol::wl_shm;

pub struct BufferManager {
    pool: SlotPool,
}

impl BufferManager {
    pub fn new(shm: &Shm) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = SlotPool::new(1920 * 1080 * 4, shm)?;
        Ok(Self { pool })
    }

    pub fn create_buffer(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<(wayland_client::protocol::wl_buffer::WlBuffer, &mut [u8]), Box<dyn std::error::Error>> {
        let stride = width as i32 * 4;
        self.pool
            .create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888)
            .map_err(|e| e.into())
    }
}
