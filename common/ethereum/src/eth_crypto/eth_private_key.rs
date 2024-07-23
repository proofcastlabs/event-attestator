use std::{convert::TryFrom, fmt, str::FromStr};

use common::{
    constants::MAX_DATA_SENSITIVITY_LEVEL,
    crypto_utils::{generate_random_private_key, keccak_hash_bytes, sha256_hash_bytes},
    strip_hex_prefix,
    traits::DatabaseInterface,
    types::{Byte, Result},
    AppError,
};
use ethereum_types::{Address as EthAddress, H256};
use secp256k1::{
    key::{PublicKey, SecretKey, ONE_KEY},
    Message,
    Secp256k1,
};

use crate::{
    eth_crypto::{eth_public_key::EthPublicKey, eth_signature::EthSignature},
    eth_traits::EthSigningCapabilities,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthPrivateKey(SecretKey);

impl EthPrivateKey {
    pub fn to_address(&self) -> EthAddress {
        self.to_public_key().to_address()
    }

    pub fn to_hex(&self) -> String {
        format!("{}", self.0)
    }
}

impl FromStr for EthPrivateKey {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_slice(&hex::decode(strip_hex_prefix(s))?)
    }
}

impl TryFrom<&str> for EthPrivateKey {
    type Error = AppError;

    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        Self::from_slice(&hex::decode(s)?)
    }
}

impl EthPrivateKey {
    pub fn from_slice(slice: &[Byte]) -> Result<Self> {
        Ok(Self(SecretKey::from_slice(slice)?))
    }

    pub fn generate_random() -> Result<Self> {
        Ok(Self(generate_random_private_key()?))
    }

    pub fn to_public_key(&self) -> EthPublicKey {
        EthPublicKey {
            compressed: true,
            public_key: PublicKey::from_secret_key(&Secp256k1::new(), &self.0),
        }
    }

    pub fn write_to_database<D>(&self, db: &D, key: &[Byte]) -> Result<()>
    where
        D: DatabaseInterface,
    {
        db.put(key.to_vec(), self.0[..].to_vec(), MAX_DATA_SENSITIVITY_LEVEL)
    }
}

impl EthSigningCapabilities for EthPrivateKey {
    fn sign_hash(&self, hash: H256) -> Result<EthSignature> {
        let msg = match Message::from_slice(hash.as_bytes()) {
            Ok(msg) => msg,
            Err(err) => return Err(err.into()),
        };
        let sig = Secp256k1::sign_recoverable(&Secp256k1::new(), &msg, &self.0);
        let (rec_id, data) = sig.serialize_compact();
        let mut data_arr = [0; 65];
        data_arr[0..64].copy_from_slice(&data[0..64]);
        data_arr[64] = rec_id.to_i32() as u8;
        Ok(EthSignature::new(data_arr))
    }

    fn sign_hash_with_normalized_parity(&self, hash: H256) -> Result<EthSignature> {
        let msg = match Message::from_slice(hash.as_bytes()) {
            Ok(msg) => msg,
            Err(err) => return Err(err.into()),
        };
        let sig = Secp256k1::sign_recoverable(&Secp256k1::new(), &msg, &self.0);
        let (rec_id, data) = sig.serialize_compact();
        let mut data_arr = [0; 65];
        data_arr[0..64].copy_from_slice(&data[0..64]);
        data_arr[64] = rec_id.to_i32() as u8 + 27;
        Ok(EthSignature::new(data_arr))
    }

    fn sign_hash_and_set_eth_recovery_param(&self, hash: H256) -> Result<EthSignature> {
        self.sign_hash(hash).map(EthSignature::set_recovery_param)
    }

    fn keccak_hash_and_sign_msg(&self, message: &[Byte]) -> Result<EthSignature> {
        self.sign_hash(keccak_hash_bytes(message))
    }

    fn sha256_hash_and_sign_msg(&self, message: &[Byte]) -> Result<EthSignature> {
        self.sign_hash(H256::from_slice(&sha256_hash_bytes(message)))
    }

    fn sha256_hash_and_sign_msg_with_normalized_parity(&self, message: &[Byte]) -> Result<EthSignature> {
        self.sign_hash_with_normalized_parity(H256::from_slice(&sha256_hash_bytes(message)))
    }

    fn hash_and_sign_msg(&self, message: &[Byte]) -> Result<EthSignature> {
        // NOTE: For backwards compatibility, where keccack was the default hashing for eth sigs
        self.keccak_hash_and_sign_msg(message)
    }

