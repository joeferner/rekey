use std::mem::size_of;
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{GetLastError, BOOL, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
            LoadCursorW, PostQuitMessage, RegisterClassExW, ShowWindow, TranslateMessage,
            IDC_ARROW, MSG, SW_SHOW, WINDOW_EX_STYLE, WM_CLOSE, WM_DESTROY, WM_INPUT, WNDCLASSEXW,
            WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_THICKFRAME,
        },
        UI::WindowsAndMessaging::{CW_USEDEFAULT, HMENU},
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
        },
        WM_DESTROY => unsafe {
            PostQuitMessage(0);
        },
        WM_INPUT => {
            println!("WM_INPUT");
            return Result::Ok(LRESULT(0));
        }
        _ => unsafe {
            return Result::Ok(DefWindowProcW(hwnd, msg, wparam, lparam));
        },
    }
    return Result::Ok(LRESULT(0));
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
