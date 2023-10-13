use boa_engine::{
    js_string, property::Attribute, Context, JsError, JsNativeError, JsResult, JsValue,
    NativeFunction, Source,
};
use lazy_static::lazy_static;
use rekey_common::{debug, KeyDirection, RekeyError};
use std::{
    fs,
    sync::{Arc, Mutex},
};

use crate::{devices::Device, get_project_config_dir, js};

struct KeyHandler {}

struct Script<'a> {
    context: Context<'a>,
}

lazy_static! {
    static ref KEY_HANDLERS: Mutex<Vec<Arc<Script>>> = Mutex::new(vec![]);
}

pub fn scripts_load() -> Result<(), RekeyError> {
    let script_dir = get_project_config_dir()?.join("scripts");
    fs::create_dir_all(&script_dir)?;
    for entry in fs::read_dir(&script_dir)? {
        let entry = entry?;
        let entry_path = entry.path();
        debug(format!("running script: {}", entry_path.display()));

        let mut context = Context::default();
        initialize_context(&mut context)?;

        let script_path = &entry_path.as_path();
        let source = Source::from_filepath(script_path)
            .map_err(|err| RekeyError::GenericError(format!("failed to load script: {}", err)))?;
        context.eval(source).map_err(|err| {
            RekeyError::GenericError(format!(
                "failed to evaluate script {}: {}",
                entry_path.display(),
                err
            ))
        })?;
    }

    return Result::Ok(());
}

fn initialize_context(context: &mut Context<'_>) -> Result<(), RekeyError> {
    let console = js::console::Console::init(context);
    context
        .register_global_property(
            js_string!(js::console::Console::NAME),
            console,
            Attribute::all(),
        )
        .map_err(|err| RekeyError::GenericError(format!("failed to register console: {}", err)))?;

    context
        .register_global_callable("rekeyRegister", 0, unsafe {
            NativeFunction::from_fn_ptr(handle_register)
        })
        .map_err(|err| {
            RekeyError::GenericError(format!("failed to register rekeyRegister: {}", err))
        })?;

    return Result::Ok(());
}

fn handle_register(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context<'_>,
) -> JsResult<JsValue> {
    let _key_handlers = KEY_HANDLERS
        .lock()
        .map_err(|err| RekeyError::GenericError(format!("could not get scripts lock: {}", err)));

    // devices, keys, callback
    if args.len() == 3 {
        let devices = args.get(0).unwrap();
        let keys = args.get(1).unwrap();
        let callback = args.get(2).unwrap();

        if devices.is_string() && keys.is_string() && callback.is_callable() {
            let devices = devices.as_string().unwrap().to_std_string_escaped();
            let keys = keys.as_string().unwrap().to_std_string_escaped();
            let callback = callback.as_callable().unwrap();
            debug("a".to_string());
        } else {
            return Result::Err(JsError::from(
                JsNativeError::error()
                    .with_message("invalid arguments, expected (string, string, callback)"),
            ));
        }
    } else {
        return Result::Err(JsError::from(JsNativeError::error().with_message(format!(
            "invalid arguments, expected 3 found {}",
            args.len()
        ))));
    }

    return Result::Ok(JsValue::Undefined);
}

pub fn scripts_handle_input(
    _vkey_code: u16,
    _direction: KeyDirection,
    _device: Option<Arc<Device>>,
) -> Result<bool, RekeyError> {
    let key_handlers = KEY_HANDLERS
        .lock()
        .map_err(|err| RekeyError::GenericError(format!("could not get scripts lock: {}", err)))?;

    for _key_handler in key_handlers.iter() {}

    return Result::Ok(false);
}
