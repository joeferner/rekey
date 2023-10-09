use std::{fmt, fs::OpenOptions, io::Write, mem::size_of};
use windows::{
    core::{s, w, PCWSTR},
    Win32::{
        Foundation::{
            FreeLibrary, GetLastError, BOOL, HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, WPARAM,
        },
        System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
            LoadCursorW, PostQuitMessage, RegisterClassExW, ShowWindow, TranslateMessage,
            IDC_ARROW, MSG, SW_SHOW, WINDOW_EX_STYLE, WM_CLOSE, WM_DESTROY, WNDCLASSEXW,
            WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_THICKFRAME,
        },
        UI::{
            Input::{RegisterRawInputDevices, RAWINPUTDEVICE, RIDEV_INPUTSINK},
            WindowsAndMessaging::{CW_USEDEFAULT, HMENU},
        },
    },
};

type PROC = unsafe extern "system" fn() -> isize;
type FnInstall = extern "stdcall" fn(dll: HMODULE) -> ();
type FnUninstall = extern "stdcall" fn() -> ();

const HID_KEYBOARD_USAGE_PAGE: u16 = 1;
const HID_KEYBOARD_USAGE: u16 = 6;

#[derive(Debug)]
pub enum RekeyError {
    Win32GetLastError(String, Result<(), windows::core::Error>),
    Win32Error(String, windows::core::Error),
}

impl fmt::Display for RekeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RekeyError::Win32GetLastError(s, error) => {
                let error_as_string = match error {
                    Result::Ok(()) => "".to_string(),
                    Result::Err(e) => format!("{}", e),
                };
                write!(f, "Win32 Error: {}: {}", s, error_as_string)
            }
            RekeyError::Win32Error(s, error) => {
                write!(f, "Win32 Error: {}: {}", s, error)
            }
        }
    }
}

fn debug(s: String) -> () {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("C:\\dev\\rekey\\target\\out.txt");
    if let Result::Ok(mut f) = file {
        let _ = writeln!(&mut f, "{}", s).is_ok();
    }
    println!("{}", s);
}

fn main() {
    match run() {
        Err(err) => {
            debug(format!("{}", err));
        }
        Ok(()) => {}
    }
}

fn run() -> Result<(), RekeyError> {
    debug("begin".to_string());
    unsafe {
        let window = create_window()?;

        let dll = LoadLibraryW(w!("rekey_lib.dll")).expect("failed to load dll");
        let install_bare = GetProcAddress(dll, s!("install")).expect("failed to find install");
        let install = std::mem::transmute::<PROC, FnInstall>(install_bare);
        let uninstall_bare =
            GetProcAddress(dll, s!("uninstall")).expect("failed to find uninstall");
        let uninstall = std::mem::transmute::<PROC, FnUninstall>(uninstall_bare);

        install(dll);

        let raw_input_device: RAWINPUTDEVICE = RAWINPUTDEVICE {
            usUsagePage: HID_KEYBOARD_USAGE_PAGE,
            usUsage: HID_KEYBOARD_USAGE,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: window,
        };
        let raw_input_devices: [RAWINPUTDEVICE; 1] = [raw_input_device];
        if raw_input_devices.len() > 4 {
            RegisterRawInputDevices(&raw_input_devices, 1)
                .expect("failed to RegisterRawInputDevices");
        }

        message_loop()?;
        println!("freeing");

        uninstall();
        FreeLibrary(dll).expect("failed to free");

        return Result::Ok(());
    }
}

fn message_loop() -> Result<(), RekeyError> {
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
        _ => unsafe {
            return Result::Ok(DefWindowProcW(hwnd, msg, wparam, lparam));
        },
    }
    return Result::Ok(LRESULT(0));
}

fn create_window() -> Result<HWND, RekeyError> {
    unsafe {
        let instance = GetModuleHandleW(PCWSTR::null()).expect("failed GetModuleHandleW");

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
