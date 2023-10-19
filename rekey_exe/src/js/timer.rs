use std::{
    cmp,
    ops::Add,
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc, Mutex,
    },
    time::{Duration, SystemTime},
};

use boa_engine::{Context, JsError, JsNativeError, JsObject, JsValue, NativeFunction};
use rekey_common::{debug, RekeyError};

use crate::scripts::Script;

static NEXT_ID: AtomicU16 = AtomicU16::new(1);

pub struct Timer {
    id: i32,
    time: Duration,
    callback: JsObject,
}

impl Timer {
    pub fn init(
        context: &mut Context<'_>,
        timers: &Arc<Mutex<Vec<Timer>>>,
    ) -> Result<(), RekeyError> {
        let set_timeout_timers = Arc::clone(timers);
        context
            .register_global_callable("setTimeout", 0, unsafe {
                NativeFunction::from_closure(move |this, args, context| {
                    return handle_set_timeout(
                        this,
                        args,
                        context,
                        Arc::clone(&set_timeout_timers),
                    );
                })
            })
            .map_err(|err| {
                RekeyError::GenericError(format!("failed to register rekeyRegister: {}", err))
            })?;

        let clear_timeout_timers = Arc::clone(timers);
        context
            .register_global_callable("clearTimeout", 0, unsafe {
                NativeFunction::from_closure(move |this, args, context| {
                    return handle_clear_timeout(
                        this,
                        args,
                        context,
                        Arc::clone(&clear_timeout_timers),
                    );
                })
            })
            .map_err(|err| {
                RekeyError::GenericError(format!("failed to register rekeyRegister: {}", err))
            })?;

        return Result::Ok(());
    }

    pub fn get_nearest_duration(scripts: &Vec<Script<'_>>) -> Result<Option<Duration>, RekeyError> {
        let mut results = Option::None;
        for script in scripts {
            let timers = script.timers.lock().map_err(|err| {
                RekeyError::GenericError(format!("could not get timers lock: {}", err))
            })?;
            for timer in timers.iter() {
                if results.is_none() {
                    results = Option::Some(timer.time);
                } else if let Option::Some(d) = results {
                    if timer.time.lt(&d) {
                        results = Option::Some(timer.time);
                    }
                }
            }
        }

        if let Option::Some(d) = results {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|err| RekeyError::GenericError(format!("failed to get now: {}", err)))?
                .as_millis();
            let d = d.as_millis();
            let dt = cmp::max(0, d - now) as u64;
            return Result::Ok(Option::Some(Duration::from_millis(dt)));
        } else {
            return Result::Ok(Option::None);
        }
    }

    pub fn run_timers(scripts: &Vec<Script<'_>>) -> Result<(), RekeyError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|err| RekeyError::GenericError(format!("failed to get now: {}", err)))?;

        for script in scripts {
            let mut timers = script.timers.lock().map_err(|err| {
                RekeyError::GenericError(format!("could not get timers lock: {}", err))
            })?;
            timers.retain(|timer| {
                if now.ge(&timer.time) {
                    run_timer(&script, &timer)
                        .unwrap_or_else(|err| debug!("failed to run timeout: {}", err));
                    return false;
                }
                return true;
            });
        }
        return Result::Ok(());
    }
}

fn run_timer(script: &Script<'_>, timer: &Timer) -> Result<(), RekeyError> {
    let mut context = script
        .context
        .lock()
        .map_err(|err| RekeyError::GenericError(format!("failed to lock context: {}", err)))?;
    let args: [JsValue; 0] = [];
    let this = JsValue::Undefined;

    timer
        .callback
        .call(&this, &args, &mut context)
        .map_err(|err| RekeyError::GenericError(format!("{}", err)))?;

    return Result::Ok(());
}

fn handle_clear_timeout(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context<'_>,
    timers: Arc<Mutex<Vec<Timer>>>,
) -> Result<JsValue, JsError> {
    match _handle_clear_timeout(this, args, context, timers) {
        Result::Ok(()) => {
            return Result::Ok(JsValue::undefined());
        }
        Result::Err(err) => {
            return Result::Err(JsError::from(
                JsNativeError::error().with_message(format!("{}", err)),
            ));
        }
    };
}

fn _handle_clear_timeout(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context<'_>,
    timers: Arc<Mutex<Vec<Timer>>>,
) -> Result<(), RekeyError> {
    if args.len() == 1 {
        let id = args.get(0).unwrap();
        if id.is_number() {
            let mut timers = timers.lock().map_err(|err| {
                RekeyError::GenericError(format!("could not get timers lock: {}", err))
            })?;

            let id = id.as_number().unwrap() as i32;
            timers.retain(|timer| timer.id != id);
            return Result::Ok(());
        } else {
            return Result::Err(RekeyError::GenericError(
                "invalid arguments, expected clearTimeout(timer: number)".to_string(),
            ));
        }
    } else {
        return Result::Err(RekeyError::GenericError(format!(
            "invalid arguments for clearTimeout, expected 1 found {}",
            args.len()
        )));
    }
}

fn handle_set_timeout(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context<'_>,
    timers: Arc<Mutex<Vec<Timer>>>,
) -> Result<JsValue, JsError> {
    match _handle_set_timeout(this, args, context) {
        Result::Ok(timer) => {
            let mut timers = timers.lock().map_err(|err| {
                JsNativeError::error().with_message(format!("could not get timers lock: {}", err))
            })?;
            let id = timer.id;
            timers.push(timer);
            return Result::Ok(JsValue::Integer(id));
        }
        Result::Err(err) => {
            return Result::Err(JsError::from(
                JsNativeError::error().with_message(format!("{}", err)),
            ));
        }
    };
}

fn _handle_set_timeout(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context<'_>,
) -> Result<Timer, RekeyError> {
    if args.len() == 2 {
        let callback = args.get(0).unwrap();
        let ms = args.get(1).unwrap();

        if callback.is_callable() && ms.is_number() {
            let ms = ms.as_number().unwrap();
            let callback = callback.as_callable().unwrap();
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|err| RekeyError::GenericError(format!("failed to get now: {}", err)))?;
            let mut id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            // javascript treats 0 as false so lets avoid that
            if id == 0 {
                id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            }
            return Result::Ok(Timer {
                id: id as i32,
                time: now.add(Duration::from_millis(ms as u64)),
                callback: callback.clone(),
            });
        } else {
            return Result::Err(RekeyError::GenericError("invalid arguments, expected setTimeout(callback: () => unknown, timeMillis: number)".to_string()));
        }
    } else {
        return Result::Err(RekeyError::GenericError(format!(
            "invalid arguments for setTimeout, expected 2 found {}",
            args.len()
        )));
    }
}
