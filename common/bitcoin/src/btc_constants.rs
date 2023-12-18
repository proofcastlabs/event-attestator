use std::str::FromStr;

#[cfg(feature = "ltc")]
use crate::bitcoin_crate_alias::locktime::absolute::PackedLockTime;
#[cfg(not(feature = "ltc"))]
use crate::bitcoin_crate_alias::PackedLockTime;
use crate::{
    bitcoin_crate_alias::{hashes::sha256d, Sequence},
    btc_utils::calculate_dust_amount,
};

#[cfg(feature = "ltc")]
pub const BTC_TAIL_LENGTH: u64 = 40;
#[cfg(not(feature = "ltc"))]
pub const BTC_TAIL_LENGTH: u64 = 10;

#[cfg(not(feature = "ltc"))]
pub const MINIMUM_REQUIRED_SATOSHIS: u64 = 5000;
#[cfg(feature = "ltc")]
pub const MINIMUM_REQUIRED_SATOSHIS: u64 = 10_000;

pub const DUST_RELAY_FEE: u64 = 3; // NOTE: Unit: satoshis-per-byte
pub const BTC_TX_VERSION: i32 = 1;
pub const MAX_NUM_OUTPUTS: usize = 2;
pub const BTC_NUM_DECIMALS: usize = 8;
pub const BTC_PUB_KEY_SLICE_LENGTH: usize = 33;
pub(crate) const BTC_FEE_HARDCAP: u64 = 500_000; // NOTE: 0.005 btc
pub const BTC_TX_LOCK_TIME: PackedLockTime = PackedLockTime::ZERO;
pub const DEFAULT_BTC_SEQUENCE: Sequence = Sequence(4_294_967_295); // NOTE: 0xFFFFFFFF
pub const BTC_CORE_IS_INITIALIZED_JSON: &str = "{btc_enclave_initialized:true}";
// NOTE: Following is used as placeholder for bad address parsing in ETH params!
pub const PLACEHOLDER_BTC_ADDRESS: &str = "msTgHeQgPZ11LRcUdtfzagEfiZyKF57DhR";

lazy_static! {
    // NOTE: As you can see, the calculation relies on the `DUST_RELAY_FEE`, which may be different
    // for different BTC forks. As such, change _that_ constant above in order to have the correct
    // dust amount calculated!
    pub static ref DUST_AMOUNT: u64 = calculate_dust_amount(DUST_RELAY_FEE);

    pub static ref ZERO_HASH: sha256d::Hash = sha256d::Hash::from_str(&hex::encode([0u8; 32]))
        .expect("This won't fail");
}
