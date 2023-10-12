use lazy_static::lazy_static;
use rekey_common::KeyDirection;
use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

use crate::{devices::Device, RekeyError};

const MAX_INPUT_LOG_LENGTH: usize = 100;

struct InputLogItem {
    time: SystemTime,
    device: Arc<Device>,
    vkey_code: u16,
    direction: KeyDirection,
}

lazy_static! {
    static ref INPUT_LOG: Mutex<Vec<InputLogItem>> = Mutex::new(vec![]);
}

pub fn input_log_add_wm_input(
    device: Arc<Device>,
    vkey_code: u16,
    direction: KeyDirection,
) -> Result<(), RekeyError> {
    let now = SystemTime::now();
    let mut input_log = INPUT_LOG.lock().map_err(|err| {
        RekeyError::GenericError(format!("could not get input log lock: {}", err))
    })?;

    input_log.push(InputLogItem {
        time: now,
        device,
        vkey_code,
        direction,
    });

    while input_log.len() > MAX_INPUT_LOG_LENGTH {
        input_log.remove(0);
    }

    return Result::Ok(());
}

pub fn input_log_get_device(
    vkey_code: u16,
    direction: KeyDirection,
) -> Result<Option<Arc<Device>>, RekeyError> {
    let now = SystemTime::now();
    let mut input_log = INPUT_LOG.lock().map_err(|err| {
        RekeyError::GenericError(format!("could not get input log lock: {}", err))
    })?;

    for i in 0..input_log.len() {
        if let Option::Some(item) = input_log.get(i) {
            if item.vkey_code == vkey_code && item.direction == direction {
                let delta_t = now
                    .duration_since(item.time)
                    .map_err(|err| RekeyError::GenericError(format!("time error: {}", err)))?;
                if delta_t.as_millis() < 1000 {
                    let removed_item = input_log.remove(i);
                    return Result::Ok(Option::Some(removed_item.device));
                }
            }
        }
    }

    return Result::Ok(Option::None);
}
