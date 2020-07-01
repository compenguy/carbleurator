use anyhow::Result;
use thiserror::Error;
// Bring in required traits
use anyhow::Context;
use btleplug::api::{Central, Peripheral};
use failure::Fail;

#[cfg(target_os = "linux")]
use btleplug::bluez::{adapter::ConnectedAdapter as BleAdapter, manager::Manager as BleManager};
#[cfg(target_os = "macos")]
use btleplug::corebluetooth::{adapter::Adapter as BleAdapter, manager::Manager as BleManager};
#[cfg(target_os = "windows")]
use btleplug::winrtble::{adapter::Adapter as BleAdapter, manager::Manager as BleManager};

#[derive(Error, Debug)]
pub enum CarbleuratorError {
    #[error("No BLE adapters found")]
    MissingBleAdapter,
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn get_central(manager: &BleManager) -> Result<BleAdapter> {
    manager
        .adapters()?
        .compat()?
        .first()
        .ok_or(CarbleuratorError::MissingBleAdapter)
}

#[cfg(target_os = "linux")]
fn get_central(manager: &BleManager) -> Result<BleAdapter> {
    let adapters = manager.adapters().map_err(|e| e.compat())?;
    let adapter = adapters
        .first()
        .ok_or(CarbleuratorError::MissingBleAdapter)?;
    adapter.connect().map_err(|e| e.compat().into())
}

fn main() -> Result<()> {
    // Init gamepads
    let mut gilrs = gilrs::Gilrs::new().expect("Failed to acquire gamepad input instance");

    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    // Init bluetooth
    let manager = BleManager::new()
        .map_err(|e| e.compat())
        .with_context(|| "Failed to initialize the BLE Manager".to_string())?;

    let central = get_central(&manager)?;

    central
        .start_scan()
        .map_err(|e| e.compat())
        .with_context(|| "Failed to scan for new BLE peripherals".to_string())?;

    std::thread::sleep(std::time::Duration::from_secs(2));

    for peripheral in central.peripherals() {
        println!(
            "{} ({})",
            peripheral.properties().local_name.unwrap_or_default(),
            peripheral.properties().address
        );
    }

    // Start event loop
    loop {
        while let Some(gilrs::Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
