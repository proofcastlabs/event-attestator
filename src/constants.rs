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
pub const PRIVATE_KEY_DATA_SENSITIVITY_LEVEL: Option<u8> = Some(255);

pub const ETH_MESSAGE_PREFIX: &[u8; 26] = b"\x19Ethereum Signed Message:\n";
pub const PREFIXED_MESSAGE_HASH_LEN: &[u8; 2] = b"32";

lazy_static! {
    pub static ref DB_KEY_PREFIX: &'static str = match option_env!(
        "DB_KEY_PREFIX"
    ) {
        Some(prefix) => prefix,
        None => "",
    };
}
