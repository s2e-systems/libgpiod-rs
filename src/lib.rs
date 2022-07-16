//! Crate for interfacing with Linux GPIO chardev module
//!
//! This crate provides an interface to the Linux GPIO using the chardev module.
//! This interface involves calling *ioctl* funcions which are unsafe and require some unintuitive variable
//! mapping. To ease this process, this crate provides a GpioChip struct which encapsulates the
//! interface in safe Rust functions. The functionality provided here is highly inspired by libgpiod.
//!
//! Since all functionality is dependent on Linux function calls, this crate only compiles for Linux systems.
//!

use std::{
    fmt,
    fs::{symlink_metadata, File, OpenOptions},
    io,
    io::Read,
    os::unix::{
        fs::{FileTypeExt, MetadataExt},
        io::FromRawFd,
        prelude::*,
    },
    path::Path,
};

macro_rules! unsafe_call {
    ($res:expr) => {
        unsafe { $res }.map_err(io::Error::from)
    };
}

mod gpio_ioctl {
    use nix::{ioctl_read, ioctl_readwrite};

    // All the structs used for ioctl must be represented in C otherwise weird memory mappings happen.
    //
    // The implementations provided inside this module are also a copy of gpio.h which is normally
    // used to access the gpio in linux. If that header file changes, this module will likely be broken
    // due to mismatched data types or GPIO command codes.
    #[derive(Debug, Default)]
    #[repr(C)]
    pub struct GpioChipInfo {
        pub name: [u8; 32],
        pub label: [u8; 32],
        pub lines: u32,
    }

    #[derive(Debug, Default)]
    #[repr(C)]
    pub struct GpioLineInfo {
        pub line_offset: u32,
        pub flags: u32,
        pub name: [u8; 32],
        pub consumer: [u8; 32],
    }

    const GPIOHANDLES_MAX: usize = 64;

    #[repr(C)]
    pub struct GpioHandleRequest {
        pub line_offsets: [u32; GPIOHANDLES_MAX],
        pub flags: u32,
        pub default_values: [u8; GPIOHANDLES_MAX],
        pub consumer_label: [u8; 32],
        pub lines: u32,
        pub fd: i32,
    }

    impl Default for GpioHandleRequest {
        fn default() -> Self {
            Self {
                line_offsets: [0; GPIOHANDLES_MAX],
                flags: 0,
                default_values: [0; GPIOHANDLES_MAX],
                consumer_label: [0; 32],
                lines: 0,
                fd: 0,
            }
        }
    }

    #[derive(Debug, Default)]
    #[repr(C)]
    pub struct GpioEventRequest {
        pub lineoffset: u32,
        pub handleflags: u32,
        pub eventflags: u32,
        pub consumer_label: [u8; 32],
        pub fd: i32,
    }

    #[repr(C)]
    pub struct GpioHandleData {
        pub values: [u8; GPIOHANDLES_MAX],
    }

    impl Default for GpioHandleData {
        fn default() -> Self {
            Self {
                values: [0; GPIOHANDLES_MAX],
            }
        }
    }

    const GPIO_MAGIC_NUMBER: u8 = 0xB4;
    const GPIO_GET_CHIPINFO_IOCTL_COMMAND_NUMBER: u8 = 0x01;
    const GPIO_GET_LINEINFO_IOCTL_COMMAND_NUMBER: u8 = 0x02;
    const GPIO_GET_LINEHANDLE_IOCTL_COMMAND_NUMBER: u8 = 0x03;
    const GPIO_GET_LINEEVENT_IOCTL_COMMAND_NUMBER: u8 = 0x04;
    const GPIO_GET_LINE_VALUES_IOCTL_COMMAND_NUMBER: u8 = 0x08;
    const GPIO_SET_LINE_VALUES_IOCTL_COMMAND_NUMBER: u8 = 0x09;

    ioctl_read!(
        gpio_get_chip_info,
        GPIO_MAGIC_NUMBER,
        GPIO_GET_CHIPINFO_IOCTL_COMMAND_NUMBER,
        GpioChipInfo
    );
    ioctl_readwrite!(
        gpio_get_line_info,
        GPIO_MAGIC_NUMBER,
        GPIO_GET_LINEINFO_IOCTL_COMMAND_NUMBER,
        GpioLineInfo
    );
    ioctl_readwrite!(
        gpio_get_line_handle,
        GPIO_MAGIC_NUMBER,
        GPIO_GET_LINEHANDLE_IOCTL_COMMAND_NUMBER,
        GpioHandleRequest
    );
    ioctl_readwrite!(
        gpio_get_line_event,
        GPIO_MAGIC_NUMBER,
        GPIO_GET_LINEEVENT_IOCTL_COMMAND_NUMBER,
        GpioEventRequest
    );
    ioctl_readwrite!(
        gpio_get_line_values,
        GPIO_MAGIC_NUMBER,
        GPIO_GET_LINE_VALUES_IOCTL_COMMAND_NUMBER,
        GpioHandleData
    );
    ioctl_readwrite!(
        gpio_set_line_values,
        GPIO_MAGIC_NUMBER,
        GPIO_SET_LINE_VALUES_IOCTL_COMMAND_NUMBER,
        GpioHandleData
    );
}

