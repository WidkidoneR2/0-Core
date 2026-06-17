//! faelight-lock-auth -- PAM authentication helper
//! Privileged component: reads username+password from stdin, authenticates via PAM
//! Outputs "OK" or "FAIL" to stdout. Exits immediately after.
//! 
//! Security design:
//! - No Wayland code
//! - No font rendering  
//! - No async runtime
//! - No network access
//! - One job: PAM auth via faelight-lock service
//! - Must be setuid root: sudo chown root:root faelight-lock-auth && sudo chmod u+s faelight-lock-auth
use pam::Client;
use std::io::{self, BufRead};
fn main() {
    // Read username and password from stdin (two lines)
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let username = match lines.next() {
        Some(Ok(u)) if !u.is_empty() => u,
        _ => {
            eprintln!("faelight-lock-auth: failed to read username");
            println!("FAIL");
            std::process::exit(1);
        }
    };
    let password = match lines.next() {
        Some(Ok(p)) => p,
        _ => {
            eprintln!("faelight-lock-auth: failed to read password");
            println!("FAIL");
            std::process::exit(1);
        }
    };
    // Authenticate via PAM using faelight-lock service
    match authenticate(&username, &password) {
        true => {
            println!("OK");
            std::process::exit(0);
        }
        false => {
            println!("FAIL");
            std::process::exit(1);
        }
    }
}
fn authenticate(username: &str, password: &str) -> bool {
    let mut auth: Client<_> = match Client::with_password("faelight-lock") {
        Ok(a) => a,
        Err(e) => {
            eprintln!("faelight-lock-auth: PAM init failed: {}", e);
            return false;
        }
    };
    auth.conversation_mut().set_credentials(username, password);
    match auth.authenticate() {
        Ok(()) => {
            match auth.open_session() {
                Ok(()) => true,
                Err(e) => {
                    eprintln!("faelight-lock-auth: session failed: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            eprintln!("faelight-lock-auth: auth failed: {}", e);
            false
        }
    }
}
