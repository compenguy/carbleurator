use anyhow::Result;
use log::trace;

mod ble;
mod carbleurator;
mod errors;
mod gamepad;
mod signaling;

fn main() -> Result<()> {
    env_logger::init();
    trace!("Starting execution...");
    let mut car = carbleurator::Carbleurator::new()?;

    trace!("Carbleurator initialized. Starting event loop...");
    car.event_loop();
    trace!("Event loop terminated.");
    Ok(())
}
