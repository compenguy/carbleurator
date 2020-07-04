#[cfg(feature = "rpi")]
const RED_LED: &str = "/sys/class/leds/led0";
#[cfg(feature = "rpi")]
const GREEN_LED: &str = "/sys/class/leds/led1";

#[cfg(feature = "rpi")]
fn get_led_state(path: &str) -> u8 {
    let mut file =
        std::fs::File::open(path).expect(format!("Failed to open led device {} for reading", path));
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect(format!(
        "Failed to read current state of led device {}",
        path
    ));
    contents.parse::<u8>().unwrap_or(0)
}

#[cfg(feature = "rpi")]
fn set_led_state(path: &str, new_state: u8) {
    let mut out_file = std::fs::File::create(path)
        .expect(format!("Failed to open led device {} for writing", path));
    out_file
        .write(new_state.to_string())
        .expect(format!("Failed to write new state for led device {}", path));
}

#[cfg(feature = "rpi")]
// /sys/class/leds/led0
pub(crate) fn update_signal_failure() {
    // Toggle red, and make sure green is off
    let new_state = match get_led_state(RED_LED) {
        255 => 0,
        _ => 255,
    };
    set_led_state(RED_LED, new_state);
    set_led_state(GREEN_LED, 0);
}

#[cfg(feature = "rpi")]
// /sys/class/leds/led0
pub(crate) fn update_signal_progress() {
    // Toggle green, and make sure red is off
    let new_state = match get_led_state(GREEN_LED) {
        255 => 0,
        _ => 255,
    };
    set_led_state(GREEN_LED, new_state);
    set_led_state(RED_LED, 0);
}

#[cfg(feature = "rpi")]
// /sys/class/leds/led0
pub(crate) fn update_signal_success() {
    // Set red to off, green to on
    set_led_state(RED_LED, 0);
    set_led_state(GREEN_LED, 255);
}

#[cfg(not(feature = "rpi"))]
pub(crate) fn update_signal_failure() {
    println!("Bad");
}

#[cfg(not(feature = "rpi"))]
pub(crate) fn update_signal_progress() {
    println!("Working");
}

#[cfg(not(feature = "rpi"))]
pub(crate) fn update_signal_success() {
    println!("Ready");
}
