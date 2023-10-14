#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod devices;
mod dll;
mod input_log;
mod js;
mod raw_input;
mod scripts;
mod win32hal;
mod window;

use std::fs;

use dll::RekeyDll;
use raw_input::RawInput;
use rekey_common::{debug, get_log_filename, RekeyError};
use window::{
    add_systray_icon, create_window, delete_systray_icon, load_scripts_notify_on_error,
    message_loop,
};

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

    load_scripts_notify_on_error(window);

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