// **************** Flags for line state **************
const GPIOLINE_FLAG_KERNEL: u32 = 1 << 0;
const GPIOLINE_FLAG_IS_OUT: u32 = 1 << 1;
const GPIOLINE_FLAG_ACTIVE_LOW: u32 = 1 << 2;
const GPIOLINE_FLAG_OPEN_DRAIN: u32 = 1 << 3;
const GPIOLINE_FLAG_OPEN_SOURCE: u32 = 1 << 4;

// **************** Flags for line requests ***************
const GPIOHANDLE_REQUEST_INPUT: u32 = 1 << 0;
const GPIOHANDLE_REQUEST_OUTPUT: u32 = 1 << 1;
const GPIOHANDLE_REQUEST_ACTIVE_LOW: u32 = 1 << 2;
const GPIOHANDLE_REQUEST_OPEN_DRAIN: u32 = 1 << 3;
const GPIOHANDLE_REQUEST_OPEN_SOURCE: u32 = 1 << 4;

/// Represents a Linux chardev GPIO chip interface.
/// It can be used to get information about the chip and lines and
/// to request GPIO lines that can be used as output or input.
pub struct GpioChip {
    name: String,
    label: String,
    num_lines: u32,
    fd: File,
}

impl fmt::Display for GpioChip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] ({} lines)",
            self.name, self.label, self.num_lines
        )
    }
}

/// Represents the direction of a GPIO line. Possible values are *Input* and *Output*.
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

/// Represents the active state condition of a line. Possible values are *Active High* or *Active Low*.
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

/// Represents the output mode of a GPIO line. Possible values are *Open Drain* and *Open Source*.
pub enum OutputMode {
    None,
    OpenDrain,
    OpenSource,
}

impl fmt::Display for OutputMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputMode::OpenDrain => write!(f, "Open drain"),
            OutputMode::OpenSource => write!(f, "Open source"),
            OutputMode::None => write!(f, ""),
        }
    }
}

/// Represents the information of a specific GPIO line. Can only be obtained through the GpioChip interface.
pub struct GpioLineInfo {
    direction: LineDirection,
    active_state: LineActiveState,
    used: bool,
    open_drain: bool,
    open_source: bool,
    name: String,
    consumer: String,
}

pub struct GpioLineValue {
    parent_chip_name: String,
    direction: LineDirection,
    offset: Vec<u32>,
    fd: File,
}

impl GpioLineValue {
    /// Get the value of GPIO lines. The values can only be read if the lines have previously been
    /// requested as either inputs, using the *request_line_values_input* method, or outputs using
    /// the *request_line_values_output*. The input vector in both the *request* and get functions
    /// must match exactly, otherwise the correct file descriptor needed to access the
    /// lines can not be retrieved and the function will fail.
    pub fn get_line_value(&self) -> io::Result<Vec<u8>> {
        let mut data = gpio_ioctl::GpioHandleData::default();

        unsafe_call!(gpio_ioctl::gpio_get_line_values(
            self.fd.as_raw_fd(),
            &mut data
        ))?;

        let mut output_data: Vec<u8> = Vec::with_capacity(self.offset.len());

        for index in 0..self.offset.len() {
            output_data.push(data.values[index]);
        }

        Ok(output_data)
    }

    /// Set the value of GPIO lines. The value can only be set if the lines have previously been
    /// requested as outputs using the *request_line_values_output*. The input vector in both
    /// functions must match exactly, otherwise the correct file descriptor needed to access the
    /// lines can not be retrieved and the function will fail.
    pub fn set_line_value(&self, value: u8) -> io::Result<()> {
        let mut data = gpio_ioctl::GpioHandleData::default();

        for line_index in 0..self.offset.len() {
            data.values[line_index] = value;
        }

        unsafe_call!(gpio_ioctl::gpio_set_line_values(
            self.fd.as_raw_fd(),
            &mut data
        ))?;

        Ok(())
    }

