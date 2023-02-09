
use common::{
    dictionaries::eth_evm::EthEvmTokenDictionary,
    metadata::{
        metadata_address::MetadataAddress,
        metadata_protocol_id::MetadataProtocolId,
        metadata_traits::{ToMetadata, ToMetadataChainId},
        Metadata,
    },
    safe_addresses::safely_convert_str_to_eth_address,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    EthChainId,
};
use common_eth::{
    encode_erc20_vault_peg_out_fxn_data_with_user_data,
    encode_erc20_vault_peg_out_fxn_data_without_user_data,
    Erc777RedeemEvent,
    EthDbUtilsExt,
    EthLog,
    EthLogExt,
    EthLogs,
    EthPrivateKey,
    EthReceipt,
    EthReceipts,
    EthState,
    EthSubmissionMaterial,
    EthTransaction as EvmTransaction,
    EthTransactions as EvmTransactions,
    ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA,
    MAX_BYTES_FOR_ETH_USER_DATA,
    ZERO_ETH_VALUE,
};
use derive_more::{Constructor, Deref, IntoIterator};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

use crate::fees_calculator::{FeeCalculator, FeesCalculator};

#[cfg_attr(test, derive(Constructor))]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Erc20OnEvmEthTxInfo {
    pub native_token_amount: U256,
    pub token_sender: EthAddress,
    pub originating_tx_hash: EthHash,
    pub evm_token_address: EthAddress,
    pub eth_token_address: EthAddress,
    pub destination_address: EthAddress,
    pub user_data: Bytes,
    pub origin_chain_id: EthChainId,
    pub eth_vault_address: EthAddress,
}

impl ToMetadata for Erc20OnEvmEthTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        let user_data = if self.user_data.len() > MAX_BYTES_FOR_ETH_USER_DATA {
            // TODO Test for this case!
            info!(
                "✘ `user_data` redacted from `Metadata` ∵ it's > {} bytes",
                MAX_BYTES_FOR_ETH_USER_DATA
            );
            vec![]
        } else {
            self.user_data.clone()
        };
        Ok(Metadata::new(
            &user_data,
            &MetadataAddress::new_from_eth_address(&self.token_sender, &self.origin_chain_id.to_metadata_chain_id())?,
        ))
    }

    fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Ethereum)
    }
}

impl FeeCalculator for Erc20OnEvmEthTxInfo {
    fn get_token_address(&self) -> EthAddress {
        debug!(
            "Getting token address in `Erc20OnEvmEthTxInfo` of {}",
            self.evm_token_address
        );
        self.evm_token_address
    }

    fn get_amount(&self) -> U256 {
        debug!(
            "Getting token amount in `Erc20OnEvmEthTxInfo` of {}",
            self.native_token_amount
        );
        self.native_token_amount
    }

    fn subtract_amount(&self, subtrahend: U256) -> Result<Self> {
        if subtrahend >= self.native_token_amount {
            Err("Cannot subtract amount from `Erc20OnEvmEthTxInfo`: subtrahend too large!".into())
        } else {
            let new_amount = self.native_token_amount - subtrahend;
            debug!(
                "Subtracting {} from {} to get final amount of {} in `Erc20OnEvmEthTxInfo`!",
                subtrahend, self.native_token_amount, new_amount
            );
            Ok(self.update_amount(new_amount))
        }
    }
}

impl Erc20OnEvmEthTxInfo {
    fn update_amount(&self, new_amount: U256) -> Self {
        let mut new_self = self.clone();
        new_self.native_token_amount = new_amount;
        new_self
    }

    pub fn to_eth_signed_tx(
        // TODO Get a sample with user data so we can use that test against!
        &self,
        nonce: u64,
        chain_id: &EthChainId,
        gas_price: u64,
        evm_private_key: &EthPrivateKey,
        vault_address: &EthAddress,
    ) -> Result<EvmTransaction> {
        let gas_limit = if self.user_data.is_empty() {
            chain_id.get_erc20_vault_pegout_without_user_data_gas_limit()
        } else {
            chain_id.get_erc20_vault_pegout_with_user_data_gas_limit()
        };
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
        debug!(
            "✔ Signing tx for token amount: {}",
            self.native_token_amount.to_string()
        );
        debug!("✔ Signing tx for vault address: {}", vault_address.to_string());
        let encoded_tx_data = if self.user_data.is_empty() {
            encode_erc20_vault_peg_out_fxn_data_without_user_data(
                self.destination_address,
                self.eth_token_address,
                self.native_token_amount,
            )?
        } else {
            encode_erc20_vault_peg_out_fxn_data_with_user_data(
                self.destination_address,
                self.eth_token_address,
                self.native_token_amount,
                self.to_metadata_bytes()?,
            )?
        };
        EvmTransaction::new_unsigned(
            encoded_tx_data,
            nonce,
            ZERO_ETH_VALUE,
            *vault_address,
            chain_id,
            gas_limit,
            gas_price,
        )
        .sign(evm_private_key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Constructor, Deref, IntoIterator, Serialize, Deserialize)]
pub struct Erc20OnEvmEthTxInfos(pub Vec<Erc20OnEvmEthTxInfo>);

impl FeesCalculator for Erc20OnEvmEthTxInfos {
    fn get_fees(&self, dictionary: &EthEvmTokenDictionary) -> Result<Vec<(EthAddress, U256)>> {
        debug!("Calculating fees in `Erc20OnEvmEthTxInfo`...");
        self.iter()
            .map(|info| info.calculate_fee_via_dictionary(dictionary))
            .collect()
    }

