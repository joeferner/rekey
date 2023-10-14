use chrono::Local;
use directories::ProjectDirs;
use std::{fmt, fs::OpenOptions, io::Write, path::PathBuf};
use windows::Win32::{Foundation::LRESULT, UI::WindowsAndMessaging::WM_USER};

pub const WM_REKEY_SHOULD_SKIP_INPUT: u32 = WM_USER + 300;
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

pub fn debug<S>(s: S) -> ()
where
    S: Into<String>,
{
    if let Result::Ok(filename) = get_log_filename() {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(filename);
        if let Result::Ok(mut f) = file {
            let now = Local::now();
            let _ = writeln!(&mut f, "{}: {}", now.format("%F %X"), s.into()).is_ok();
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
