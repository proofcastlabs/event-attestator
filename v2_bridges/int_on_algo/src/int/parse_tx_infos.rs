use std::str::FromStr;

use common::{
    chains::eth::{
        eth_contracts::erc20_vault::{Erc20VaultPegInEventParams, ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2},
        eth_database_utils::EthDbUtilsExt,
        eth_log::{EthLog, EthLogExt, EthLogs},
        eth_receipt::EthReceipt,
        eth_submission_material::EthSubmissionMaterial,
        EthState,
    },
    dictionaries::evm_algo::EvmAlgoTokenDictionary,
    safe_addresses::SAFE_ALGO_ADDRESS,
    traits::DatabaseInterface,
    types::Result,
};
use common_algorand::AlgoDbUtils;
use ethereum_types::Address as EthAddress;
use rust_algorand::{AlgorandAddress, AlgorandAppId};

use crate::int::algo_tx_info::{IntOnAlgoAlgoTxInfo, IntOnAlgoAlgoTxInfos};

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
        app_id: &AlgorandAppId,
    ) -> Result<Self> {
        info!("✔ Getting `IntOnAlgoAlgoTxInfo` from receipt...");
        Ok(Self::new(
            Self::get_supported_logs(receipt, vault_address)
                .iter()
                .map(|log| {
                    let event_params = Erc20VaultPegInEventParams::from_eth_log(log)?;

                    let (destination_address, destination_app_id) =
                        match AlgorandAddress::from_str(&event_params.destination_address) {
                            Ok(address) => (Some(address), None),
                            Err(_) => match AlgorandAppId::from_str(&event_params.destination_address) {
                                Ok(app_id) => (None, Some(app_id)),
                                Err(_) => {
                                    warn!("✘ Neither address nor app ID was parsed! Defaulting to safe address!");
                                    (Some(*SAFE_ALGO_ADDRESS), None)
                                },
                            },
                        };

                    let tx_info = IntOnAlgoAlgoTxInfo {
                        destination_app_id,
                        destination_address,
                        vault_address: *vault_address,
                        router_address: *router_address,
                        issuance_manager_app_id: app_id.clone(),
                        token_sender: event_params.token_sender,
                        user_data: event_params.user_data.clone(),
                        int_token_address: event_params.token_address,
                        originating_tx_hash: receipt.transaction_hash,
                        native_token_amount: event_params.token_amount,
                        origin_chain_id: event_params.get_origin_chain_id()?,
                        destination_chain_id: event_params.get_destination_chain_id()?,
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
        app_id: &AlgorandAppId,
    ) -> Result<Self> {
        info!("✔ Getting `IntOnAlgoAlgoTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Self::from_eth_receipt(receipt, vault_address, dictionary, router_address, app_id))
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
        .and_then(|submission_material| {
            if submission_material.receipts.is_empty() {
                info!("✔ No receipts in canon block ∴ no info to parse!");
                Ok(state)
            } else {
                info!(
                    "✔ {} receipts in canon block ∴ parsing info...",
                    submission_material.receipts.len()
                );
                IntOnAlgoAlgoTxInfos::from_submission_material(
                    &submission_material,
                    &state.eth_db_utils.get_int_on_algo_smart_contract_address()?,
                    state.get_evm_algo_token_dictionary()?,
                    &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
                    &AlgoDbUtils::new(state.db).get_algo_app_id()?,
                )
                .and_then(|tx_infos| tx_infos.to_bytes())
                .map(|bytes| state.add_tx_infos(bytes))
            }
        })
}
