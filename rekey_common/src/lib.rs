use chrono::Utc;
use std::{fmt, fs::OpenOptions, io::Write};
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

pub fn debug(s: String) -> () {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("C:\\dev\\rekey\\target\\out.txt");
    if let Result::Ok(mut f) = file {
        let now = Utc::now();
        let _ = writeln!(&mut f, "{}: {}", now.format("%+"), s).is_ok();
    }
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
