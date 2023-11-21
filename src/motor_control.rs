use log::debug;

const INPUT_DEADZONE_MIN: i8 = -63;
const INPUT_DEADZONE_MAX: i8 = 63;
const INPUT_ACTIVE_POS_MIN: i8 = 64;
const INPUT_ACTIVE_POS_MAX: i8 = 127;
const INPUT_ACTIVE_NEG_MIN: i8 = -128;
const INPUT_ACTIVE_NEG_MAX: i8 = -64;

#[allow(dead_code)]
pub(crate) fn input_to_message_digital(x: i8, y: i8) -> u8 {
    match (x, y) {
        (INPUT_DEADZONE_MIN..=INPUT_DEADZONE_MAX, INPUT_DEADZONE_MIN..=INPUT_DEADZONE_MAX) => b's',
        (_, INPUT_ACTIVE_POS_MIN..=INPUT_ACTIVE_POS_MAX) => b'b',
        (_, INPUT_ACTIVE_NEG_MIN..=INPUT_ACTIVE_NEG_MAX) => b'f',
        (INPUT_ACTIVE_NEG_MIN..=INPUT_ACTIVE_NEG_MAX, INPUT_DEADZONE_MIN..=INPUT_DEADZONE_MAX) => {
            b'l'
        }
        (INPUT_ACTIVE_POS_MIN..=INPUT_ACTIVE_POS_MAX, INPUT_DEADZONE_MIN..=INPUT_DEADZONE_MAX) => {
            b'r'
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum ThreeBitAnalogMotor {
    S0 = 0b000,
    S25 = 0b001,
    S50 = 0b010,
    S75 = 0b011,
    S100 = 0b100,
    R33 = 0b101,
    R66 = 0b110,
    R100 = 0b111,
}

impl From<i8> for ThreeBitAnalogMotor {
    fn from(speed: i8) -> Self {
        match speed {
            0 => Self::S0,
            1..=32 => Self::S25,
            33..=64 => Self::S50,
            65..=97 => Self::S75,
            98..=127 => Self::S100,
            -43..=-1 => Self::R33,
            -86..=-44 => Self::R66,
            -128..=-87 => Self::R100,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SixBitAnalogFullDrive {
    left: ThreeBitAnalogMotor,
    right: ThreeBitAnalogMotor,
}

impl From<(i8, i8)> for SixBitAnalogFullDrive {
    fn from(xy: (i8, i8)) -> Self {
        let rotation = (xy.0 as f32) / (i8::MAX as f32);
        let speed = (xy.1 as f32) / (i8::MAX as f32);
        debug!(
            "Analog drive forward speed: {}, turn speed: {}",
            speed, rotation
        );
        let mut left: f32 = speed;
        let mut right: f32 = speed;

        left -= rotation;
        right += rotation;
        debug!("Analog drive prescaling left  speed: {}", left);
        debug!("Analog drive prescaling right speed: {}", right);

        // Now determine the scaling factor to apply to re-scale it to -1.0 to 1.0
        let greater: f32 = speed.abs().min(rotation.abs());
        let lesser: f32 = speed.abs().max(rotation.abs());

        // Avoid a divide-by-zero
        if greater == 0.0 {
            left = 0.0;
            right = 0.0;
        } else {
            let scaling_factor: f32 = (greater + lesser) / greater;
            left /= scaling_factor;
            right /= scaling_factor;
        }

        debug!("Analog drive scaled left  speed: {}", left);
        debug!("Analog drive scaled right speed: {}", right);

        let left_analog = ThreeBitAnalogMotor::from((left * i8::MAX as f32) as i8);
        let right_analog = ThreeBitAnalogMotor::from((right * i8::MAX as f32) as i8);

        SixBitAnalogFullDrive {
            left: left_analog,
            right: right_analog,
        }
    }
}

impl From<SixBitAnalogFullDrive> for u8 {
    fn from(sixbitdrive: SixBitAnalogFullDrive) -> u8 {
        ((sixbitdrive.left as u8) << 3) | (sixbitdrive.right as u8)
    }
}

#[allow(dead_code)]
pub(crate) fn input_to_message_analog(x: i8, y: i8) -> u8 {
    let six_bit_analog = SixBitAnalogFullDrive::from((x, y));
    debug!("Analog drive levels: {:?}", six_bit_analog);
    let message: u8 = u8::from(six_bit_analog);
    debug!("Analog drive encoded message: {:0<8b}", message);
    message
}
