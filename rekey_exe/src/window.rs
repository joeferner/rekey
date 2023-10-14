use rekey_common::{KeyDirection, DONT_SKIP_INPUT, SKIP_INPUT, WM_REKEY_SHOULD_SKIP_INPUT};
use std::mem::size_of;
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{GetLastError, BOOL, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{PeekMessageW, CW_USEDEFAULT, HMENU, PM_REMOVE},
        UI::{
            Input::RIM_TYPEKEYBOARD,
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
                LoadCursorW, PostQuitMessage, RegisterClassExW, ShowWindow, TranslateMessage,
                IDC_ARROW, MSG, SW_SHOW, WINDOW_EX_STYLE, WM_CLOSE, WM_DESTROY, WM_INPUT,
                WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSEXW, WS_CAPTION,
                WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_THICKFRAME,
            },
        },
    },
};

use crate::{
    debug,
    devices::find_device,
    input_log::{input_log_add_wm_input, input_log_get_device},
    should_skip_input,
    win32hal::get_raw_input_data,
    RekeyError, SkipInput,
};

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
            debug!("window proc error {}", err);
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
        WM_REKEY_SHOULD_SKIP_INPUT => {
            return handle_should_skip_input(hwnd, wparam, lparam);
        }
        _ => unsafe {
            return Result::Ok(DefWindowProcW(hwnd, msg, wparam, lparam));
        },
    }
}

fn handle_should_skip_input(
    hwnd: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Result<LRESULT, RekeyError> {
    let vkey_code = wparam.0 as u16;
    let direction = if lparam.0 >> 31 == 0 {
        KeyDirection::Down
    } else {
        KeyDirection::Up
    };
    let mut device = input_log_get_device(vkey_code, direction)?;
    if device.is_none() {
        process_waiting_input_messages(hwnd)?;
        device = input_log_get_device(vkey_code, direction)?;
    }

    let result = should_skip_input(vkey_code, direction, device)?;
    if result == SkipInput::Skip {
        return Result::Ok(SKIP_INPUT);
    } else {
        return Result::Ok(DONT_SKIP_INPUT);
    }
}

fn process_waiting_input_messages(hwnd: HWND) -> Result<(), RekeyError> {
    let mut msg: MSG = MSG::default();
    while unsafe { PeekMessageW(&mut msg, hwnd, WM_INPUT, WM_INPUT, PM_REMOVE).as_bool() } {
        handle_wm_input(msg.hwnd, msg.message, msg.wParam, msg.lParam)?;
    }
    return Result::Ok(());
}

fn handle_wm_input(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Result<LRESULT, RekeyError> {
    let raw_input_data = get_raw_input_data(lparam)?;
    if raw_input_data.header.dwType == RIM_TYPEKEYBOARD.0 {
        let keyboard_message = unsafe { raw_input_data.data.keyboard.Message };
        let vkey_code = unsafe { raw_input_data.data.keyboard.VKey };
        let direction = match keyboard_message {
            WM_KEYDOWN => KeyDirection::Down,
            WM_SYSKEYDOWN => KeyDirection::Down,
            WM_KEYUP => KeyDirection::Up,
            WM_SYSKEYUP => KeyDirection::Up,
            _ => KeyDirection::Down,
        };
        let device = find_device(raw_input_data.header.hDevice)?;
        input_log_add_wm_input(device, vkey_code, direction)?;
    }
    unsafe {
        return Result::Ok(DefWindowProcW(hwnd, msg, wparam, lparam));
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
