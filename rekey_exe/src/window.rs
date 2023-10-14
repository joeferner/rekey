use rekey_common::{
    get_log_filename, get_scripts_dir, KeyDirection, DONT_SKIP_INPUT, SKIP_INPUT,
    WM_USER_SHELL_ICON, WM_USER_SHOULD_SKIP_INPUT,
};
use std::mem::size_of;
use windows::{
    core::{w, HSTRING, PCWSTR},
    Win32::{
        Foundation::{GetLastError, BOOL, HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::RIM_TYPEKEYBOARD,
            Shell::{
                ShellExecuteW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_DELETE, NOTIFYICONDATAW,
                NOTIFY_ICON_DATA_FLAGS,
            },
            WindowsAndMessaging::{
                CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
                GetCursorPos, GetMessageW, InsertMenuW, LoadCursorW, LoadIconW, MessageBoxW,
                PostMessageW, PostQuitMessage, RegisterClassExW, TrackPopupMenu, TranslateMessage,
                IDC_ARROW, MB_ICONEXCLAMATION, MB_RETRYCANCEL, MF_BYPOSITION, MF_STRING, MSG,
                SW_NORMAL, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_LEFTBUTTON, WINDOW_EX_STYLE,
                WM_CLOSE, WM_COMMAND, WM_DESTROY, WM_INPUT, WM_KEYDOWN, WM_KEYUP, WM_RBUTTONDOWN,
                WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSEXW, WS_CAPTION, WS_MAXIMIZEBOX,
                WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_THICKFRAME, IDRETRY,
            },
        },
        UI::{
            Shell::{Shell_NotifyIconW, NIM_ADD},
            WindowsAndMessaging::{PeekMessageW, CW_USEDEFAULT, HMENU, PM_REMOVE},
        },
    },
};

use crate::{
    debug,
    devices::find_device,
    input_log::{input_log_add_wm_input, input_log_get_device},
    scripts::{scripts_handle_input, scripts_load},
    win32hal::get_raw_input_data,
    RekeyError, SkipInput,
};

const SYS_TRAY_ID: u32 = 1001;

const ID_MENU_EXIT: usize = 1;
const ID_MENU_RELOAD_SCRIPTS: usize = 2;
const ID_MENU_OPEN_SCRIPTS_FOLDER: usize = 3;
const ID_MENU_OPEN_LOG: usize = 4;

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
        WM_USER_SHOULD_SKIP_INPUT => {
            return handle_should_skip_input(hwnd, wparam, lparam);
        }
        WM_USER_SHELL_ICON => {
            return handle_shell_icon(hwnd, wparam, lparam);
        }
        WM_COMMAND => {
            return handle_menu_click(hwnd, wparam, lparam);
        }
        _ => unsafe {
            return Result::Ok(DefWindowProcW(hwnd, msg, wparam, lparam));
        },
    }
}

fn handle_menu_click(hwnd: HWND, wparam: WPARAM, _lparam: LPARAM) -> Result<LRESULT, RekeyError> {
    match wparam.0 {
        ID_MENU_EXIT => unsafe {
            PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).map_err(|err| {
                RekeyError::GenericError(format!("failed to close window: {}", err))
            })?;
            return Result::Ok(LRESULT(0));
        },
        ID_MENU_OPEN_LOG => unsafe {
            let log_file = HSTRING::from(get_log_filename()?.as_os_str());
            ShellExecuteW(
                hwnd,
                w!("open"),
                &log_file,
                PCWSTR::null(),
                PCWSTR::null(),
                SW_NORMAL,
            );
            return Result::Ok(LRESULT(0));
        },
        ID_MENU_OPEN_SCRIPTS_FOLDER => unsafe {
            let scripts_dir = HSTRING::from(get_scripts_dir()?.as_os_str());
            ShellExecuteW(
                hwnd,
                w!("explore"),
                &scripts_dir,
                PCWSTR::null(),
                PCWSTR::null(),
                SW_NORMAL,
            );
            return Result::Ok(LRESULT(0));
        },
        ID_MENU_RELOAD_SCRIPTS => {
            load_scripts_notify_on_error(hwnd);
            return Result::Ok(LRESULT(0));
        }
        _ => {
            return Result::Ok(LRESULT(0));
        }
    }
}

fn handle_shell_icon(hwnd: HWND, _wparam: WPARAM, lparam: LPARAM) -> Result<LRESULT, RekeyError> {
    let msg = (lparam.0 & 0xffff) as u32;
    match msg {
        WM_RBUTTONDOWN => {
            return handle_shell_icon_right_click(hwnd);
        }
        _ => {
            return Result::Ok(LRESULT(0));
        }
    }
}

