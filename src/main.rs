use anyhow::Result;
use log::debug;

mod bleserial;
mod carbleurator;
mod errors;
mod gamepad;
mod motor_control;
mod signaling;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();
    debug!("Starting execution...");
    let mut car = carbleurator::Carbleurator::new()?;

    debug!("Carbleurator initialized. Starting event loop...");
    car.event_loop().await;
    debug!("Event loop terminated.");
    Ok(())
}
