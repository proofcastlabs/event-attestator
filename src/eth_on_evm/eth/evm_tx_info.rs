use std::str::FromStr;

use derive_more::{Constructor, Deref};
use eos_primitives::AccountName as EosAccountName;
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    chains::{
        eos::eos_utils::remove_symbol_from_eos_asset,
        eth::{
            eth_constants::{
                ETH_ADDRESS_SIZE_IN_BYTES,
                ETH_ON_EVM_PEG_IN_EVENT_TOPIC,
                ETH_ON_EVM_PEG_IN_EVENT_TOPIC_HEX,
                ETH_WORD_SIZE_IN_BYTES,
            },
            eth_contracts::eth_on_evm::EthOnEvmPegInEventParams,
            eth_database_utils::{
                get_erc20_on_eos_smart_contract_address_from_db,
                get_eth_canon_block_from_db,
                get_eth_on_evm_smart_contract_address_from_db,
            },
            eth_log::{EthLog, EthLogs},
            eth_receipt::{EthReceipt, EthReceipts},
            eth_state::EthState,
            eth_submission_material::EthSubmissionMaterial,
        },
    },
    constants::SAFE_ETH_ADDRESS,
    dictionaries::eth_evm::EthEvmTokenDictionary,
    traits::DatabaseInterface,
    types::{Bytes, Result},
};

pub const NOT_ENOUGH_BYTES_IN_LOG_DATA_ERR: &str = "Not enough bytes in log data!";

#[derive(Debug, Clone, PartialEq, Eq, Constructor)]
pub struct EthOnEvmEvmTxInfo {
    pub token_amount: U256,
    pub token_sender: EthAddress,
    pub originating_tx_hash: EthHash,
    pub evm_token_address: EthAddress,
    pub eth_token_address: EthAddress,
    pub destination_address: EthAddress,
    pub user_data: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct EthOnEvmEvmTxInfos(pub Vec<EthOnEvmEvmTxInfo>);

impl EthOnEvmEvmTxInfos {
    pub fn filter_out_zero_values(&self) -> Result<Self> {
        Ok(Self::new(
            self.iter()
                .filter(|tx_info| match tx_info.token_amount != U256::zero() {
                    true => true,
                    false => {
                        info!("✘ Filtering out peg in info due to zero asset amount: {:?}", tx_info);
                        false
                    },
                })
                .cloned()
                .collect::<Vec<EthOnEvmEvmTxInfo>>(),
        ))
    }

    fn is_log_eth_on_evm_peg_in(log: &EthLog) -> Result<bool> {
        Ok(log.contains_topic(&EthHash::from_slice(
            &hex::decode(&ETH_ON_EVM_PEG_IN_EVENT_TOPIC_HEX)?[..],
        )))
    }

    pub fn is_log_supported_eth_on_evm_peg_in(log: &EthLog, dictionary: &EthEvmTokenDictionary) -> Result<bool> {
        match Self::is_log_eth_on_evm_peg_in(log)? {
            false => Ok(false),
            true => Self::get_token_contract_address_from_log(log)
                .map(|token_contract_address| dictionary.is_eth_token_supported(&token_contract_address)),
        }
    }

    fn get_token_contract_address_from_log(log: &EthLog) -> Result<EthAddress> {
        EthOnEvmPegInEventParams::from_log(log).map(|params| params.token_address)
    }

    fn check_log_is_eth_on_evm_peg_in(log: &EthLog) -> Result<()> {
        trace!("✔ Checking if log is an `ETH-on-ETVM` peg in...");
        match Self::is_log_eth_on_evm_peg_in(log)? {
            true => Ok(()),
            false => Err("✘ Log is not from an `ETH-on-EVM` peg in event!".into()),
        }
    }

    fn receipt_contains_supported_eth_on_evm_peg_in(receipt: &EthReceipt, dictionary: &EthEvmTokenDictionary) -> bool {
        Self::get_supported_eth_on_evm_logs_from_receipt(receipt, dictionary).len() > 0
    }

    fn get_supported_eth_on_evm_logs_from_receipt(receipt: &EthReceipt, dictionary: &EthEvmTokenDictionary) -> EthLogs {
        EthLogs::new(
            receipt
                .logs
                .iter()
                .filter(|log| matches!(Self::is_log_supported_eth_on_evm_peg_in(&log, dictionary), Ok(true)))
                .cloned()
                .collect(),
        )
    }

