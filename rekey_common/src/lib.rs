use chrono::Local;
use directories::ProjectDirs;
use std::{fmt, fs::OpenOptions, io::Write, path::PathBuf};
use vkeys::VKEY_LOOKUP_BY_NAME;
use windows::Win32::{
    Foundation::LRESULT,
    UI::{
        Input::KeyboardAndMouse::{VkKeyScanW, VIRTUAL_KEY, VK_0, VK_9, VK_NUMPAD0, VK_NUMPAD9, VK_Z, VK_A},
        WindowsAndMessaging::WM_USER,
    },
};

pub mod vkeys;

pub const WM_USER_SHOULD_SKIP_INPUT: u32 = WM_USER + 300;
pub const WM_USER_SHELL_ICON: u32 = WM_USER + 301;
pub const DONT_SKIP_INPUT: LRESULT = LRESULT(1);
pub const SKIP_INPUT: LRESULT = LRESULT(42);
pub const REKEY_API_JS_FILENAME: &str = "rekey-api.js";

#[derive(Debug)]
pub enum RekeyError {
    GenericError(String),
    Win32GetLastError(String, Result<(), windows::core::Error>),
    Win32Error(String, windows::core::Error),
    IoError(std::io::Error),
}

impl From<std::io::Error> for RekeyError {
    fn from(err: std::io::Error) -> Self {
        return RekeyError::IoError(err);
    }
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
            RekeyError::IoError(error) => {
                write!(f, "IO Error: {}", error)
            }
        }
    }
}

pub fn get_user_dir() -> Result<PathBuf, RekeyError> {
    match ProjectDirs::from("com.github", "joeferner", "rekey") {
        Option::Some(proj_dir) => {
            let mut p = proj_dir.config_dir().to_path_buf();
            if p.ends_with("config") {
                p = p.parent().unwrap_or(p.as_path()).to_path_buf();
            }
            return Result::Ok(p);
        }
        Option::None => {
            return Result::Err(RekeyError::GenericError(
                "could not get project dir".to_string(),
            ));
        }
    }
}

pub fn get_log_filename() -> Result<PathBuf, RekeyError> {
    return Result::Ok(get_user_dir()?.join("rekey.log"));
}

pub fn get_scripts_dir() -> Result<PathBuf, RekeyError> {
    return Result::Ok(get_user_dir()?.join("scripts"));
}

pub fn debug<S>(s: S) -> ()
where
    S: Into<String>,
{
    let now = Local::now();
    let s = s.into();

    if cfg!(debug_assertions) {
        println!("{}: {}", now.format("%F %X"), s);
    }

    if let Result::Ok(filename) = get_log_filename() {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(filename);
        if let Result::Ok(mut f) = file {
            let _ = writeln!(&mut f, "{}: {}", now.format("%F %X"), s).is_ok();
        }
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        let res = $crate::debug(format!($($arg)*));
        res
    }}
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum KeyDirection {
    Down,
    Up,
}

impl fmt::Display for KeyDirection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            KeyDirection::Down => write!(f, "Down"),
            KeyDirection::Up => write!(f, "Up"),
        }
    }
}

pub struct KeyboardModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub hankaku: bool,
}

pub struct ToVirtualKeyResult {
    pub vkey: VIRTUAL_KEY,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub hankaku: bool,
}

impl ToVirtualKeyResult {
    fn from_vkey(vkey: VIRTUAL_KEY) -> Self {
        return ToVirtualKeyResult {
            vkey,
            shift: false,
            ctrl: false,
            alt: false,
            hankaku: false,
        };
    }
}

pub fn to_virtual_key(s: &str) -> Result<ToVirtualKeyResult, RekeyError> {
    if let Option::Some(lookup_value) = VKEY_LOOKUP_BY_NAME.get(s.to_ascii_lowercase().as_str()) {
        return Result::Ok(ToVirtualKeyResult::from_vkey(lookup_value.code));
    }

    let s = s.to_ascii_lowercase();
    if s.len() == 1 {
        if let Option::Some(ch) = s.chars().next() {
            let r = unsafe { VkKeyScanW(ch as u16) as u16 };
            let low = (r & 0xff) as i8;
            let high = ((r >> 8) & 0xff) as i8;
            if low >= 0 && high >= 0 {
                let vkey = VIRTUAL_KEY(low as u16);
                return Result::Ok(ToVirtualKeyResult {
                    vkey,
                    shift: high & 1 == 1,
                    ctrl: high & 2 == 2,
                    alt: high & 4 == 4,
                    hankaku: high & 8 == 8,
                });
            }
        }
    }

    return Result::Err(RekeyError::GenericError(format!(
        "could not convert key {} to virtual key",
        s
    )));
}

pub fn char_from_vcode(vkey_code: u16) -> Option<char> {
    if vkey_code >= VK_0.0 && vkey_code <= VK_9.0 {
        return char::from_u32(('0' as u32) + (vkey_code - VK_0.0) as u32);
    }
    if vkey_code >= VK_NUMPAD0.0 && vkey_code <= VK_NUMPAD9.0 {
        return char::from_u32(('0' as u32) + (vkey_code - VK_NUMPAD0.0) as u32);
    }
    if vkey_code >= VK_A.0 && vkey_code <= VK_Z.0 {
        return char::from_u32(('a' as u32) + (vkey_code - VK_A.0) as u32);
    }
    return Option::None;
}
