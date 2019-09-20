use std::fs;
use std::env;
use libgpio::GpioChip;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {

        let dev_files = fs::read_dir("/dev/").unwrap();

        let gpiochips : Vec<_> = dev_files.
                filter_map(Result::ok)
                .filter(|f| f.path().to_str().unwrap().starts_with("/dev/gpiochip"))
                .map(|f| GpioChip::new(&f.path()).unwrap())
                .collect();

        println!("Info for all {} GPIO chips", gpiochips.len());
        
        for index in (0..gpiochips.len()).rev() {
            let gpiochip = &gpiochips[index];
            println!("{}", gpiochip);
            for line_index in 0..*gpiochip.num_lines() {
                let line_info = gpiochip.get_line_info(&line_index).unwrap();
                print!("\t Line \t {}: \t", line_index);
                println!("{}", line_info);
            }
        }
    }
}