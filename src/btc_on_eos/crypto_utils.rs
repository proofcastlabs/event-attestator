use ethereum_types::H256;
use tiny_keccak::keccak256;
use secp256k1::{
    Secp256k1,
    Message as Secp256k1Message,
};
use rand::{
    RngCore,
    thread_rng,
};
use secp256k1::key::{
    SecretKey,
    PublicKey,
};
use bitcoin_hashes::{
    sha256,
    Hash as HashTrait
};
use crate::btc_on_eos::{
    types::{
        Bytes,
        Result,
    },
};
pub fn keccak_hash_bytes(bytes: Bytes) -> H256 {
    H256::from(keccak256(&bytes[..]))
}

pub fn sha256_hash_message_bytes(
    message_bytes: &Bytes
) -> Result<Secp256k1Message> {
    Ok(Secp256k1Message::from_slice(&sha256::Hash::hash(message_bytes))?)
}

fn get_public_key_from_secret(secret_key: SecretKey) -> PublicKey {
    PublicKey::from_secret_key(&Secp256k1::new(), &secret_key)
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::btc_on_eos::utils::convert_hex_to_h256;

    #[test]
    fn should_keccak_hash_bytes() {
        let bytes = vec![0xc0, 0xff, 0xee];
        let result = keccak_hash_bytes(bytes);
        let expected_result_hex =
            "7924f890e12acdf516d6278e342cd34550e3bafe0a3dec1b9c2c3e991733711a"
            .to_string();
        let expected_result = convert_hex_to_h256(expected_result_hex)
            .unwrap();
        assert!(result == expected_result);
    }

    #[test]
    fn should_generate_32_random_bytes() {
        let result = get_32_random_bytes_arr();
        assert!(result.len() == 32);
    }

    #[test]
    fn should_generate_x_random_bytes() {
        let x: usize = 100; // TODO: get a random num here!
        let result = get_x_random_bytes(x);
        assert!(result.len() == x);
    }

    #[test]
    fn should_generate_random_private_key() {
        generate_random_private_key()
            .unwrap();
    }

    #[test]
    fn should_convert_private_key_to_public() {
        let secret_bytes = hex::decode(
            "7c4495fe8341d1144c259f0b21979cbabd03814bbd747e70762c1c59b004d617"
        ).unwrap();
        let public_string = "0259419bf5cce6f6411ec3a90f0873b3156c43631403cb832dc710d00ec5690fe0"
            .to_string();
        let secret = SecretKey::from_slice(&secret_bytes[..])
            .unwrap();
        let result = get_public_key_from_secret(secret);
        assert!(result.to_string() == public_string);
    }
}
