use super::{GPIO_MAGIC, GPIO_MAX_NAME_SIZE};

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

pub const GPIOEVENT_REQUEST_RISING_EDGE: u32 = 1 << 0;
pub const GPIOEVENT_REQUEST_FALLING_EDGE: u32 = 1 << 1;
pub const GPIOEVENT_REQUEST_BOTH_EDGES: u32 =
    GPIOEVENT_REQUEST_RISING_EDGE | GPIOEVENT_REQUEST_FALLING_EDGE;

pub const GPIOEVENT_EVENT_RISING_EDGE: u32 = 0x01;
pub const GPIOEVENT_EVENT_FALLING_EDGE: u32 = 0x02;

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct GpioLineInfo {
    pub line_offset: u32,
    pub flags: u32,
    pub name: [u8; GPIO_MAX_NAME_SIZE],
    pub consumer: [u8; GPIO_MAX_NAME_SIZE],
}

const GPIOHANDLES_MAX: usize = 64;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GpioHandleRequest {
    pub line_offsets: [u32; GPIOHANDLES_MAX],
    pub flags: u32,
    pub default_values: [u8; GPIOHANDLES_MAX],
    pub consumer_label: [u8; GPIO_MAX_NAME_SIZE],
    pub lines: u32,
    pub fd: i32,
}

impl Default for GpioHandleRequest {
    fn default() -> Self {
        Self {
            line_offsets: [Default::default(); GPIOHANDLES_MAX],
            flags: Default::default(),
            default_values: [Default::default(); GPIOHANDLES_MAX],
            consumer_label: Default::default(),
            lines: Default::default(),
            fd: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct GpioEventRequest {
    pub lineoffset: u32,
    pub handleflags: u32,
    pub eventflags: u32,
    pub consumer_label: [u8; GPIO_MAX_NAME_SIZE],
    pub fd: i32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GpioHandleConfig {
    pub flags: u32,
    pub default_values: [u8; GPIOHANDLES_MAX],
    padding: [u32; 4],
}

impl Default for GpioHandleConfig {
    fn default() -> Self {
        Self {
            flags: Default::default(),
            default_values: [Default::default(); GPIOHANDLES_MAX],
            padding: Default::default(),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GpioHandleData {
    pub values: [u8; GPIOHANDLES_MAX],
}

impl Default for GpioHandleData {
    fn default() -> Self {
        Self {
            values: [Default::default(); GPIOHANDLES_MAX],
        }
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct GpioEventData {
    pub timestamp: u64,
    pub id: u32,
}

impl AsMut<[u8; core::mem::size_of::<GpioEventData>()]> for GpioEventData {
    fn as_mut(&mut self) -> &mut [u8; core::mem::size_of::<GpioEventData>()] {
        unsafe { core::mem::transmute(self) }
    }
}

nix::ioctl_readwrite!(gpio_get_line_info, GPIO_MAGIC, 0x02, GpioLineInfo);
nix::ioctl_readwrite!(gpio_get_line_handle, GPIO_MAGIC, 0x03, GpioHandleRequest);
nix::ioctl_readwrite!(gpio_get_line_event, GPIO_MAGIC, 0x04, GpioEventRequest);
nix::ioctl_readwrite!(gpio_get_line_values, GPIO_MAGIC, 0x08, GpioHandleData);
nix::ioctl_readwrite!(gpio_set_line_values, GPIO_MAGIC, 0x09, GpioHandleData);
nix::ioctl_readwrite!(gpio_set_config, GPIO_MAGIC, 0x0a, GpioHandleConfig);
nix::ioctl_readwrite!(gpio_get_line_info_watch, GPIO_MAGIC, 0x0b, GpioLineInfo);
