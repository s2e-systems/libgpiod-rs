use std::path::Path;
use std::fs;
use std::env;

use libgpio::libgpio::GpioChip;

fn main() {
    let dev_files = fs::read_dir("/dev/").unwrap();

    dev_files.
        filter_map(Result::ok)
        .filter(|f| f.path().to_str().unwrap().starts_with("/dev/gpiochip"))
        .for_each(|f| println!("{} [{}] ({} lines)"
            ,GpioChip::new(&f.path()).unwrap().name()
            ,GpioChip::new(&f.path()).unwrap().label()
            ,GpioChip::new(&f.path()).unwrap().num_lines()));
}