use crate::utils::get_prefixed_db_key_hash;

#[cfg(test)] // NOTE Because of real BTC tx test-vectors
pub const PTOKEN_P2SH_SCRIPT_BYTES: usize = 0;

#[cfg(not(test))]
pub const PTOKEN_P2SH_SCRIPT_BYTES: usize = 101;

pub const BTC_TAIL_LENGTH: u64 = 10;

pub const DEFAULT_BTC_SEQUENCE: u32 = 4_294_967_295; // NOTE: 0xFFFFFFFF

// NOTE: Following is used as placeholder for bad address parsing in ETH params!
pub const DEFAULT_BTC_ADDRESS: &str = "msTgHeQgPZ11LRcUdtfzagEfiZyKF57DhR";

lazy_static! {
    pub static ref BTC_DIFFICULTY_THRESHOLD: [u8; 32] = get_prefixed_db_key_hash(
        "btc-difficulty"
    );
}

lazy_static! {
    pub static ref BTC_ADDRESS_KEY: [u8; 32] = get_prefixed_db_key_hash(
        "btc-address"
    );
}

lazy_static! {
    pub static ref BTC_CANON_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key_hash(
        "btc-canon-block"
    );
}

lazy_static! {
    pub static ref BTC_LATEST_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key_hash(
        "btc-latest-block"
    );
}

lazy_static! {
    pub static ref BTC_LINKER_HASH_KEY: [u8; 32] = get_prefixed_db_key_hash(
        "btc-linker-hash"
    );
}

lazy_static! {
    pub static ref BTC_ANCHOR_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key_hash(
        "btc-anchor-block"
    );
}

lazy_static! {
    pub static ref BTC_PRIVATE_KEY_DB_KEY: [u8; 32] = get_prefixed_db_key_hash(
        "btc-private-key"
    );
}

lazy_static! {
    pub static ref BTC_CANON_TO_TIP_LENGTH_KEY: [u8; 32] = get_prefixed_db_key_hash(
        "btc-canon-to-tip-length"
    );
}

lazy_static! {
    pub static ref PTOKEN_GENESIS_HASH: [u8; 32] = get_prefixed_db_key_hash(
        "provable-ptoken"
    );
}

lazy_static! {
    pub static ref BTC_NETWORK_KEY: [u8; 32] = get_prefixed_db_key_hash(
        "btc-network-key"
    );
}

lazy_static! {
    pub static ref BTC_FEE_KEY: [u8; 32] = get_prefixed_db_key_hash(
        "btc-fee-key"
    );
}

lazy_static! {
    pub static ref BTC_ACCOUNT_NONCE_KEY: [u8; 32] = get_prefixed_db_key_hash(
        "btc-account-nonce-key"
    );
}

lazy_static! {
    pub static ref BTC_TAIL_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key_hash(
        "btc-tail-block-hash-key"
    );
}
