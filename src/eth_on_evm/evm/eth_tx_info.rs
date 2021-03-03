use std::str::FromStr;

use derive_more::{Constructor, Deref};
use eos_chain::AccountName as EosAccountName;
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    chains::{
        eth::{
            eth_constants::{
                ETH_ADDRESS_SIZE_IN_BYTES,
                ETH_ON_EVM_REDEEM_EVENT_TOPIC,
                ETH_ON_EVM_REDEEM_EVENT_TOPIC_HEX,
                ETH_WORD_SIZE_IN_BYTES,
                ZERO_ETH_VALUE,
            },
            eth_contracts::{
                eth_on_evm_erc777::{encode_erc777_mint_fxn_maybe_with_data, EthOnEvmErc777RedeemEvent},
                eth_on_evm_vault::ETH_ON_EVM_PEGOUT_WITH_USER_DATA_GAS_LIMIT,
            },
            eth_crypto::{
                eth_private_key::EthPrivateKey,
                eth_transaction::{EthTransaction as EvmTransaction, EthTransactions as EvmTransactions},
            },
            eth_database_utils::{
                get_eth_account_nonce_from_db,
                get_eth_chain_id_from_db,
                get_eth_gas_price_from_db,
                get_eth_private_key_from_db,
            },
        },
        evm::{
            eth_database_utils::get_eth_canon_block_from_db as get_evm_canon_block_from_db,
            eth_log::{EthLog, EthLogs},
            eth_receipt::{EthReceipt, EthReceipts},
            eth_state::EthState as EvmState,
            eth_submission_material::EthSubmissionMaterial as EvmSubmissionMaterial,
        },
    },
    constants::SAFE_ETH_ADDRESS,
    dictionaries::eth_evm::EthEvmTokenDictionary,
    traits::DatabaseInterface,
    types::{Bytes, Result},
};

pub const NOT_ENOUGH_BYTES_IN_LOG_DATA_ERR: &str = "Not enough bytes in log data!";

#[derive(Debug, Clone, PartialEq, Eq, Constructor)]
pub struct EthOnEvmEthTxInfo {
    pub token_amount: U256,
    pub token_sender: EthAddress,
    pub originating_tx_hash: EthHash,
    pub evm_token_address: EthAddress,
    pub eth_token_address: EthAddress,
    pub destination_address: EthAddress,
    pub user_data: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct EthOnEvmEthTxInfos(pub Vec<EthOnEvmEthTxInfo>);

impl EthOnEvmEthTxInfos {
    pub fn filter_out_zero_values(&self) -> Result<Self> {
        Ok(Self::new(
            self.iter()
                .filter(|tx_info| match tx_info.token_amount != U256::zero() {
                    true => true,
                    false => {
                        info!("✘ Filtering out redeem info due to zero asset amount: {:?}", tx_info);
                        false
                    },
                })
                .cloned()
                .collect::<Vec<EthOnEvmEthTxInfo>>(),
        ))
    }

    fn is_log_eth_on_evm_redeem(log: &EthLog) -> Result<bool> {
        Ok(log.contains_topic(&EthHash::from_slice(
            &hex::decode(&ETH_ON_EVM_REDEEM_EVENT_TOPIC_HEX)?[..],
        )))
    }

    pub fn is_log_supported_eth_on_evm_redeem(log: &EthLog, dictionary: &EthEvmTokenDictionary) -> Result<bool> {
        match Self::is_log_eth_on_evm_redeem(log)? {
            false => Ok(false),
            true => Ok(dictionary.is_evm_token_supported(&log.address)),
        }
    }

    fn check_log_is_eth_on_evm_redeem(log: &EthLog) -> Result<()> {
        trace!("✔ Checking if log is an `ETH-on-ETVM` redeem...");
        match Self::is_log_eth_on_evm_redeem(log)? {
            true => Ok(()),
            false => Err("✘ Log is not from an `ETH-on-EVM` redeem event!".into()),
        }
    }

