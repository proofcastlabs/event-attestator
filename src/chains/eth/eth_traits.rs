use ethereum_types::H256;

use crate::{
    chains::eth::{
        any_sender::relay_transaction::RelayTransaction,
        eth_crypto_utils::keccak_hash_bytes,
        eth_types::EthSignature,
    },
    types::{Byte, Bytes, Result},
};

pub trait EthTxInfoCompatible {
    fn is_any_sender(&self) -> bool;
    fn any_sender_tx(&self) -> Option<RelayTransaction>;
    fn eth_tx_hex(&self) -> Option<String>;
    fn serialize_bytes(&self) -> Bytes;
    fn get_tx_hash(&self) -> String {
        hex::encode(keccak_hash_bytes(&self.serialize_bytes()))
    }
}

pub trait EthSigningCapabilities {
    fn sign_hash(&self, hash: H256) -> Result<EthSignature>;
    fn sign_message_bytes(&self, message: &[Byte]) -> Result<EthSignature>;
    fn sign_eth_prefixed_msg_bytes(&self, message: &[Byte]) -> Result<EthSignature>;
}
