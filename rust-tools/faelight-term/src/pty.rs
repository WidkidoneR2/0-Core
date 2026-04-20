//! faelight-term v2 -- PTY layer
//! Spawns shell in a pseudoterminal.
use nix::{
    pty::{openpty, OpenptyResult},
    unistd::{dup2, execvp, fork, setsid, ForkResult},
};
use std::{ffi::CString, os::unix::io::{AsRawFd, IntoRawFd, RawFd}};
pub struct Pty {
    pub master: RawFd,
}
impl Pty {
    pub fn spawn(shell: &str, cols: u16, rows: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let OpenptyResult { master, slave } = openpty(None, None)?;
        let master_raw = master.as_raw_fd();
        let slave_raw  = slave.as_raw_fd();
        let winsize = nix::pty::Winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 };
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
                execvp(&shell_c, &[shell_c.clone()]).ok();
                std::process::exit(1);
            }
            ForkResult::Parent { .. } => {
                drop(slave);
                let fd = master.into_raw_fd();
                // Set non-blocking
                unsafe {
                    let flags = nix::libc::fcntl(fd, nix::libc::F_GETFL);
                    nix::libc::fcntl(fd, nix::libc::F_SETFL, flags | nix::libc::O_NONBLOCK);
                }
                Ok(Self { master: fd })
            }
        }
    }
    pub fn write(&self, data: &[u8]) -> std::io::Result<usize> {
        let n = unsafe {
            nix::libc::write(self.master, data.as_ptr() as *const _, data.len())
        };
        if n < 0 { Err(std::io::Error::last_os_error()) } else { Ok(n as usize) }
    }
    pub fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = unsafe {
            nix::libc::read(self.master, buf.as_mut_ptr() as *mut _, buf.len())
        };
        if n < 0 {
            let e = std::io::Error::last_os_error();
            if e.raw_os_error() == Some(nix::libc::EAGAIN) ||
               e.raw_os_error() == Some(nix::libc::EWOULDBLOCK) {
                return Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, "would block"));
            }
            Err(e)
        } else {
            Ok(n as usize)
        }
    }
    pub fn resize(&self, cols: u16, rows: u16) {
        let winsize = nix::pty::Winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 };
        unsafe { nix::libc::ioctl(self.master, nix::libc::TIOCSWINSZ, &winsize) };
    }
}
impl Drop for Pty {
    fn drop(&mut self) {
        unsafe { nix::libc::close(self.master) };
    }
}
