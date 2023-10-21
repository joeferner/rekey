use lazy_static::lazy_static;
use std::{
    env,
    fs::{self, File},
    io::Write,
    sync::Mutex,
};

use rekey_common::{debug, RekeyError, SKIP_INPUT, WM_USER_SHOULD_SKIP_INPUT};
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
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

type PROC = unsafe extern "system" fn() -> isize;

struct GlobalData {
    hhook: HHOOK,
    hwnd: HWND,
}

lazy_static! {
    static ref MY_DATA: Mutex<Option<GlobalData>> = Mutex::new(Option::None);
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: u32,
    _: *mut ())
    -> bool
{
    match call_reason {
        DLL_PROCESS_ATTACH => {
            debug!("dll: attach");
        }
        DLL_PROCESS_DETACH => {
            debug!("dll: detach");
        }
        _ => ()
    }

    return true;
}

#[no_mangle]
pub extern "C" fn install(dll: u64, hwnd: u64) -> i32 {
    debug!("dll: installing");
    match _install(HMODULE(dll as isize), HWND(hwnd as isize)) {
        Result::Err(err) => {
            debug!("dll: install failed {}", err);
            return 1;
        }
        Result::Ok(()) => {
            debug!("dll: install success");
            return 0;
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
    return Result::Ok(());
}

#[no_mangle]
pub extern "C" fn uninstall() -> i32 {
    debug!("dll: uninstalling");
    match _uninstall() {
        Result::Err(err) => {
            debug!("dll: uninstall failed {}", err);
            return 1;
        }
        Result::Ok(()) => {
            debug!("dll: uninstall success");
            return 0;
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
    return Result::Ok(());
}

#[no_mangle]
pub extern "C" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match _keyboard_hook(code, wparam, lparam) {
        Result::Err(err) => {
            debug!("dll: keyboard_hook failed {}", err);
            return LRESULT(0);
        }
        Result::Ok(r) => {
            return r;
        }
    };
}

fn _keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> Result<LRESULT, RekeyError> {
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

            let result = SendMessageW(d.hwnd, WM_USER_SHOULD_SKIP_INPUT, wparam, lparam);
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
    debug!("reading {}", filename.display());

    let contents = fs::read_to_string(filename)?;
    let mut parts = contents.split(",");

    let hhook_str = parts
        .nth(0)
        .ok_or_else(|| RekeyError::GenericError("failed to get hhook from file".to_string()))?;

    let hwnd_str = parts
        .nth(0)
        .ok_or_else(|| RekeyError::GenericError("failed to get hwnd from file".to_string()))?;

    let hhook = hhook_str.parse::<isize>().map_err(|err| {
        RekeyError::GenericError(format!("failed to parse hhook to string {}", err))
    })?;

    let hwnd = hwnd_str.parse::<isize>().map_err(|err| {
        RekeyError::GenericError(format!("failed to parse hwnd to string {}", err))
    })?;

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
    write!(file, "{},{}", data.hhook.0, data.hwnd.0)?;
    return Result::Ok(());
}
