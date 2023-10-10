#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod dll;
mod raw_input;
mod window;

use dll::RekeyDll;
use raw_input::RawInput;
use std::{fmt, fs::OpenOptions, io::Write};
use window::{create_window, message_loop};

fn main() {
    match _main() {
        Result::Ok(()) => {}
        Result::Err(err) => {
            debug(format!("main failed: {}", err));
        }
    };
}

fn _main() -> Result<(), RekeyError> {
    let window = create_window()?;

    let mut dll = RekeyDll::new()?;
    dll.install()?;

    let mut raw_input = RawInput::new(window)?;

    message_loop()?;

    dll.uninstall()?;
    raw_input.uninstall()?;

    return Result::Ok(());
}

#[derive(Debug)]
pub enum RekeyError {
    GenericError(String),
    Win32GetLastError(String, Result<(), windows::core::Error>),
    Win32Error(String, windows::core::Error),
}

impl fmt::Display for RekeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RekeyError::GenericError(s) => {
                write!(f, "Generic Error: {}", s)
            }
            RekeyError::Win32GetLastError(s, error) => {
                let error_as_string = match error {
                    Result::Ok(()) => "".to_string(),
                    Result::Err(e) => format!("{}", e),
                };
                write!(f, "Win32 Error: {}: {}", s, error_as_string)
            }
            RekeyError::Win32Error(s, error) => {
                write!(f, "Win32 Error: {}: {}", s, error)
            }
        }
    }
}

pub fn debug(s: String) -> () {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("C:\\dev\\rekey\\target\\out.txt");
    if let Result::Ok(mut f) = file {
        let _ = writeln!(&mut f, "{}", s).is_ok();
    }
    println!("{}", s);
}
