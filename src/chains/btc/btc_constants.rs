use crate::chains::btc::btc_utils::calculate_dust_amount;

pub const DUST_RELAY_FEE: u64 = 3; // NOTE: Unit: satoshis-per-byte
pub const BTC_TX_VERSION: i32 = 1;
pub const BTC_TX_LOCK_TIME: u32 = 0;
pub const BTC_TAIL_LENGTH: u64 = 10;
pub const MAX_NUM_OUTPUTS: usize = 2;
pub const BTC_PUB_KEY_SLICE_LENGTH: usize = 33;
pub const MINIMUM_REQUIRED_SATOSHIS: u64 = 5000;
pub const DEFAULT_BTC_SEQUENCE: u32 = 4_294_967_295; // NOTE: 0xFFFFFFFF
pub const BTC_CORE_IS_INITIALIZED_JSON: &str = "{btc_enclave_initialized:true}";
// NOTE: Following is used as placeholder for bad address parsing in ETH params!
pub const PLACEHOLDER_BTC_ADDRESS: &str = "msTgHeQgPZ11LRcUdtfzagEfiZyKF57DhR";

lazy_static! {
    // NOTE: The calculation relies on the above `DUST_RELAY_FEE` constant, which may be different
    // for different BTC forks. As such, change _that_ constant, not this one, in order to have the
    // correct dust amount calculated.
    pub static ref DUST_AMOUNT: u64 = calculate_dust_amount().expect("Getting dust amount should not fail!");
}
