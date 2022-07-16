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

// Flags for line state
pub const GPIOLINE_FLAG_KERNEL: u32 = 1 << 0;
pub const GPIOLINE_FLAG_IS_OUT: u32 = 1 << 1;
pub const GPIOLINE_FLAG_ACTIVE_LOW: u32 = 1 << 2;
pub const GPIOLINE_FLAG_OPEN_DRAIN: u32 = 1 << 3;
pub const GPIOLINE_FLAG_OPEN_SOURCE: u32 = 1 << 4;
pub const GPIOLINE_FLAG_BIAS_PULL_UP: u32 = 1 << 5;
pub const GPIOLINE_FLAG_BIAS_PULL_DOWN: u32 = 1 << 6;

// Flags for line requests
pub const GPIOHANDLE_REQUEST_INPUT: u32 = 1 << 0;
pub const GPIOHANDLE_REQUEST_OUTPUT: u32 = 1 << 1;
pub const GPIOHANDLE_REQUEST_ACTIVE_LOW: u32 = 1 << 2;
pub const GPIOHANDLE_REQUEST_OPEN_DRAIN: u32 = 1 << 3;
pub const GPIOHANDLE_REQUEST_OPEN_SOURCE: u32 = 1 << 4;
pub const GPIOHANDLE_REQUEST_BIAS_PULL_UP: u32 = 1 << 5;
pub const GPIOHANDLE_REQUEST_BIAS_PULL_DOWN: u32 = 1 << 6;
pub const GPIOHANDLE_REQUEST_BIAS_DISABLE: u32 = 1 << 7;
