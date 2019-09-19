use std::fs;
use libgpio::GpioChip;

fn main() {
    let dev_files = fs::read_dir("/dev/").unwrap();

    let gpiochips : Vec<_> = dev_files.
            filter_map(Result::ok)
            .filter(|f| f.path().to_str().unwrap().starts_with("/dev/gpiochip"))
            .map(|f| GpioChip::new(&f.path()).unwrap())
            .collect();
    
    gpiochips.iter().rev() //Do it in reverse order because the numbers of the GPIO chips go from high to low
        .for_each(|f| println!("{}",f));
}