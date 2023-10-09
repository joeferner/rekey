use std::{fs::OpenOptions, io::Write};

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

#[no_mangle]
pub extern "C" fn install(dll: HMODULE) -> () {
    unsafe {
        let keyboard_hook_bare =
            GetProcAddress(dll, s!("keyboard_hook")).expect("failed to find keyboard_hook");
        let keyboard_hook =
            std::mem::transmute::<Option<PROC>, HOOKPROC>(Option::Some(keyboard_hook_bare));
        let hook =
            SetWindowsHookExW(WH_KEYBOARD, keyboard_hook, dll, 0).expect("failed to set hook");

        MY_HOOK = Option::Some(hook);
    }
    debug("installed".to_string());
}

#[no_mangle]
pub extern "C" fn uninstall() -> () {
    unsafe {
        if let Some(hook) = MY_HOOK {
            UnhookWindowsHookEx(hook).expect("failed to unhook");
        }
    }
    debug("uninstalled".to_string());
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
