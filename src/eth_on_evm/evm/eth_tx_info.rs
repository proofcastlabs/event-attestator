use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    chains::{
        eth::{
            eth_constants::{ETH_ON_EVM_REDEEM_EVENT_TOPIC, ETH_ON_EVM_REDEEM_EVENT_TOPIC_HEX, ZERO_ETH_VALUE},
            eth_contracts::{
                erc777::Erc777RedeemEvent,
                erc20_vault::{encode_erc20_vault_peg_out_fxn_data, ERC20_VAULT_PEGOUT_WITH_USER_DATA_GAS_LIMIT},
            },
            eth_crypto::{
                eth_private_key::EthPrivateKey,
                eth_transaction::{EthTransaction as EvmTransaction, EthTransactions as EvmTransactions},
            },
            eth_database_utils::{
                get_eth_account_nonce_from_db,
                get_eth_chain_id_from_db,
                get_eth_gas_price_from_db,
                get_eth_on_evm_smart_contract_address_from_db,
                get_eth_private_key_from_db,
            },
            eth_utils::safely_convert_hex_to_eth_address,
        },
        evm::{
            eth_database_utils::get_eth_canon_block_from_db as get_evm_canon_block_from_db,
            eth_log::{EthLog, EthLogs},
            eth_receipt::{EthReceipt, EthReceipts},
            eth_state::EthState as EvmState,
            eth_submission_material::EthSubmissionMaterial as EvmSubmissionMaterial,
        },
    },
    dictionaries::eth_evm::EthEvmTokenDictionary,
    traits::DatabaseInterface,
    types::{Bytes, Result},
};

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

impl EthOnEvmEthTxInfo {
    pub fn to_eth_signed_tx(
        &self,
        nonce: u64,
        chain_id: u8,
        gas_limit: usize,
        gas_price: u64,
        evm_private_key: &EthPrivateKey,
        vault_address: &EthAddress,
    ) -> Result<EvmTransaction> {
        info!("✔ Signing ETH transaction for tx info: {:?}", self);
        debug!("✔ Signing with nonce:     {}", nonce);
        debug!("✔ Signing with chain id:  {}", chain_id);
        debug!("✔ Signing with gas limit: {}", gas_limit);
        debug!("✔ Signing with gas price: {}", gas_price);
        debug!(
            "✔ Signing tx to token recipient: {}",
            self.destination_address.to_string()
        );
        debug!(
            "✔ Signing tx for token address : {}",
            self.eth_token_address.to_string()
        );
        debug!("✔ Signing tx for token amount: {}", self.token_amount.to_string());
        debug!("✔ Signing tx for vault address: {}", vault_address.to_string());
        encode_erc20_vault_peg_out_fxn_data(self.destination_address, self.eth_token_address, self.token_amount)
            .map(|data| {
                EvmTransaction::new_unsigned(
                    data,
                    nonce,
                    ZERO_ETH_VALUE,
                    *vault_address,
                    chain_id,
                    gas_limit,
                    gas_price,
                )
            })
            .and_then(|unsigned_tx| unsigned_tx.sign(evm_private_key))
    }
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

    fn is_log_eth_on_evm_redeem(log: &EthLog, dictionary: &EthEvmTokenDictionary) -> Result<bool> {
        debug!("✔ Checking log contains topic: {}", ETH_ON_EVM_REDEEM_EVENT_TOPIC_HEX);
        let token_is_supported = dictionary.is_evm_token_supported(&log.address);
        let log_contains_topic = log.contains_topic(&EthHash::from_slice(
            &hex::decode(&ETH_ON_EVM_REDEEM_EVENT_TOPIC_HEX)?[..],
        ));
        debug!("✔ Log is supported: {}", token_is_supported);
        debug!("✔ Log has correct topic: {}", log_contains_topic);
        Ok(token_is_supported && log_contains_topic)
    }

