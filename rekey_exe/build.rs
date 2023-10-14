use {std::io, winres::WindowsResource};

fn main() -> io::Result<()> {
    if cfg!(target_os = "windows") {
        WindowsResource::new()
            .set_icon_with_id("assets/rekey.ico", "ICON_REKEY")
            .compile()?;
    }
    Ok(())
}
