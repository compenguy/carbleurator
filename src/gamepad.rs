use anyhow::Result;
use log::{debug, trace, warn};

use crate::errors::CarbleuratorError;
use gilrs::ev::{Axis, Button, EventType};

pub(crate) struct Gamepad {
    gilrs: gilrs::Gilrs,
    d_x: i8,
    d_y: i8,
}

impl Gamepad {
    pub(crate) fn new() -> Result<Self> {
        let gilrs = gilrs::Gilrs::new().map_err(CarbleuratorError::from)?;
        if gilrs.gamepads().count() == 0 {
            return Err(CarbleuratorError::MissingGamepad.into());
        }
        for (_id, gamepad) in gilrs.gamepads() {
            debug!("{} is {:?}", gamepad.name(), gamepad.power_info());
        }
        Ok(Self {
            gilrs,
            d_x: 0,
            d_y: 0,
        })
    }

    pub(crate) fn update(&mut self) -> Result<()> {
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
                EventType::AxisChanged(Axis::LeftStickX, d_x, _) => self.d_x = (d_x * 128f32) as i8,
                EventType::AxisChanged(Axis::LeftStickY, d_y, _) => self.d_y = (d_y * 128f32) as i8,
                EventType::AxisChanged(Axis::RightStickX, d_x, _) => {
                    self.d_x = (d_x * 128f32) as i8
                }
                EventType::AxisChanged(Axis::RightStickY, d_y, _) => {
                    self.d_y = (d_y * 128f32) as i8
                }
                n => {
                    warn!("Unhandled input event {:?}", n);
                }
            }
            debug!(
                "Updated gamepad state: x: {:3} y: {:3}",
                &self.d_x, &self.d_y
            );
        }
        Ok(())
    }

    pub(crate) fn read(&self) -> (i8, i8) {
        (self.d_x, self.d_y)
    }
}
