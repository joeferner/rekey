use std::{fs::OpenOptions, io::Write, thread::sleep, time::Duration};

use windows::{
    core::{s, w},
    Win32::{
        Foundation::{FreeLibrary, HMODULE},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
    },
};

type PROC = unsafe extern "system" fn() -> isize;
type FnInstall = extern "stdcall" fn(dll: HMODULE) -> ();
type FnUninstall = extern "stdcall" fn() -> ();

fn debug(s: String) -> () {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("C:\\dev\\rekey\\target\\out.txt");
    if let Result::Ok(mut f) = file {
        let _ = writeln!(&mut f, "{}", s).is_ok();
    }
    println!("{}", s);
}

fn main() {
    debug("begin".to_string());
    unsafe {
        let dll = LoadLibraryW(w!("rekey_lib.dll")).expect("failed to load dll");
        let install_bare = GetProcAddress(dll, s!("install")).expect("failed to find install");
        let install = std::mem::transmute::<PROC, FnInstall>(install_bare);
        let uninstall_bare =
            GetProcAddress(dll, s!("uninstall")).expect("failed to find uninstall");
        let uninstall = std::mem::transmute::<PROC, FnUninstall>(uninstall_bare);

        install(dll);

        println!("sleeping 10s");
        sleep(Duration::from_secs(10));
        println!("freeing");

        uninstall();
        FreeLibrary(dll).expect("failed to free");
    }
}
