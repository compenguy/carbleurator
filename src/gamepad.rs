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

    pub(crate) fn read(&mut self) -> Option<(i8, i8)> {
        let mut new_x = self.d_x;
        let mut new_y = self.d_y;
        while let Some(gilrs::Event { event, .. }) = self.gilrs.next_event() {
            trace!("Processing input event {:?}", event);
            match event {
                EventType::ButtonPressed(Button::DPadLeft, _) => new_x = -128,
                EventType::ButtonReleased(Button::DPadLeft, _) => new_x = 0,
                EventType::ButtonPressed(Button::DPadRight, _) => new_x = 127,
                EventType::ButtonReleased(Button::DPadRight, _) => new_x = 0,
                EventType::ButtonPressed(Button::DPadUp, _) => new_y = -128,
                EventType::ButtonReleased(Button::DPadUp, _) => new_y = 0,
                EventType::ButtonPressed(Button::DPadDown, _) => new_y = 127,
                EventType::ButtonReleased(Button::DPadDown, _) => new_y = 0,
                EventType::AxisChanged(Axis::DPadX, d_x, _) => new_x = (d_x * 128f32) as i8,
                EventType::AxisChanged(Axis::DPadY, d_y, _) => new_y = (d_y * 128f32) as i8,
                EventType::AxisChanged(Axis::LeftStickX, d_x, _) => new_x = (d_x * 128f32) as i8,
                EventType::AxisChanged(Axis::LeftStickY, d_y, _) => new_y = (d_y * 128f32) as i8,
                EventType::AxisChanged(Axis::RightStickX, d_x, _) => new_x = (d_x * 128f32) as i8,
                EventType::AxisChanged(Axis::RightStickY, d_y, _) => new_y = (d_y * 128f32) as i8,
                n => {
                    warn!("Unhandled input event {:?}", n);
                }
            }
            trace!(
                "Intermediate gamepad state: x: {:3} y: {:3}",
                &new_x,
                &new_y
            );
        }
        if new_x == self.d_x && new_y == self.d_y {
            None
        } else {
            debug!("Updated gamepad state: x: {:3} y: {:3}", &new_x, &new_y);
            self.d_x = new_x;
            self.d_y = new_y;
            Some((new_x, new_y))
        }
    }
}
