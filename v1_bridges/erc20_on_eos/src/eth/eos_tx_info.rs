use std::str::FromStr;

use common::{
    chains::{
        eos::{
            eos_chain_id::EosChainId,
            eos_crypto::{
                eos_private_key::EosPrivateKey,
                eos_transaction::{get_signed_eos_ptoken_issue_tx, EosSignedTransaction, EosSignedTransactions},
            },
            eos_utils::{get_eos_tx_expiration_timestamp_with_offset, remove_symbol_from_eos_asset},
        },
        eth::{
            eth_chain_id::EthChainId,
            eth_contracts::erc20_vault::{
                Erc20VaultPegInEventParams,
                ERC20_VAULT_PEG_IN_EVENT_WITHOUT_USER_DATA_TOPIC,
                ERC20_VAULT_PEG_IN_EVENT_WITH_USER_DATA_TOPIC,
            },
            eth_database_utils::EthDbUtilsExt,
            eth_log::{EthLog, EthLogExt, EthLogs},
            eth_receipt::{EthReceipt, EthReceipts},
            eth_submission_material::EthSubmissionMaterial,
            EthState,
        },
    },
    dictionaries::eos_eth::EosEthTokenDictionary,
    metadata::{
        metadata_address::MetadataAddress,
        metadata_protocol_id::MetadataProtocolId,
        metadata_traits::{ToMetadata, ToMetadataChainId},
        Metadata,
    },
    safe_addresses::safely_convert_str_to_eos_address,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
};
use derive_more::{Constructor, Deref};
use eos_chain::AccountName as EosAccountName;
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

use crate::fees_calculator::{FeeCalculator, FeesCalculator};

#[cfg_attr(test, derive(Constructor))]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Erc20OnEosEosTxInfo {
    pub token_amount: U256,
    pub token_sender: EthAddress,
    pub eth_token_address: EthAddress,
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub eos_token_address: String,
    pub eos_asset_amount: String,
    pub user_data: Bytes,
    pub origin_chain_id: EthChainId,
}

impl ToMetadata for Erc20OnEosEosTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        Ok(Metadata::new(
            &self.user_data,
            &MetadataAddress::new_from_eth_address(&self.token_sender, &self.origin_chain_id.to_metadata_chain_id())?,
        ))
    }

    fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Eos)
    }
}

impl FeesCalculator for Erc20OnEosEosTxInfos {
    fn get_fees(&self, dictionary: &EosEthTokenDictionary) -> Result<Vec<(EthAddress, U256)>> {
        debug!("Calculating fees in `Erc20OnEosEosTxInfos`...");
        self.iter()
            .map(|info| info.calculate_peg_in_fee_via_dictionary(dictionary))
            .collect()
    }

    fn subtract_fees(&self, dictionary: &EosEthTokenDictionary) -> Result<Self> {
        self.get_fees(dictionary).and_then(|fee_tuples| {
            Ok(Self::new(
                self.iter()
                    .zip(fee_tuples.iter())
                    .map(|(info, (_, fee))| {
                        if fee.is_zero() {
                            debug!("Not subtracting fee because `fee` is 0!");
                            Ok(info.clone())
                        } else {
                            info.subtract_amount(*fee)
                        }
                    })
                    .collect::<Result<Vec<_>>>()?,
            ))
        })
    }
}

impl FeeCalculator for Erc20OnEosEosTxInfo {
    fn get_amount(&self) -> U256 {
        info!(
            "✔ Getting token amount in `Erc20OnEosEosTxInfo` of {}",
            self.token_amount
        );
        self.token_amount
    }

    fn get_eth_token_address(&self) -> EthAddress {
        debug!(
            "Getting EOS token address in `Erc20OnEosEosTxInfo` of {}",
            self.eth_token_address
        );
        self.eth_token_address
    }

    fn get_eos_token_address(&self) -> Result<EosAccountName> {
        debug!(
            "Getting EOS token address in `Erc20OnEosEosTxInfo` of {}",
            self.eos_token_address
        );
        Ok(EosAccountName::from_str(&self.eos_token_address)?)
    }

