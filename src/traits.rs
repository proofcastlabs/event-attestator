use crate::{
    chains::eth::{
        eth_crypto_utils::keccak_hash_bytes,
        any_sender::relay_transaction::RelayTransaction,
    },
    types::{
        Bytes,
        Result
    },
};

pub trait DatabaseInterface {
    fn end_transaction(&self) -> Result<()>;

    fn start_transaction(&self) -> Result<()>;

    fn delete(&self, key: Bytes) -> Result<()>;

    fn get(
        &self,
        key: Bytes,
        data_sensitivity: Option<u8>
    ) -> Result<Bytes>;

    fn put(
        &self,
        key: Bytes,
        value: Bytes,
        data_sensitivity: Option<u8>
    ) -> Result<()>;
}


pub trait EthTxInfoCompatible {
    fn is_any_sender(&self) -> bool;

    fn any_sender_tx(&self) -> Option<RelayTransaction>;

    fn eth_tx_hex(&self) -> Option<String>;

    fn serialize_bytes(&self) -> Bytes;

    fn get_tx_hash(&self) -> String {
        hex::encode(keccak_hash_bytes(&self.serialize_bytes()))
    }
}
