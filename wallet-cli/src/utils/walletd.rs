//! Interact with walletd daemon.

use std::env;
use std::fs;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::process::Command;
use std::str;

use crate::error::Error;

pub const WALLETD_PID_FILE: &str = "/tmp/walletd.pid";

extern "C" {
    fn proc_name(pid: c_int, buffer: *mut c_char, buffersize: u32) -> c_int;
}

pub fn ensure_walletd() -> Result<(), Error> {
    match get_walletd_pid() {
        Ok(_pid) => {
            // eprintln!("walletd: running at pid={:}", pid);
            Ok(())
        }
        Err(e) => {
            eprintln!("walletd: {:?}", e);
            run_walletd()
        }
    }
}

pub fn get_walletd_pid() -> Result<c_int, Error> {
    let pid_file = Path::new(WALLETD_PID_FILE);

    if pid_file.exists() {
        let pid: c_int = fs::read_to_string(&pid_file)
            .unwrap()
            .parse()
            .expect("pid file format error");

        let mut buffer = [0u8; 64];
        let n = unsafe { proc_name(pid, &mut buffer[0] as *mut u8 as *mut c_char, 64) };
        let name = str::from_utf8(&buffer[..n as usize]).unwrap();
        if n != 0 && name == "walletd" {
            return Ok(pid);
        }
    }

    Err(Error::Runtime("walletd process not found"))
}

pub fn run_walletd() -> Result<(), Error> {
    let walletd_path = env::current_exe()?.parent().unwrap().join("walletd");
    if walletd_path.exists() {
        Command::new(walletd_path)
            .status()
            .map_err(From::from)
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    Err(Error::Runtime("walletd exits abnormality"))
                }
            })
    } else {
        Err(Error::Runtime("walletd executable not found!"))
    }
}
