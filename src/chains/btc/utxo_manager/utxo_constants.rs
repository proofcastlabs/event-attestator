use crate::utils::get_prefixed_db_key_hash;

lazy_static! {
    pub static ref UTXO_FIRST: [u8; 32] = get_prefixed_db_key_hash(
        "utxo-first"
    );
}

lazy_static! {
    pub static ref UTXO_LAST: [u8; 32] = get_prefixed_db_key_hash(
        "utxo-last"
    );
}

lazy_static! {
    pub static ref UTXO_BALANCE: [u8; 32] = get_prefixed_db_key_hash(
        "utxo-balance"
    );
}

lazy_static! {
    pub static ref UTXO_NONCE: [u8; 32] = get_prefixed_db_key_hash(
        "utxo-nonce"
    );
}
