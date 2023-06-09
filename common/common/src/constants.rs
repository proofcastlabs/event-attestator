pub const ETH_HASH_LENGTH: usize = 32;
pub const ALGO_TAIL_LENGTH: u64 = 30;
pub const MAX_FEE_BASIS_POINTS: u64 = 100;
pub const PTOKEN_ERC777_NUM_DECIMALS: u32 = 18;
pub const FIELD_NOT_SET_MSG: &str = "Not set!";
pub const SUCCESS_JSON: &str = "{success:true}";
pub const MIN_DATA_SENSITIVITY_LEVEL: Option<u8> = None;
pub const DEBUG_OUTPUT_MARKER: &str = "DEBUG_OUTPUT_MARKER";
pub const MAX_DATA_SENSITIVITY_LEVEL: Option<u8> = Some(255);
pub const CORE_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
pub const ETH_ZERO_ADDRESS_STR: &str = "0x0000000000000000000000000000000000000000";
pub const ZERO_CONFS_WARNING: &str = "WARNING: NUMBER OF CONFIRMATIONS IS SET TO ZERO!";

lazy_static! {
    pub static ref THIRTY_TWO_ZERO_BYTES: Vec<u8> = vec![0; 32];
    // NOTE: Used to create a core with DB key prefixes if one is supplied as an env var. Because
    // a core's DB keys are hashed, this can be used to avoid collisions between similar bridges.
    pub static ref DB_KEY_PREFIX: &'static str = option_env!("DB_KEY_PREFIX").unwrap_or("");
}

lazy_static! {
    pub static ref ALGO_PTOKEN_GENESIS_HASH: rust_algorand::AlgorandHash = rust_algorand::AlgorandHash::default();
}
