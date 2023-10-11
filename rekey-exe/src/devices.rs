use lazy_static::lazy_static;
use std::sync::{Arc, Mutex, MutexGuard};

use windows::Win32::Foundation::HANDLE;

use crate::{
    win32hal::{get_raw_input_device_info_device_name, get_raw_input_device_list},
    RekeyError,
};

pub struct Device {
    pub hdevice: HANDLE,
    pub device_name: String,
}

lazy_static! {
    static ref DEVICES: Mutex<Vec<Arc<Device>>> = Mutex::new(vec![]);
}

pub fn find_device(hdevice: HANDLE) -> Result<Arc<Device>, RekeyError> {
    let mut devices = DEVICES
        .lock()
        .map_err(|err| RekeyError::GenericError(format!("could not get devices lock: {}", err)))?;

    for device in devices.iter() {
        if device.hdevice == hdevice {
            return Result::Ok(device.clone());
        }
    }

    // update the list and try again
    update_device_list(&mut devices)?;
    for device in devices.iter() {
        if device.hdevice == hdevice {
            return Result::Ok(device.clone());
        }
    }

    // if all else fails create an unknown device and return that
    let device = Arc::new(Device {
        hdevice,
        device_name: "Unknown".to_string(),
    });
    devices.push(device.clone());

    return Result::Ok(device);
}

fn update_device_list(devices: &mut MutexGuard<Vec<Arc<Device>>>) -> Result<(), RekeyError> {
    let device_list = get_raw_input_device_list()?;
    for device in device_list {
        let device_name = get_raw_input_device_info_device_name(device.hDevice)?;
        devices.push(Arc::new(Device {
            hdevice: device.hDevice,
            device_name,
        }));
    }
    return Result::Ok(());
}
