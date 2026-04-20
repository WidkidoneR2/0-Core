//! faelight-term v2 -- PTY layer
//! Spawns faelight-shell in a pseudoterminal.
use nix::{
    pty::{openpty, OpenptyResult},
    unistd::{dup2, execvp, fork, setsid, ForkResult},
};
use std::{
    ffi::CString,
    os::unix::io::{IntoRawFd, RawFd},
};
pub struct Pty {
    pub master: RawFd,
}
impl Pty {
    pub fn spawn(shell: &str, cols: u16, rows: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let OpenptyResult { master, slave } = openpty(None, None)?;
        // Set terminal size via raw fd
        let master_raw = master.as_raw_fd();
        let slave_raw  = slave.as_raw_fd();
        let winsize = nix::pty::Winsize {
            ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0
        };
        unsafe { nix::libc::ioctl(master_raw, nix::libc::TIOCSWINSZ, &winsize) };
        match unsafe { fork()? } {
            ForkResult::Child => {
                drop(master);
                setsid().ok();
                unsafe { nix::libc::ioctl(slave_raw, nix::libc::TIOCSCTTY, 0) };
                dup2(slave_raw, 0).ok();
                dup2(slave_raw, 1).ok();
                dup2(slave_raw, 2).ok();
                drop(slave);
                let shell_c = CString::new(shell).unwrap();
                let args = vec![shell_c.clone()];
                execvp(&shell_c, &args).ok();
                std::process::exit(1);
            }
            ForkResult::Parent { .. } => {
                drop(slave);
                let master_fd = master.into_raw_fd();
                Ok(Self { master: master_fd })
            }
        }
    }
    pub fn write(&self, data: &[u8]) -> std::io::Result<usize> {
        use std::io::Write;
        let mut f = unsafe { std::fs::File::from_raw_fd(self.master) };
        let n = f.write(data)?;
        std::mem::forget(f);
        Ok(n)
    }
    pub fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::Read;
        let mut f = unsafe { std::fs::File::from_raw_fd(self.master) };
        let n = f.read(buf)?;
        std::mem::forget(f);
        Ok(n)
    }
    pub fn resize(&self, cols: u16, rows: u16) {
        let winsize = nix::pty::Winsize {
            ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0
        };
        unsafe { nix::libc::ioctl(self.master, nix::libc::TIOCSWINSZ, &winsize) };
    }
}
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
