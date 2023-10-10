use std::{fmt, fs::OpenOptions, io::Write};

use windows::{
    core::s,
    Win32::{
        Foundation::{HMODULE, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetProcAddress,
        UI::WindowsAndMessaging::{
            CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, HHOOK, HOOKPROC,
            WH_KEYBOARD,
        },
    },
};

type PROC = unsafe extern "system" fn() -> isize;
static mut MY_HOOK: Option<HHOOK> = Option::None;

fn debug(s: String) -> () {
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("C:\\dev\\rekey\\target\\out.txt");
    if let Result::Ok(mut f) = file {
        let _ = writeln!(&mut f, "{}", s).is_ok();
    }
}

#[derive(Debug)]
pub enum RekeyError {
    GenericError(String),
    Win32GetLastError(String, Result<(), windows::core::Error>),
    Win32Error(String, windows::core::Error),
}

impl fmt::Display for RekeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RekeyError::GenericError(s) => {
                write!(f, "Generic Error: {}", s)
            }
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

#[no_mangle]
pub extern "C" fn install(dll: HMODULE) -> bool {
    match _install(dll) {
        Result::Err(err) => {
            debug(format!("install failed {}", err));
            return false;
        }
        Result::Ok(()) => {
            return true;
        }
    };
}

fn _install(dll: HMODULE) -> Result<(), RekeyError> {
    unsafe {
        let keyboard_hook_bare = GetProcAddress(dll, s!("keyboard_hook"))
            .ok_or_else(|| RekeyError::GenericError("failed to find keyboard_hook".to_string()))?;
        let keyboard_hook =
            std::mem::transmute::<Option<PROC>, HOOKPROC>(Option::Some(keyboard_hook_bare));
        let hook = SetWindowsHookExW(WH_KEYBOARD, keyboard_hook, dll, 0)
            .map_err(|err| RekeyError::Win32Error("failed to set hook".to_string(), err))?;

        MY_HOOK = Option::Some(hook);
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
    unsafe {
        if let Some(hook) = MY_HOOK {
            UnhookWindowsHookEx(hook)
                .map_err(|err| RekeyError::Win32Error("failed to unhook".to_string(), err))?;
        }
    }
    debug("uninstalled".to_string());
    return Result::Ok(());
}

#[no_mangle]
pub extern "C" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    debug(format!(
        "keyboard_hook {}, {}, {}",
        code, wparam.0, lparam.0
    ));

    unsafe {
        if let Some(hook) = MY_HOOK {
            if code < 0 || code != HC_ACTION as i32 {
                return CallNextHookEx(hook, code, wparam, lparam);
            }

            return CallNextHookEx(hook, code, wparam, lparam);
        } else {
            return LRESULT(0);
        }
    }
}
