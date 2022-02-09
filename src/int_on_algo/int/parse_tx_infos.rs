use std::str::FromStr;

use ethereum_types::Address as EthAddress;
use rust_algorand::AlgorandAddress;

use crate::{
    chains::eth::{
        eth_contracts::erc20_vault::{Erc20VaultPegInEventParams, ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2},
        eth_database_utils::EthDbUtilsExt,
        eth_log::{EthLog, EthLogExt, EthLogs},
        eth_receipt::EthReceipt,
        eth_state::EthState,
        eth_submission_material::EthSubmissionMaterial,
    },
    dictionaries::evm_algo::EvmAlgoTokenDictionary,
    int_on_algo::int::algo_tx_info::{IntOnAlgoAlgoTxInfo, IntOnAlgoAlgoTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

impl IntOnAlgoAlgoTxInfos {
    fn is_log_a_supported_peg_in(log: &EthLog, vault_address: &EthAddress) -> Result<bool> {
        let log_contains_topic = log.contains_topic(&ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2);
        let log_is_from_vault_address = log.address == *vault_address;
        Ok(log_contains_topic && log_is_from_vault_address)
    }

    pub fn get_supported_logs(receipt: &EthReceipt, vault_address: &EthAddress) -> EthLogs {
        EthLogs::new(
            receipt
                .logs
                .iter()
                .filter(|log| matches!(Self::is_log_a_supported_peg_in(log, vault_address), Ok(true)))
                .cloned()
                .collect(),
        )
    }

    fn from_eth_receipt(
        receipt: &EthReceipt,
        vault_address: &EthAddress,
        dictionary: &EvmAlgoTokenDictionary,
        router_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `IntOnAlgoAlgoTxInfo` from receipt...");
        Ok(Self::new(
            Self::get_supported_logs(receipt, vault_address)
                .iter()
                .map(|log| {
                    let event_params = Erc20VaultPegInEventParams::from_eth_log(log)?;
                    let tx_info = IntOnAlgoAlgoTxInfo {
                        router_address: *router_address,
                        token_sender: event_params.token_sender,
                        user_data: event_params.user_data.clone(),
                        int_token_address: event_params.token_address,
                        originating_tx_hash: receipt.transaction_hash,
                        native_token_amount: event_params.token_amount,
                        origin_chain_id: event_params.get_origin_chain_id()?,
                        destination_chain_id: event_params.get_destination_chain_id()?,
                        destination_address: AlgorandAddress::from_str(&event_params.destination_address)?,
                        algo_asset_id: dictionary.get_algo_asset_id_from_evm_address(&event_params.token_address)?,
                        host_token_amount: dictionary.convert_evm_amount_to_algo_amount(
                            &event_params.token_address,
                            event_params.token_amount,
                        )?,
                    };
                    info!("✔ Parsed tx info: {:?}", tx_info);
                    Ok(tx_info)
                })
                .collect::<Result<Vec<IntOnAlgoAlgoTxInfo>>>()?,
        ))
    }

    pub fn from_submission_material(
        submission_material: &EthSubmissionMaterial,
        vault_address: &EthAddress,
        dictionary: &EvmAlgoTokenDictionary,
        router_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `IntOnAlgoAlgoTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Self::from_eth_receipt(receipt, vault_address, dictionary, router_address))
                .collect::<Result<Vec<IntOnAlgoAlgoTxInfos>>>()?
                .iter()
                .map(|infos| infos.iter().cloned().collect())
                .collect::<Vec<Vec<IntOnAlgoAlgoTxInfo>>>()
                .concat(),
        ))
    }
}

pub fn maybe_parse_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `IntOnAlgoAlgoTxInfos`...");
    state
        .eth_db_utils
        .get_eth_canon_block_from_db()
        .and_then(|submission_material| match submission_material.receipts.is_empty() {
            true => {
                info!("✔ No receipts in canon block ∴ no info to parse!");
                Ok(state)
            },
            false => {
                info!(
                    "✔ {} receipts in canon block ∴ parsing info...",
                    submission_material.receipts.len()
                );
                IntOnAlgoAlgoTxInfos::from_submission_material(
                    &submission_material,
                    &state.eth_db_utils.get_int_on_algo_smart_contract_address()?,
                    state.get_evm_algo_token_dictionary()?,
                    &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
                )
                .and_then(|tx_infos| state.add_int_on_algo_algo_tx_infos(tx_infos))
            },
        })
}

// TODO test!