fn handle_shell_icon_right_click(hwnd: HWND) -> Result<LRESULT, RekeyError> {
    unsafe {
        let mut click_point = POINT::default();
        GetCursorPos(&mut click_point).map_err(|err| {
            RekeyError::GenericError(format!("failed to get cursor pos: {}", err))
        })?;

        let menu = CreatePopupMenu().map_err(|err| {
            RekeyError::GenericError(format!("failed to create popup menu: {}", err))
        })?;

        InsertMenuW(
            menu,
            0xFFFFFFFF,
            MF_BYPOSITION | MF_STRING,
            ID_MENU_RELOAD_SCRIPTS,
            w!("Reload Scripts"),
        )
        .map_err(|err| RekeyError::GenericError(format!("failed to insert menu item: {}", err)))?;

        InsertMenuW(
            menu,
            0xFFFFFFFF,
            MF_BYPOSITION | MF_STRING,
            ID_MENU_OPEN_SCRIPTS_FOLDER,
            w!("Open Scripts Folder"),
        )
        .map_err(|err| RekeyError::GenericError(format!("failed to insert menu item: {}", err)))?;

        InsertMenuW(
            menu,
            0xFFFFFFFF,
            MF_BYPOSITION | MF_STRING,
            ID_MENU_OPEN_LOG,
            w!("Open Log"),
        )
        .map_err(|err| RekeyError::GenericError(format!("failed to insert menu item: {}", err)))?;

        InsertMenuW(
            menu,
            0xFFFFFFFF,
            MF_BYPOSITION | MF_STRING,
            ID_MENU_EXIT,
            w!("Exit"),
        )
        .map_err(|err| RekeyError::GenericError(format!("failed to insert menu item: {}", err)))?;

        TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_LEFTBUTTON | TPM_BOTTOMALIGN,
            click_point.x,
            click_point.y,
            0,
            hwnd,
            Option::None,
        )
        .map_err(|err| RekeyError::GenericError(format!("failed to track popup menu: {}", err)))?;
    }
    return Result::Ok(LRESULT(0));
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

    let result = scripts_handle_input(vkey_code, direction, device)?;
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

fn get_hinstance() -> Result<HINSTANCE, RekeyError> {
    unsafe {
        let instance = GetModuleHandleW(PCWSTR::null())
            .map_err(|err| RekeyError::Win32Error("failed GetModuleHandleW".to_string(), err))?;
        return Result::Ok(HINSTANCE(instance.0));
    }
}

pub fn add_systray_icon(hwnd: HWND) -> Result<(), RekeyError> {
    unsafe {
        let hinstance = get_hinstance()?;

        let mut tray_tooltip = [0; 128];
        let tray_tooltip_w = w!("ReKey");
        let tray_tooltip_wide = tray_tooltip_w.as_wide();
        tray_tooltip[..tray_tooltip_wide.len()].copy_from_slice(&tray_tooltip_wide);

        let main_icon = LoadIconW(hinstance, w!("ICON_REKEY"))
            .map_err(|err| RekeyError::GenericError(format!("failed to load icon: {}", err)))?;

        let mut notify_icon_data = NOTIFYICONDATAW::default();
        notify_icon_data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        notify_icon_data.hWnd = hwnd;
        notify_icon_data.uID = SYS_TRAY_ID;
        notify_icon_data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        notify_icon_data.hIcon = main_icon;
        notify_icon_data.uCallbackMessage = WM_USER_SHELL_ICON;
        notify_icon_data.szTip = tray_tooltip;
        if !Shell_NotifyIconW(NIM_ADD, &notify_icon_data).as_bool() {
            return Result::Err(RekeyError::GenericError(
                "failed Shell_NotifyIcon".to_string(),
            ));
        }

        return Result::Ok(());
    }
}

pub fn delete_systray_icon(hwnd: HWND) -> Result<(), RekeyError> {
    unsafe {
        let mut notify_icon_data = NOTIFYICONDATAW::default();
        notify_icon_data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        notify_icon_data.hWnd = hwnd;
        notify_icon_data.uID = SYS_TRAY_ID;
        notify_icon_data.uFlags = NOTIFY_ICON_DATA_FLAGS(0);
        if !Shell_NotifyIconW(NIM_DELETE, &notify_icon_data).as_bool() {
            return Result::Err(RekeyError::GenericError(
                "failed delete Shell_NotifyIcon".to_string(),
            ));
        }

        return Result::Ok(());
    }
}

pub fn create_window() -> Result<HWND, RekeyError> {
    unsafe {
        let hinstance = get_hinstance()?;
        let window_class_name = w!("rekey");

        let mut wnd_class = WNDCLASSEXW::default();
        wnd_class.cbSize = size_of::<WNDCLASSEXW>() as u32;
        wnd_class.lpfnWndProc = Option::Some(window_proc_system);
        wnd_class.hInstance = hinstance;
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
            hinstance,
            Option::None,
        );

        return Result::Ok(window);
    }
}

pub fn load_scripts_notify_on_error(hwnd: HWND) -> () {
    if let Result::Err(err) = scripts_load() {
        unsafe {
            let message = HSTRING::from(format!("{}", err));
            let results = MessageBoxW(
                hwnd,
                &message,
                w!("Error Loading Scripts"),
                MB_ICONEXCLAMATION | MB_RETRYCANCEL,
            );
            if results == IDRETRY {
                load_scripts_notify_on_error(hwnd);
                return;
            }
        }
    }
}
