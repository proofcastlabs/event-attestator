use ethereum_types::H256;
use tiny_keccak::{Hasher, Keccak};

use crate::{chains::evm::eth_types::EthSignature, types::Byte};

pub fn keccak_hash_bytes(bytes: &[Byte]) -> H256 {
    let mut keccak = Keccak::v256();
    let mut hashed = [0u8; 32];
    keccak.update(&bytes);
    keccak.finalize(&mut hashed);
    H256::from(hashed)
}

pub fn set_eth_signature_recovery_param(signature: &mut EthSignature) {
    signature[64] = if signature[64] == 1 { 0x1c } else { 0x1b };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::eth::eth_utils::convert_hex_to_h256;

    #[test]
    fn should_keccak_hash_bytes() {
        let bytes = vec![0xc0, 0xff, 0xee];
        let result = keccak_hash_bytes(&bytes);
        let expected_result_hex = "7924f890e12acdf516d6278e342cd34550e3bafe0a3dec1b9c2c3e991733711a";
        let expected_result = convert_hex_to_h256(expected_result_hex).unwrap();
        assert_eq!(result, expected_result);
    }
}
