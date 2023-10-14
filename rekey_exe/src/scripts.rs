use boa_engine::{
    js_string, property::Attribute, Context, JsError, JsNativeError, JsObject, JsValue,
    NativeFunction, Source,
};
use lazy_static::lazy_static;
use rekey_common::{debug, KeyDirection, RekeyError};
use std::{
    fs,
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::{devices::Device, get_project_config_dir, js, SkipInput};

#[derive(PartialEq, Eq)]
enum KeyHandlerDevices {
    All,
}

#[derive(PartialEq, Eq)]
enum KeyHandlerKeys {
    All,
}

struct KeyHandler {
    devices: KeyHandlerDevices,
    keys: KeyHandlerKeys,
    callback: JsObject,
}

struct Script<'a> {
    context: Arc<Mutex<Context<'a>>>,
    key_handlers: Arc<Mutex<Vec<KeyHandler>>>,
}

struct InputMessage {
    vkey_code: u16,
    direction: KeyDirection,
    device: Option<Arc<Device>>,
}

enum ThreadMessage {
    Exit,
    HandleInput(mpsc::Sender<ThreadResponseMessage>, InputMessage),
}

type ThreadResponseMessage = Result<SkipInput, RekeyError>;

lazy_static! {
    static ref CHANNEL: Mutex<Option<mpsc::Sender<ThreadMessage>>> = Mutex::new(Option::None);
}

pub fn scripts_load() -> Result<(), RekeyError> {
    let mut channel = CHANNEL
        .lock()
        .map_err(|err| RekeyError::GenericError(format!("could not get scripts lock: {}", err)))?;
    if let Option::Some(ch) = &mut *channel {
        ch.send(ThreadMessage::Exit).map_err(|err| {
            RekeyError::GenericError(format!("failed to send exit to thread: {}", err))
        })?;
        *channel = Option::None;
    }

    let (tx, rx) = mpsc::channel::<ThreadMessage>();

    let script_dir = get_project_config_dir()?.join("scripts");
    fs::create_dir_all(&script_dir)?;

    *channel = Option::Some(tx);
    thread::spawn(move || {
        scripts_thread(rx, script_dir);
    });

    return Result::Ok(());
}

fn scripts_thread(rx: mpsc::Receiver<ThreadMessage>, script_dir: PathBuf) -> () {
    let scripts = match load_scripts(script_dir) {
        Result::Err(err) => {
            // send back to main thread
            debug(format!("failed to load scripts: {}", err));
            return;
        }
        Result::Ok(scripts) => scripts,
    };

    loop {
        match rx.recv() {
            Result::Err(_err) => {
                break;
            }
            Result::Ok(msg) => match msg {
                ThreadMessage::Exit => {
                    break;
                }
                ThreadMessage::HandleInput(tx, msg) => {
                    tx.send(thread_handle_input_message(msg, &scripts))
                        .unwrap_or_else(|err| {
                            debug(format!("failed to send message: {}", err));
                            return ();
                        });
                }
            },
        }
    }
}

fn thread_handle_input_message(msg: InputMessage, scripts: &Vec<Script>) -> ThreadResponseMessage {
    let mut result = SkipInput::DontSkip;
    for script in scripts {
        if thread_run_script_callbacks(&msg, script)? == SkipInput::Skip {
            result = SkipInput::Skip;
        }
    }
    return Result::Ok(result);
}

fn thread_run_script_callbacks(msg: &InputMessage, script: &Script) -> ThreadResponseMessage {
    let mut result = SkipInput::DontSkip;
    let key_handlers = script
        .key_handlers
        .lock()
        .map_err(|err| RekeyError::GenericError(format!("failed to lock key handlers: {}", err)))?;
    for key_handler in key_handlers.iter() {
        if thread_run_key_handler_callbacks(&msg, script, &key_handler)? == SkipInput::Skip {
            result = SkipInput::Skip;
        }
    }
    return Result::Ok(result);
}