    fn get_supported_eth_on_evm_logs_from_receipt(receipt: &EthReceipt, dictionary: &EthEvmTokenDictionary) -> EthLogs {
        EthLogs::new(
            receipt
                .logs
                .iter()
                .filter(|log| matches!(Self::is_log_supported_eth_on_evm_redeem(&log, dictionary), Ok(true)))
                .cloned()
                .collect(),
        )
    }

    fn receipt_contains_supported_eth_on_evm_redeem(receipt: &EthReceipt, dictionary: &EthEvmTokenDictionary) -> bool {
        Self::get_supported_eth_on_evm_logs_from_receipt(receipt, dictionary).len() > 0
    }

    fn from_eth_receipt(receipt: &EthReceipt, dictionary: &EthEvmTokenDictionary) -> Result<Self> {
        info!("✔ Getting `EthOnEvmEthTxInfos` from receipt...");
        Ok(Self::new(
            Self::get_supported_eth_on_evm_logs_from_receipt(receipt, dictionary)
                .iter()
                .enumerate()
                .map(|(i, log)| {
                    let event_params = EthOnEvmErc777RedeemEvent::from_log(log)?;
                    let tx_info = EthOnEvmEthTxInfo {
                        eth_token_address: log.address,
                        token_amount: event_params.value.clone(),
                        user_data: event_params.user_data.clone(),
                        token_sender: event_params.redeemer.clone(),
                        originating_tx_hash: receipt.transaction_hash,
                        destination_address: event_params.underlying_asset_recipient,
                        evm_token_address: dictionary.get_eth_address_from_evm_address(&log.address)?,
                    };
                    info!("✔ Parsed tx info: {:?}", tx_info);
                    Ok(tx_info)
                })
                .collect::<Result<Vec<EthOnEvmEthTxInfo>>>()?,
        ))
    }

    fn filter_eth_submission_material_for_supported_redeems(
        submission_material: &EvmSubmissionMaterial,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<EvmSubmissionMaterial> {
        info!("✔ Filtering submission material receipts for those pertaining to `ETH-on-EVM` redeems...");
        info!(
            "✔ Num receipts before filtering: {}",
            submission_material.receipts.len()
        );
        let filtered_receipts = EthReceipts::new(
            submission_material
                .receipts
                .iter()
                .filter(|receipt| {
                    EthOnEvmEthTxInfos::receipt_contains_supported_eth_on_evm_redeem(&receipt, dictionary)
                })
                .cloned()
                .collect(),
        );
        info!("✔ Num receipts after filtering: {}", filtered_receipts.len());
        Ok(EvmSubmissionMaterial::new(
            submission_material.get_block()?,
            filtered_receipts,
            None,
            None,
        ))
    }

    pub fn from_submission_material(
        submission_material: &EvmSubmissionMaterial,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<Self> {
        info!("✔ Getting `EthOnEvmEthTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Self::from_eth_receipt(&receipt, dictionary))
                .collect::<Result<Vec<EthOnEvmEthTxInfos>>>()?
                .iter()
                .map(|infos| infos.iter().cloned().collect())
                .collect::<Vec<Vec<EthOnEvmEthTxInfo>>>()
                .concat(),
        ))
    }

    fn to_evm_signed_tx(
        tx_info: &EthOnEvmEthTxInfo,
        nonce: u64,
        chain_id: u8,
        gas_limit: usize,
        gas_price: u64,
        evm_private_key: &EthPrivateKey,
    ) -> Result<EvmTransaction> {
        info!("✔ Signing ETH transaction for tx info: {:?}", tx_info);
        debug!("✔ Signing with nonce:     {}", nonce);
        debug!("✔ Signing with chain id:  {}", chain_id);
        debug!("✔ Signing with gas limit: {}", gas_limit);
        debug!("✔ Signing with gas price: {}", gas_price);
        let operator_data = None;
        encode_erc777_mint_fxn_maybe_with_data(
            &tx_info.destination_address,
            &tx_info.token_amount,
            Some(&tx_info.user_data),
            operator_data,
        )
        .map(|data| {
            EvmTransaction::new_unsigned(
                data,
                nonce,
                ZERO_ETH_VALUE,
                tx_info.evm_token_address,
                chain_id,
                gas_limit,
                gas_price,
            )
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(evm_private_key))
    }

    pub fn to_evm_signed_txs(
        &self,
        start_nonce: u64,
        chain_id: u8,
        gas_limit: usize,
        gas_price: u64,
        evm_private_key: &EthPrivateKey,
    ) -> Result<EvmTransactions> {
        info!("✔ Signing `ETH-on-EVM` ETH transactions...");
        Ok(EvmTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, ref tx_info)| {
                    Self::to_evm_signed_tx(
                        tx_info,
                        start_nonce + i as u64,
                        chain_id,
                        gas_limit,
                        gas_price,
                        evm_private_key,
                    )
                })
                .collect::<Result<Vec<EvmTransaction>>>()?,
        ))
    }
}

