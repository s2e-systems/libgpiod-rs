use std::env;
use std::path::Path;
use libgpiod::GpioChip;
use std::{thread, time};

fn main()  -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        return Err("Too few arguments. Usage <gpiochip path> [offset1] [offset2] ... ");
    }

    let gpiodev = &args[1];

    let offset : Vec<u32> = args.iter().enumerate()
        .filter(|&(i, _)| i >= 2)
        .map(|(_,x)| x.parse().unwrap())
        .collect();

    let gpiochip = GpioChip::new(&Path::new(gpiodev)).unwrap();

    let line = gpiochip.request_line_values_input(&offset, true, "gpioget").unwrap();

    println!("GPIO get {} offset {:?}. Values {:?}", gpiodev, offset, line.get_line_value().unwrap());

    thread::sleep(time::Duration::from_secs(60));

    Ok(())
}