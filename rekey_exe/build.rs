use rekey_common::{vkeys::VKEY_LOOKUP_BY_NAME, REKEY_API_JS_FILENAME};
use std::{fs, io};
use winres::WindowsResource;

fn main() -> io::Result<()> {
    if cfg!(target_os = "windows") {
        WindowsResource::new()
            .set_icon_with_id("assets/rekey.ico", "ICON_REKEY")
            .compile()?;
    }

    generate_javascript_template()?;

    Ok(())
}

fn generate_javascript_template() -> io::Result<()> {
    let mut contents = fs::read_to_string("src/js/rekey-api.template.js")?;
    fs::create_dir_all("target/generated")?;

    for vkey in VKEY_LOOKUP_BY_NAME.values() {
        let s = format!(
            "/**\n * virtual key {}\n * @global\n */\nconst VK_{} = {};\n",
            vkey.name,
            vkey.name.to_uppercase(),
            vkey.code.0
        );
        contents.push_str(s.as_str());
    }

    fs::write(
        "target/generated/".to_string() + REKEY_API_JS_FILENAME,
        contents,
    )?;

    return Ok(());
}
