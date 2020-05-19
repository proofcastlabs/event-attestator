use tiny_keccak::keccak256;

#[cfg(any(feature="pbtc-on-eth", feature="pbtc-on-eos"))]
lazy_static! {
    pub static ref UTXO_FIRST: [u8; 32] = keccak256(b"utxo-first");
}

#[cfg(any(feature="pbtc-on-eth", feature="pbtc-on-eos"))]
lazy_static! {
    pub static ref UTXO_LAST: [u8; 32] = keccak256(b"utxo-last");
}

#[cfg(any(feature="pbtc-on-eth", feature="pbtc-on-eos"))]
lazy_static! {
    pub static ref UTXO_BALANCE: [u8; 32] = keccak256(b"utxo-balance");
}

#[cfg(any(feature="pbtc-on-eth", feature="pbtc-on-eos"))]
lazy_static! {
    pub static ref UTXO_NONCE: [u8; 32] = keccak256(b"utxo-nonce");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_expected_keccak_hash() {
        let expected_result =
            "2674b2e116a8fe42de73cd7e81f67c7e42c788c2da9711f2e5f628a001368b22";
        let result = hex::encode(keccak256(b"utxo-first"));
        assert_eq!(result, expected_result);
    }
}
