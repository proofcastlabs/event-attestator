use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    chains::{
        eth::{
            eth_constants::ZERO_ETH_VALUE,
            eth_contracts::{
                erc20_vault::{Erc20VaultPegInEventParams, ERC20_VAULT_PEG_IN_EVENT_WITH_USER_DATA_TOPIC},
                erc777::{encode_erc777_mint_fxn_maybe_with_data, ERC777_MINT_WITH_DATA_GAS_LIMIT},
            },
            eth_crypto::eth_transaction::{EthTransaction as EvmTransaction, EthTransactions as EvmTransactions},
            eth_database_utils::{get_eth_canon_block_from_db, get_eth_on_evm_smart_contract_address_from_db},
            eth_log::{EthLog, EthLogs},
            eth_receipt::{EthReceipt, EthReceipts},
            eth_state::EthState,
            eth_submission_material::EthSubmissionMaterial,
            eth_utils::safely_convert_hex_to_eth_address,
        },
        evm::{
            eth_crypto::eth_private_key::EthPrivateKey as EvmPrivateKey,
            eth_database_utils::{
                get_eth_account_nonce_from_db as get_evm_account_nonce_from_db,
                get_eth_chain_id_from_db as get_evm_chain_id_from_db,
                get_eth_gas_price_from_db as get_evm_gas_price_from_db,
                get_eth_private_key_from_db as get_evm_private_key_from_db,
            },
        },
    },
    dictionaries::eth_evm::EthEvmTokenDictionary,
    traits::DatabaseInterface,
    types::{Bytes, Result},
};

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

