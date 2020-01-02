//! Interact with walletd daemon.

use std::env;
use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::process::Command;

use crate::error::Error;

pub const WALLETD_PID_FILE: &str = "/tmp/walletd.pid";

extern "C" {
    fn proc_name(pid: c_int, buffer: *mut c_char, buffersize: u32) -> c_int;
}

pub fn assure_walletd() -> Result<(), Error> {
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

        unsafe {
            let c_string = CString::from_vec_unchecked(Vec::with_capacity(64));
            let buffer = c_string.into_raw();
            let n = proc_name(pid, buffer, 64);
            let c_string = CString::from_raw(buffer);
            if n != 0 && c_string.to_str().expect("utf8 error") == "walletd" {
                return Ok(pid);
            }
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
