use lazy_static::lazy_static;
use std::{
    env,
    fs::File,
    io::{Read, Write},
    sync::Mutex,
};

use rekey_common::{debug, RekeyError, SKIP_INPUT, WM_REKEY_SHOULD_SKIP_INPUT};
use windows::{
    core::s,
    Win32::{
        Foundation::{HMODULE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetProcAddress,
        UI::WindowsAndMessaging::{
            CallNextHookEx, SendMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, HHOOK,
            HOOKPROC, WH_KEYBOARD,
        },
    },
};

type PROC = unsafe extern "system" fn() -> isize;

struct GlobalData {
    hhook: HHOOK,
    hwnd: HWND,
}

lazy_static! {
    static ref MY_DATA: Mutex<Option<GlobalData>> = Mutex::new(Option::None);
}

#[no_mangle]
pub extern "C" fn install(dll: HMODULE, hwnd: HWND) -> bool {
    match _install(dll, hwnd) {
        Result::Err(err) => {
            debug(format!("install failed {}", err));
            return false;
        }
        Result::Ok(()) => {
            return true;
        }
    };
}

fn _install(dll: HMODULE, hwnd: HWND) -> Result<(), RekeyError> {
    let mut data = MY_DATA
        .lock()
        .map_err(|err| RekeyError::GenericError(format!("could not get data lock: {}", err)))?;

    unsafe {
        if data.is_some() {
            return Result::Err(RekeyError::GenericError("already installed".to_string()));
        }

        let keyboard_hook_bare = GetProcAddress(dll, s!("keyboard_hook"))
            .ok_or_else(|| RekeyError::GenericError("failed to find keyboard_hook".to_string()))?;
        let keyboard_hook =
            std::mem::transmute::<Option<PROC>, HOOKPROC>(Option::Some(keyboard_hook_bare));
        let hhook = SetWindowsHookExW(WH_KEYBOARD, keyboard_hook, dll, 0)
            .map_err(|err| RekeyError::Win32Error("failed to set hook".to_string(), err))?;

        let d = GlobalData { hhook, hwnd };
        write_global_data(&d)?;
        *data = Option::Some(d);
    }
    debug("installed".to_string());
    return Result::Ok(());
}

#[no_mangle]
pub extern "C" fn uninstall() -> bool {
    match _uninstall() {
        Result::Err(err) => {
            debug(format!("uninstall failed {}", err));
            return false;
        }
        Result::Ok(()) => {
            return true;
        }
    };
}

fn _uninstall() -> Result<(), RekeyError> {
    let mut data = MY_DATA
        .lock()
        .map_err(|err| RekeyError::GenericError(format!("could not get data lock: {}", err)))?;

    unsafe {
        if data.is_none() {
            return Result::Err(RekeyError::GenericError("not installed".to_string()));
        }

        if let Some(d) = data.as_ref() {
            UnhookWindowsHookEx(d.hhook)
                .map_err(|err| RekeyError::Win32Error("failed to unhook".to_string(), err))?;
            *data = Option::None;
        }
    }
    debug("uninstalled".to_string());
    return Result::Ok(());
}

#[no_mangle]
pub extern "C" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match _keyboard_hook(code, wparam, lparam) {
        Result::Err(err) => {
            debug(format!("keyboard_hook failed {}", err));
            return LRESULT(0);
        }
        Result::Ok(r) => {
            return r;
        }
    };
}

fn _keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> Result<LRESULT, RekeyError> {
    debug(format!(
        "keyboard_hook {}, {}, {}",
        code, wparam.0, lparam.0
    ));

    let mut data = MY_DATA
        .lock()
        .map_err(|err| RekeyError::GenericError(format!("could not get data lock: {}", err)))?;

    unsafe {
        if data.is_none() {
            let d = read_global_data()?;
            *data = Option::Some(d);
        }

        if let Some(d) = data.as_ref() {
            if code < 0 || code != HC_ACTION as i32 {
                return Result::Ok(CallNextHookEx(d.hhook, code, wparam, lparam));
            }

            let result = SendMessageW(d.hwnd, WM_REKEY_SHOULD_SKIP_INPUT, wparam, lparam);
            if result == SKIP_INPUT {
                return Result::Ok(LRESULT(1));
            }
            return Result::Ok(CallNextHookEx(d.hhook, code, wparam, lparam));
        }
        return Result::Ok(LRESULT(0));
    }
}

fn read_global_data() -> Result<GlobalData, RekeyError> {
    let dir = env::temp_dir();
    let filename = dir.join("rekey.dat");
    let filename_clone = filename.clone();

    let mut file = File::open(filename).map_err(|error| {
        RekeyError::GenericError(format!(
            "failed to create file: {}: {}",
            filename_clone.display(),
            error
        ))
    })?;

    let mut buffer = [0; 8];
    file.read_exact(&mut buffer)?;
    let hhook = isize::from_le_bytes(buffer);
    file.read_exact(&mut buffer)?;
    let hwnd = isize::from_le_bytes(buffer);

    return Result::Ok(GlobalData {
        hhook: HHOOK(hhook),
        hwnd: HWND(hwnd),
    });
}

fn write_global_data(data: &GlobalData) -> Result<(), RekeyError> {
    let dir = env::temp_dir();
    let filename = dir.join("rekey.dat");
    let filename_clone = filename.clone();

    let mut file = File::create(filename).map_err(|error| {
        RekeyError::GenericError(format!(
            "failed to create file: {}: {}",
            filename_clone.display(),
            error
        ))
    })?;
    file.write_all(&data.hhook.0.to_le_bytes())?;
    file.write_all(&data.hwnd.0.to_le_bytes())?;

    return Result::Ok(());
}
