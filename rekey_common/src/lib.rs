use chrono::Local;
use directories::ProjectDirs;
use std::{fmt, fs::OpenOptions, io::Write, path::PathBuf};
use windows::Win32::{
    Foundation::LRESULT,
    UI::{
        Input::KeyboardAndMouse::{self, VkKeyScanW, VIRTUAL_KEY},
        WindowsAndMessaging::WM_USER,
    },
};

pub const WM_USER_SHOULD_SKIP_INPUT: u32 = WM_USER + 300;
pub const WM_USER_SHELL_ICON: u32 = WM_USER + 301;
pub const DONT_SKIP_INPUT: LRESULT = LRESULT(1);
pub const SKIP_INPUT: LRESULT = LRESULT(42);

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
    let s = s.to_ascii_lowercase();
    match s.as_str() {
        "ctrl" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_CONTROL));
        }
        "alt" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_MENU));
        }
        "shift" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_SHIFT));
        }
        "win" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_LWIN));
        }
        "esc" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_ESCAPE));
        }
        "space" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_SPACE));
        }
        "backspace" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_BACK));
        }
        "tab" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_TAB));
        }
        "enter" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_RETURN));
        }
        "pause" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_PAUSE));
        }
        "left" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_LEFT));
        }
        "right" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_RIGHT));
        }
        "up" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_UP));
        }
        "down" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_DOWN));
        }
        "insert" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_INSERT));
        }
        "delete" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_DELETE));
        }
        "f1" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F1));
        }
        "f2" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F2));
        }
        "f3" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F3));
        }
        "f4" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F4));
        }
        "f5" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F5));
        }
        "f6" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F6));
        }
        "f7" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F7));
        }
        "f8" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F8));
        }
        "f9" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F9));
        }
        "f10" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F10));
        }
        "f11" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F11));
        }
        "f12" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F12));
        }
        "f13" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F13));
        }
        "f14" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F14));
        }
        "f15" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F15));
        }
        "f16" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F16));
        }
        "f17" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F17));
        }
        "f18" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F18));
        }
        "f19" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F19));
        }
        "f20" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F20));
        }
        "f21" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F21));
        }
        "f22" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F22));
        }
        "f23" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F23));
        }
        "f24" => {
            return Result::Ok(ToVirtualKeyResult::from_vkey(KeyboardAndMouse::VK_F23));
        }
        _ => {
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
    };
}
