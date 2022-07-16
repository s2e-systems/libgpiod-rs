//! Crate for interfacing with Linux GPIO chardev module
//!
//! This crate provides an interface to the Linux GPIO using the chardev module.
//! This interface involves calling *ioctl* funcions which are unsafe and require some unintuitive variable
//! mapping. To ease this process, this crate provides a GpioChip struct which encapsulates the
//! interface in safe Rust functions. The functionality provided here is highly inspired by libgpiod.
//!
//! Since all functionality is dependent on Linux function calls, this crate only compiles for Linux systems.
//!

mod raw;

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

/// Represents a Linux chardev GPIO chip interface.
/// It can be used to get information about the chip and lines and
/// to request GPIO lines that can be used as output or input.
pub struct Chip {
    name: String,
    label: String,
    num_lines: u32,
    fd: File,
}

impl fmt::Display for Chip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] ({} lines)",
            self.name, self.label, self.num_lines
        )
    }
}

/// Represents the direction of a GPIO line. Possible values are *Input* and *Output*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    Input,
    Output,
}

impl AsRef<str> for Direction {
    fn as_ref(&self) -> &str {
        match self {
            Self::Input => "Input",
            Self::Output => "Output",
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

/// Represents the active state condition of a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Active {
    Low,
    High,
}

impl AsRef<str> for Active {
    fn as_ref(&self) -> &str {
        match self {
            Self::Low => "low",
            Self::High => "high",
        }
    }
}

impl fmt::Display for Active {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

/// Represents the input bias of a GPIO line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Bias {
    Disable,
    PullUp,
    PullDown,
}

impl AsRef<str> for Bias {
    fn as_ref(&self) -> &str {
        match self {
            Self::Disable => "Disable",
            Self::PullUp => "Pull up",
            Self::PullDown => "Pull down",
        }
    }
}

impl fmt::Display for Bias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

/// Represents the output mode of a GPIO line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Drive {
    PushPull,
    OpenDrain,
    OpenSource,
}

impl AsRef<str> for Drive {
    fn as_ref(&self) -> &str {
        match self {
            Self::PushPull => "Push pull",
            Self::OpenDrain => "Open drain",
            Self::OpenSource => "Open source",
        }
    }
}

impl fmt::Display for Drive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

/// Represents the information of a specific GPIO line. Can only be obtained through the Chip interface.
pub struct LineInfo {
    direction: Direction,
    active: Active,
    used: bool,
    bias: Bias,
    drive: Drive,
    name: String,
    consumer: String,
}

/// Represents the line values.
pub struct LineValue {
    parent_chip_name: String,
    direction: Direction,
    offset: Vec<u32>,
    fd: File,
}

impl LineValue {
    /// Get the value of GPIO lines. The values can only be read if the lines have previously been
    /// requested as either inputs, using the *request_line_values_input* method, or outputs using
    /// the *request_line_values_output*. The input vector in both the *request* and get functions
    /// must match exactly, otherwise the correct file descriptor needed to access the
    /// lines can not be retrieved and the function will fail.
    pub fn get_line_value(&self) -> io::Result<Vec<u8>> {
        let mut data = raw::GpioHandleData::default();

        unsafe_call!(raw::gpio_get_line_values(self.fd.as_raw_fd(), &mut data))?;

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
        let mut data = raw::GpioHandleData::default();

        for line_index in 0..self.offset.len() {
            data.values[line_index] = value;
        }

        unsafe_call!(raw::gpio_set_line_values(self.fd.as_raw_fd(), &mut data))?;

        Ok(())
    }

    /// Get line chip name
    pub fn parent_chip_name(&self) -> &str {
        &self.parent_chip_name
    }

    /// Get line direction
    pub fn direction(&self) -> &Direction {
        &self.direction
    }
}

impl fmt::Display for LineInfo {
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
        write!(f, "\t Active {}", self.active)?;
        if !matches!(self.drive, Drive::PushPull) {
            write!(f, "\t {}", self.drive)?;
        }

        Ok(())
    }
}

impl LineInfo {
    /// Get direction of line
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Get active state of line
    pub fn active(&self) -> Active {
        self.active
    }

    /// Get input bias of line
    pub fn bias(&self) -> Bias {
        self.bias
    }

    /// In line configured as pull up input
    pub fn is_pull_up(&self) -> bool {
        matches!(self.bias, Bias::PullUp)
    }

    /// In line configured as pull down input
    pub fn is_pull_down(&self) -> bool {
        matches!(self.bias, Bias::PullDown)
    }

    /// Get output mode of line
    pub fn drive(&self) -> Drive {
        self.drive
    }

    /// Is line used
    pub fn is_used(&self) -> bool {
        self.used
    }

    /// Is line configured as open drain output
    pub fn is_open_drain(&self) -> bool {
        matches!(self.drive, Drive::OpenDrain)
    }

    /// Is line configured as open source output
    pub fn is_open_source(&self) -> bool {
        matches!(self.drive, Drive::OpenSource)
    }

    /// Get line name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get line consumer
    pub fn consumer(&self) -> &str {
        &self.consumer
    }
}

