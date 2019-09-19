#[macro_use]
extern crate nix;

pub mod libgpio {
	use std::fmt;
	use std::collections::HashMap;
	use std::io;
	use std::io::{Error, ErrorKind};
	use std::fs::File;
	use std::io::Read;
	use std::fs::OpenOptions;
	use std::fs::symlink_metadata;
	use std::os::unix::fs::{MetadataExt, FileTypeExt};
	use std::path::Path;
	use std::os::unix::prelude::*;

	/* ****************** C structs for ioctl *********************** */
	/* All the structs used for ioctl must be represented in C otherwise weird memory mappings happen */

	#[derive(Debug, Default)]
	#[repr(C)]
	pub struct GpioChipInfo {
	name: [u8; 32],
	label: [u8; 32],
	lines: u32,
	}

	#[derive(Debug, Default)]
	#[repr(C)]
	pub struct GpioLineInfo {
		line_offset: u32,
		flags : u32,
		name: [u8; 32],
		consumer: [u8; 32],
	}

	const GPIOHANDLES_MAX: usize = 64;

	#[repr(C)]
	pub struct GpioHandleRequest {
		line_offsets: [u32; GPIOHANDLES_MAX],
		flags: u32,
		default_values: [u8; GPIOHANDLES_MAX],
		consumer_label: [u8; 32],
		lines: u32,
		fd: i32,
	}

	impl Default for GpioHandleRequest {
		fn default() -> Self {
			Self {
				line_offsets: [0; GPIOHANDLES_MAX],
				flags: 0,
				default_values: [0; GPIOHANDLES_MAX],
				consumer_label: [0;32],
				lines: 0,
				fd: 0,
			}
		}
	}

	#[derive(Debug, Default)]
	#[repr(C)]
	pub struct GpioEventRequest {
		lineoffset: u32,
		handleflags: u32,
		eventflags: u32,
		consumer_label: [u8; 32],
		fd: i32,
	}

	#[repr(C)]
	pub struct GpioHandleData {
		values: [u8; GPIOHANDLES_MAX],
	}

	impl Default for GpioHandleData {
    	fn default() -> Self {
			Self {
				values: [0;GPIOHANDLES_MAX],
			}
		}
	}

	/* ***************** Defines for ioctl **************** */
	const GPIO_MAGIC_NUMBER: u8 = 0xB4;
	const GPIO_GET_CHIPINFO_IOCTL_COMMAND_NUMBER: u8 = 0x01;
	const GPIO_GET_LINEINFO_IOCTL_COMMAND_NUMBER: u8 = 0x02;
	const GPIO_GET_LINEHANDLE_IOCTL_COMMAND_NUMBER: u8 = 0x03;
	const GPIO_GET_LINEEVENT_IOCTL_COMMAND_NUMBER: u8 = 0x04;
	const GPIO_GET_LINE_VALUES_IOCTL_COMMAND_NUMBER: u8 = 0x08;
	const GPIO_SET_LINE_VALUES_IOCTL_COMMAND_NUMBER: u8 = 0x09;

	/* **************** Flags for line state ************** */
	const GPIOLINE_FLAG_KERNEL: u32 = 1 << 0;
	const GPIOLINE_FLAG_IS_OUT: u32 = 1 << 1;
	const GPIOLINE_FLAG_ACTIVE_LOW: u32 = 1 << 2;
	const GPIOLINE_FLAG_OPEN_DRAIN: u32 = 1 << 3;
	const GPIOLINE_FLAG_OPEN_SOURCE: u32  = 1 << 4;

	/* **************** Flags for line requests *************** */
	const GPIOHANDLE_REQUEST_INPUT: u32 = 1 << 0;
	const GPIOHANDLE_REQUEST_OUTPUT: u32 = 	1 << 1;
	const GPIOHANDLE_REQUEST_ACTIVE_LOW: u32 = 1 << 2;
	const GPIOHANDLE_REQUEST_OPEN_DRAIN: u32 = 1 << 3;
	const GPIOHANDLE_REQUEST_OPEN_SOURCE: u32 = 1 << 4;

