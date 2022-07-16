use libgpiod::{GpioChip, OutputMode};
use std::env;
use std::path::Path;
use std::{thread, time};

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        return Err(
            "Too few arguments. Usage <gpiochip path> {options} [offset1] [offset2] ... \n
                        Options are: \n
                            \t --open-source | --open-drain \n
                            \t --active-low",
        );
    }

    let gpiodev = &args[1];

    let offset: Vec<u32> = args
        .iter()
        .enumerate()
        .filter(|&(i, _)| i >= 2)
        .map(|(_, x)| x.parse().unwrap())
        .collect();

    let gpiochip = GpioChip::new(&Path::new(gpiodev)).unwrap();

    let line = gpiochip
        .request_line_values_output(&offset, OutputMode::None, false, "gpioset")
        .unwrap();

    println!(
        "GPIO get {} offset {:?}. Values {:?}",
        gpiodev,
        offset,
        line.set_line_value(1)
    );

    thread::sleep(time::Duration::from_secs(60));

    Ok(())
}