    pub fn parent_chip_name(&self) -> &str {
        &self.parent_chip_name
    }

    pub fn direction(&self) -> &LineDirection {
        &self.direction
    }
}

impl fmt::Display for GpioLineInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\t {}", self.direction)?;
        if self.used {
            write!(f, "\t Used")?;
        } else {
            write!(f, "\t Unused")?;
        }
        if self.consumer.is_empty() {
            write!(f, "\t Unnamed")?;
        } else {
            write!(f, "\t {}", self.consumer)?;
        }
        write!(f, "\t {}", self.active_state())?;
        if self.open_drain {
            write!(f, "\t Open drain")?;
        } else if self.open_source {
            write!(f, "\t Open source")?;
        }

        Ok(())
    }
}

impl GpioLineInfo {
    pub fn direction(&self) -> &LineDirection {
        &self.direction
    }

    pub fn active_state(&self) -> &LineActiveState {
        &self.active_state
    }

    /// Get output mode of line
    pub fn output_mode(&self) -> OutputMode {
        match (self.open_drain, self.open_source) {
            (true, false) => OutputMode::OpenDrain,
            (false, true) => OutputMode::OpenSource,
            _ => OutputMode::None,
        }
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn consumer(&self) -> &str {
        &self.consumer
    }
}

impl GpioChip {
    /// Create a new GPIO chip interface.
    pub fn new(path: &dyn AsRef<Path>) -> io::Result<GpioChip> {
        let dev_file = OpenOptions::new().read(true).write(true).open(path)?;

        GpioChip::is_gpiochip_cdev(path)?;

        let mut gpio_chip_info = gpio_ioctl::GpioChipInfo::default();

        unsafe_call!(gpio_ioctl::gpio_get_chip_info(
            dev_file.as_raw_fd(),
            &mut gpio_chip_info
        ))?;

        Ok(GpioChip {
            name: String::from_utf8(gpio_chip_info.name.to_vec())
                .unwrap()
                .trim_end_matches(char::from(0))
                .to_string(),
            label: String::from_utf8(gpio_chip_info.label.to_vec())
                .unwrap()
                .trim_end_matches(char::from(0))
                .to_string(),
            num_lines: gpio_chip_info.lines,
            fd: dev_file,
        })
    }

