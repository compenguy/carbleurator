use anyhow::Result;

use crate::errors::CarbleuratorError;

pub(crate) fn init_gamepads() -> Result<gilrs::Gilrs> {
    let gilrs = gilrs::Gilrs::new().map_err(CarbleuratorError::from)?;
    if gilrs.gamepads().count() == 0 {
        return Err(CarbleuratorError::MissingGamepad.into());
    }
    Ok(gilrs)
}
