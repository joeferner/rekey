#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod devices;
mod dll;
mod input_log;
mod js;
mod raw_input;
mod scripts;
mod win32hal;
mod window;

use std::{fs, sync::Arc};

use devices::Device;
use dll::RekeyDll;
use raw_input::RawInput;
use rekey_common::{debug, get_log_filename, KeyDirection, RekeyError};
use scripts::{scripts_handle_input, scripts_load};
use window::{add_systray_icon, create_window, message_loop, delete_systray_icon};

fn main() {
    match _main() {
        Result::Ok(()) => {}
        Result::Err(err) => {
            debug!("main failed: {}", err);
        }
    };
}

fn _main() -> Result<(), RekeyError> {
    reset_log_file()?;
    debug("BEGIN");

    let window = create_window()?;
    add_systray_icon(window)?;

    scripts_load()?;

    let mut dll = RekeyDll::new()?;
    dll.install(window)?;

    let mut raw_input = RawInput::new(window)?;

    message_loop()?;

    dll.uninstall()?;
    raw_input.uninstall()?;
    delete_systray_icon(window)?;

    debug("END");
    return Result::Ok(());
}

fn reset_log_file() -> Result<(), RekeyError> {
    let log_filename = get_log_filename()?;
    fs::create_dir_all(log_filename.parent().unwrap_or(&log_filename))?;
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_filename)?;
    return Result::Ok(());
}

#[derive(PartialEq, Eq)]
pub enum SkipInput {
    Skip,
    DontSkip,
}

pub fn should_skip_input(
    vkey_code: u16,
    direction: KeyDirection,
    device: Option<Arc<Device>>,
) -> Result<SkipInput, RekeyError> {
    return scripts_handle_input(vkey_code, direction, device);
}
