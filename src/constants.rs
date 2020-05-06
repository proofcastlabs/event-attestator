#[cfg(feature="debug")]
pub const DEBUG_MODE: bool = true;

#[cfg(not(feature="debug"))]
pub const DEBUG_MODE: bool = false;

pub const U64_NUM_BYTES: usize = 8;
pub const MIN_DATA_SENSITIVITY_LEVEL: Option<u8> = None;
