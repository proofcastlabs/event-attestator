#[cfg(feature="debug")]
pub const DEBUG_MODE: bool = true;

#[cfg(not(feature="debug"))]
pub const DEBUG_MODE: bool = false;

pub const HASH_LENGTH: usize  = 32;
pub const U64_NUM_BYTES: usize = 8;
pub const HASH_HEX_CHARS: usize  = 64;
pub const HEX_PREFIX_LENGTH: usize = 2;
pub const MINIMUM_REQUIRED_SATOSHIS: u64 = 10; // FIXME RM!
pub static LOG_FILE_PATH: &'static str = "logs/";
pub static EOS_TOKEN_TICKER: &'static str = "BTC"; // TODO Maybe get from db?
pub static SAFE_EOS_ADDRESS: &'static str = "provabletest";
pub const PRIVATE_KEY_DATA_SENSITIVITY_LEVEL: Option<u8> = Some(255);
pub static SAFE_BTC_ADDRESS: &'static str = "136CTERaocm8dLbEtzCaFtJJX9jfFhnChK";
