use std::fs;
use libgpio::libgpio::GpioChip;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    let gpiodev = &args[1];
    let offset : u32 = args[2].parse().unwrap();;

    let mut gpiochip = GpioChip::new(Path::new(gpiodev)).unwrap();

    gpiochip.request_line_values_input(offset);

    println!("GPIO get {} offset {}. Value {}", gpiodev, offset,gpiochip.get_line_value(offset));
}