    fn subtract_amount(&self, subtrahend: U256) -> Result<Self> {
        if subtrahend >= self.token_amount {
            Err("Cannot subtract amount from `Erc20OnEosEosTxInfo`: subtrahend too large!".into())
        } else {
            let new_amount = self.token_amount - subtrahend;
            debug!(
                "Subtracting {} from {} to get final amount of {} in `Erc20OnEosEosTxInfo`!",
                subtrahend, self.token_amount, new_amount
            );
            let mut new_self = self.clone();
            new_self.token_amount = new_amount;
            Ok(new_self)
        }
    }
}

impl Erc20OnEosEosTxInfo {
    pub fn is_zero_eos_amount(&self, dictionary: &EosEthTokenDictionary) -> Result<bool> {
        let entry = dictionary.get_entry_via_eos_address(&EosAccountName::from_str(&self.eos_token_address)?)?;
        let eos_amount = remove_symbol_from_eos_asset(&entry.convert_u256_to_eos_asset_string(&self.token_amount)?)
            .parse::<f64>()?;
        Ok(eos_amount == 0.0)
    }

    pub fn to_eos_signed_tx(
        &self,
        ref_block_num: u16,
        ref_block_prefix: u32,
        chain_id: &EosChainId,
        private_key: &EosPrivateKey,
        timestamp: u32,
        dictionary: &EosEthTokenDictionary,
    ) -> Result<EosSignedTransaction> {
        info!("✔ Signing EOS tx from `Erc20OnEosEosTxInfo`: {:?}", self);
        let dictionary_entry =
            dictionary.get_entry_via_eos_address(&EosAccountName::from_str(&self.eos_token_address)?)?;
        let eos_amount = dictionary_entry.convert_u256_to_eos_asset_string(&self.token_amount)?;
        get_signed_eos_ptoken_issue_tx(
            ref_block_num,
            ref_block_prefix,
            &self.destination_address,
            &eos_amount,
            chain_id,
            private_key,
            &self.eos_token_address,
            timestamp,
            if self.user_data.is_empty() {
                None
            } else {
                info!("✔ Wrapping `user_data` in metadata for `Erc20OnEosEosTxInfo¬");
                Some(
                    self.to_metadata()?
                        .to_bytes_for_protocol(&chain_id.to_metadata_chain_id().to_protocol_id())?,
                )
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq, Default, Eq, Constructor, Deref, Serialize, Deserialize)]
pub struct Erc20OnEosEosTxInfos(pub Vec<Erc20OnEosEosTxInfo>);

impl Erc20OnEosEosTxInfos {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        if bytes.is_empty() {
            Ok(Self::default())
        } else {
            Ok(serde_json::from_slice(bytes)?)
        }
    }

    pub fn to_eos_signed_txs(
        &self,
        ref_block_num: u16,
        ref_block_prefix: u32,
        chain_id: &EosChainId,
        private_key: &EosPrivateKey,
        dictionary: &EosEthTokenDictionary,
    ) -> Result<EosSignedTransactions> {
        info!("✔ Signing {} EOS txs from `erc20-on-eos` peg in infos...", self.len());
        Ok(EosSignedTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, info)| {
                    info.to_eos_signed_tx(
                        ref_block_num,
                        ref_block_prefix,
                        chain_id,
                        private_key,
                        get_eos_tx_expiration_timestamp_with_offset(i as u32)?,
                        dictionary,
                    )
                })
                .collect::<Result<Vec<EosSignedTransaction>>>()?,
        ))
    }

    pub fn filter_out_zero_eos_values(&self, dictionary: &EosEthTokenDictionary) -> Result<Self> {
        Ok(Self::new(
            self.iter()
                .map(|eos_tx_info| {
                    let is_zero_eos_amount = eos_tx_info.is_zero_eos_amount(dictionary)?;
                    Ok((eos_tx_info.clone(), is_zero_eos_amount))
                })
                .collect::<Result<Vec<(Erc20OnEosEosTxInfo, bool)>>>()?
                .iter()
                .filter_map(|(eos_tx_info, is_zero_amount)| {
                    if *is_zero_amount {
                        info!(
                            "✘ Filtering out peg in info due to zero EOS asset amount: {:?}",
                            eos_tx_info
                        );
                        None
                    } else {
                        Some(eos_tx_info)
                    }
                })
                .cloned()
                .collect::<Vec<Erc20OnEosEosTxInfo>>(),
        ))
    }

    fn is_log_erc20_peg_in(log: &EthLog) -> Result<bool> {
        Ok(log.contains_topic(&ERC20_VAULT_PEG_IN_EVENT_WITHOUT_USER_DATA_TOPIC)
            || log.contains_topic(&ERC20_VAULT_PEG_IN_EVENT_WITH_USER_DATA_TOPIC))
    }

    pub fn is_log_supported_erc20_peg_in(log: &EthLog, token_dictionary: &EosEthTokenDictionary) -> Result<bool> {
        match Self::is_log_erc20_peg_in(log)? {
            false => Ok(false),
            true => Erc20VaultPegInEventParams::from_eth_log(log)
                .map(|params| token_dictionary.is_token_supported(&params.token_address)),
        }
    }

    fn receipt_contains_supported_erc20_peg_in(receipt: &EthReceipt, token_dictionary: &EosEthTokenDictionary) -> bool {
        Self::get_supported_erc20_peg_in_logs_from_receipt(receipt, token_dictionary).len() > 0
    }

    fn get_supported_erc20_peg_in_logs_from_receipt(
        receipt: &EthReceipt,
        token_dictionary: &EosEthTokenDictionary,
    ) -> EthLogs {
        EthLogs::new(
            receipt
                .logs
                .iter()
                .filter(|log| matches!(Self::is_log_supported_erc20_peg_in(log, token_dictionary), Ok(true)))
                .cloned()
                .collect(),
        )
    }

    fn from_eth_receipt(
        receipt: &EthReceipt,
        token_dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EthChainId,
    ) -> Result<Self> {
        info!("✔ Getting `erc20-on-eos` peg in infos from receipt...");
        Ok(Self::new(
            Self::get_supported_erc20_peg_in_logs_from_receipt(receipt, token_dictionary)
                .iter()
                .map(|log| {
                    let params = Erc20VaultPegInEventParams::from_eth_log(log)?;
                    let eos_tx_info = Erc20OnEosEosTxInfo {
                        token_sender: params.token_sender,
                        originating_tx_hash: receipt.transaction_hash,
                        destination_address: safely_convert_str_to_eos_address(&params.destination_address).to_string(),
                        eos_token_address: token_dictionary
                            .get_eos_account_name_from_eth_token_address(&params.token_address)?,
                        eos_asset_amount: token_dictionary
                            .convert_u256_to_eos_asset_string(&params.token_address, &params.token_amount)?,
                        eth_token_address: params.token_address,
                        token_amount: params.token_amount,
                        user_data: params.user_data,
                        origin_chain_id: origin_chain_id.clone(),
                    };
                    info!("✔ Parsed peg in info: {:?}", eos_tx_info);
                    Ok(eos_tx_info)
                })
                .collect::<Result<Vec<Erc20OnEosEosTxInfo>>>()?,
        ))
    }

    fn filter_eth_sub_mat_for_supported_eos_tx_infos(
        submission_material: &EthSubmissionMaterial,
        token_dictionary: &EosEthTokenDictionary,
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
                    Erc20OnEosEosTxInfos::receipt_contains_supported_erc20_peg_in(receipt, token_dictionary)
                })
                .cloned()
                .collect(),
        );
        info!("✔ Num receipts after filtering: {}", filtered_receipts.len());
        Ok(EthSubmissionMaterial::new(
            submission_material.get_block()?,
            filtered_receipts,
            submission_material.eos_ref_block_num,
            submission_material.eos_ref_block_prefix,
        ))
    }

    pub fn from_submission_material(
        submission_material: &EthSubmissionMaterial,
        eos_eth_token_dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EthChainId,
    ) -> Result<Self> {
        info!("✔ Getting `Erc20OnEosEosTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Self::from_eth_receipt(receipt, eos_eth_token_dictionary, origin_chain_id))
                .collect::<Result<Vec<Erc20OnEosEosTxInfos>>>()?
                .iter()
                .map(|infos| infos.iter().cloned().collect())
                .collect::<Vec<Vec<Erc20OnEosEosTxInfo>>>()
                .concat(),
        ))
    }
}

