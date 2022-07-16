use std::{fmt, time::SystemTime};

/// Line offset
pub type LineId = u32;

/// Bit offset
pub type BitId = u8;

/// Line values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct Values {
    pub bits: u64,
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
    /// Get value of bit
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

    /// Set value of bit
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

impl Default for Direction {
    fn default() -> Self {
        Self::Input
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

impl Default for Active {
    fn default() -> Self {
        Self::High
    }
}

/// Represents the detectected edge of a GPIO line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Edge {
    Rising,
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

/// Represent input event.
#[derive(Clone, Copy)]
pub struct Event {
    pub line: BitId,
    pub edge: Edge,
    pub time: SystemTime,
}

/// Represents the edge detection of a GPIO line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EdgeDetect {
    Disable,
    Rising,
    Falling,
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

impl Default for Bias {
    fn default() -> Self {
        Self::Disable
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

impl Default for Drive {
    fn default() -> Self {
        Self::PushPull
    }
}
