//! PTY (Pseudo-Terminal) module for spawning and managing shell processes
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::pty::{openpty, Winsize};
use nix::sys::termios::{self, LocalFlags, InputFlags, OutputFlags, SetArg};
use nix::unistd::{dup2, execvp, fork, setsid, ForkResult};
use std::ffi::CString;
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd};

pub struct Pty {
    pub master: File,
    #[allow(dead_code)]
    pub pid: i32,
}

impl Pty {
    /// Spawn a new shell process in a PTY
    #[allow(unreachable_code)]
    pub fn spawn_shell(rows: u16, cols: u16) -> io::Result<Self> {
        // Set up terminal window size
        let winsize = Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        // Get default termios from stdin or create minimal config
        let termios = if let Ok(t) = termios::tcgetattr(std::io::stdin()) {
            let mut t = t;
            // Enable signal processing (Ctrl+C, Ctrl+Z, etc)
            t.local_flags |= LocalFlags::ISIG;
            // Keep echo and canonical mode
            t.local_flags |= LocalFlags::ECHO | LocalFlags::ICANON;
            t.input_flags |= InputFlags::ICRNL;
            t.output_flags |= OutputFlags::OPOST | OutputFlags::ONLCR;
            Some(t)
        } else {
            None  // Fallback to no termios config
        };
        
        // Open a PTY (master/slave pair) with termios config
        let pty_result = openpty(&winsize, termios.as_ref())
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
                
                // Configure termios for signal processing on the slave (now stdin)
                unsafe {
                    use std::os::fd::BorrowedFd;
                    let stdin_fd = BorrowedFd::borrow_raw(0);
                    if let Ok(mut t) = termios::tcgetattr(&stdin_fd) {
                        t.local_flags |= LocalFlags::ISIG;  // Enable signals
                        t.local_flags |= LocalFlags::ECHO | LocalFlags::ICANON;
                        t.input_flags |= InputFlags::ICRNL;
                        t.output_flags |= OutputFlags::OPOST | OutputFlags::ONLCR;
                        let _ = termios::tcsetattr(&stdin_fd, SetArg::TCSANOW, &t);
                    }
                }
                
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