    fn hash_and_sign_msg_with_eth_prefix(&self, message: &[Byte]) -> Result<EthSignature> {
        let eth_msg_prefix = b"\x19Ethereum Signed Message:\n";

        // NOTE: See here: https://github.com/ethers-io/ethers.js/blob/77fcc7fdab9a7123f67bbc8c4d1c013ee2f6edca/src.ts/hash/message.ts#L37
        let msg = [eth_msg_prefix, format!("{}", message.len()).as_bytes(), message].concat();

        self.hash_and_sign_msg(&msg).map(EthSignature::set_recovery_param)
    }
}

impl fmt::Display for EthPrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "✘ Cannot print ETH private key!")
    }
}

impl Drop for EthPrivateKey {
    fn drop(&mut self) {
        unsafe { ::std::ptr::write_volatile(&mut self.0, ONE_KEY) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        convert_hex_to_eth_address,
        test_utils::{get_sample_eth_private_key, get_sample_eth_private_key_slice},
    };

    #[test]
    fn should_create_random_eth_private_key() {
        if let Err(e) = EthPrivateKey::generate_random() {
            panic!("Error generating random eth private key: {}", e);
        }
    }

    #[test]
    fn should_create_eth_private_key_from_slice() {
        if let Err(e) = EthPrivateKey::from_slice(&get_sample_eth_private_key_slice()) {
            panic!("Error generating eth private key from slice: {}", e);
        }
    }

    #[test]
    fn should_hash_and_sign_msg() {
        let key = get_sample_eth_private_key();
        let message_bytes = vec![0xc0, 0xff, 0xee];
        if let Err(e) = key.hash_and_sign_msg(&message_bytes) {
            panic!("Error signing message bytes: {}", e);
        }
    }

    #[test]
    fn should_sign_message_hash() {
        let key = get_sample_eth_private_key();
        let message_bytes = vec![0xc0, 0xff, 0xee];
        let message_hash = keccak_hash_bytes(&message_bytes);
        if let Err(e) = key.sign_hash(message_hash) {
            panic!("Error signing message hash: {}", e);
        }
    }

    #[test]
    fn should_hash_and_sign_msg_with_eth_prefix() {
        let key = get_sample_eth_private_key();
        let message = "Arbitrary message";
        if let Err(e) = key.hash_and_sign_msg_with_eth_prefix(message.as_bytes()) {
            panic!("Error signing eth prefixed message bytes: {}", e);
        }
    }

    #[test]
    fn should_hash_and_sign_msg_with_eth_prefix_recoverable_with_solidity() {
        let eth_private_key =
            EthPrivateKey::from_str("841734cb439af03575c37c29b332619f3da9ea2fbaed58a1c8b1188ecff2a8dd").unwrap();
        let msg = hex::decode("00000000000000000000000053c2048dad4fcfab44c3ef3d16e882b5178df42b00000000000000000000000000000000000000000000000000000000000005390000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let expected_result = "6c7739aefe46a4bbef64ea98ff3719204b2e23b0b45f7b213642b1ec13b3021f47a5b6c3f5f1b8dd60c37014eb1403f85bf2c586529927674800609fe5582d261c";
        let result = eth_private_key
            .hash_and_sign_msg_with_eth_prefix(&msg)
            .unwrap()
            .to_string();
        assert_eq!(expected_result, result);
    }

    #[test]
    fn should_get_public_key_from_private() {
        let expected_result = hex::decode(
            "04d95149f2ea3a078523d28fb8fb0d589f8a8c8e90d9688a9bdcbcd97f43e157a74ec521b7fd317e4a02bd81ed5822d6ff93ea78d529cd2a7c2d196ec992d00754"
        ).unwrap();
        let private_key = get_sample_eth_private_key();
        let result = private_key.to_public_key().to_bytes();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_address_from_pk() {
        let pk = get_sample_eth_private_key();
        let result = pk.to_address();
        let expected_result = convert_hex_to_eth_address("0x1739624f5cd969885a224da84418d12b8570d61a").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_pk_as_hex() {
        let pk = get_sample_eth_private_key();
        let result = pk.to_hex();
        let expected_result = "e8eeb2631ab476dacd68f84eb0b9ee558b872f5155a088bf74381b5f2c63a130".to_string();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_eth_pk_from_non_hex_prefixed_str() {
        let s = "e8eeb2631ab476dacd68f84eb0b9ee558b872f5155a088bf74381b5f2c63a130";
        let result = EthPrivateKey::from_str(s).unwrap();
        let expected_result = get_sample_eth_private_key();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_eth_pk_from_hex_prefixed_str() {
        let s = "0xe8eeb2631ab476dacd68f84eb0b9ee558b872f5155a088bf74381b5f2c63a130";
        let result = EthPrivateKey::from_str(s).unwrap();
        let expected_result = get_sample_eth_private_key();
        assert_eq!(result, expected_result);
    }
}
