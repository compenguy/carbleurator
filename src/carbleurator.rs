use anyhow::Result;
use log::{debug, error};

use crate::bleserial::BleSerial;
use crate::gamepad::Gamepad;
use crate::motor_control;
use crate::signaling::{update_signal_failure, update_signal_progress, update_signal_success};
use btleplug::api::bleuuid;

const MAX_TIME_TX_DELAY: std::time::Duration = std::time::Duration::from_millis(200);
const LOOP_SLEEP_INCREMENT_MILLIS: u64 = 25;
const LOOP_SLEEP_MIN_MILLIS: u64 = 25;
const LOOP_SLEEP_MAX_MILLIS: u64 = 100;

const SUPPORTED_INTERFACES: [(&str, u16); 2] = [
    ("HC-08", 0xFFE0),  // Out-of-box configuration
    ("Zazoom", 0xFFE1), // Custom configuration
];

pub(crate) struct Carbleurator {
    gamepad: Gamepad,
    serial_if: BleSerial,
}

impl Carbleurator {
    pub(crate) fn new() -> Result<Self> {
        debug!("Initializing gamepads...");
        let gamepad = Gamepad::new()?;
        let characteristic_uuid = bleuuid::uuid_from_u16(SUPPORTED_INTERFACES[1].1);
        let serial_if = BleSerial::new(characteristic_uuid, SUPPORTED_INTERFACES[1].0.to_owned());
        debug!("Carbleurator initialized.");
        update_signal_progress();
        Ok(Carbleurator { gamepad, serial_if })
    }

    pub(crate) async fn event_loop(&mut self) {
        update_signal_progress();
        debug!("Gamepad input configured, connected to compatible car, starting control loop...");
        loop {
            debug!("Starting event processing...");
            if let Err(e) = self.run_events().await {
                error!("Event processing failed with error {}", e);
                update_signal_failure();
            }

            std::thread::sleep(std::time::Duration::from_secs(3));
            update_signal_progress();
        }
    }

    fn read_gamepad(&mut self) -> Option<u8> {
        self.gamepad
            .read()
            .map(|(x, y)| motor_control::input_to_message_analog(x, y))
    }

    async fn run_events(&mut self) -> Result<()> {
        update_signal_success();
        let mut last_msg: u8 = 0;
        let mut last_update = std::time::Instant::now() - MAX_TIME_TX_DELAY;
        let mut loop_sleep_period = LOOP_SLEEP_MIN_MILLIS;
        loop {
            let msg = self.read_gamepad().unwrap_or(last_msg);
            if msg != last_msg || last_update.elapsed() > MAX_TIME_TX_DELAY {
                self.serial_if.send_message(msg).await?;
                last_msg = msg;
                last_update = std::time::Instant::now();
                loop_sleep_period = LOOP_SLEEP_MIN_MILLIS;
            } else {
                loop_sleep_period = std::cmp::min(
                    loop_sleep_period + LOOP_SLEEP_INCREMENT_MILLIS,
                    LOOP_SLEEP_MAX_MILLIS,
                );
            }

            tokio::time::sleep(std::time::Duration::from_millis(loop_sleep_period)).await;
        }
    }
}