impl EthOnEvmEvmTxInfo {
    pub fn to_evm_signed_tx(
        &self,
        nonce: u64,
        chain_id: u8,
        gas_limit: usize,
        gas_price: u64,
        evm_private_key: &EvmPrivateKey,
    ) -> Result<EvmTransaction> {
        info!("✔ Signing EVM transaction for tx info: {:?}", self);
        debug!("✔ Signing with nonce:     {}", nonce);
        debug!("✔ Signing with chain id:  {}", chain_id);
        debug!("✔ Signing with gas limit: {}", gas_limit);
        debug!("✔ Signing with gas price: {}", gas_price);
        let operator_data = None;
        encode_erc777_mint_fxn_maybe_with_data(
            &self.destination_address,
            &self.token_amount,
            Some(&self.user_data),
            operator_data,
        )
        .map(|data| {
            EvmTransaction::new_unsigned(
                data,
                nonce,
                ZERO_ETH_VALUE,
                self.evm_token_address,
                chain_id,
                gas_limit,
                gas_price,
            )
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(evm_private_key))
    }
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

    fn is_log_eth_on_evm_peg_in(log: &EthLog, vault_address: &EthAddress) -> Result<bool> {
        let log_contains_topic = log.contains_topic(&ERC20_VAULT_PEG_IN_EVENT_WITH_USER_DATA_TOPIC);
        let log_is_from_vault_address = &log.address == vault_address;
        Ok(log_contains_topic && log_is_from_vault_address)
    }

    fn receipt_contains_supported_eth_on_evm_peg_in(receipt: &EthReceipt, vault_address: &EthAddress) -> bool {
        Self::get_supported_eth_on_evm_logs_from_receipt(receipt, vault_address).len() > 0
    }

    fn get_supported_eth_on_evm_logs_from_receipt(receipt: &EthReceipt, vault_address: &EthAddress) -> EthLogs {
        EthLogs::new(
            receipt
                .logs
                .iter()
                .filter(|log| matches!(Self::is_log_eth_on_evm_peg_in(&log, vault_address), Ok(true)))
                .cloned()
                .collect(),
        )
    }

    fn from_eth_receipt(
        receipt: &EthReceipt,
        vault_address: &EthAddress,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<Self> {
        info!("✔ Getting `ETH-on-EVM` peg in infos from receipt...");
        Ok(Self::new(
            Self::get_supported_eth_on_evm_logs_from_receipt(receipt, vault_address)
                .iter()
                .map(|log| {
                    let event_params = Erc20VaultPegInEventParams::from_eth_log(log)?;
                    let tx_info = EthOnEvmEvmTxInfo {
                        user_data: event_params.user_data.clone(),
                        eth_token_address: event_params.token_address,
                        originating_tx_hash: receipt.transaction_hash,
                        token_amount: event_params.token_amount,
                        token_sender: event_params.token_sender,
                        destination_address: safely_convert_hex_to_eth_address(&event_params.destination_address)?,
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
        vault_address: &EthAddress,
    ) -> Result<EthSubmissionMaterial> {
        info!("✔ Filtering submission material receipts for those pertaining to `ETH-on-EVM` peg-ins...");
        info!(
            "✔ Num receipts before filtering: {}",
            submission_material.receipts.len()
        );
        let filtered_receipts = EthReceipts::new(
            submission_material
                .receipts
                .iter()
                .filter(|receipt| {
                    EthOnEvmEvmTxInfos::receipt_contains_supported_eth_on_evm_peg_in(&receipt, vault_address)
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
        vault_address: &EthAddress,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<Self> {
        info!("✔ Getting `EthOnEvmEvmTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Self::from_eth_receipt(&receipt, vault_address, dictionary))
                .collect::<Result<Vec<EthOnEvmEvmTxInfos>>>()?
                .iter()
                .map(|infos| infos.iter().cloned().collect())
                .collect::<Vec<Vec<EthOnEvmEvmTxInfo>>>()
                .concat(),
        ))
    }

    pub fn to_evm_signed_txs(
        &self,
        start_nonce: u64,
        chain_id: u8,
        gas_limit: usize,
        gas_price: u64,
        evm_private_key: &EvmPrivateKey,
    ) -> Result<EvmTransactions> {
        info!("✔ Signing `ETH-on-EVM` EVM transactions...");
        Ok(EvmTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, ref tx_info)| {
                    EthOnEvmEvmTxInfo::to_evm_signed_tx(
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
                EthOnEvmEvmTxInfos::from_submission_material(
                    &submission_material,
                    &get_eth_on_evm_smart_contract_address_from_db(&state.db)?,
                    &EthEvmTokenDictionary::get_from_db(&state.db)?,
                )
                .and_then(|tx_infos| state.add_eth_on_evm_evm_tx_infos(tx_infos))
            },
        }
    })
}

pub fn filter_out_zero_value_tx_infos_from_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Maybe filtering out zero value `EthOnEvmEvmTxInfos`...");
    debug!(
        "✔ Num `EthOnEvmEvmTxInfos` before: {}",
        state.eth_on_evm_evm_signed_txs.len()
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
    let vault_address = get_eth_on_evm_smart_contract_address_from_db(&state.db)?;
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_address_and_with_topics(&vault_address, &[
            *ERC20_VAULT_PEG_IN_EVENT_WITH_USER_DATA_TOPIC,
        ])
        .and_then(|filtered_submission_material| {
            EthOnEvmEvmTxInfos::filter_eth_submission_material_for_supported_peg_ins(
                &filtered_submission_material,
                &vault_address,
            )
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}

pub fn maybe_sign_evm_txs_and_add_to_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.eth_on_evm_evm_tx_infos.is_empty() {
        info!("✔ No tx infos in state ∴ no EVM transactions to sign!");
        Ok(state)
    } else {
        state
            .eth_on_evm_evm_tx_infos
            .to_evm_signed_txs(
                get_evm_account_nonce_from_db(&state.db)?,
                get_evm_chain_id_from_db(&state.db)?,
                ERC777_MINT_WITH_DATA_GAS_LIMIT,
                get_evm_gas_price_from_db(&state.db)?,
                &get_evm_private_key_from_db(&state.db)?,
            )
            .and_then(|signed_txs| {
                #[cfg(feature = "debug")]
                {
                    debug!("✔ Signed transactions: {:?}", signed_txs);
                }
                state.add_eth_on_evm_evm_signed_txs(signed_txs)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::eth_traits::EthTxInfoCompatible,
        eth_on_evm::test_utils::{
            get_eth_submission_material_n,
            get_sample_eth_evm_token_dictionary,
            get_sample_evm_private_key,
            get_sample_vault_address,
        },
    };

    #[test]
    fn should_filter_submission_info_for_supported_redeems() {
        let material = get_eth_submission_material_n(1);
        let vault_address = get_sample_vault_address();
        let result =
            EthOnEvmEvmTxInfos::filter_eth_submission_material_for_supported_peg_ins(&material, &vault_address)
                .unwrap();
        let expected_num_receipts = 1;
        assert_eq!(result.receipts.len(), expected_num_receipts);
    }

    #[test]
    fn should_get_eth_on_evm_evm_tx_info_from_submission_material() {
        let material = get_eth_submission_material_n(1);
        let vault_address = get_sample_vault_address();
        let dictionary = get_sample_eth_evm_token_dictionary();
        let result = EthOnEvmEvmTxInfos::from_submission_material(&material, &vault_address, &dictionary).unwrap();
        let expected_num_results = 1;
        assert_eq!(result.len(), expected_num_results);
        let expected_result = EthOnEvmEvmTxInfos::new(vec![EthOnEvmEvmTxInfo {
            user_data: vec![],
            token_amount: U256::from_dec_str("1000000000000000000").unwrap(),
            token_sender: EthAddress::from_slice(&hex::decode("8127192c2e4703dfb47f087883cc3120fe061cb8").unwrap()),
            evm_token_address: EthAddress::from_slice(
                &hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap(),
            ),
            eth_token_address: EthAddress::from_slice(
                &hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap(),
            ),
            // NOTE It's the `SAFE_ETH_ADDRESS_HEX` ∵ @bertani accidentally included the `"`s in the pegin!
            destination_address: EthAddress::from_slice(
                &hex::decode("71a440ee9fa7f99fb9a697e96ec7839b8a1643b8").unwrap(),
            ),
            originating_tx_hash: EthHash::from_slice(
                &hex::decode("578670d0e08ca172eb8e862352e731814564fd6a12c3143e88bfb28292cd1535").unwrap(),
            ),
        }]);
        assert_eq!(result, expected_result);
    }

    // FIXME / TODO test one without the safe eth address!

    #[test]
    fn should_get_signaures_from_evm_tx_info() {
        let material = get_eth_submission_material_n(1);
        let pk = get_sample_evm_private_key();
        let vault_address = get_sample_vault_address();
        let dictionary = get_sample_eth_evm_token_dictionary();
        let infos = EthOnEvmEvmTxInfos::from_submission_material(&material, &vault_address, &dictionary).unwrap();
        let nonce = 0_u64;
        let chain_id = 4_u8;
        let gas_limit = 300_000_usize;
        let gas_price = 20_000_000_000_u64;
        let signed_txs = infos
            .to_evm_signed_txs(nonce, chain_id, gas_limit, gas_price, &pk)
            .unwrap();
        let expected_num_results = 1;
        assert_eq!(signed_txs.len(), expected_num_results);
        let tx_hex = signed_txs[0].eth_tx_hex().unwrap();
        let expected_tx_hex =
"f9012a808504a817c800830493e094daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af9280b8c4dcdc7dd000000000000000000000000071a440ee9fa7f99fb9a697e96ec7839b8a1643b80000000000000000000000000000000000000000000000000de0b6b3a7640000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002ba0e692dd01449b9d70b4f6a98f07ea2ebab8de8f92a55f45dc92dae4e0cd962a0ba0113b961254c8a47f9b318157a6ffb172223093884e7c807ca8e92f9bc143464a"
;
        assert_eq!(tx_hex, expected_tx_hex);
    }
}
