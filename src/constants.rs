#[cfg(feature="debug")]
pub const DEBUG_MODE: bool = true;

#[cfg(not(feature="debug"))]
pub const DEBUG_MODE: bool = false;

#[cfg(feature="non-validating")]
pub const CORE_IS_VALIDATING: bool = false;

#[cfg(not(feature="non-validating"))]
pub const CORE_IS_VALIDATING: bool = true;

pub const U64_NUM_BYTES: usize = 8;
pub const MIN_DATA_SENSITIVITY_LEVEL: Option<u8> = None;

#[cfg(feature="pbtc-on-eth")]
pub static DB_KEY_PREFIX: &str = "pbtc-on-eth-";

#[cfg(feature="pbtc-on-eos")]
pub static DB_KEY_PREFIX: &str = "pbtc-on-eos-";

#[cfg(feature="legacy")]
pub static DB_KEY_PREFIX: &str = "";

#[cfg(feature="legacy")]
warn!("✘ !!WARNING!! Legacy mode enabled - DB key conflicts are possible!");
