use anyhow::{Context, Result};
use btleplug::api::{Central, Peripheral};
use failure::Fail;

use gilrs::ev::{Axis, Button, EventType};

use crate::ble;
use crate::errors::CarbleuratorError;
use crate::gamepad;
use crate::signaling::{update_signal_failure, update_signal_progress, update_signal_success};

const BLE_PERIPH_NAME: &str = "HC-08";
//const BLE_SVC_UUID: &str = "0000FFE0-0000-1000-8000-00805F9B34FB";
//const BLE_SVC_HANDLE: u16 = 0xFFE0;
//const BLE_CHR_UUID: &str = "0000FFE1-0000-1000-8000-00805F9B34FB";
const BLE_CHR_HANDLE: u16 = 0xFFE1;

pub(crate) struct Carbleurator {
    gilrs: gilrs::Gilrs,
    adapter: ble::Adapter,
    d_x: i8,
    d_y: i8,
}

impl Carbleurator {
    pub(crate) fn new() -> Result<Self> {
        let result = Self::init();
        match result {
            Ok(_) => update_signal_progress(),
            Err(_) => update_signal_failure(),
        }
        result
    }

    fn init() -> Result<Self> {
        update_signal_progress();
        let gilrs = gamepad::init_gamepads()?;
        // debug:
        //for (_id, gamepad) in gilrs.gamepads() {
        //    println!("{} is {:?}", gamepad.name(), gamepad.power_info());
        //}
        update_signal_progress();

        // Init bluetooth
        let manager = ble::Manager::new().map_err(|e| e.compat())?;

        update_signal_progress();

        let adapter = ble::get_central(&manager)?;

        Ok(Carbleurator {
            gilrs,
            adapter,
            d_x: 0,
            d_y: 0,
        })
    }

    pub(crate) fn event_loop(&mut self) {
        loop {
            let _result = self.run_events();
            update_signal_failure();
            std::thread::sleep(std::time::Duration::from_secs(1));
            update_signal_progress();
        }
    }

    fn run_events(&mut self) -> Result<()> {
        update_signal_progress();
        self.adapter
            .start_scan()
            .map_err(|e| e.compat())
            .with_context(|| "Failed to scan for new BLE peripherals".to_string())?;

        update_signal_progress();

        std::thread::sleep(std::time::Duration::from_secs(1));

        update_signal_progress();

        let mut counter = 0;
        let mut opt_peripheral = None;
        while counter <= 5 && opt_peripheral.is_none() {
            opt_peripheral = self
                .adapter
                .peripherals()
                .into_iter()
                .find(|x| x.properties().local_name == Some(BLE_PERIPH_NAME.to_string()));
            if opt_peripheral.is_none() {
                counter += 1;
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            update_signal_progress();
        }

        let peripheral = opt_peripheral.ok_or(CarbleuratorError::BleAdapterDiscoveryTimeout)?;
        peripheral.connect().map_err(|e| e.compat())?;
        update_signal_progress();

        let res_characteristics = peripheral
            .discover_characteristics_in_range(BLE_CHR_HANDLE, BLE_CHR_HANDLE)
            .map_err(|e| e.compat())?;
        let characteristic = res_characteristics
            .first()
            .ok_or(CarbleuratorError::BleAdapterMissingCharacteristic)?;

        update_signal_success();
        loop {
            while let Some(gilrs::Event { event, .. }) = self.gilrs.next_event() {
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
                (_, 64..=127) => b"f",
                (_, -128..=-64) => b"b",
                (-128..=-64, -63..=63) => b"l",
                (64..=127, -63..=63) => b"r",
            };
            peripheral
                .command(characteristic, msg)
                .map_err(|e| e.compat())?;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
