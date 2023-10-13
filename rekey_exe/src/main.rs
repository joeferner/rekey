#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod devices;
mod dll;
mod input_log;
mod js;
mod raw_input;
mod scripts;
mod win32hal;
mod window;

use directories::ProjectDirs;
use std::{path::PathBuf, sync::Arc};

use devices::Device;
use dll::RekeyDll;
use raw_input::RawInput;
use rekey_common::{debug, KeyDirection, RekeyError};
use scripts::{scripts_handle_input, scripts_load};
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

    scripts_load()?;

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
    return scripts_handle_input(vkey_code, direction, device);
}

pub fn get_project_config_dir() -> Result<PathBuf, RekeyError> {
    match ProjectDirs::from("com.github", "joeferner", "rekey") {
        Option::Some(proj_dir) => {
            return Result::Ok(proj_dir.config_dir().to_path_buf());
        }
        Option::None => {
            return Result::Err(RekeyError::GenericError(
                "could not get project dir".to_string(),
            ));
        }
    }
}
