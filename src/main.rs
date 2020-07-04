use anyhow::Result;
// Bring in required traits
use anyhow::Context;
use btleplug::api::{Central, Peripheral};
use failure::Fail;

mod ble;
mod errors;
mod gamepad;
mod signaling;

fn main() -> Result<()> {
    signaling::update_signal_progress();
    // Init gamepads
    let mut gilrs = gamepad::init_gamepads()?;
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }
    signaling::update_signal_progress();

    // Init bluetooth
    let manager = ble::Manager::new().map_err(|e| e.compat())?;

    signaling::update_signal_progress();

    let central = ble::get_central(&manager)?;

    central
        .start_scan()
        .map_err(|e| e.compat())
        .with_context(|| "Failed to scan for new BLE peripherals".to_string())?;

    signaling::update_signal_progress();
    std::thread::sleep(std::time::Duration::from_secs(2));
    signaling::update_signal_progress();

    for peripheral in central.peripherals() {
        println!(
            "{} ({})",
            peripheral.properties().local_name.unwrap_or_default(),
            peripheral.properties().address
        );
    }

    signaling::update_signal_success();
    // Start event loop
    loop {
        while let Some(gilrs::Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