    pub fn is_log_supported_eth_on_evm_redeem(log: &EthLog, dictionary: &EthEvmTokenDictionary) -> Result<bool> {
        match Self::is_log_eth_on_evm_redeem(log, dictionary)? {
            false => Ok(false),
            true => Ok(dictionary.is_evm_token_supported(&log.address)),
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
                .map(|log| {
                    let event_params = Erc777RedeemEvent::from_eth_log(log)?;
                    let tx_info = EthOnEvmEthTxInfo {
                        evm_token_address: log.address,
                        token_amount: event_params.value,
                        token_sender: event_params.redeemer,
                        user_data: event_params.user_data.clone(),
                        originating_tx_hash: receipt.transaction_hash,
                        eth_token_address: dictionary.get_eth_address_from_evm_address(&log.address)?,
                        destination_address: safely_convert_hex_to_eth_address(
                            &event_params.underlying_asset_recipient,
                        )?,
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

    pub fn to_eth_signed_txs(
        &self,
        start_nonce: u64,
        chain_id: u8,
        gas_limit: usize,
        gas_price: u64,
        evm_private_key: &EthPrivateKey,
        vault_address: &EthAddress,
    ) -> Result<EvmTransactions> {
        info!("✔ Signing `ETH-on-EVM` ETH transactions...");
        Ok(EvmTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, ref tx_info)| {
                    EthOnEvmEthTxInfo::to_eth_signed_tx(
                        tx_info,
                        start_nonce + i as u64,
                        chain_id,
                        gas_limit,
                        gas_price,
                        evm_private_key,
                        vault_address,
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
            .to_eth_signed_txs(
                get_eth_account_nonce_from_db(&state.db)?,
                get_eth_chain_id_from_db(&state.db)?,
                ERC20_VAULT_PEGOUT_WITH_USER_DATA_GAS_LIMIT,
                get_eth_gas_price_from_db(&state.db)?,
                &get_eth_private_key_from_db(&state.db)?,
                &get_eth_on_evm_smart_contract_address_from_db(&state.db)?,
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

// FIXME Re-instate once we have new sample material
// #[cfg(test)]
// mod tests {
// use super::*;
// use crate::{
// chains::eth::eth_traits::EthTxInfoCompatible,
// eth_on_evm::test_utils::{
// get_evm_submission_material_n,
// get_sample_eth_evm_token_dictionary,
// get_sample_eth_private_key,
// },
// };
//
// #[test]
// fn should_filter_submission_info_for_supported_redeems() {
// let dictionary = get_sample_eth_evm_token_dictionary();
// let material = get_evm_submission_material_n(1);
// let result =
// EthOnEvmEthTxInfos::filter_eth_submission_material_for_supported_redeems(&material, &dictionary).unwrap();
// let expected_num_receipts = 1;
// assert_eq!(result.receipts.len(), expected_num_receipts);
// }
//
// #[test]
// fn should_get_eth_on_evm_eth_tx_info_from_submission_material() {
// let dictionary = get_sample_eth_evm_token_dictionary();
// let material = get_evm_submission_material_n(1);
// let result = EthOnEvmEthTxInfos::from_submission_material(&material, &dictionary).unwrap();
// let expected_num_results = 1;
// assert_eq!(result.len(), expected_num_results);
// let expected_result = EthOnEvmEthTxInfos::new(vec![EthOnEvmEthTxInfo {
// user_data: vec![0xde, 0xca, 0xff],
// token_amount: U256::from_dec_str("666").unwrap(),
// token_sender: EthAddress::from_slice(&hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap()),
// evm_token_address: EthAddress::from_slice(
// &hex::decode("6819bbfdf803b8b87850916d3eeb3642dde6c24f").unwrap(),
// ),
// eth_token_address: EthAddress::from_slice(
// &hex::decode("bf6ab900f1a3d8f94bc98f1d2ba1b8f00d532078").unwrap(),
// ),
// destination_address: EthAddress::from_slice(
// &hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap(),
// ),
// originating_tx_hash: EthHash::from_slice(
// &hex::decode("27e094774335279a8ba2ca2f8675071cf5e3c09a9d80ce11b3f5ea49c806d268").unwrap(),
// ),
// }]);
// assert_eq!(result, expected_result);
// }
//
// #[test]
// fn should_get_signaures_from_eth_tx_info() {
// let dictionary = get_sample_eth_evm_token_dictionary();
// let material = get_evm_submission_material_n(1);
// let infos = EthOnEvmEthTxInfos::from_submission_material(&material, &dictionary).unwrap();
// let vault_address = EthAddress::from_slice(&hex::decode("e72479bf662ca5047301f80733cb1c5c23a338af").unwrap());
// let pk = get_sample_eth_private_key();
// let nonce = 0_u64;
// let chain_id = 4_u8;
// let gas_limit = 300_000_usize;
// let gas_price = 20_000_000_000_u64;
// let signed_txs = infos
// .to_eth_signed_txs(nonce, chain_id, gas_limit, gas_price, &pk, &vault_address)
// .unwrap();
// let expected_num_results = 1;
// assert_eq!(signed_txs.len(), expected_num_results);
// let tx_hex = signed_txs[0].eth_tx_hex().unwrap();
// let expected_tx_hex =
// "f8ca808504a817c800830493e094e72479bf662ca5047301f80733cb1c5c23a338af80b86483c09d42000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac000000000000000000000000bf6ab900f1a3d8f94bc98f1d2ba1b8f00d532078000000000000000000000000000000000000000000000000000000000000029a2ca0031c56126894d95ef87d515048877b3e3e213737506965d7dea74854fc934930a066eb76208b12e2b872e559d077fd7fa0aaa94ec501214b60fd454d4a7d4b418d"
// ; assert_eq!(tx_hex, expected_tx_hex);
// }
// }
