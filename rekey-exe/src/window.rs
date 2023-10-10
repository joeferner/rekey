use std::ffi::c_void;
use std::mem::size_of;
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{GetLastError, BOOL, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::{GetRawInputData, RAWINPUT, RAWINPUTHEADER},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
                LoadCursorW, PostQuitMessage, RegisterClassExW, ShowWindow, TranslateMessage,
                IDC_ARROW, MSG, SW_SHOW, WINDOW_EX_STYLE, WM_CLOSE, WM_DESTROY, WM_INPUT,
                WNDCLASSEXW, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU,
                WS_THICKFRAME,
            },
        },
        UI::{
            Input::{HRAWINPUT, RID_INPUT},
            WindowsAndMessaging::{CW_USEDEFAULT, HMENU},
        },
    },
};

use crate::{debug, RekeyError};

pub fn message_loop() -> Result<(), RekeyError> {
    unsafe {
        let mut msg: MSG = MSG::default();
        loop {
            let message_return = GetMessageW(&mut msg, HWND::default(), 0, 0);
            if message_return == BOOL(0) {
                break;
            } else if message_return == BOOL(-1) {
                return Result::Err(RekeyError::Win32GetLastError(
                    "GetMessageW".to_string(),
                    GetLastError(),
                ));
            } else {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
    return Result::Ok(());
}

unsafe extern "system" fn window_proc_system(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match window_proc(hwnd, msg, wparam, lparam) {
        Ok(r) => {
            return r;
        }
        Err(err) => {
            debug(format!("window proc error {}", err));
            return LRESULT(0);
        }
    }
}

fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Result<LRESULT, RekeyError> {
    match msg {
        WM_CLOSE => unsafe {
            DestroyWindow(hwnd)
                .map_err(|e| RekeyError::Win32Error("DestroyWindow".to_string(), e))?;
            return Result::Ok(LRESULT(0));
        },
        WM_DESTROY => unsafe {
            PostQuitMessage(0);
            return Result::Ok(LRESULT(0));
        },
        WM_INPUT => {
            return handle_wm_input(hwnd, msg, wparam, lparam);
        }
        _ => unsafe {
            return Result::Ok(DefWindowProcW(hwnd, msg, wparam, lparam));
        },
    }
}

fn handle_wm_input(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Result<LRESULT, RekeyError> {
    unsafe {
        let raw_input_data = get_raw_input_data(lparam)?;
        println!("dwSize: {}", raw_input_data.header.dwSize);
        println!("dwType: {}", raw_input_data.header.dwType);
        println!("hDevice: {:#04x}", raw_input_data.header.hDevice.0);
        return Result::Ok(DefWindowProcW(hwnd, msg, wparam, lparam));
    }
}

fn get_raw_input_data(lparam: LPARAM) -> Result<RAWINPUT, RekeyError> {
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

pub fn create_window() -> Result<HWND, RekeyError> {
    unsafe {
        let instance = GetModuleHandleW(PCWSTR::null())
            .map_err(|err| RekeyError::Win32Error("failed GetModuleHandleW".to_string(), err))?;

        let window_class_name = w!("rekey");

        let mut wnd_class = WNDCLASSEXW::default();
        wnd_class.cbSize = size_of::<WNDCLASSEXW>() as u32;
        wnd_class.lpfnWndProc = Option::Some(window_proc_system);
        wnd_class.hInstance = HINSTANCE::from(instance);
        wnd_class.lpszClassName = window_class_name;
        wnd_class.hCursor = LoadCursorW(HINSTANCE::default(), IDC_ARROW)
            .map_err(|e| RekeyError::Win32Error("LoadCursorW".to_string(), e))?;
        if RegisterClassExW(&wnd_class) == 0 {
            return Result::Err(RekeyError::Win32GetLastError(
                "failed to register class".to_string(),
                GetLastError(),
            ));
        }

        let window = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            window_class_name,
            w!("ReKey"),
            WS_OVERLAPPED
                | WS_CAPTION
                | WS_SYSMENU
                | WS_THICKFRAME
                | WS_MINIMIZEBOX
                | WS_MAXIMIZEBOX,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            HWND(0),
            HMENU(0),
            instance,
            Option::None,
        );

        ShowWindow(window, SW_SHOW);

        return Result::Ok(window);
    }
}
