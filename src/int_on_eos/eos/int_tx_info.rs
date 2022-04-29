use std::str::{from_utf8, FromStr};

use derive_more::{Constructor, Deref};
use eos_chain::{AccountName as EosAccountName, Checksum256};
use ethereum_types::{Address as EthAddress, U256};

use crate::{
    chains::{
        eos::{
            eos_action_proofs::EosActionProof,
            eos_chain_id::EosChainId,
            eos_global_sequences::{GlobalSequence, GlobalSequences, ProcessedGlobalSequences},
            eos_state::EosState,
        },
        eth::{eth_constants::MAX_BYTES_FOR_ETH_USER_DATA, eth_database_utils::EthDbUtilsExt},
    },
    dictionaries::eos_eth::{EosEthTokenDictionary, EosEthTokenDictionaryEntry},
    metadata::{
        metadata_address::MetadataAddress,
        metadata_chain_id::MetadataChainId,
        metadata_protocol_id::MetadataProtocolId,
        metadata_traits::{ToMetadata, ToMetadataChainId},
        Metadata,
    },
    safe_addresses::SAFE_ETH_ADDRESS,
    traits::DatabaseInterface,
    types::{Bytes, Result},
    utils::{convert_bytes_to_u64, strip_hex_prefix},
};

#[derive(Clone, Debug, PartialEq, Eq, Deref, Constructor)]
pub struct IntOnEosIntTxInfos(pub Vec<IntOnEosIntTxInfo>);

#[derive(Clone, Debug, Default, PartialEq, Eq, Constructor)]
pub struct IntOnEosIntTxInfo {
    pub amount: U256,
    pub user_data: Bytes,
    pub eos_tx_amount: String,
    pub eos_token_address: String,
    pub int_token_address: String,
    pub int_vault_address: String,
    pub router_address: EthAddress,
    pub destination_address: String,
    pub origin_address: EosAccountName,
    pub originating_tx_id: Checksum256,
    pub global_sequence: GlobalSequence,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}
