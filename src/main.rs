use anyhow::Result;
use log::trace;

mod carbleurator;
mod errors;
mod gamepad;
mod signaling;

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    trace!("Starting execution...");
    let mut car = carbleurator::Carbleurator::new()?;

    trace!("Carbleurator initialized. Starting event loop...");
    car.event_loop().await;
    trace!("Event loop terminated.");
    Ok(())
}
