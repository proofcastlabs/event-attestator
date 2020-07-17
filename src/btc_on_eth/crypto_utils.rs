use ethereum_types::H256;
use tiny_keccak::keccak256;
use secp256k1::key::SecretKey;
use rand::{
    RngCore,
    thread_rng,
};
use crate::{
    types::{
        Bytes,
        Result,
    },
    btc_on_eth::eth::eth_types::EthSignature,
};

pub fn keccak_hash_bytes(bytes: Bytes) -> H256 {
    H256::from(keccak256(&bytes[..]))
}

fn get_x_random_bytes(num_bytes: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; num_bytes];
    thread_rng().fill_bytes(&mut bytes);
    bytes
}

fn get_32_random_bytes_arr() -> [u8; 32] {
    let mut arr = [0; 32];
    arr.copy_from_slice(&get_x_random_bytes(32));
    arr
}

pub fn generate_random_private_key() -> Result<SecretKey> {
    Ok(SecretKey::from_slice(&get_32_random_bytes_arr())?)
}

pub fn set_eth_signature_recovery_param(signature: &mut EthSignature) {
    signature[64] = if signature[64] == 1 { 0x1c } else { 0x1b };
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::btc_on_eth::utils::convert_hex_to_h256;

    #[test]
    fn should_keccak_hash_bytes() {
        let bytes = vec![0xc0, 0xff, 0xee];
        let result = keccak_hash_bytes(bytes);
        let expected_result_hex = "7924f890e12acdf516d6278e342cd34550e3bafe0a3dec1b9c2c3e991733711a";
        let expected_result = convert_hex_to_h256(expected_result_hex).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_generate_32_random_bytes() {
        let result = get_32_random_bytes_arr();
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn should_generate_x_random_bytes() {
        let x: usize = 100;
        let result = get_x_random_bytes(x);
        assert_eq!(result.len(), x);
    }

    #[test]
    fn should_generate_random_private_key() {
        generate_random_private_key()
            .unwrap();
    }
}
