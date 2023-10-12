#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod devices;
mod dll;
mod raw_input;
mod win32hal;
mod window;
mod input_log;

use dll::RekeyDll;
use raw_input::RawInput;
use rekey_common::{debug, RekeyError};
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
    dll.install(window)?;

    let mut raw_input = RawInput::new(window)?;

    message_loop()?;

    dll.uninstall()?;
    raw_input.uninstall()?;

    return Result::Ok(());
}