	/* *********** Implementation ********************/

	pub struct GpioChip {
		name: String,
		label: String,
		num_lines: u32,
		fd: File,
		lines: HashMap<Vec<u32>, i32>,
	}

	impl fmt::Display for GpioChip {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(f, "{} [{}] ({} lines)", self.name, self.label, self.num_lines)
		}
	}

	pub enum LineDirection {
		Input,
		Output,
	}

	impl fmt::Display for LineDirection {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			match self {
				LineDirection::Input => write!(f, "Input"),
				LineDirection::Output => write!(f, "Output"), 
			}
		}
	}

	pub enum LineActiveState {
		ActiveLow,
		ActiveHigh,
	}

	impl fmt::Display for LineActiveState {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			match self {
				LineActiveState::ActiveLow => write!(f, "Active low"),
				LineActiveState::ActiveHigh => write!(f, "Active high"), 
			}
		}
	}

	pub enum OutputMode {
		OpenDrain,
		OpenSource,
	}

	impl fmt::Display for OutputMode {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			match self {
				OutputMode::OpenDrain => write!(f, "Open drain"),
				OutputMode::OpenSource => write!(f, "Open source"), 
			}
		}
	}

	pub struct GpioLine {
		direction: LineDirection,
		active_state: LineActiveState,
		used: bool,
		open_drain: bool,
		open_source: bool,
	}

	impl fmt::Display for GpioLine {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(f, "{} {} {} {} {}", self.direction, self.active_state, self.used, self.open_drain, self.open_source)
		}
	}

	impl GpioLine {
		pub fn direction(&self) -> &LineDirection {
			&self.direction
		}

		pub fn active_state(&self) -> &LineActiveState {
			&self.active_state
		}

		pub fn is_used(&self) -> &bool {
			&self.used
		}

		pub fn is_open_drain(&self) -> &bool {
			&self.open_drain
		}

		pub fn is_open_source(&self) -> &bool {
			&self.open_source
		}
	}

	impl GpioChip {
		ioctl_read!(gpio_get_chip_info, GPIO_MAGIC_NUMBER, GPIO_GET_CHIPINFO_IOCTL_COMMAND_NUMBER, GpioChipInfo);
		ioctl_readwrite!(gpio_get_line_info, GPIO_MAGIC_NUMBER, GPIO_GET_LINEINFO_IOCTL_COMMAND_NUMBER, GpioLineInfo);
		ioctl_readwrite!(gpio_get_line_handle, GPIO_MAGIC_NUMBER, GPIO_GET_LINEHANDLE_IOCTL_COMMAND_NUMBER, GpioHandleRequest);
		ioctl_readwrite!(gpio_get_line_event, GPIO_MAGIC_NUMBER, GPIO_GET_LINEEVENT_IOCTL_COMMAND_NUMBER, GpioEventRequest);
		ioctl_readwrite!(gpio_get_line_values, GPIO_MAGIC_NUMBER, GPIO_GET_LINE_VALUES_IOCTL_COMMAND_NUMBER, GpioHandleData);
		ioctl_readwrite!(gpio_set_line_values, GPIO_MAGIC_NUMBER, GPIO_SET_LINE_VALUES_IOCTL_COMMAND_NUMBER, GpioHandleData);

		pub fn new(path: &Path) -> io::Result<GpioChip> {
			let dev_file = OpenOptions::new().read(true).write(true).open(path)?;

			GpioChip::is_gpiochip_cdev(path)?;

			let mut gpio_chip_info = GpioChipInfo::default();

			unsafe { 
				GpioChip::gpio_get_chip_info(dev_file.as_raw_fd(), &mut gpio_chip_info).unwrap();
			}

			Ok (GpioChip{
					name: String::from_utf8(gpio_chip_info.name.to_vec()).unwrap().trim_end_matches(char::from(0)).to_string(),
					label: String::from_utf8(gpio_chip_info.label.to_vec()).unwrap().trim_end_matches(char::from(0)).to_string(),
					num_lines: gpio_chip_info.lines,
					fd: dev_file,
					lines: HashMap::new() })
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

		pub fn get_line_info(&self, line_number: u32) -> io::Result<GpioLine>{
			let mut gpio_line_info = GpioLineInfo::default();

			gpio_line_info.line_offset = line_number;

			unsafe { 
				GpioChip::gpio_get_line_info(self.fd.as_raw_fd(), &mut gpio_line_info).unwrap();
			}

			let direction = if gpio_line_info.flags & GPIOLINE_FLAG_IS_OUT == 1 {
				LineDirection::Output
			} else {
				LineDirection::Input
			};

			let active_state = if gpio_line_info.flags & GPIOLINE_FLAG_ACTIVE_LOW == 1 {
				LineActiveState::ActiveLow
			} else {
				LineActiveState::ActiveHigh
			};

			let used = gpio_line_info.flags & GPIOLINE_FLAG_KERNEL == 1;
			let open_drain = gpio_line_info.flags & GPIOLINE_FLAG_OPEN_DRAIN == 1; 
			let open_source = gpio_line_info.flags & GPIOLINE_FLAG_OPEN_SOURCE == 1;
			
			Ok(GpioLine{
				direction,
				active_state,
				used,
				open_drain,
				open_source
			})
		}

		pub fn request_line_values_output(&mut self, line_offset: &Vec<u32>, output_mode: OutputMode, active_low: bool) {
			let mut gpio_handle_request = GpioHandleRequest::default();

			gpio_handle_request.lines = line_offset.len() as u32;

			for index in 0..line_offset.len() {
				gpio_handle_request.line_offsets[index] = line_offset[index];
			}
			
			gpio_handle_request.flags |= GPIOHANDLE_REQUEST_OUTPUT;
			match output_mode {
				OutputMode::OpenDrain => gpio_handle_request.flags |= GPIOHANDLE_REQUEST_OPEN_DRAIN,
				OutputMode::OpenSource => gpio_handle_request.flags |= GPIOHANDLE_REQUEST_OPEN_SOURCE,
			};

			if active_low {
				gpio_handle_request.flags |= GPIOHANDLE_REQUEST_ACTIVE_LOW;
			}

			unsafe {
				GpioChip::gpio_get_line_handle(self.fd.as_raw_fd(),&mut gpio_handle_request).unwrap();
			}

			self.lines.insert(line_offset.clone(), gpio_handle_request.fd);
		}

		pub fn request_line_values_input(&mut self, line_offset: &Vec<u32>) {
			let mut gpio_handle_request = GpioHandleRequest::default();
			
			for index in 0 .. line_offset.len() {
				gpio_handle_request.line_offsets[index] = line_offset[index];
			}

			gpio_handle_request.lines = line_offset.len() as u32;
			
			gpio_handle_request.flags |= GPIOHANDLE_REQUEST_INPUT;

			unsafe {
				GpioChip::gpio_get_line_handle(self.fd.as_raw_fd(), &mut gpio_handle_request).unwrap();
			}

			self.lines.insert(line_offset.clone(), gpio_handle_request.fd);
		}

		pub fn get_line_value(&self, line_offset: &Vec<u32>) -> Vec<u8>{
			let line_fd = self.lines.get(line_offset).unwrap();

			let mut data = GpioHandleData::default();

			unsafe {
				GpioChip::gpio_get_line_values(*line_fd, &mut data).unwrap();
			}

			let mut output_data : Vec<u8> = Vec::with_capacity(line_offset.len());

			for index in 0..line_offset.len() {
				output_data.push(data.values[index]);
			}

			output_data
		}

		pub fn set_line_value(&self, line_offset: &Vec<u32>, value: u8) -> io::Result<()>{
			let line_fd = self.lines.get(line_offset).unwrap();

			let mut data = GpioHandleData::default();
			data.values[0] = value;

			unsafe {
				GpioChip::gpio_set_line_values(*line_fd, &mut data).unwrap();
			}

			Ok(())
		}

		pub fn name(&self) -> &str {
			&self.name
		}

		pub fn label(&self) -> &str {
			&self.label
		}

		pub fn num_lines(&self) -> &u32 {
			&self.num_lines
		}
	}
}