    fn is_gpiochip_cdev(path: &dyn AsRef<Path>) -> io::Result<bool> {
        const LINE_FEED: u8 = 10;

        /*rv = lstat(path, &statbuf);*/
        let file_metadata = symlink_metadata(path)?;

        /*if (!S_ISCHR(statbuf.st_mode)) */
        if !file_metadata.file_type().is_char_device() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "File is not character device",
            ));
        }

        /*basename(pathcpy);*/
        let basename = path.as_ref().file_name().unwrap();

        let sysfs = format! {"/sys/bus/gpio/devices/{}/dev", basename.to_str().unwrap()};

        /*if (access(sysfsp, R_OK) != 0)*/
        if !Path::new(&sysfs).is_file()
        /*I check if it is a file instead of read access done in libgpiod */
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Matching GPIO in sys not found",
            ));
        }

        let mut sysfs_rdev: [u8; 16] = [0; 16];
        {
            let mut fd = OpenOptions::new().read(true).open(sysfs)?;

            fd.read(&mut sysfs_rdev)?; /*Ignoring any error for now*/
        }

        let lf_pos = sysfs_rdev
            .iter()
            .position(|&x| x == LINE_FEED)
            .unwrap_or(sysfs_rdev.len() - 1);

        let file_rdev = format!(
            "{}:{}",
            file_metadata.rdev() >> 8,
            file_metadata.rdev() & 0xFF
        );

        if String::from_utf8(sysfs_rdev[0..lf_pos - 1].to_vec()).unwrap() == file_rdev {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Unmatched device versions",
            ));
        }

        Ok(true)
    }

    /// Request the info of a specific GPIO line.
    pub fn get_line_info(&self, line_number: &u32) -> io::Result<GpioLineInfo> {
        let mut gpio_line_info = gpio_ioctl::GpioLineInfo::default();

        gpio_line_info.line_offset = *line_number;

        unsafe_call!(gpio_ioctl::gpio_get_line_info(
            self.fd.as_raw_fd(),
            &mut gpio_line_info,
        ))?;

        let direction = if gpio_line_info.flags & GPIOLINE_FLAG_IS_OUT == GPIOLINE_FLAG_IS_OUT {
            LineDirection::Output
        } else {
            LineDirection::Input
        };

        let active_state =
            if gpio_line_info.flags & GPIOLINE_FLAG_ACTIVE_LOW == GPIOLINE_FLAG_ACTIVE_LOW {
                LineActiveState::ActiveLow
            } else {
                LineActiveState::ActiveHigh
            };

        let used = (gpio_line_info.flags & GPIOLINE_FLAG_KERNEL) == GPIOLINE_FLAG_KERNEL;
        let open_drain =
            (gpio_line_info.flags & GPIOLINE_FLAG_OPEN_DRAIN) == GPIOLINE_FLAG_OPEN_DRAIN;
        let open_source =
            (gpio_line_info.flags & GPIOLINE_FLAG_OPEN_SOURCE) == GPIOLINE_FLAG_OPEN_SOURCE;
        let name = String::from_utf8(gpio_line_info.name.to_vec())
            .unwrap()
            .trim_end_matches(char::from(0))
            .to_string();
        let consumer = String::from_utf8(gpio_line_info.consumer.to_vec())
            .unwrap()
            .trim_end_matches(char::from(0))
            .to_string();

        Ok(GpioLineInfo {
            direction,
            active_state,
            used,
            open_drain,
            open_source,
            name,
            consumer,
        })
    }

    /// Request the GPIO chip to configure the lines passed as argument as outputs. Calling this
    /// operation is a precondition to being able to set the state of the GPIO lines. All the lines
    /// passed in one request must share the output mode and the active state. The state of lines configured
    /// as outputs can also be read using the *get_line_value* method.
    pub fn request_line_values_output(
        &self,
        line_offset: &Vec<u32>,
        output_mode: OutputMode,
        active_low: bool,
        label: &str,
    ) -> io::Result<GpioLineValue> {
        let mut gpio_handle_request = gpio_ioctl::GpioHandleRequest::default();

        gpio_handle_request.lines = line_offset.len() as u32;

        for index in 0..line_offset.len() {
            gpio_handle_request.line_offsets[index] = line_offset[index];
        }

        gpio_handle_request.flags |= GPIOHANDLE_REQUEST_OUTPUT;

        match output_mode {
            OutputMode::OpenDrain => gpio_handle_request.flags |= GPIOHANDLE_REQUEST_OPEN_DRAIN,
            OutputMode::OpenSource => gpio_handle_request.flags |= GPIOHANDLE_REQUEST_OPEN_SOURCE,
            _ => (),
        };

        if active_low {
            gpio_handle_request.flags |= GPIOHANDLE_REQUEST_ACTIVE_LOW;
        }

        if label.len() > 32 {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        gpio_handle_request.consumer_label[..label.len()].copy_from_slice(label.as_bytes());

        unsafe_call!(gpio_ioctl::gpio_get_line_handle(
            self.fd.as_raw_fd(),
            &mut gpio_handle_request,
        ))?;

        Ok(GpioLineValue {
            parent_chip_name: self.name.clone(),
            direction: LineDirection::Output,
            offset: line_offset.clone(),
            fd: unsafe { File::from_raw_fd(gpio_handle_request.fd) },
        })
    }

    /// Request the GPIO chip to configure the lines passed as argument as inputs. Calling this
    /// operation is a precondition to being able to read the state of the GPIO lines.
    pub fn request_line_values_input(
        &self,
        line_offset: &Vec<u32>,
        active_low: bool,
        label: &str,
    ) -> io::Result<GpioLineValue> {
        let mut gpio_handle_request = gpio_ioctl::GpioHandleRequest::default();

        for index in 0..line_offset.len() {
            gpio_handle_request.line_offsets[index] = line_offset[index];
        }

        gpio_handle_request.lines = line_offset.len() as u32;

        gpio_handle_request.flags |= GPIOHANDLE_REQUEST_INPUT;

        if active_low {
            gpio_handle_request.flags |= GPIOHANDLE_REQUEST_ACTIVE_LOW;
        }

        if label.len() > 32 {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        gpio_handle_request.consumer_label[..label.len()].copy_from_slice(label.as_bytes());

        unsafe_call!(gpio_ioctl::gpio_get_line_handle(
            self.fd.as_raw_fd(),
            &mut gpio_handle_request,
        ))?;

        Ok(GpioLineValue {
            parent_chip_name: self.name.clone(),
            direction: LineDirection::Input,
            offset: line_offset.clone(),
            fd: unsafe { File::from_raw_fd(gpio_handle_request.fd) },
        })
    }

    /// Get the GPIO chip name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the GPIO chip label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Get the total number of lines of the GPIO chip.
    pub fn num_lines(&self) -> &u32 {
        &self.num_lines
    }
}
