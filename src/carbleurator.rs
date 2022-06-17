use anyhow::{Context, Result};
use btleplug::api::Manager as _;
use btleplug::api::{bleuuid, Central, Peripheral, ScanFilter, WriteType};
//use failure::Fail;
use log::{debug, error, trace, warn};

use btleplug::platform::{Adapter, Manager};
use gilrs::ev::{Axis, Button, EventType};

use crate::errors::CarbleuratorError;
use crate::gamepad;
use crate::signaling::{update_signal_failure, update_signal_progress, update_signal_success};

const BLE_PERIPH_NAME: &str = "HC-08";
//const BLE_SVC_UUID: &str = "0000FFE0-0000-1000-8000-00805F9B34FB";
//const BLE_SVC_UUID_SHORT: u16 = 0xFFE0;
//const BLE_CHR_UUID: &str = "0000FFE1-0000-1000-8000-00805F9B34FB";
const BLE_CHR_UUID_SHORT: u16 = 0xFFE1;

pub(crate) struct Carbleurator {
    gilrs: gilrs::Gilrs,
    adapter: Option<Adapter>,
    d_x: i8,
    d_y: i8,
}

impl Carbleurator {
    pub(crate) fn new() -> Result<Self> {
        trace!("Initializing gamepads...");
        let gilrs = gamepad::init_gamepads()?;
        for (_id, gamepad) in gilrs.gamepads() {
            debug!("{} is {:?}", gamepad.name(), gamepad.power_info());
        }
        trace!("Carbleurator initialized.");
        Ok(Carbleurator {
            gilrs,
            adapter: None,
            d_x: 0,
            d_y: 0,
        })
    }

    async fn connect(&mut self) -> Result<&Adapter> {
        if self.adapter.is_none() {
            update_signal_progress();

            trace!("Initializing bluetooth...");
            let manager = Manager::new().await?;

            update_signal_progress();

            trace!("Initializing BLE central...");
            let adapter = manager
                .adapters()
                .await?
                .into_iter()
                .next()
                .ok_or(CarbleuratorError::MissingBleAdapter)?;
            self.adapter = Some(adapter);
            update_signal_progress();
        }
        if let Some(adapter) = &self.adapter {
            Ok(adapter)
        } else {
            Err(CarbleuratorError::MissingBleAdapter.into())
        }
    }

    pub(crate) async fn event_loop(&mut self) {
        loop {
            trace!("Starting event processing...");
            if let Err(e) = self.run_events().await {
                error!("Event processing failed with error {}", e);
                update_signal_failure();
            }

            std::thread::sleep(std::time::Duration::from_secs(3));
            update_signal_progress();
            trace!("Retrying event processing...");
        }
    }

    async fn run_events(&mut self) -> Result<()> {
        let adapter = self.connect().await?;
        let peripheral_uuid = bleuuid::uuid_from_u16(BLE_CHR_UUID_SHORT);
        update_signal_progress();
        trace!("Starting scan for BLE peripherals...");
        adapter
            .start_scan(ScanFilter {
                services: vec![peripheral_uuid],
            })
            .await
            .with_context(|| "Failed to scan for new BLE peripherals".to_string())?;

        update_signal_progress();

        trace!("Waiting for devices to appear...");
        std::thread::sleep(std::time::Duration::from_secs(1));

        update_signal_progress();

        let mut counter = 0;
        let mut opt_peripheral = None;
        trace!("Iterating over discovered devices searching for a compatible peripheral...");
        while counter <= 5 && opt_peripheral.is_none() {
            let mut peripherals = adapter.peripherals().await?.into_iter();
            opt_peripheral = loop {
                if let Some(peripheral) = peripherals.next() {
                    if let Some(props) = peripheral.properties().await? {
                        if props.local_name == Some(BLE_PERIPH_NAME.to_string()) {
                            break Some(peripheral);
                        }
                    }
                } else {
                    break None;
                }
            };
            if opt_peripheral.is_none() {
                warn!("No compatible BLE peripherals found. Retrying...");
                counter += 1;
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            update_signal_progress();
        }

        let peripheral = opt_peripheral.ok_or(CarbleuratorError::BleAdapterDiscoveryTimeout)?;
        trace!("BLE peripheral found. Connecting...");
        peripheral.connect().await?;
        update_signal_progress();

        trace!("Searching for correct peripheral characteristic for communication...");
        peripheral.discover_services().await?;
        let characteristic = peripheral
            .characteristics()
            .into_iter()
            .find(|x| x.uuid == peripheral_uuid)
            .ok_or(CarbleuratorError::BleAdapterMissingCharacteristic)?;

        trace!("Gamepad input configured, connected to compatible car, starting control loop...");
        update_signal_success();
        loop {
            while let Some(gilrs::Event { event, .. }) = self.gilrs.next_event() {
                trace!("Processing input event {:?}", event);
                match event {
                    EventType::ButtonPressed(Button::DPadLeft, _) => self.d_x = -128,
                    EventType::ButtonReleased(Button::DPadLeft, _) => self.d_x = 0,
                    EventType::ButtonPressed(Button::DPadRight, _) => self.d_x = 127,
                    EventType::ButtonReleased(Button::DPadRight, _) => self.d_x = 0,
                    EventType::ButtonPressed(Button::DPadUp, _) => self.d_y = -128,
                    EventType::ButtonReleased(Button::DPadUp, _) => self.d_y = 0,
                    EventType::ButtonPressed(Button::DPadDown, _) => self.d_y = 127,
                    EventType::ButtonReleased(Button::DPadDown, _) => self.d_y = 0,
                    EventType::AxisChanged(Axis::DPadX, d_x, _) => self.d_x = (d_x * 128f32) as i8,
                    EventType::AxisChanged(Axis::DPadY, d_y, _) => self.d_y = (d_y * 128f32) as i8,
                    EventType::AxisChanged(Axis::LeftStickX, d_x, _) => {
                        self.d_x = (d_x * 128f32) as i8
                    }
                    EventType::AxisChanged(Axis::LeftStickY, d_y, _) => {
                        self.d_y = (d_y * 128f32) as i8
                    }
                    EventType::AxisChanged(Axis::RightStickX, d_x, _) => {
                        self.d_x = (d_x * 128f32) as i8
                    }
                    EventType::AxisChanged(Axis::RightStickY, d_y, _) => {
                        self.d_y = (d_y * 128f32) as i8
                    }
                    _ => {}
                }
            }
            let msg: &[u8; 1] = match (self.d_x, self.d_y) {
                (-63..=63, -63..=63) => b"s",
                (_, 64..=127) => b"b",
                (_, -128..=-64) => b"f",
                (-128..=-64, -63..=63) => b"l",
                (64..=127, -63..=63) => b"r",
            };
            // TODO: only send message if the msg value changed or after x amount of seconds have
            // passed
            trace!("Preparing to send message to vehicle: {:?}", msg);
            peripheral
                .write(&characteristic, msg, WriteType::WithoutResponse)
                .await?;

            // TODO: crank up sleep time each period we go without getting input, up to a
            // predetermined limit, but reset the sleep time once we do get input
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
