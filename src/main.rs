#[macro_use]
extern crate nix;

use std::io;
use std::io::{Error, ErrorKind};
use std::fs::File;
use std::io::Read;
use std::fs::OpenOptions;
use std::fs::symlink_metadata;
use std::os::unix::fs::{MetadataExt, FileTypeExt, PermissionsExt};
use std::env;
use std::path::Path;
use std::os::unix::prelude::*;

/*#define GPIO_GET_CHIPINFO_IOCTL _IOR(0xB4, 0x01, struct gpiochip_info)
#define GPIO_GET_LINEINFO_IOCTL _IOWR(0xB4, 0x02, struct gpioline_info)*/

/* All the structs used for ioctl must be represented in C otherwise weird memory mappings happen */
#[derive(Debug, Default)]
#[repr(C)]
pub struct GpioChipInfo {
   name: [u8;32],
   label: [u8;32],
   lines: u32,
}

/*  struct gpiochip_info {
	char name[32];
	char label[32];
	__u32 lines;
};*/

const GPIO_GET_CHIPINFO_IOCTL_TYPE: u8 = 0xB4;
const GPIO_GET_CHIPINFO_IOCTL_NUMBER: u8 = 0x01;

ioctl_read!(gpio_get_chip_info, GPIO_GET_CHIPINFO_IOCTL_TYPE, GPIO_GET_CHIPINFO_IOCTL_NUMBER, GpioChipInfo);

fn gpiod_chip_open(path: &Path) -> io::Result<File> {
    
    let dev_file = OpenOptions::new().read(true).write(true).open(path)?;

    is_gpiochip_cdev(path)?;

    let mut gpio_chip_info = GpioChipInfo::default();

    println!("Read chip info. Name: {}, Label: {}, Lines: {}",String::from_utf8(gpio_chip_info.name.to_vec()).unwrap(),String::from_utf8(gpio_chip_info.label.to_vec()).unwrap(),gpio_chip_info.lines);

    unsafe { 
        gpio_get_chip_info(dev_file.as_raw_fd(), &mut gpio_chip_info).unwrap();
    }

    println!("Read chip info. Name: {}, Label: {}, Lines: {}",String::from_utf8(gpio_chip_info.name.to_vec()).unwrap(),String::from_utf8(gpio_chip_info.label.to_vec()).unwrap(),gpio_chip_info.lines);

    Ok(dev_file)
}

fn is_gpiochip_cdev(path: &Path) -> io::Result<bool>{

	const LINE_FEED : u8 = 10;

	/*rv = lstat(path, &statbuf);*/
    let file_metadata = symlink_metadata(path)?; 

	/*if (!S_ISCHR(statbuf.st_mode)) */
	if !file_metadata.file_type().is_char_device() { 
		return Err(Error::new(ErrorKind::InvalidInput,"File is not character device"));
	}

	/*basename(pathcpy);*/
	let basename = path.file_name().unwrap(); 

	let sysfs = format!{"/sys/bus/gpio/devices/{}/dev", basename.to_str().unwrap()};

	/*if (access(sysfsp, R_OK) != 0)*/
	if !Path::new(&sysfs).is_file() /*I check if it is a file instead of read access done in libgpiod */ {
		return  Err(Error::new(ErrorKind::InvalidInput,"Matching GPIO in sys not found"));
	}

	let mut sysfs_rdev: [u8; 16] = [0; 16];
	{
		let mut fd = OpenOptions::new().read(true).open(sysfs)?;

		fd.read(&mut sysfs_rdev)?; /*Ignoring any error for now*/
	}

	let lf_pos = sysfs_rdev.iter().position(|&x| x == LINE_FEED).unwrap_or(sysfs_rdev.len()-1);

	let file_rdev = format!("{}:{}", file_metadata.rdev() >> 8, file_metadata.rdev() & 0xFF);

	if String::from_utf8(sysfs_rdev[0 .. lf_pos-1].to_vec()).unwrap() == file_rdev {
		return Err(Error::new(ErrorKind::Other,"Unmatched device versions"));
	}

    Ok(true)
}

fn main() {

    /*fd = open(path, O_RDWR | O_CLOEXEC);
	if (fd < 0)
		return NULL;*/
        //  let mut f = File::open("foo.txt")?;

	let args: Vec<String> = env::args().collect();

    let path = &args[1];

    println!("Testing file in {}. Chardev result {}", path, is_gpiochip_cdev(Path::new(path)).unwrap());

    gpiod_chip_open(Path::new(path));
}
