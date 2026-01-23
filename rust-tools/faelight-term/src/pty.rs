//! PTY (Pseudo-Terminal) module for spawning and managing shell processes
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::pty::{openpty, Winsize};
use nix::unistd::{dup2, execvp, fork, setsid, ForkResult};
use std::ffi::CString;
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::{AsFd, AsRawFd, FromRawFd, IntoRawFd};

pub struct Pty {
    pub master: File,
    pub pid: i32,
}

impl Pty {
    /// Spawn a new shell process in a PTY
    pub fn spawn_shell(rows: u16, cols: u16) -> io::Result<Self> {
        // Set up terminal window size
        let winsize = Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        // Open a PTY (master/slave pair)
        let pty_result = openpty(&winsize, None)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Fork the process
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                // Parent process: close slave, keep master
                drop(pty_result.slave); // OwnedFd will auto-close
                
                // Convert master to raw fd
                let master_fd = pty_result.master.as_raw_fd();
                
                // Make master non-blocking
                fcntl(master_fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                
                // Convert to File (consuming the OwnedFd)
                let master = unsafe { File::from_raw_fd(pty_result.master.into_raw_fd()) };
                
                Ok(Pty {
                    master,
                    pid: child.as_raw(),
                })
            }
            Ok(ForkResult::Child) => {
                // Child process: set up slave as controlling terminal
                
                // Create new session
                setsid().expect("Failed to create new session");
                
                // Get raw fds
                let slave_fd = pty_result.slave.as_raw_fd();
                
                // Redirect stdin/stdout/stderr to slave
                dup2(slave_fd, 0).expect("Failed to dup2 stdin");
                dup2(slave_fd, 1).expect("Failed to dup2 stdout");
                dup2(slave_fd, 2).expect("Failed to dup2 stderr");
                
                // Close master and slave in child (fds 0,1,2 are now connected)
                drop(pty_result.master);
                drop(pty_result.slave);
                
                // Execute shell
                let shell = CString::new("/bin/zsh").unwrap();
                let args = [
                    CString::new("zsh").unwrap(),
                ];
                
                execvp(&shell, &args).expect("Failed to exec shell");
                unreachable!()
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }
    
    /// Read available data from PTY
    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.master.read(buf)
    }
    
    /// Write data to PTY (send to shell)
    pub fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        use std::io::Write;
        self.master.write(data)
    }
}
