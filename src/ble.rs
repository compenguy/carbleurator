use anyhow::Result;
// Bring in required traits
use failure::Fail;

use crate::errors::CarbleuratorError;

#[cfg(target_os = "linux")]
pub(crate) use btleplug::bluez::{adapter::ConnectedAdapter as Adapter, manager::Manager};
#[cfg(target_os = "macos")]
pub(crate) use btleplug::corebluetooth::{adapter::Adapter, manager::Manager};
#[cfg(target_os = "windows")]
pub(crate) use btleplug::winrtble::{adapter::Adapter, manager::Manager};

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub(crate) fn get_central(manager: &Manager) -> Result<Adapter> {
    manager
        .adapters()?
        .compat()?
        .first()
        .ok_or(CarbleuratorError::MissingBleAdapter)
}

#[cfg(target_os = "linux")]
pub(crate) fn get_central(manager: &Manager) -> Result<Adapter> {
    let adapters = manager.adapters().map_err(|e| e.compat())?;
    let adapter = adapters
        .first()
        .ok_or(CarbleuratorError::MissingBleAdapter)?;
    adapter.connect().map_err(|e| e.compat().into())
}
