use anyhow::Result;

mod ble;
mod carbleurator;
mod errors;
mod gamepad;
mod signaling;

fn main() -> Result<()> {
    let mut car = carbleurator::Carbleurator::new()?;

    car.event_loop();
    Ok(())
}
