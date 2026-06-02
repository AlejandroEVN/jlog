use std::time::{SystemTime, UNIX_EPOCH};

pub struct Utils {}

impl Utils {
    pub(crate) fn get_current_time() -> Result<i64, String> {
        let current_time_as_u128 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "System clock is set before UNIX EPOCH".to_string())?
            .as_millis();

        let current_time = i64::try_from(current_time_as_u128)
            .map_err(|_| "System timestamp is too large to fit into i64".to_string())?;

        Ok(current_time)
    }
}
