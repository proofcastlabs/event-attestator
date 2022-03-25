pub const DEBUG_MODE: bool = cfg!(feature = "debug");
pub const CORE_IS_VALIDATING: bool = !cfg!(feature = "non-validating");
pub const CORE_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

pub const NOT_VALIDATING_WHEN_NOT_IN_DEBUG_MODE_ERROR: &str =
    "✘ Not allowed to skip validation when core is not built in `DEBUG` mode!`";

pub const ETH_HASH_LENGTH: usize = 32;
pub const PTOKEN_ERC777_NUM_DECIMALS: u32 = 18;
pub const SUCCESS_JSON: &str = "{success:true}";
pub const FIELD_NOT_SET_MSG: &str = "Not set!";
pub const FEE_BASIS_POINTS_DIVISOR: u64 = 10_000;
pub const MIN_DATA_SENSITIVITY_LEVEL: Option<u8> = None;
pub const DEBUG_OUTPUT_MARKER: &str = "DEBUG_OUTPUT_MARKER";
pub const MAX_DATA_SENSITIVITY_LEVEL: Option<u8> = Some(255);

lazy_static! {
    pub static ref THIRTY_TWO_ZERO_BYTES: Vec<u8> = vec![0; 32];
    pub static ref DB_KEY_PREFIX: &'static str = option_env!("DB_KEY_PREFIX").unwrap_or("");
}