fn thread_run_key_handler_callbacks(
    msg: &InputMessage,
    script: &Script,
    key_handler: &KeyHandler,
) -> ThreadResponseMessage {
    if key_handler.keys != KeyHandlerKeys::All || key_handler.devices != KeyHandlerDevices::All {
        return Result::Err(RekeyError::GenericError(
            "unhandled keys/devices".to_string(),
        ));
    }

    let mut context = script
        .context
        .lock()
        .map_err(|err| RekeyError::GenericError(format!("failed to lock context: {}", err)))?;

    let direction = format!("{}", msg.direction);
    let direction = direction.to_lowercase();

    let ctx = JsObject::default();

    ctx.set(
        js_string!("vKeyCode"),
        JsValue::from(msg.vkey_code),
        false,
        &mut context,
    )
    .map_err(|err| RekeyError::GenericError(format!("failed to set: {}", err)))?;

    ctx.set(
        js_string!("direction"),
        JsValue::from(js_string!(direction)),
        false,
        &mut context,
    )
    .map_err(|err| RekeyError::GenericError(format!("failed to set: {}", err)))?;

    if let Option::Some(device) = &msg.device {
        ctx.set(
            js_string!("deviceName"),
            JsValue::from(js_string!(device.device_name.clone())),
            false,
            &mut context,
        )
        .map_err(|err| RekeyError::GenericError(format!("failed to set: {}", err)))?;
    }

    let args: [JsValue; 1] = [JsValue::Object(ctx)];
    let this = JsValue::Undefined;

    let results = key_handler
        .callback
        .call(&this, &args, &mut context)
        .map_err(|err| RekeyError::GenericError(format!("failed to run callback: {}", err)))?;
    if results.to_boolean() == false {
        return Result::Ok(SkipInput::DontSkip);
    } else {
        return Result::Ok(SkipInput::Skip);
    }
}

fn load_scripts<'a>(script_dir: PathBuf) -> Result<Vec<Script<'a>>, RekeyError> {
    let mut results: Vec<Script> = vec![];
    for entry in fs::read_dir(&script_dir)? {
        let entry = entry?;
        let entry_path = entry.path();
        debug(format!("running script: {}", entry_path.display()));

        let mut context = Context::default();
        let key_handlers: Arc<Mutex<Vec<KeyHandler>>> = Arc::new(Mutex::new(vec![]));
        initialize_context(&mut context, &key_handlers)?;

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
        results.push(Script {
            context: Arc::new(Mutex::new(context)),
            key_handlers,
        });
    }
    return Result::Ok(results);
}

fn initialize_context(
    context: &mut Context<'_>,
    key_handlers: &Arc<Mutex<Vec<KeyHandler>>>,
) -> Result<(), RekeyError> {
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
            let my_key_handlers = Arc::clone(key_handlers);
            NativeFunction::from_closure(move |this, args, context| {
                match handle_register(this, args, context) {
                    Result::Ok(key_handler) => {
                        let mut my_key_handlers = my_key_handlers.lock().map_err(|err| {
                            JsNativeError::error()
                                .with_message(format!("could not get key handlers lock: {}", err))
                        })?;
                        my_key_handlers.push(key_handler);
                        return Result::Ok(JsValue::Undefined);
                    }
                    Result::Err(err) => {
                        return Result::Err(err);
                    }
                };
            })
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
) -> Result<KeyHandler, JsError> {
    // devices, keys, callback
    if args.len() == 3 {
        let devices = args.get(0).unwrap();
        let keys = args.get(1).unwrap();
        let callback = args.get(2).unwrap();

        if devices.is_string() && keys.is_string() && callback.is_callable() {
            let devices = devices.as_string().unwrap().to_std_string_escaped();
            let keys = keys.as_string().unwrap().to_std_string_escaped();
            let callback = callback.as_callable().unwrap();
            if devices != "*" {
                return Result::Err(JsError::from(
                    JsNativeError::error()
                        .with_message("invalid devices arguments, expected \"*\""),
                ));
            }
            if keys != "*" {
                return Result::Err(JsError::from(
                    JsNativeError::error().with_message("invalid keys arguments, expected \"*\""),
                ));
            }
            return Result::Ok(KeyHandler {
                devices: KeyHandlerDevices::All,
                keys: KeyHandlerKeys::All,
                callback: callback.clone(),
            });
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
}

pub fn scripts_handle_input(
    vkey_code: u16,
    direction: KeyDirection,
    device: Option<Arc<Device>>,
) -> Result<SkipInput, RekeyError> {
    let mut channel = CHANNEL
        .lock()
        .map_err(|err| RekeyError::GenericError(format!("could not get scripts lock: {}", err)))?;
    if let Option::Some(ch) = &mut *channel {
        let (tx, rx) = mpsc::channel::<ThreadResponseMessage>();
        ch.send(ThreadMessage::HandleInput(
            tx,
            InputMessage {
                vkey_code,
                direction,
                device,
            },
        ))
        .map_err(|err| {
            RekeyError::GenericError(format!("failed to send input message to thread: {}", err))
        })?;
        let result = rx.recv().map_err(|err| {
            RekeyError::GenericError(format!(
                "failed to receive response from input thread: {}",
                err
            ))
        })?;
        return result;
    }

    return Result::Ok(SkipInput::DontSkip);
}
