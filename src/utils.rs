use std::time::{SystemTime, UNIX_EPOCH};

use crate::jlog;

pub struct Utils {}

impl Utils {
    pub(crate) fn get_current_time() -> jlog::Result<i64> {
        let current_time_as_u128 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

        let current_time = i64::try_from(current_time_as_u128)?;

        Ok(current_time)
    }
}