    fn subtract_fees(&self, dictionary: &EthEvmTokenDictionary) -> Result<Self> {
        self.get_fees(dictionary).and_then(|fee_tuples| {
            Ok(Self::new(
                self.iter()
                    .zip(fee_tuples.iter())
                    .map(|(info, (_, fee))| {
                        if *fee == U256::zero() {
                            debug!("Not subtracting fee because `fee` is 0!");
                            Ok(info.clone())
                        } else {
                            info.subtract_amount(*fee)
                        }
                    })
                    .collect::<Result<Vec<Erc20OnEvmEthTxInfo>>>()?,
            ))
        })
    }
}

impl Erc20OnEvmEthTxInfos {
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

    pub fn filter_out_zero_values(&self) -> Result<Self> {
        Ok(Self::new(
            self.iter()
                .filter(|tx_info| match tx_info.native_token_amount != U256::zero() {
                    true => true,
                    false => {
                        info!("✘ Filtering out redeem info due to zero asset amount: {:?}", tx_info);
                        false
                    },
                })
                .cloned()
                .collect::<Vec<Erc20OnEvmEthTxInfo>>(),
        ))
    }

    fn is_log_erc20_on_evm_redeem(log: &EthLog, dictionary: &EthEvmTokenDictionary) -> Result<bool> {
        debug!(
            "✔ Checking log contains topic: {}",
            hex::encode(ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA.as_bytes())
        );
        let token_is_supported = dictionary.is_evm_token_supported(&log.address);
        let log_contains_topic = log.contains_topic(&ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA);
        debug!("✔ Log is supported: {}", token_is_supported);
        debug!("✔ Log has correct topic: {}", log_contains_topic);
        Ok(token_is_supported && log_contains_topic)
    }

    pub fn is_log_supported_erc20_on_evm_redeem(log: &EthLog, dictionary: &EthEvmTokenDictionary) -> Result<bool> {
        match Self::is_log_erc20_on_evm_redeem(log, dictionary)? {
            false => Ok(false),
            true => Ok(dictionary.is_evm_token_supported(&log.address)),
        }
    }

    fn get_supported_erc20_on_evm_logs_from_receipt(
        receipt: &EthReceipt,
        dictionary: &EthEvmTokenDictionary,
    ) -> EthLogs {
        EthLogs::new(
            receipt
                .logs
                .iter()
                .filter(|log| matches!(Self::is_log_supported_erc20_on_evm_redeem(log, dictionary), Ok(true)))
                .cloned()
                .collect(),
        )
    }

    fn receipt_contains_supported_erc20_on_evm_redeem(
        receipt: &EthReceipt,
        dictionary: &EthEvmTokenDictionary,
    ) -> bool {
        Self::get_supported_erc20_on_evm_logs_from_receipt(receipt, dictionary).len() > 0
    }

    fn from_eth_receipt(
        receipt: &EthReceipt,
        dictionary: &EthEvmTokenDictionary,
        origin_chain_id: &EthChainId,
        eth_vault_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `Erc20OnEvmEthTxInfos` from receipt...");
        Ok(Self::new(
            Self::get_supported_erc20_on_evm_logs_from_receipt(receipt, dictionary)
                .iter()
                .map(|log| {
                    let event_params = Erc777RedeemEvent::from_eth_log(log)?;
                    let tx_info = Erc20OnEvmEthTxInfo {
                        evm_token_address: log.address,
                        token_sender: event_params.redeemer,
                        eth_vault_address: *eth_vault_address,
                        origin_chain_id: origin_chain_id.clone(),
                        user_data: event_params.user_data.clone(),
                        originating_tx_hash: receipt.transaction_hash,
                        eth_token_address: dictionary.get_eth_address_from_evm_address(&log.address)?,
                        destination_address: safely_convert_str_to_eth_address(
                            &event_params.underlying_asset_recipient,
                        ),
                        native_token_amount: dictionary
                            .convert_evm_amount_to_eth_amount(&log.address, event_params.value)?,
                    };
                    info!("✔ Parsed tx info: {:?}", tx_info);
                    Ok(tx_info)
                })
                .collect::<Result<Vec<Erc20OnEvmEthTxInfo>>>()?,
        ))
    }

