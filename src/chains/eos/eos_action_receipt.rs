use std::str::FromStr;

use bitcoin_hashes::{sha256, Hash};
use eos_primitives::{AccountName as EosAccountName, AuthSequences, Checksum256, NumBytes, Read, SerializeData, Write};

use crate::{chains::eos::eos_utils::convert_hex_to_checksum256, types::Bytes};

#[derive(Clone, Debug, Serialize, Deserialize, Read, Write, NumBytes, Default, PartialEq, Eq, PartialOrd, Ord)]
#[eosio_core_root_path = "::eos_primitives"]
pub struct EosActionReceipt {
    pub recipient: EosAccountName,
    pub act_digest: Checksum256,
    pub global_sequence: u64,
    pub recv_sequence: u64,
    pub auth_sequence: AuthSequences,
    pub code_sequence: usize,
    pub abi_sequence: usize,
}

impl SerializeData for EosActionReceipt {}

impl EosActionReceipt {
    pub fn new(
        recipient: &str,
        act_digest_string: &str,
        recv_sequence: u64,
        abi_sequence: usize,
        global_sequence: u64,
        code_sequence: usize,
        auth_sequences: AuthSequences,
    ) -> crate::Result<Self> {
        Ok(Self {
            abi_sequence,
            code_sequence,
            recv_sequence,
            global_sequence,
            auth_sequence: auth_sequences,
            recipient: EosAccountName::from_str(recipient)?,
            act_digest: convert_hex_to_checksum256(act_digest_string)?,
        })
    }

    pub fn serialize(&self) -> Bytes {
        self.to_serialize_data()
    }

    pub fn to_digest(&self) -> Bytes {
        sha256::Hash::hash(&self.serialize()).to_vec()
    }
}