pub fn maybe_parse_eos_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `erc20-on-eos` peg-in infos...");
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
                EosEthTokenDictionary::get_from_db(state.db)
                    .and_then(|account_names| {
                        Erc20OnEosEosTxInfos::from_submission_material(
                            &submission_material,
                            &account_names,
                            &state.eth_db_utils.get_eth_chain_id_from_db()?,
                        )
                    })
                    .and_then(|infos| infos.to_bytes())
                    .map(|bytes| state.add_tx_infos(bytes))
                    .and_then(filter_out_zero_value_eos_tx_infos_from_state)
            }
        })
}

pub fn filter_out_zero_value_eos_tx_infos_from_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ NOT filtering `erc20-on-eos` peg-in infos because there are none!");
        Ok(state)
    } else {
        info!("✔ Maybe filtering `erc20-on-eos` peg-in infos...");
        Erc20OnEosEosTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| {
                debug!("✔ Num peg-in infos before: {}", infos.len());
                infos.filter_out_zero_eos_values(&EosEthTokenDictionary::get_from_db(state.db)?)
            })
            .and_then(|infos| {
                debug!("✔ Num peg-in infos after: {}", infos.len());
                infos.to_bytes()
            })
            .map(|bytes| state.add_tx_infos(bytes))
    }
}