impl Chip {
    /// Create a new GPIO chip interface.
    pub fn new(path: impl AsRef<Path>) -> io::Result<Chip> {
        let dev_file = OpenOptions::new().read(true).write(true).open(&path)?;

        Chip::is_gpiochip_cdev(path)?;

        let mut gpio_chip_info = raw::GpioChipInfo::default();

        unsafe_call!(raw::gpio_get_chip_info(
            dev_file.as_raw_fd(),
            &mut gpio_chip_info
        ))?;

        Ok(Chip {
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

    fn is_gpiochip_cdev(path: impl AsRef<Path>) -> io::Result<bool> {
        const LINE_FEED: u8 = 10;

        /*rv = lstat(path, &statbuf);*/
        let file_metadata = symlink_metadata(&path)?;

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
    pub fn get_line_info(&self, line_number: &u32) -> io::Result<LineInfo> {
        let mut gpio_line_info = raw::GpioLineInfo::default();

        gpio_line_info.line_offset = *line_number;

        unsafe_call!(raw::gpio_get_line_info(
            self.fd.as_raw_fd(),
            &mut gpio_line_info,
        ))?;

        let direction =
            if gpio_line_info.flags & raw::GPIOLINE_FLAG_IS_OUT == raw::GPIOLINE_FLAG_IS_OUT {
                Direction::Output
            } else {
                Direction::Input
            };

        let active = if gpio_line_info.flags & raw::GPIOLINE_FLAG_ACTIVE_LOW
            == raw::GPIOLINE_FLAG_ACTIVE_LOW
        {
            Active::Low
        } else {
            Active::High
        };

        let used = (gpio_line_info.flags & raw::GPIOLINE_FLAG_KERNEL) == raw::GPIOLINE_FLAG_KERNEL;

        let bias = match (
            (gpio_line_info.flags & raw::GPIOLINE_FLAG_BIAS_PULL_UP)
                == raw::GPIOLINE_FLAG_BIAS_PULL_UP,
            (gpio_line_info.flags & raw::GPIOLINE_FLAG_BIAS_PULL_DOWN)
                == raw::GPIOLINE_FLAG_BIAS_PULL_DOWN,
        ) {
            (true, false) => Bias::PullUp,
            (false, true) => Bias::PullDown,
            _ => Bias::Disable,
        };

        let drive = match (
            (gpio_line_info.flags & raw::GPIOLINE_FLAG_OPEN_DRAIN) == raw::GPIOLINE_FLAG_OPEN_DRAIN,
            (gpio_line_info.flags & raw::GPIOLINE_FLAG_OPEN_SOURCE)
                == raw::GPIOLINE_FLAG_OPEN_SOURCE,
        ) {
            (true, false) => Drive::OpenDrain,
            (false, true) => Drive::OpenSource,
            _ => Drive::PushPull,
        };
        let name = String::from_utf8(gpio_line_info.name.to_vec())
            .unwrap()
            .trim_end_matches(char::from(0))
            .to_string();
        let consumer = String::from_utf8(gpio_line_info.consumer.to_vec())
            .unwrap()
            .trim_end_matches(char::from(0))
            .to_string();

        Ok(LineInfo {
            direction,
            active,
            used,
            bias,
            drive,
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
        drive: Drive,
        active: Active,
        label: &str,
    ) -> io::Result<LineValue> {
        let mut gpio_handle_request = raw::GpioHandleRequest::default();

        gpio_handle_request.lines = line_offset.len() as u32;

        for index in 0..line_offset.len() {
            gpio_handle_request.line_offsets[index] = line_offset[index];
        }

        gpio_handle_request.flags |= raw::GPIOHANDLE_REQUEST_OUTPUT;

        match drive {
            Drive::OpenDrain => gpio_handle_request.flags |= raw::GPIOHANDLE_REQUEST_OPEN_DRAIN,
            Drive::OpenSource => gpio_handle_request.flags |= raw::GPIOHANDLE_REQUEST_OPEN_SOURCE,
            _ => (),
        };

        if matches!(active, Active::Low) {
            gpio_handle_request.flags |= raw::GPIOHANDLE_REQUEST_ACTIVE_LOW;
        }

        if label.len() > 32 {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        gpio_handle_request.consumer_label[..label.len()].copy_from_slice(label.as_bytes());

        unsafe_call!(raw::gpio_get_line_handle(
            self.fd.as_raw_fd(),
            &mut gpio_handle_request,
        ))?;

        Ok(LineValue {
            parent_chip_name: self.name.clone(),
            direction: Direction::Output,
            offset: line_offset.clone(),
            fd: unsafe { File::from_raw_fd(gpio_handle_request.fd) },
        })
    }

    /// Request the GPIO chip to configure the lines passed as argument as inputs. Calling this
    /// operation is a precondition to being able to read the state of the GPIO lines.
    pub fn request_line_values_input(
        &self,
        line_offset: &Vec<u32>,
        bias: Option<Bias>,
        active: Active,
        label: &str,
    ) -> io::Result<LineValue> {
        let mut gpio_handle_request = raw::GpioHandleRequest::default();

        for index in 0..line_offset.len() {
            gpio_handle_request.line_offsets[index] = line_offset[index];
        }

        gpio_handle_request.lines = line_offset.len() as u32;

        gpio_handle_request.flags |= raw::GPIOHANDLE_REQUEST_INPUT;

        if let Some(bias) = bias {
            match bias {
                Bias::PullUp => gpio_handle_request.flags |= raw::GPIOHANDLE_REQUEST_BIAS_PULL_UP,
                Bias::PullDown => {
                    gpio_handle_request.flags |= raw::GPIOHANDLE_REQUEST_BIAS_PULL_DOWN
                }
                Bias::Disable => gpio_handle_request.flags |= raw::GPIOHANDLE_REQUEST_BIAS_DISABLE,
            }
        }

        if matches!(active, Active::Low) {
            gpio_handle_request.flags |= raw::GPIOHANDLE_REQUEST_ACTIVE_LOW;
        }

        if label.len() > 32 {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        gpio_handle_request.consumer_label[..label.len()].copy_from_slice(label.as_bytes());

        unsafe_call!(raw::gpio_get_line_handle(
            self.fd.as_raw_fd(),
            &mut gpio_handle_request,
        ))?;

        Ok(LineValue {
            parent_chip_name: self.name.clone(),
            direction: Direction::Input,
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
