use derive_more::{Constructor, Deref};
use eos_chain::{AccountName as EosAccountName, Checksum256};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

use crate::chains::eos::eos_global_sequences::GlobalSequence;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Deref, Constructor)]
pub struct EosOnIntIntTxInfos(pub Vec<EosOnIntIntTxInfo>);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Constructor)]
pub struct EosOnIntIntTxInfo {
    pub token_amount: U256,
    pub from: EosAccountName,
    pub destination_address: EthAddress,
    pub originating_tx_id: Checksum256,
    pub global_sequence: GlobalSequence,
    pub eth_token_address: EthAddress,
    pub eos_token_address: String,
}
