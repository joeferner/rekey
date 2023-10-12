use std::mem::size_of;

use windows::Win32::{
    Foundation::HWND,
    UI::Input::{RegisterRawInputDevices, RAWINPUTDEVICE, RIDEV_INPUTSINK},
};

use crate::RekeyError;

pub struct RawInput {}

const HID_KEYBOARD_USAGE_PAGE: u16 = 1;
const HID_KEYBOARD_USAGE: u16 = 6;

impl RawInput {
    pub fn new(window: HWND) -> Result<Self, RekeyError> {
        let raw_input_device: RAWINPUTDEVICE = RAWINPUTDEVICE {
            usUsagePage: HID_KEYBOARD_USAGE_PAGE,
            usUsage: HID_KEYBOARD_USAGE,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: window,
        };
        let raw_input_devices: [RAWINPUTDEVICE; 1] = [raw_input_device];
        let cbsize = size_of::<RAWINPUTDEVICE>();
        unsafe {
            RegisterRawInputDevices(&raw_input_devices, cbsize as u32).map_err(|err| {
                RekeyError::Win32Error("failed to register raw input devices".to_string(), err)
            })?;
        }
        return Result::Ok(RawInput {});
    }

    pub fn uninstall(&mut self) -> Result<(), RekeyError> {
        return Result::Ok(());
    }
}
