use windows::{
    core::{s, w},
    Win32::{
        Foundation::{FreeLibrary, HMODULE, HWND},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
    },
};

use crate::{debug, RekeyError};

type PROC = unsafe extern "system" fn() -> isize;
type FnInstall = extern "C" fn(dll: u64, hwnd: u64) -> i32;
type FnUninstall = extern "C" fn() -> i32;

pub struct RekeyDll {
    dll: HMODULE,
    install: Option<FnInstall>,
    uninstall: Option<FnUninstall>,
}

impl RekeyDll {
    pub fn new() -> Result<Self, RekeyError> {
        unsafe {
            let dll = LoadLibraryW(w!("rekey_lib.dll"))
                .map_err(|err| RekeyError::Win32Error("failed to load dll".to_string(), err))?;
            let install_bare = GetProcAddress(dll, s!("install"))
                .ok_or_else(|| RekeyError::GenericError("failed to find install".to_string()))?;
            let install = std::mem::transmute::<PROC, FnInstall>(install_bare);
            let uninstall_bare = GetProcAddress(dll, s!("uninstall"))
                .ok_or_else(|| RekeyError::GenericError("failed to find uninstall".to_string()))?;
            let uninstall = std::mem::transmute::<PROC, FnUninstall>(uninstall_bare);
            return Result::Ok(Self {
                dll,
                install: Option::Some(install),
                uninstall: Option::Some(uninstall),
            });
        }
    }

    pub fn install(&mut self, hwnd: HWND) -> Result<(), RekeyError> {
        if let Option::Some(install) = self.install.take() {
            let install_ret = install(self.dll.0 as u64, hwnd.0 as u64);
            if install_ret != 0 {
                return Result::Err(RekeyError::GenericError(format!(
                    "failed to install: {}",
                    install_ret
                )));
            }
            return Result::Ok(());
        } else {
            return Result::Err(RekeyError::GenericError("already installed".to_string()));
        }
    }

    pub fn uninstall(&mut self) -> Result<(), RekeyError> {
        if let Option::Some(uninstall) = self.uninstall.take() {
            let uninstall_ret = uninstall();
            if uninstall_ret != 0 {
                return Result::Err(RekeyError::GenericError(format!(
                    "failed to uninstall: {}",
                    uninstall_ret
                )));
            }
            unsafe {
                FreeLibrary(self.dll).map_err(|err| {
                    RekeyError::Win32Error("failed to free library".to_string(), err)
                })?;
            }
            return Result::Ok(());
        } else {
            return Result::Err(RekeyError::GenericError("already uninstalled".to_string()));
        }
    }
}

impl Drop for RekeyDll {
    fn drop(&mut self) {
        if self.uninstall.is_some() {
            self.uninstall().unwrap_or_else(|err| {
                debug!("failed to uninstall {}", err);
                return ();
            });
        }
    }
}
