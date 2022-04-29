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
    int_on_eos::eos::int_tx_info::{IntOnEosIntTxInfo, IntOnEosIntTxInfos},
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

//NOTE: smart contract address: &state.eth_db_utils.get_int_on_eos_smart_contract_address_from_db()?,

impl IntOnEosIntTxInfos {
    pub fn get_global_sequences(&self) -> GlobalSequences {
        GlobalSequences::new(
            self.iter()
                .map(|infos| infos.global_sequence)
                .collect::<Vec<GlobalSequence>>(),
        )
    }

    pub fn filter_out_already_processed_txs(&self, processed_tx_ids: &ProcessedGlobalSequences) -> Result<Self> {
        Ok(IntOnEosIntTxInfos::new(
            self.iter()
                .filter(|info| !processed_tx_ids.contains(&info.global_sequence))
                .cloned()
                .collect::<Vec<IntOnEosIntTxInfo>>(),
        ))
    }
}

pub fn maybe_filter_out_already_processed_tx_infos_from_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Filtering out already processed tx infos...");
    state
        .erc20_on_eos_redeem_infos
        .filter_out_already_processed_txs(&state.processed_tx_ids)
        .and_then(|filtered| state.add_erc20_on_eos_redeem_infos(filtered))
}