pub fn filter_submission_material_for_peg_in_events_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing `erc20-on-eos` peg in events...");
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_address_and_with_topics(
            &state.eth_db_utils.get_erc20_on_eos_smart_contract_address_from_db()?,
            &[
                *ERC20_VAULT_PEG_IN_EVENT_WITHOUT_USER_DATA_TOPIC,
                *ERC20_VAULT_PEG_IN_EVENT_WITH_USER_DATA_TOPIC,
            ],
        )
        .and_then(|filtered_submission_material| {
            Erc20OnEosEosTxInfos::filter_eth_sub_mat_for_supported_eos_tx_infos(
                &filtered_submission_material,
                state.get_eos_eth_token_dictionary()?,
            )
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}

pub fn maybe_sign_eos_txs_and_add_to_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ NOT Signing `erc20-on-eos` peg in txs because there are none!");
        Ok(state)
    } else {
        info!("✔ Signing `erc20-on-eos` peg in txs...");
        let submission_material = state.get_eth_submission_material()?;
        Erc20OnEosEosTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| {
                infos.to_eos_signed_txs(
                    submission_material.get_eos_ref_block_num()?,
                    submission_material.get_eos_ref_block_prefix()?,
                    &state.eos_db_utils.get_eos_chain_id_from_db()?,
                    &EosPrivateKey::get_from_db(state.db)?,
                    &EosEthTokenDictionary::get_from_db(state.db)?,
                )
            })
            .and_then(|signed_txs| state.add_eos_transactions(signed_txs))
    }
}

#[cfg(test)]
mod tests {
    use common::{
        chains::eos::eos_test_utils::get_sample_eos_private_key,
        dictionaries::eos_eth::{test_utils::get_sample_eos_eth_token_dictionary, EosEthTokenDictionaryEntry},
    };
    use serde_json::json;

    use super::*;
    use crate::test_utils::{
        get_sample_eos_eth_dictionary,
        get_sample_erc20_on_eos_eos_tx_info,
        get_sample_erc20_on_eos_eos_tx_infos,
        get_sample_erc20_on_eos_eos_tx_infos_2,
        get_sample_log_with_erc20_peg_in_event,
        get_sample_log_with_erc20_peg_in_event_2,
        get_sample_receipt_with_erc20_peg_in_event,
        get_sample_submission_material_with_erc20_peg_in_event,
    };

    fn get_sample_zero_eos_asset_eos_tx_info() -> Erc20OnEosEosTxInfo {
        let user_data = vec![0xde, 0xca, 0xff];
        Erc20OnEosEosTxInfo::new(
            U256::from_dec_str("1337").unwrap(),
            EthAddress::from_slice(&hex::decode("edB86cd455ef3ca43f0e227e00469C3bDFA40628").unwrap()),
            EthAddress::from_slice(&hex::decode("d879D3C8782aB95a43C69Fa73d8DCC50C8815d5e").unwrap()),
            "aneosaddress".to_string(),
            EthHash::from_slice(
                &hex::decode("7b7f73183fe4d1d6e23c494ba0b579718c7dd6e1c62426fd5411a6a21b3203aa").unwrap(),
            ),
            "aneosaccount".to_string(),
            "0.00000000 SAM".to_string(),
            user_data,
            EthChainId::Mainnet,
        )
    }

