use boa_engine::{
    js_string, property::Attribute, Context, JsError, JsNativeError, JsObject, JsValue,
    NativeFunction, Source,
};
use lazy_static::lazy_static;
use rekey_common::{debug, get_scripts_dir, to_virtual_key, KeyDirection, RekeyError};
use std::{
    fs,
    mem::size_of,
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY,
    VK_CONTROL, VK_MENU, VK_SHIFT,
};

use crate::{devices::Device, js, SkipInput};

#[derive(PartialEq, Eq)]
enum KeyHandlerDevices {
    All,
    Contains(String),
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
        debug("stopping old scripts thread");
        ch.send(ThreadMessage::Exit).map_err(|err| {
            RekeyError::GenericError(format!("failed to send exit to thread: {}", err))
        })?;
        *channel = Option::None;
    }

    debug("loading scripts");

    let (tx, rx) = mpsc::channel::<ThreadMessage>();

    let script_dir = get_scripts_dir()?;
    fs::create_dir_all(&script_dir)?;

    let (init_tx, init_rx) = mpsc::channel();
    thread::spawn(move || {
        scripts_thread(init_tx, rx, script_dir);
    });

    let init_result = init_rx
        .recv()
        .map_err(|err| RekeyError::GenericError(format!("failed to receive: {}", err)))?;
    if let Result::Err(err) = init_result {
        return Result::Err(err);
    }

    *channel = Option::Some(tx);
    return Result::Ok(());
}

fn scripts_thread(
    tx: mpsc::Sender<Result<(), RekeyError>>,
    rx: mpsc::Receiver<ThreadMessage>,
    script_dir: PathBuf,
) -> () {
    debug("script thread starting");
    let scripts = match load_scripts(script_dir) {
        Result::Err(err) => {
            debug!("init error: {}", err);
            tx.send(Result::Err(err))
                .unwrap_or_else(move |err| debug!("failed to send init error: {}", err));
            return;
        }
        Result::Ok(scripts) => scripts,
    };
    tx.send(Result::Ok(()))
        .unwrap_or_else(|err| debug!("failed to send init: {}", err));

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
                            debug!("failed to send message: {}", err);
                            return ();
                        });
                }
            },
        }
    }
    debug("script thread stopped");
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
    match key_handler.keys {
        KeyHandlerKeys::All => {}
    }

    match &key_handler.devices {
        KeyHandlerDevices::All => {}
        KeyHandlerDevices::Contains(contains_str) => {
            if let Option::Some(device) = &msg.device {
                if !device.device_name.contains(contains_str) {
                    return Result::Ok(SkipInput::DontSkip);
                }
            }
        }
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
        if entry_path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            != "js"
        {
            continue;
        }
        debug!("loading script: {}", entry_path.display());

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

    context
        .register_global_callable("sendKey", 0, NativeFunction::from_fn_ptr(handle_send_key))
        .map_err(|err| {
            RekeyError::GenericError(format!("failed to register rekeyRegister: {}", err))
        })?;

    return Result::Ok(());
}

fn handle_send_key(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context<'_>,
) -> Result<JsValue, JsError> {
    if args.len() != 1 && args.len() != 2 {
        return Result::Err(JsError::from(JsNativeError::error().with_message(
            format!("invalid number of arguments, expected sendKey(expr: string, direction?: 'up' | 'down') found {}",args.len())
        )));
    }

    let arg0 = args.get(0).unwrap();
    if !arg0.is_string() {
        return Result::Err(JsError::from(JsNativeError::error().with_message(format!(
            "invalid first argument, expected sendKey(expr: string, direction?: 'up' | 'down')"
        ))));
    }

    let mut key_direction = "both".to_string();
    if args.len() == 2 {
        let arg1 = args.get(1).unwrap();
        if !arg1.is_string() {
            return Result::Err(JsError::from(JsNativeError::error().with_message(format!(
                "invalid second argument, expected sendKey(expr: string, direction?: 'up' | 'down')"
            ))));
        }
        key_direction = arg1.as_string().unwrap().to_std_string_escaped();
    }
    if key_direction != "both" && key_direction != "up" && key_direction != "down" {
        return Result::Err(JsError::from(JsNativeError::error().with_message(format!(
            "invalid second argument, expected sendKey(expr: string, direction?: 'up' | 'down')"
        ))));
    }

    let key_expr = arg0.as_string().unwrap().to_std_string_escaped();

    let mut inputs: Vec<INPUT> = vec![];

    fn add_key_to_input(
        inputs: &mut Vec<INPUT>,
        key_expr_part: &str,
        up: bool,
    ) -> Result<(), JsError> {
        let r = to_virtual_key(key_expr_part).map_err(|err| {
            JsError::from(
                JsNativeError::error()
                    .with_message(format!("could not covert key {}: {}", key_expr_part, err)),
            )
        })?;

        if up {
            inputs.push(create_input(r.vkey, true));
        }
        if r.ctrl {
            inputs.push(create_input(VK_CONTROL, up));
        }
        if r.alt {
            inputs.push(create_input(VK_MENU, up));
        }
        if r.shift {
            inputs.push(create_input(VK_SHIFT, up));
        }
        if r.hankaku {
            return Result::Err(JsError::from(
                JsNativeError::error().with_message("could not handle hankaku"),
            ));
        }
        if !up {
            inputs.push(create_input(r.vkey, false));
        }
        return Result::Ok(());
    }

    let key_expr_parts: Vec<&str> = key_expr.split("+").collect();
    if key_direction == "both" || key_direction == "down" {
        for key_expr_part in &key_expr_parts {
            add_key_to_input(&mut inputs, key_expr_part, false)?;
        }
    }

    if key_direction == "both" || key_direction == "up" {
        let key_expr_parts_rev: Vec<&str> = key_expr_parts.iter().copied().rev().collect();
        for key_expr_part in key_expr_parts_rev {
            add_key_to_input(&mut inputs, key_expr_part, true)?;
        }
    }

    let input_size = size_of::<INPUT>();
    unsafe {
        let r = SendInput(&inputs, input_size as i32) as usize;
        if r != inputs.len() {
            return Result::Err(JsError::from(
                JsNativeError::error().with_message("failed to send all inputs"),
            ));
        }
    }

    return Result::Ok(JsValue::Undefined);
}

fn create_input(vkey: VIRTUAL_KEY, up: bool) -> INPUT {
    let mut input = INPUT::default();
    input.r#type = INPUT_KEYBOARD;
    input.Anonymous = INPUT_0::default();
    input.Anonymous.ki = KEYBDINPUT::default();
    input.Anonymous.ki.wVk = vkey;
    if up {
        input.Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
    }
    return input;
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
            if keys != "*" {
                return Result::Err(JsError::from(
                    JsNativeError::error()
                        .with_message("invalid keys arguments for rekeyRegister, expected \"*\""),
                ));
            }
            let devices = if devices == "*" {
                KeyHandlerDevices::All
            } else {
                KeyHandlerDevices::Contains(devices)
            };
            return Result::Ok(KeyHandler {
                devices,
                keys: KeyHandlerKeys::All,
                callback: callback.clone(),
            });
        } else {
            return Result::Err(JsError::from(
                JsNativeError::error()
                    .with_message("invalid arguments, expected rekeyRegister(devices: string, keys: string, callback: (ctx) => boolean)"),
            ));
        }
    } else {
        return Result::Err(JsError::from(JsNativeError::error().with_message(format!(
            "invalid arguments for rekeyRegister, expected 3 found {}",
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
