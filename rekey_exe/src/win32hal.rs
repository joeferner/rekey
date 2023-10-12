use std::ffi::{c_void, OsString};
use std::mem::size_of;
use std::os::windows::prelude::OsStringExt;
use windows::Win32::{
    Foundation::{GetLastError, HANDLE, LPARAM},
    UI::Input::{
        GetRawInputData, GetRawInputDeviceInfoW, GetRawInputDeviceList, RAWINPUT,
        RAWINPUTDEVICELIST, RAWINPUTHEADER, RIDI_DEVICENAME,
    },
    UI::Input::{HRAWINPUT, RID_INPUT},
};

use crate::RekeyError;

const MAX_RAW_INPUT_DEVICE_COUNT: usize = 1000;
const MAX_RAW_INPUT_DEVICE_INFO_DEVICE_NAME: usize = 2000;

pub fn get_raw_input_data(lparam: LPARAM) -> Result<RAWINPUT, RekeyError> {
    unsafe {
        let mut pcbsize: u32 = 0;
        let get_raw_input_data_size_result = GetRawInputData(
            HRAWINPUT(lparam.0),
            RID_INPUT,
            Option::None,
            &mut pcbsize,
            size_of::<RAWINPUTHEADER>() as u32,
        );
        if get_raw_input_data_size_result != 0 {
            return Result::Err(RekeyError::GenericError(
                "failed to get raw input data size".to_string(),
            ));
        }
        if pcbsize > size_of::<RAWINPUT>() as u32 {
            return Result::Err(RekeyError::GenericError(format!(
                "unexpected raw input size expected size less than {} but found {}",
                size_of::<RAWINPUT>() as u32,
                pcbsize
            )));
        }

        let mut raw_input: RAWINPUT = RAWINPUT::default();
        let get_raw_input_data_result = GetRawInputData(
            HRAWINPUT(lparam.0),
            RID_INPUT,
            Option::Some(std::ptr::addr_of_mut!(raw_input) as *mut c_void),
            &mut pcbsize,
            size_of::<RAWINPUTHEADER>() as u32,
        );
        if get_raw_input_data_result != pcbsize {
            return Result::Err(RekeyError::GenericError(
                "failed to get raw input data".to_string(),
            ));
        }

        return Result::Ok(raw_input);
    }
}

pub fn get_raw_input_device_list() -> Result<Vec<RAWINPUTDEVICELIST>, RekeyError> {
    unsafe {
        let mut device_count: u32 = 0;
        let get_raw_input_device_list_size_results = GetRawInputDeviceList(
            Option::None,
            &mut device_count,
            size_of::<RAWINPUTDEVICELIST>() as u32,
        );
        if get_raw_input_device_list_size_results == -1i32 as u32 {
            return Result::Err(RekeyError::Win32GetLastError(
                "failed GetRawInputDeviceList size".to_string(),
                GetLastError(),
            ));
        }
        if device_count as usize > MAX_RAW_INPUT_DEVICE_COUNT {
            return Result::Err(RekeyError::GenericError(format!(
                "too many raw input devices expected less than {} but found {}",
                MAX_RAW_INPUT_DEVICE_COUNT, device_count
            )));
        }

        let mut device_list: [RAWINPUTDEVICELIST; MAX_RAW_INPUT_DEVICE_COUNT] =
            [RAWINPUTDEVICELIST::default(); MAX_RAW_INPUT_DEVICE_COUNT];
        let get_raw_input_device_list_results = GetRawInputDeviceList(
            Option::Some(device_list.as_mut_ptr()),
            &mut device_count,
            size_of::<RAWINPUTDEVICELIST>() as u32,
        );
        if get_raw_input_device_list_results == -1i32 as u32 {
            return Result::Err(RekeyError::Win32GetLastError(
                "failed GetRawInputDeviceList items".to_string(),
                GetLastError(),
            ));
        }

        return Result::Ok(device_list[0..get_raw_input_device_list_results as usize].to_vec());
    }
}

pub fn get_raw_input_device_info_device_name(hdevice: HANDLE) -> Result<String, RekeyError> {
    unsafe {
        let mut pcbsize: u32 = 0;
        let get_raw_input_device_info_size_result =
            GetRawInputDeviceInfoW(hdevice, RIDI_DEVICENAME, Option::None, &mut pcbsize);
        if get_raw_input_device_info_size_result == -1i32 as u32 {
            return Result::Err(RekeyError::Win32GetLastError(
                "failed GetRawInputDeviceInfoW size".to_string(),
                GetLastError(),
            ));
        }
        if pcbsize as usize > MAX_RAW_INPUT_DEVICE_INFO_DEVICE_NAME {
            return Result::Err(RekeyError::GenericError(format!(
                "device name too big expected less than {} but found {}",
                MAX_RAW_INPUT_DEVICE_INFO_DEVICE_NAME, pcbsize
            )));
        }

        let mut device_name: [u16; MAX_RAW_INPUT_DEVICE_INFO_DEVICE_NAME] =
            [0; MAX_RAW_INPUT_DEVICE_INFO_DEVICE_NAME];
        let get_raw_input_device_info_result = GetRawInputDeviceInfoW(
            hdevice,
            RIDI_DEVICENAME,
            Option::Some(device_name.as_mut_ptr() as *mut c_void),
            &mut pcbsize,
        );
        if get_raw_input_device_info_result == -1i32 as u32 {
            return Result::Err(RekeyError::Win32GetLastError(
                "failed GetRawInputDeviceInfoW data".to_string(),
                GetLastError(),
            ));
        }

        let device_name_slice = &device_name[0..(get_raw_input_device_info_result - 1) as usize];
        let converted_string: OsString = OsStringExt::from_wide(device_name_slice);
        return match converted_string.into_string() {
            Ok(str) => Result::Ok(str),
            Err(_) => Result::Err(RekeyError::GenericError(format!(
                "could not convert bytes to string"
            ))),
        };
    }
}
