use anyhow::Result;
use log::{debug, error};

use crate::bleserial::BleSerial;
use crate::gamepad::Gamepad;
use crate::motor_control;
use crate::signaling::{update_signal_failure, update_signal_progress, update_signal_success};
use btleplug::api::bleuuid;

const MAX_TIME_TX_DELAY: std::time::Duration = std::time::Duration::from_secs(5);
const LOOP_SLEEP_INCREMENT_MILLIS: u64 = 100;
const LOOP_SLEEP_MIN_MILLIS: u64 = 100;
const LOOP_SLEEP_MAX_MILLIS: u64 = 2000;

//const BLE_PERIPH_NAME: &str = "HC-08";
const BLE_PERIPH_NAME: &str = "Zazoom";
//const BLE_SVC_UUID: &str = "0000FFE0-0000-1000-8000-00805F9B34FB";
const BLE_SVC_UUID_SHORT: u16 = 0xFFE0;
//const BLE_CHR_UUID: &str = "0000FFE1-0000-1000-8000-00805F9B34FB";
//const BLE_CHR_UUID_SHORT: u16 = 0xFFE1;

pub(crate) struct Carbleurator {
    gamepad: Gamepad,
    serial_if: BleSerial,
}

impl Carbleurator {
    pub(crate) fn new() -> Result<Self> {
        debug!("Initializing gamepads...");
        let gamepad = Gamepad::new()?;
        let characteristic_uuid = bleuuid::uuid_from_u16(BLE_SVC_UUID_SHORT);
        let serial_if = BleSerial::new(characteristic_uuid, BLE_PERIPH_NAME.to_owned());
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

    async fn run_events(&mut self) -> Result<()> {
        update_signal_success();
        let mut last_x = 0;
        let mut last_y = 0;
        let mut last_update = std::time::Instant::now() - MAX_TIME_TX_DELAY;
        let mut loop_sleep_period = LOOP_SLEEP_MIN_MILLIS;
        loop {
            self.gamepad.update()?;
            let (x, y) = self.gamepad.read();
            //let msg = motor_control::input_to_message_digital(x, y);
            let msg = motor_control::input_to_message_analog(x, y);
            if x != last_x || y != last_y || last_update.elapsed() > MAX_TIME_TX_DELAY {
                debug!("Preparing to send message to vehicle: {:?}", msg);
                self.serial_if.send_message(&msg).await?;
                last_x = x;
                last_y = y;
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