    fn from_eth_receipt(receipt: &EthReceipt, dictionary: &EthEvmTokenDictionary) -> Result<Self> {
        info!("✔ Getting `ETH-on-EVM` peg in infos from receipt...");
        Ok(Self::new(
            Self::get_supported_eth_on_evm_logs_from_receipt(receipt, dictionary)
                .iter()
                .enumerate()
                .map(|(i, log)| {
                    let event_params = EthOnEvmPegInEventParams::from_log(log)?;
                    let tx_info = EthOnEvmEvmTxInfo {
                        eth_token_address: log.address,
                        user_data: event_params.user_data.clone(),
                        originating_tx_hash: receipt.transaction_hash,
                        token_amount: event_params.token_amount.clone(),
                        token_sender: event_params.token_sender.clone(),
                        destination_address: event_params.destination_address,
                        evm_token_address: dictionary.get_evm_address_from_eth_address(&event_params.token_address)?,
                    };
                    info!("✔ Parsed tx info: {:?}", tx_info);
                    Ok(tx_info)
                })
                .collect::<Result<Vec<EthOnEvmEvmTxInfo>>>()?,
        ))
    }

    fn filter_eth_submission_material_for_supported_peg_ins(
        submission_material: &EthSubmissionMaterial,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<EthSubmissionMaterial> {
        info!("✔ Filtering submission material receipts for those pertaining to `erc20-on-eos` peg-ins...");
        info!(
            "✔ Num receipts before filtering: {}",
            submission_material.receipts.len()
        );
        let filtered_receipts = EthReceipts::new(
            submission_material
                .receipts
                .iter()
                .filter(|receipt| {
                    EthOnEvmEvmTxInfos::receipt_contains_supported_eth_on_evm_peg_in(&receipt, dictionary)
                })
                .cloned()
                .collect(),
        );
        info!("✔ Num receipts after filtering: {}", filtered_receipts.len());
        Ok(EthSubmissionMaterial::new(
            submission_material.get_block()?,
            filtered_receipts,
            None,
            None,
        ))
    }

    pub fn from_submission_material(
        submission_material: &EthSubmissionMaterial,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<Self> {
        info!("✔ Getting `EthOnEvmEvmTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Self::from_eth_receipt(&receipt, dictionary))
                .collect::<Result<Vec<EthOnEvmEvmTxInfos>>>()?
                .iter()
                .map(|infos| infos.iter().cloned().collect())
                .collect::<Vec<Vec<EthOnEvmEvmTxInfo>>>()
                .concat(),
        ))
    }
    // TODO To signed EVM txs!
}

pub fn maybe_parse_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `ETH-on-EVM` peg-in infos...");
    get_eth_canon_block_from_db(&state.db).and_then(|submission_material| {
        match submission_material.receipts.is_empty() {
            true => {
                info!("✔ No receipts in canon block ∴ no info to parse!");
                Ok(state)
            },
            false => {
                info!(
                    "✔ {} receipts in canon block ∴ parsing info...",
                    submission_material.receipts.len()
                );
                EthEvmTokenDictionary::get_from_db(&state.db)
                    .and_then(|account_names| {
                        EthOnEvmEvmTxInfos::from_submission_material(&submission_material, &account_names)
                    })
                    .and_then(|tx_infos| state.add_eth_on_evm_evm_tx_infos(tx_infos))
            },
        }
    })
}

pub fn filter_out_zero_value_tx_infos_from_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Maybe filtering out zero value `EthOnEvmEvmTxInfos`...");
    debug!(
        "✔ Num `EthOnEvmEvmTxInfos` before: {}",
        state.erc20_on_eos_peg_in_infos.len()
    );
    state
        .eth_on_evm_evm_tx_infos
        .filter_out_zero_values()
        .and_then(|filtered_tx_infos| {
            debug!("✔ Num `EthOnEvmEvmTxInfos` after: {}", filtered_tx_infos.len());
            state.replace_eth_on_evm_evm_tx_infos(filtered_tx_infos)
        })
}

pub fn filter_submission_material_for_peg_in_events_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing `ETH-on-EVM` peg in events...");
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_address_and_with_topics(
            &get_eth_on_evm_smart_contract_address_from_db(&state.db)?,
            &ETH_ON_EVM_PEG_IN_EVENT_TOPIC.to_vec(),
        )
        .and_then(|filtered_submission_material| {
            EthOnEvmEvmTxInfos::filter_eth_submission_material_for_supported_peg_ins(
                &filtered_submission_material,
                state.get_eth_evm_token_dictionary()?,
            )
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}
