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

lazy_static! {
    pub static ref DB_KEY_PREFIX: &'static str = match option_env!(
        "DB_KEY_PREFIX"
    ) {
        Some(prefix) => prefix,
        None => "",
    };
}
