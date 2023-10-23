use anyhow::Result;
use log::trace;

mod bleserial;
mod carbleurator;
mod errors;
mod gamepad;
mod motor_control;
mod signaling;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();
    trace!("Starting execution...");
    let mut car = carbleurator::Carbleurator::new()?;

    trace!("Carbleurator initialized. Starting event loop...");
    car.event_loop().await;
    trace!("Event loop terminated.");
    Ok(())
}
