use derive_more::{Constructor, Deref};
use eos_chain::{AccountName as EosAccountName, Checksum256};
use ethereum_types::U256;
use serde::{Deserialize, Serialize};

use crate::{chains::eos::eos_global_sequences::GlobalSequence, metadata::MetadataChainId, types::Bytes};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Deref, Constructor)]
pub struct EosOnIntIntTxInfos(pub Vec<EosOnIntIntTxInfo>);

#[derive(Clone, Debug, Default, PartialEq, Eq, Constructor, Serialize, Deserialize)]
pub struct EosOnIntIntTxInfo {
    pub amount: U256,
    pub user_data: Bytes,
    pub eos_tx_amount: String,
    pub router_address: String,
    pub eos_token_address: String,
    pub int_token_address: String,
    pub destination_address: String,
    pub origin_address: EosAccountName,
    pub originating_tx_id: Checksum256,
    pub global_sequence: GlobalSequence,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}
