#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod devices;
mod dll;
mod input_log;
mod raw_input;
mod win32hal;
mod window;

use std::sync::Arc;

use devices::Device;
use dll::RekeyDll;
use raw_input::RawInput;
use rekey_common::{debug, KeyDirection, RekeyError};
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

pub fn should_skip_input(
    vkey_code: u16,
    direction: KeyDirection,
    device: Option<Arc<Device>>,
) -> Result<bool, RekeyError> {
    return Result::Ok(true);
}
