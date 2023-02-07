use algorand::{AlgoChainId, AlgoNoteMetadata, AlgoState};
use common::{
    chains::eth::eth_database_utils::EthDbUtilsExt,
    dictionaries::evm_algo::EvmAlgoTokenDictionary,
    traits::DatabaseInterface,
    types::Result,
};
use ethereum_types::Address as EthAddress;
use rust_algorand::{AlgorandGenesisId, AlgorandHash, AlgorandTransaction, AlgorandTransactions};

use crate::algo::int_tx_info::{IntOnAlgoIntTxInfo, IntOnAlgoIntTxInfos};

impl IntOnAlgoIntTxInfos {
    fn get_missing_field_err(s: &str) -> String {
        format!("Attempting to parse ALGO tx with no {} field!", s)
    }

    fn from_algo_tx(
        tx: &AlgorandTransaction,
        dictionary: &EvmAlgoTokenDictionary,
        router_address: &EthAddress,
        genesis_hash: &AlgorandHash,
        vault_address: &EthAddress,
    ) -> Result<IntOnAlgoIntTxInfo> {
        info!("✔ Getting `IntOnAlgoIntTxInfos` from ALGO tx...");
        let metadata = match tx.note {
            Some(ref bytes) => AlgoNoteMetadata::from_bytes_or_default(bytes),
            None => AlgoNoteMetadata::default(),
        };
        let asset_id = match tx.transfer_asset_id {
            Some(id) => Result::Ok(id),
            None => Result::Err(Self::get_missing_field_err("asset id").into()),
        }?;
        let token_sender = match &tx.sender {
            Some(sender) => Result::Ok(sender),
            None => Result::Err(Self::get_missing_field_err("sender").into()),
        }?;
        let host_asset_amount = match tx.asset_amount {
            Some(amount) => Result::Ok(amount),
            None => Result::Err(Self::get_missing_field_err("asset amount").into()),
        }?;
        let tx_info = IntOnAlgoIntTxInfo {
            algo_asset_id: asset_id,
            token_sender: *token_sender,
            router_address: *router_address,
            originating_tx_hash: tx.to_id()?,
            int_vault_address: *vault_address,
            user_data: metadata.user_data.clone(),
            destination_address: metadata.destination_address,
            destination_chain_id: metadata.destination_chain_id,
            int_token_address: dictionary.get_evm_address_from_asset_id(asset_id)?,
            native_token_amount: dictionary.convert_algo_amount_to_evm_amount(asset_id, host_asset_amount)?,
            origin_chain_id: AlgoChainId::from_genesis_id(&AlgorandGenesisId::from_hash(genesis_hash)?)?
                .to_metadata_chain_id(),
        };
        info!("✔ Parsed tx info: {:?}", tx_info);
        Ok(tx_info)
    }

    pub fn from_algo_txs(
        txs: &AlgorandTransactions,
        dictionary: &EvmAlgoTokenDictionary,
        router_address: &EthAddress,
        genesis_hash: &AlgorandHash,
        vault_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `IntOnAlgoIntTxInfos` from ALGO txs...");
        Ok(Self::new(
            txs.iter()
                .map(|tx| Self::from_algo_tx(tx, dictionary, router_address, genesis_hash, vault_address))
                .collect::<Result<Vec<IntOnAlgoIntTxInfo>>>()?,
        ))
    }
}

pub fn maybe_parse_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    info!("✔ Maybe parsing `IntOnAlgoIntTxInfos`...");
    state
        .algo_db_utils
        .get_canon_submission_material()
        .and_then(|material| match material.block.transactions {
            None => {
                info!("✔ No transactions in canon submission material ∴ no tx info to parse!");
                Ok(state)
            },
            Some(txs) => {
                info!(
                    "✔ {} transactions in canon submission material ∴ parsing info...",
                    txs.len()
                );
                state
                    .get_evm_algo_token_dictionary()
                    .and_then(|dictionary| {
                        IntOnAlgoIntTxInfos::from_algo_txs(
                            &txs,
                            &dictionary,
                            &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
                            &state.algo_db_utils.get_genesis_hash()?,
                            &state.eth_db_utils.get_int_on_algo_smart_contract_address()?,
                        )
                    })
                    .and_then(|tx_infos| tx_infos.to_bytes())
                    .map(|bytes| state.add_tx_infos(bytes))
            },
        })
}
