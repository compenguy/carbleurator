fn main() {
    let mut gilrs = gilrs::Gilrs::new().expect("Failed to acquire gamepad input instance");

    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    loop {
        while let Some(gilrs::Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