    fn filter_eth_submission_material_for_supported_redeems(
        submission_material: &EthSubmissionMaterial,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<EthSubmissionMaterial> {
        info!("✔ Filtering submission material receipts for those pertaining to `ERC20-on-EVM` redeems...");
        info!(
            "✔ Num receipts before filtering: {}",
            submission_material.receipts.len()
        );
        let filtered_receipts = EthReceipts::new(
            submission_material
                .receipts
                .iter()
                .filter(|receipt| {
                    Erc20OnEvmEthTxInfos::receipt_contains_supported_erc20_on_evm_redeem(receipt, dictionary)
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
        origin_chain_id: &EthChainId,
        eth_vault_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `Erc20OnEvmEthTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Self::from_eth_receipt(receipt, dictionary, origin_chain_id, eth_vault_address))
                .collect::<Result<Vec<Erc20OnEvmEthTxInfos>>>()?
                .into_iter()
                .flatten()
                .collect(),
        ))
    }

    pub fn to_eth_signed_txs(
        &self,
        start_nonce: u64,
        chain_id: &EthChainId,
        gas_price: u64,
        evm_private_key: &EthPrivateKey,
        vault_address: &EthAddress,
    ) -> Result<EvmTransactions> {
        info!("✔ Signing `ERC20-on-EVM` ETH transactions...");
        Ok(EvmTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, tx_info)| {
                    Erc20OnEvmEthTxInfo::to_eth_signed_tx(
                        tx_info,
                        start_nonce + i as u64,
                        chain_id,
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
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `Erc20OnEvmEthTxInfos`...");
    state
        .evm_db_utils
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
                EthEvmTokenDictionary::get_from_db(state.db)
                    .and_then(|account_names| {
                        Erc20OnEvmEthTxInfos::from_submission_material(
                            &submission_material,
                            &account_names,
                            &state.evm_db_utils.get_eth_chain_id_from_db()?,
                            &state.evm_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                        )
                    })
                    .and_then(|infos| infos.to_bytes())
                    .map(|bytes| state.add_tx_infos(bytes))
            }
        })
}

pub fn filter_out_zero_value_eth_tx_infos_from_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ NOT filtering out zero value `Erc20OnEvmEthTxInfos` because there are none to filter!");
        Ok(state)
    } else {
        info!("✔ Maybe filtering out zero value `Erc20OnEvmEthTxInfos`...");
        Erc20OnEvmEthTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| {
                debug!("✔ Num `Erc20OnEvmEthTxInfos` before: {}", infos.len());
                infos.filter_out_zero_values()
            })
            .and_then(|infos| {
                debug!("✔ Num `Erc20OnEvmEthTxInfos` after: {}", infos.len());
                infos.to_bytes()
            })
            .map(|bytes| state.add_tx_infos(bytes))
    }
}

pub fn filter_submission_material_for_redeem_events_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing `ERC20-on-EVM` redeem events...");
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_addresses_and_with_topics(
            &state.get_eth_evm_token_dictionary()?.to_evm_addresses(),
            &[*ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA],
        )
        .and_then(|filtered_submission_material| {
            Erc20OnEvmEthTxInfos::filter_eth_submission_material_for_supported_redeems(
                &filtered_submission_material,
                state.get_eth_evm_token_dictionary()?,
            )
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}

pub fn maybe_sign_eth_txs_and_add_to_evm_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ No tx infos in state ∴ no ETH transactions to sign!");
        Ok(state)
    } else {
        Erc20OnEvmEthTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| {
                infos.to_eth_signed_txs(
                    state.eth_db_utils.get_eth_account_nonce_from_db()?,
                    &state.eth_db_utils.get_eth_chain_id_from_db()?,
                    state.eth_db_utils.get_eth_gas_price_from_db()?,
                    &state.eth_db_utils.get_eth_private_key_from_db()?,
                    &state.eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                )
            })
            .and_then(|signed_txs| {
                debug!("✔ Signed transactions: {:?}", signed_txs);
                state.add_erc20_on_evm_eth_signed_txs(signed_txs)
            })
    }
}