pub fn maybe_parse_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: EvmState<D>,
) -> Result<EvmState<D>> {
    info!("✔ Maybe parsing `EthOnEvmEthTxInfos`...");
    get_evm_canon_block_from_db(&state.db).and_then(|submission_material| {
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
                        EthOnEvmEthTxInfos::from_submission_material(&submission_material, &account_names)
                    })
                    .and_then(|tx_infos| state.add_eth_on_evm_eth_tx_infos(tx_infos))
            },
        }
    })
}

pub fn filter_out_zero_value_tx_infos_from_state<D: DatabaseInterface>(state: EvmState<D>) -> Result<EvmState<D>> {
    info!("✔ Maybe filtering out zero value `EthOnEvmEthTxInfos`...");
    debug!(
        "✔ Num `EthOnEvmEthTxInfos` before: {}",
        state.eth_on_evm_eth_signed_txs.len()
    );
    state
        .eth_on_evm_eth_tx_infos
        .filter_out_zero_values()
        .and_then(|filtered_tx_infos| {
            debug!("✔ Num `EthOnEvmEthTxInfos` after: {}", filtered_tx_infos.len());
            state.replace_eth_on_evm_eth_tx_infos(filtered_tx_infos)
        })
}

pub fn filter_submission_material_for_redeem_events_in_state<D: DatabaseInterface>(
    state: EvmState<D>,
) -> Result<EvmState<D>> {
    info!("✔ Filtering receipts for those containing `ETH-on-EVM` redeem events...");
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_addresses_and_with_topics(
            &state.get_eth_evm_token_dictionary()?.to_evm_addresses(),
            &ETH_ON_EVM_REDEEM_EVENT_TOPIC.to_vec(),
        )
        .and_then(|filtered_submission_material| {
            EthOnEvmEthTxInfos::filter_eth_submission_material_for_supported_redeems(
                &filtered_submission_material,
                state.get_eth_evm_token_dictionary()?,
            )
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}

pub fn maybe_sign_eth_txs_and_add_to_evm_state<D: DatabaseInterface>(state: EvmState<D>) -> Result<EvmState<D>> {
    if state.eth_on_evm_eth_tx_infos.is_empty() {
        info!("✔ No tx infos in state ∴ no ETH transactions to sign!");
        Ok(state)
    } else {
        state
            .eth_on_evm_eth_tx_infos
            .to_evm_signed_txs(
                get_eth_account_nonce_from_db(&state.db)?,
                get_eth_chain_id_from_db(&state.db)?,
                ETH_ON_EVM_PEGOUT_WITH_USER_DATA_GAS_LIMIT,
                get_eth_gas_price_from_db(&state.db)?,
                &get_eth_private_key_from_db(&state.db)?,
            )
            .and_then(|signed_txs| {
                #[cfg(feature = "debug")]
                {
                    debug!("✔ Signed transactions: {:?}", signed_txs);
                }
                state.add_eth_on_evm_eth_signed_txs(signed_txs)
            })
    }
}
