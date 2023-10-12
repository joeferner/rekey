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
static mut MY_HOOK: Option<HHOOK> = Option::None;
static mut MY_HWND: Option<HWND> = Option::None;

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
    unsafe {
        if MY_HOOK.is_some() {
            return Result::Err(RekeyError::GenericError("already installed".to_string()));
        }

        let keyboard_hook_bare = GetProcAddress(dll, s!("keyboard_hook"))
            .ok_or_else(|| RekeyError::GenericError("failed to find keyboard_hook".to_string()))?;
        let keyboard_hook =
            std::mem::transmute::<Option<PROC>, HOOKPROC>(Option::Some(keyboard_hook_bare));
        let hook = SetWindowsHookExW(WH_KEYBOARD, keyboard_hook, dll, 0)
            .map_err(|err| RekeyError::Win32Error("failed to set hook".to_string(), err))?;

        MY_HWND = Option::Some(hwnd);
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
        if MY_HOOK.is_none() {
            return Result::Err(RekeyError::GenericError("not installed".to_string()));
        }

        if let Some(hook) = MY_HOOK {
            UnhookWindowsHookEx(hook)
                .map_err(|err| RekeyError::Win32Error("failed to unhook".to_string(), err))?;
            MY_HOOK = Option::None;
            MY_HWND = Option::None;
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
        debug("1".to_string());
        if let Some(hook) = MY_HOOK {
            debug("2".to_string());
            if let Some(hwnd) = MY_HWND {
                debug("3".to_string());
                if code < 0 || code != HC_ACTION as i32 {
                    return CallNextHookEx(hook, code, wparam, lparam);
                }

                debug("4".to_string());
                let result = SendMessageW(hwnd, WM_REKEY_SHOULD_SKIP_INPUT, wparam, lparam);
                debug(format!("5 {}", result.0));
                if result == SKIP_INPUT {
                    return LRESULT(1);
                }
                return CallNextHookEx(hook, code, wparam, lparam);
            }
        }
        return LRESULT(0);
    }
}