    #[test]
    fn should_filter_out_zero_eos_asset_eos_tx_infos() {
        let expected_num_eos_tx_infos_before = 1;
        let expected_num_eos_tx_infos_after = 0;
        let eos_tx_infos = Erc20OnEosEosTxInfos::new(vec![get_sample_zero_eos_asset_eos_tx_info()]);
        assert_eq!(eos_tx_infos.len(), expected_num_eos_tx_infos_before);
        let dictionary = EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::from_str(
            &json!({
                "eth_token_decimals": 18,
                "eos_token_decimals": 8,
                "eth_symbol": "TOK",
                "eos_symbol": "TOK",
                "eth_address": "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "eos_address": "aneosaccount",
            })
            .to_string(),
        )
        .unwrap()]);
        let result = eos_tx_infos.filter_out_zero_eos_values(&dictionary).unwrap();
        assert_eq!(result.len(), expected_num_eos_tx_infos_after);
    }

    #[test]
    fn log_is_supported_erc20_peg_in_should_be_true_if_supported() {
        let eth_token_decimals = 18;
        let eos_token_decimals = 9;
        let eth_symbol = "SAM".to_string();
        let eos_symbol = "SYM".to_string();
        let token_name = "SampleToken".to_string();
        let token_address = EthAddress::from_slice(&hex::decode("9f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let eth_basis_points = 0;
        let eos_basis_points = 0;
        let token_dictionary = EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::new(
            eth_token_decimals,
            eos_token_decimals,
            eth_symbol,
            eos_symbol,
            token_name,
            token_address,
            eth_basis_points,
            eos_basis_points,
            U256::zero(),
            0,
            0,
            "".to_string(),
        )]);
        let log = get_sample_log_with_erc20_peg_in_event();
        let result = Erc20OnEosEosTxInfos::is_log_supported_erc20_peg_in(&log, &token_dictionary).unwrap();
        assert!(result);
    }

    #[test]
    fn log_is_supported_erc20_peg_in_should_be_false_if_not_supported() {
        let eth_token_decimals = 18;
        let eos_token_decimals = 9;
        let eth_symbol = "SAM".to_string();
        let eos_symbol = "SYM".to_string();
        let token_name = "SampleToken".to_string();
        let token_address = EthAddress::from_slice(&hex::decode("8f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let eth_basis_points = 0;
        let eos_basis_points = 0;
        let token_dictionary = EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::new(
            eth_token_decimals,
            eos_token_decimals,
            eth_symbol,
            eos_symbol,
            token_name,
            token_address,
            eth_basis_points,
            eos_basis_points,
            U256::zero(),
            0,
            0,
            "".to_string(),
        )]);
        let log = get_sample_log_with_erc20_peg_in_event();
        let result = Erc20OnEosEosTxInfos::is_log_supported_erc20_peg_in(&log, &token_dictionary).unwrap();
        assert!(!result);
    }

    #[test]
    fn log_is_supported_erc20_peg_in_2_should_be_true_if_supported() {
        let eth_token_decimals = 18;
        let eos_token_decimals = 9;
        let eth_symbol = "SAM".to_string();
        let eos_symbol = "SYM".to_string();
        let token_name = "SampleToken".to_string();
        let token_address = EthAddress::from_slice(
            &hex::decode("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(), // NOTE wETH address on mainnet!
        );
        let eth_basis_points = 0;
        let eos_basis_points = 0;
        let token_dictionary = EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::new(
            eth_token_decimals,
            eos_token_decimals,
            eth_symbol,
            eos_symbol,
            token_name,
            token_address,
            eth_basis_points,
            eos_basis_points,
            U256::zero(),
            0,
            0,
            "".to_string(),
        )]);
        let log = get_sample_log_with_erc20_peg_in_event_2();
        let result = Erc20OnEosEosTxInfos::is_log_supported_erc20_peg_in(&log, &token_dictionary).unwrap();
        assert!(result);
    }

    #[test]
    fn erc20_log_with_peg_in_should_be_erc20_log_with_peg_in() {
        let log = get_sample_log_with_erc20_peg_in_event();
        let result = Erc20OnEosEosTxInfos::is_log_erc20_peg_in(&log).unwrap();
        assert!(result);
    }

    #[test]
    fn should_return_true_if_receipt_contains_log_with_erc20_peg_in() {
        let dictionary = get_sample_eos_eth_token_dictionary();
        let receipt = get_sample_receipt_with_erc20_peg_in_event();
        let result = Erc20OnEosEosTxInfos::receipt_contains_supported_erc20_peg_in(&receipt, &dictionary);
        assert!(result);
    }

    #[test]
    fn should_return_false_if_receipt_does_not_contain_log_with_erc20_peg_in() {
        let dictionary = EosEthTokenDictionary::new(vec![]);
        let receipt = get_sample_receipt_with_erc20_peg_in_event();
        let result = Erc20OnEosEosTxInfos::receipt_contains_supported_erc20_peg_in(&receipt, &dictionary);
        assert!(!result);
    }

    #[test]
    fn should_get_supported_erc20_peg_in_logs() {
        let expected_result = EthLogs::new(vec![EthLog {
            address: EthAddress::from_slice(&hex::decode("d0a3d2d3d19a6ac58e60254fd606ec766638c3ba").unwrap()),
            topics: vec![EthHash::from_slice(&hex::decode("42877668473c4cba073df41397388516dc85c3bbae14b33603513924cec55e36").unwrap())],
            data: hex::decode("0000000000000000000000009f57cb2a4f462a5258a49e88b4331068a391de66000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac00000000000000000000000000000000000000000000000000000000000005390000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000c616e656f73616464726573730000000000000000000000000000000000000000").unwrap(),
        }]);
        let expected_num_logs = 1;
        let dictionary = get_sample_eos_eth_token_dictionary();
        let receipt = get_sample_receipt_with_erc20_peg_in_event();
        let result = Erc20OnEosEosTxInfos::get_supported_erc20_peg_in_logs_from_receipt(&receipt, &dictionary);
        assert_eq!(result.len(), expected_num_logs);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_get_erc20_eth_tx_infos_from_receipt() {
        let eth_token_decimals = 18;
        let eos_token_decimals = 9;
        let eth_symbol = "SAM".to_string();
        let eos_symbol = "SAM".to_string();
        let token_name = "SampleToken".to_string();
        let token_address = EthAddress::from_slice(&hex::decode("9f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let eth_basis_points = 0;
        let eos_basis_points = 0;
        let token_dictionary = EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::new(
            eth_token_decimals,
            eos_token_decimals,
            eth_symbol,
            eos_symbol,
            token_name,
            token_address,
            eth_basis_points,
            eos_basis_points,
            U256::zero(),
            0,
            0,
            "".to_string(),
        )]);
        let expected_num_results = 1;
        let expected_result = get_sample_erc20_on_eos_eos_tx_info().unwrap();
        let receipt = get_sample_receipt_with_erc20_peg_in_event();
        let origin_chain_id = EthChainId::Mainnet;
        let result = Erc20OnEosEosTxInfos::from_eth_receipt(&receipt, &token_dictionary, &origin_chain_id).unwrap();
        assert_eq!(result.len(), expected_num_results);
        assert_eq!(result.0[0], expected_result);
    }

    #[test]
    fn should_not_get_get_erc20_eth_tx_infos_from_receipt_if_token_not_supported() {
        let token_dictionary = EosEthTokenDictionary::new(vec![]);
        let expected_num_results = 0;
        let origin_chain_id = EthChainId::Mainnet;
        let receipt = get_sample_receipt_with_erc20_peg_in_event();
        let result = Erc20OnEosEosTxInfos::from_eth_receipt(&receipt, &token_dictionary, &origin_chain_id).unwrap();
        assert_eq!(result.len(), expected_num_results);
    }

    #[test]
    fn should_filter_submission_material_for_receipts_containing_supported_erc20_eos_tx_infos() {
        let expected_num_receipts_after = 1;
        let expected_num_receipts_before = 69;
        let token_dictionary = get_sample_eos_eth_token_dictionary();
        let submission_material = get_sample_submission_material_with_erc20_peg_in_event();
        let num_receipts_before = submission_material.receipts.len();
        assert_eq!(num_receipts_before, expected_num_receipts_before);
        let filtered_submission_material = Erc20OnEosEosTxInfos::filter_eth_sub_mat_for_supported_eos_tx_infos(
            &submission_material,
            &token_dictionary,
        )
        .unwrap();
        let num_receipts_after = filtered_submission_material.receipts.len();
        assert_eq!(num_receipts_after, expected_num_receipts_after);
    }

    #[test]
    fn should_get_erc20_on_eos_eos_tx_infos() {
        let eth_token_decimals = 18;
        let eos_token_decimals = 9;
        let eth_symbol = "SAM".to_string();
        let eos_symbol = "SAM".to_string();
        let token_name = "SampleToken".to_string();
        let token_address = EthAddress::from_slice(&hex::decode("9f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let eth_basis_points = 0;
        let eos_basis_points = 0;
        let token_dictionary = EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::new(
            eth_token_decimals,
            eos_token_decimals,
            eth_symbol,
            eos_symbol,
            token_name,
            token_address,
            eth_basis_points,
            eos_basis_points,
            U256::zero(),
            0,
            0,
            "".to_string(),
        )]);
        let expected_num_results = 1;
        let submission_material = get_sample_submission_material_with_erc20_peg_in_event();
        let origin_chain_id = EthChainId::Mainnet;
        let expected_result = get_sample_erc20_on_eos_eos_tx_infos().unwrap();
        let result =
            Erc20OnEosEosTxInfos::from_submission_material(&submission_material, &token_dictionary, &origin_chain_id)
                .unwrap();
        assert_eq!(result.len(), expected_num_results);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_erc20_on_eos_eos_tx_info_to_metadata() {
        let info = get_sample_erc20_on_eos_eos_tx_info().unwrap();
        let result = info.to_metadata();
        assert!(result.is_ok());
    }

    #[test]
    fn should_convert_erc20_on_eos_eos_tx_info_to_metadata_bytes() {
        let info = get_sample_zero_eos_asset_eos_tx_info();
        let result = info.to_metadata_bytes().unwrap();
        let expected_result = "0103decaff04005fe7f92a307865646238366364343535656633636134336630653232376530303436396333626466613430363238";
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_get_eos_signed_txs_from_eos_tx_infos() {
        let user_data = vec![];
        let info = Erc20OnEosEosTxInfo::new(
            U256::from_dec_str("1337").unwrap(),
            EthAddress::from_slice(&hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap()),
            EthAddress::from_slice(&hex::decode("9f57cb2a4f462a5258a49e88b4331068a391de66").unwrap()),
            "aneosaddress".to_string(),
            EthHash::from_slice(
                &hex::decode("241f386690b715422102edf42f5c9edcddea16b64f17d02bad572f5f341725c0").unwrap(),
            ),
            "sampletoken".to_string(),
            "0.00000000 SAM".to_string(),
            user_data,
            EthChainId::Mainnet,
        );
        let infos = Erc20OnEosEosTxInfos::new(vec![info]);
        let pk = get_sample_eos_private_key();
        let ref_block_num = 1;
        let ref_block_prefix = 2;
        let chain_id = EosChainId::EosMainnet;
        let dictionary = get_sample_eos_eth_dictionary();
        let result = infos
            .to_eos_signed_txs(ref_block_num, ref_block_prefix, &chain_id, &pk, &dictionary)
            .unwrap();
        let expected_result = "010002000000000000000100a68234ab58a5c10000000000a531760100a68234ab58a5c100000000a8ed32321980b1ba29194cd53400000000000000000853414d000000000000";
        let result_without_timestamp = &result[0].transaction[8..];
        assert_eq!(result_without_timestamp, expected_result);
    }

    #[test]
    fn should_subtract_amount_from_eos_on_eth_eos_tx_info() {
        let info = get_sample_erc20_on_eos_eos_tx_info().unwrap();
        let subtrahend = U256::from(337);
        let info_after = info.subtract_amount(subtrahend).unwrap();
        let result = info_after.get_amount();
        let expected_result = U256::from(1000);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_fees_from_eos_on_eth_eos_tx_infos() {
        let eth_address = EthAddress::from_slice(&hex::decode("9e57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let expected_result = vec![
            (eth_address, U256::from_dec_str("3342500000").unwrap()),
            (eth_address, U256::from_dec_str("1665000000").unwrap()),
        ];
        let infos = get_sample_erc20_on_eos_eos_tx_infos_2();
        let dictionary = get_sample_eos_eth_token_dictionary();
        let result = infos.get_fees(&dictionary).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_subtract_fees_from_eos_on_eth_eos_tx_infos() {
        let infos = get_sample_erc20_on_eos_eos_tx_infos_2();
        let dictionary = get_sample_eos_eth_token_dictionary();
        let result = infos.subtract_fees(&dictionary).unwrap();
        let expected_amount_1 = U256::from_dec_str("1333657500000").unwrap();
        let expected_amount_2 = U256::from_dec_str("664335000000").unwrap();
        assert_eq!(result[0].token_amount, expected_amount_1);
        assert_eq!(result[1].token_amount, expected_amount_2);
    }
}
