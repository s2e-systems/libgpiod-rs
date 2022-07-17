use std::{fmt, time::SystemTime};

/// Line offset
pub type LineId = u32;

/// Bit offset
pub type BitId = u8;

/// Line values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct Values {
    /// Logic values of lines
    pub bits: u64,
    /// Mask of lines to get or set
    pub mask: u64,
}

macro_rules! values_conv {
    ($($type:ty,)*) => {
        $(
            impl From<$type> for Values {
                fn from(bits: $type) -> Self {
                    Self {
                        bits: bits as _,
                        mask: <$type>::MAX as _,
                    }
                }
            }

            impl From<Values> for $type {
                fn from(values: Values) -> Self {
                    values.bits as _
                }
            }
        )*
    };
}

values_conv! {
    u8,
    u16,
    u32,
    u64,
}

impl Values {
    /// Get the value of specific bit
    ///
    /// If bit is out of range (0..64) or not masked then None will be returned.
    pub fn get(&self, bit: BitId) -> Option<bool> {
        if bit > 64 {
            return None;
        }

        let mask = 1 << bit;

        if (self.mask & mask) == 0 {
            return None;
        }

        Some(self.bits & mask != 0)
    }

    /// Set the value of specific bit and mask it
    ///
    /// If bit if out of range (0..64) then nothing will be set.
    pub fn set(&mut self, bit: BitId, val: bool) {
        if bit > 64 {
            return;
        }

        let mask = 1 << bit;

        self.mask |= mask;

        if val {
            self.bits |= mask;
        } else {
            self.bits &= !mask;
        }
    }
}

/// Direction of a GPIO line
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    /// Line acts as input (default)
    Input,
    /// Line acts as output
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

impl Default for Direction {
    fn default() -> Self {
        Self::Input
    }
}

/// Active state condition of a line
///
/// If active state of line is **high** then physical and logical levels is same.
/// Otherwise if it is **low** then physical level will be inverted from logical.
///
/// Also this may be treated as polarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Active {
    /// Active level is low
    Low,
    /// Active level is high (default)
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

impl Default for Active {
    fn default() -> Self {
        Self::High
    }
}

/// Signal edge or level transition of a GPIO line
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Edge {
    /// Rising edge detected
    Rising,
    /// Falling edge detected
    Falling,
}

impl AsRef<str> for Edge {
    fn as_ref(&self) -> &str {
        match self {
            Self::Rising => "rising",
            Self::Falling => "falling",
        }
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

/// Signal edge detection event
#[derive(Clone, Copy)]
pub struct Event {
    /// GPIO line where edge detected
    pub line: BitId,
    /// Detected edge or level transition
    pub edge: Edge,
    /// Time when edge actually detected
    pub time: SystemTime,
}

/// Edge detection setting for GPIO line
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EdgeDetect {
    /// Detection disabled (default)
    Disable,
    /// Detect rising edge only
    Rising,
    /// Detect falling edge only
    Falling,
    /// Detect both rising and falling edges
    Both,
}

impl AsRef<str> for EdgeDetect {
    fn as_ref(&self) -> &str {
        match self {
            Self::Disable => "disable",
            Self::Rising => "rising",
            Self::Falling => "falling",
            Self::Both => "both",
        }
    }
}

impl fmt::Display for EdgeDetect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl Default for EdgeDetect {
    fn default() -> Self {
        Self::Disable
    }
}

/// Input bias of a GPIO line
///
/// Sometimes GPIO lines shall be pulled to up (power rail) or down (ground)
/// through resistor to avoid floating level on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Bias {
    /// Disabled bias (default)
    Disable,
    /// Pull line up
    PullUp,
    /// Pull line down
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

impl Default for Bias {
    fn default() -> Self {
        Self::Disable
    }
}

/// Output drive mode of a GPIO line
///
/// Usually GPIO lines configured as push-pull but sometimes it required to drive via open drain or source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Drive {
    /// Drive push-pull (default)
    PushPull,
    /// Drive with open-drain
    OpenDrain,
    /// Drive with open-source
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

impl Default for Drive {
    fn default() -> Self {
        Self::PushPull
    }
}
